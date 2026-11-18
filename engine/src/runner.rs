//! EngineRunner: recebe um `Request` já validado pelo protocolo, resolve a
//! entrada e despacha para a análise pedida (`analysis_kind`).
//!
//! Antes de existir este módulo, esse despacho era um `match` de dois braços
//! dentro do `main.rs`; a spec já aponta novos formatos como próximos
//! candidatos, então o ponto de extensão passa a ser um só lugar. Cada análise
//! continua uma função concreta — não há trait de "parser" (ver ADR-0016).

use crate::{
    detect,
    error::EngineFailure,
    evtx_timeline, input,
    limits::{MAX_RECORD_ERRORS, RESULT_BATCH_SIZE},
    output::{emit_protocol_line, emit_result_batch, result_line_size},
    yara_scan,
};
use dfir_protocol::{
    ANALYSIS_KIND_EVTX_DETECTION, ANALYSIS_KIND_EVTX_TIMELINE, ANALYSIS_KIND_YARA_SCAN,
    ErrorMessage, MAX_LINE_BYTES, NormalizedEvent, Request, normalize,
};
use evtx::EvtxParser;
use serde_json::Value;
use std::{
    io::Write,
    path::{Path, PathBuf},
};

pub struct EngineRunner {
    input_root: PathBuf,
    /// Único arquivo que este processo aceita analisar (F1-22). `None` só no
    /// uso manual/QA, sem o supervisor.
    pinned_file: Option<PathBuf>,
}

impl EngineRunner {
    pub fn new(input_root: PathBuf) -> Self {
        Self {
            input_root,
            pinned_file: None,
        }
    }

    /// Fixa o único arquivo aceito (já canônico).
    pub fn with_pinned_file(mut self, pinned_file: Option<PathBuf>) -> Self {
        self.pinned_file = pinned_file;
        self
    }

    /// Usa a raiz de entrada do ambiente (`DFIR_ENGINE_INPUT_ROOT`, senão
    /// `/input`), lida uma única vez na criação do runner.
    pub fn from_env() -> Self {
        Self::new(input::input_root())
    }

    pub fn process_request(
        &self,
        request: &Request,
        stdout: &mut impl Write,
    ) -> Result<(), EngineFailure> {
        let path = input::resolve_input(&self.input_root, &request.input_reference)?;
        if let Some(pinned) = &self.pinned_file
            && &path != pinned
        {
            return Err((
                "INPUT_REJECTED",
                "input is not the single file pinned for this engine process".into(),
            ));
        }
        match request.analysis_kind.as_str() {
            ANALYSIS_KIND_EVTX_TIMELINE => process_evtx_request(request, &path, stdout, false),
            ANALYSIS_KIND_EVTX_DETECTION => process_evtx_request(request, &path, stdout, true),
            ANALYSIS_KIND_YARA_SCAN => process_yara_request(request, &path, stdout),
            other => Err((
                "UNSUPPORTED_ANALYSIS_KIND",
                format!("engine does not support analysis_kind {other:?}"),
            )),
        }
    }
}

/// Acrescenta um item ao lote e envia o lote quando ele enche ou quando a linha
/// do protocolo estouraria. Compartilhado pela timeline e pelas detecções.
fn push_batched(
    request: &Request,
    stdout: &mut impl Write,
    batch: &mut Vec<Value>,
    item: Value,
    result_bytes: &mut u64,
) -> Result<(), EngineFailure> {
    batch.push(item);
    if result_line_size(request, batch, false)? > MAX_LINE_BYTES && batch.len() > 1 {
        let last = batch.pop().expect("batch contains the just-pushed item");
        let full = std::mem::replace(batch, vec![last]);
        *result_bytes = emit_result_batch(request, stdout, full, false, *result_bytes)?;
    }
    if result_line_size(request, batch, false)? > MAX_LINE_BYTES {
        return Err((
            "RESULT_LINE_TOO_LARGE",
            "single result item exceeds protocol line limit".into(),
        ));
    }
    if batch.len() >= RESULT_BATCH_SIZE {
        let full = std::mem::replace(batch, Vec::with_capacity(RESULT_BATCH_SIZE));
        *result_bytes = emit_result_batch(request, stdout, full, false, *result_bytes)?;
    }
    Ok(())
}

/// Timeline (`detect == false`) ou detecção por regras (`detect == true`)
/// sobre o mesmo parser: a diferença é só o que se faz com cada registro.
fn process_evtx_request(
    request: &Request,
    path: &Path,
    stdout: &mut impl Write,
    detect: bool,
) -> Result<(), EngineFailure> {
    if !evtx_timeline::has_evtx_extension(path) {
        return Err((
            "UNSUPPORTED_FORMAT",
            "MVP parser supports EVTX files only".into(),
        ));
    }

    let mut parser = EvtxParser::from_path(path).map_err(|error| {
        (
            "PARSER_ERROR",
            format!("failed to open EVTX input: {error}"),
        )
    })?;

    // Avisos NÃO fatais sobre arquivo truncado (AUD-36): o app os guarda no job.
    let header_info = evtx_timeline::read_header_info(path);
    if let Some(warning) = header_info
        .as_ref()
        .and_then(evtx_timeline::truncation_warning)
    {
        emit_protocol_line(
            stdout,
            &ErrorMessage::new(
                &request.request_id,
                &request.job_id,
                "EVTX_TRUNCATED",
                &warning,
            ),
        )
        .map_err(|io_error| ("OUTPUT_ERROR", io_error.to_string()))?;
    }

    let mut batch = Vec::with_capacity(RESULT_BATCH_SIZE);
    let mut item_count = 0_u64;
    let mut result_bytes = 0_u64;
    let mut record_error_count = 0_usize;
    let mut max_record_id: Option<u64> = None;
    let mut detector = detect.then(|| detect::Detector::new(&request.input_reference));

    let records = parser.serialized_records(|record| {
        let record = record?;
        let event_record_id = record.event_record_id;
        let record_timestamp = record.timestamp.to_string();
        let data = record.into_json_value()?.data;
        let timestamp = evtx_timeline::event_timestamp(&record_timestamp, &data);
        Ok((event_record_id, timestamp, data))
    });
    for record in records {
        if item_count >= request.max_items {
            return Err((
                "ITEM_LIMIT",
                "engine result item limit reached before the evidence stream ended".into(),
            ));
        }

        match record {
            Ok((event_record_id, timestamp, data)) => {
                max_record_id =
                    Some(max_record_id.map_or(event_record_id, |m| m.max(event_record_id)));
                let fields = normalize::event_fields(&data);
                if let Some(detector) = detector.as_mut() {
                    // Detecção: o registro alimenta as regras; nada é emitido por registro.
                    if let Some(id) = normalize::event_id(&data) {
                        detector.observe(&detect::Ev {
                            record_id: event_record_id,
                            timestamp: &timestamp,
                            micros: normalize::epoch_micros(&timestamp),
                            id,
                            data: &data,
                            fields: &fields,
                        });
                    }
                    continue;
                }
                let item = NormalizedEvent::new(
                    event_record_id,
                    timestamp,
                    data,
                    request.input_reference.clone(),
                )
                .with_fields(fields)
                .into_item()
                .map_err(|error| {
                    (
                        "SERIALIZATION_ERROR",
                        format!("failed to encode normalized event: {error}"),
                    )
                })?;
                item_count += 1;
                push_batched(request, stdout, &mut batch, item, &mut result_bytes)?;
            }
            Err(error) => {
                if record_error_count >= MAX_RECORD_ERRORS {
                    emit_protocol_line(
                        stdout,
                        &ErrorMessage::new(
                            &request.request_id,
                            &request.job_id,
                            "PARSER_RECORD_ERROR_LIMIT",
                            "maximum parser record error count exceeded",
                        ),
                    )
                    .map_err(|io_error| ("OUTPUT_ERROR", io_error.to_string()))?;
                    return Err((
                        "PARSER_RECORD_ERROR_LIMIT",
                        "maximum parser record error count exceeded".into(),
                    ));
                }
                emit_protocol_line(
                    stdout,
                    &ErrorMessage::new(
                        &request.request_id,
                        &request.job_id,
                        "PARSER_RECORD_ERROR",
                        &error.to_string(),
                    ),
                )
                .map_err(|io_error| ("OUTPUT_ERROR", io_error.to_string()))?;
                record_error_count += 1;
            }
        }
    }

    if let Some(warning) = header_info
        .as_ref()
        .and_then(|info| evtx_timeline::missing_records_warning(info, max_record_id))
    {
        emit_protocol_line(
            stdout,
            &ErrorMessage::new(
                &request.request_id,
                &request.job_id,
                "EVTX_RECORDS_MISSING",
                &warning,
            ),
        )
        .map_err(|io_error| ("OUTPUT_ERROR", io_error.to_string()))?;
    }

    if let Some(detector) = detector {
        let (candidates, stats) = detector.finish();
        if stats.dropped_candidates > 0 || stats.dropped_burst_keys > 0 {
            // Não é fatal, mas também não pode passar despercebido: o teto de
            // memória das regras foi atingido e parte dos candidatos ficou de fora.
            emit_protocol_line(
                stdout,
                &ErrorMessage::new(
                    &request.request_id,
                    &request.job_id,
                    "DETECTION_LIMIT_REACHED",
                    &format!(
                        "detection limits reached: {} candidate(s) and {} burst key(s) were not tracked",
                        stats.dropped_candidates, stats.dropped_burst_keys
                    ),
                ),
            )
            .map_err(|io_error| ("OUTPUT_ERROR", io_error.to_string()))?;
        }
        if candidates.len() as u64 > request.max_items {
            return Err((
                "ITEM_LIMIT",
                "engine result item limit reached before the detection candidates were emitted"
                    .into(),
            ));
        }
        for candidate in candidates {
            let item = candidate.into_item().map_err(|error| {
                (
                    "SERIALIZATION_ERROR",
                    format!("failed to encode detection candidate: {error}"),
                )
            })?;
            push_batched(request, stdout, &mut batch, item, &mut result_bytes)?;
        }
    }

    // Always send a terminating RESULT message, even when no records were parsed.
    let _ = emit_result_batch(request, stdout, batch, true, result_bytes)?;

    Ok(())
}

fn process_yara_request(
    request: &Request,
    path: &Path,
    stdout: &mut impl Write,
) -> Result<(), EngineFailure> {
    // Unlike EVTX parsing, a YARA scan produces its full item list in one
    // pass (there is no per-record streaming API to page through), so the
    // item/output-limit checks happen once against the complete list rather
    // than incrementally per record.
    let items = yara_scan::scan_path(request, path)?;
    if items.len() as u64 > request.max_items {
        return Err((
            "ITEM_LIMIT",
            "engine result item limit reached before the evidence stream ended".into(),
        ));
    }

    let mut batch = Vec::with_capacity(RESULT_BATCH_SIZE);
    let mut result_bytes = 0_u64;
    for item in items {
        batch.push(item);
        if result_line_size(request, &batch, false)? > MAX_LINE_BYTES && batch.len() > 1 {
            let last = batch.pop().expect("batch contains the just-pushed item");
            result_bytes = emit_result_batch(request, stdout, batch, false, result_bytes)?;
            batch = vec![last];
        }
        if result_line_size(request, &batch, false)? > MAX_LINE_BYTES {
            return Err((
                "RESULT_LINE_TOO_LARGE",
                "single result item exceeds protocol line limit".into(),
            ));
        }
        if batch.len() >= RESULT_BATCH_SIZE {
            result_bytes = emit_result_batch(request, stdout, batch, false, result_bytes)?;
            batch = Vec::with_capacity(RESULT_BATCH_SIZE);
        }
    }

    // Always send a terminating RESULT message, even when no rule matched.
    let _ = emit_result_batch(request, stdout, batch, true, result_bytes)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::EngineRunner;
    use dfir_protocol::{
        ANALYSIS_KIND_EVTX_TIMELINE, ANALYSIS_KIND_YARA_SCAN, MAX_ITEMS, MAX_OUTPUT_BYTES, Request,
    };
    use serde_json::Value;

    const EICAR: &[u8] = br#"X5O!P%@AP[4\PZX54(P^)7CC)7}$EICAR-STANDARD-ANTIVIRUS-TEST-FILE!$H+H*"#;

    fn request(input_reference: &str, analysis_kind: &str) -> Request {
        Request {
            protocol_version: dfir_protocol::PROTOCOL_VERSION.into(),
            schema_version: dfir_protocol::SCHEMA_VERSION.into(),
            message_type: dfir_protocol::REQUEST_MESSAGE.into(),
            request_id: "req-test".into(),
            job_id: "job-test".into(),
            input_reference: input_reference.into(),
            output_reference: None,
            max_output_bytes: MAX_OUTPUT_BYTES,
            max_items: MAX_ITEMS,
            analysis_kind: analysis_kind.into(),
        }
    }

    fn runner_with_file(name: &str, bytes: &[u8]) -> (tempfile::TempDir, EngineRunner) {
        let dir = tempfile::tempdir().expect("temp dir");
        std::fs::write(dir.path().join(name), bytes).expect("write fixture");
        let runner = EngineRunner::new(dir.path().to_path_buf());
        (dir, runner)
    }

    fn jsonl(out: &[u8]) -> Vec<Value> {
        String::from_utf8(out.to_vec())
            .expect("utf-8 output")
            .lines()
            .map(|line| serde_json::from_str(line).expect("each output line is JSON"))
            .collect()
    }

    #[test]
    fn yara_scan_is_dispatched_and_ends_with_a_completed_result() {
        let (_dir, runner) = runner_with_file("sample.bin", EICAR);
        let mut out = Vec::new();
        runner
            .process_request(
                &request("evidence/sample.bin", ANALYSIS_KIND_YARA_SCAN),
                &mut out,
            )
            .expect("scan succeeds");

        let messages = jsonl(&out);
        let last = messages.last().expect("at least one message");
        assert_eq!(last["message_type"], "RESULT");
        assert_eq!(last["status"], "COMPLETED");
        assert_eq!(last["items"][0]["rule_id"], "eicar_test_string");
    }

    #[test]
    fn yara_scan_without_matches_still_sends_a_terminating_result() {
        let (_dir, runner) = runner_with_file("sample.bin", b"nothing to see here");
        let mut out = Vec::new();
        runner
            .process_request(
                &request("evidence/sample.bin", ANALYSIS_KIND_YARA_SCAN),
                &mut out,
            )
            .expect("scan succeeds");
        let messages = jsonl(&out);
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0]["status"], "COMPLETED");
        assert_eq!(messages[0]["items"].as_array().map(Vec::len), Some(0));
    }

    #[test]
    fn yara_item_limit_is_enforced_before_anything_is_emitted() {
        let (_dir, runner) = runner_with_file("sample.bin", EICAR);
        let mut limited = request("evidence/sample.bin", ANALYSIS_KIND_YARA_SCAN);
        limited.max_items = 0;
        let mut out = Vec::new();
        let error = runner
            .process_request(&limited, &mut out)
            .expect_err("limit must apply");
        assert_eq!(error.0, "ITEM_LIMIT");
        assert!(out.is_empty());
    }

    #[test]
    fn evtx_timeline_rejects_a_file_without_the_evtx_extension() {
        let (_dir, runner) = runner_with_file("sample.bin", b"not evtx");
        let mut out = Vec::new();
        let error = runner
            .process_request(
                &request("evidence/sample.bin", ANALYSIS_KIND_EVTX_TIMELINE),
                &mut out,
            )
            .expect_err("wrong extension");
        assert_eq!(error.0, "UNSUPPORTED_FORMAT");
    }

    #[test]
    fn evtx_timeline_reports_a_controlled_error_for_a_corrupt_evtx_file() {
        let (_dir, runner) = runner_with_file("broken.evtx", b"definitely not an evtx file");
        let mut out = Vec::new();
        let error = runner
            .process_request(
                &request("evidence/broken.evtx", ANALYSIS_KIND_EVTX_TIMELINE),
                &mut out,
            )
            .expect_err("corrupt input must not parse");
        assert_eq!(error.0, "PARSER_ERROR");
    }

    #[test]
    fn an_unknown_analysis_kind_is_rejected_by_the_runner_itself() {
        // O protocolo já rejeita isto em `validate`; o runner não confia nisso.
        let (_dir, runner) = runner_with_file("sample.bin", EICAR);
        let mut out = Vec::new();
        let error = runner
            .process_request(&request("evidence/sample.bin", "NOT_A_REAL_KIND"), &mut out)
            .expect_err("unknown kind");
        assert_eq!(error.0, "UNSUPPORTED_ANALYSIS_KIND");
    }

    #[test]
    fn input_errors_surface_before_any_analysis_runs() {
        let (_dir, runner) = runner_with_file("sample.bin", EICAR);
        let mut out = Vec::new();
        let error = runner
            .process_request(
                &request("evidence/absent.bin", ANALYSIS_KIND_YARA_SCAN),
                &mut out,
            )
            .expect_err("missing input");
        assert_eq!(error.0, "INPUT_UNAVAILABLE");
        assert!(out.is_empty());
    }
}
