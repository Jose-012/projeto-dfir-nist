//! Saída do engine: linhas JSONL no stdout, com contabilização de tamanho.

use crate::error::EngineFailure;
use dfir_protocol::{MAX_LINE_BYTES, Request, ResultMessage};
use serde_json::Value;
use std::io::{self, Write};

pub fn result_line_size(
    request: &Request,
    items: &[Value],
    completed: bool,
) -> Result<usize, EngineFailure> {
    let value = serde_json::json!({
        "protocol_version": dfir_protocol::PROTOCOL_VERSION,
        "schema_version": dfir_protocol::SCHEMA_VERSION,
        "message_type": dfir_protocol::RESULT_MESSAGE,
        "request_id": request.request_id,
        "job_id": request.job_id,
        "status": if completed { "COMPLETED" } else { "PARTIAL" },
        "items": items,
    });
    serde_json::to_vec(&value)
        .map(|encoded| encoded.len())
        .map_err(|error| {
            (
                "SERIALIZATION_ERROR",
                format!("failed to size result batch: {error}"),
            )
        })
}

pub fn emit_result_batch(
    request: &Request,
    stdout: &mut impl Write,
    items: Vec<Value>,
    completed: bool,
    previous_bytes: u64,
) -> Result<u64, EngineFailure> {
    let result = ResultMessage::batch(request, items, completed);
    let encoded = serde_json::to_vec(&result).map_err(|error| {
        (
            "SERIALIZATION_ERROR",
            format!("failed to encode result batch: {error}"),
        )
    })?;
    if encoded.len() > MAX_LINE_BYTES {
        return Err((
            "RESULT_LINE_TOO_LARGE",
            "result batch exceeds protocol line limit; reduce max_items or parser batch size"
                .into(),
        ));
    }
    let next_bytes = previous_bytes.saturating_add(encoded.len() as u64 + 1);
    if next_bytes > request.max_output_bytes {
        return Err(("OUTPUT_LIMIT", "result exceeds max_output_bytes".into()));
    }
    stdout
        .write_all(&encoded)
        .and_then(|_| stdout.write_all(b"\n"))
        .and_then(|_| stdout.flush())
        .map_err(|error| ("OUTPUT_ERROR", error.to_string()))?;
    Ok(next_bytes)
}

pub fn emit_protocol_line<T: serde::Serialize>(
    stdout: &mut impl Write,
    message: &T,
) -> Result<(), io::Error> {
    serde_json::to_writer(&mut *stdout, message)
        .map_err(|error| io::Error::other(error.to_string()))?;
    stdout.write_all(b"\n")?;
    stdout.flush()
}

#[cfg(test)]
mod tests {
    use super::{emit_protocol_line, emit_result_batch, result_line_size};
    use dfir_protocol::{ANALYSIS_KIND_YARA_SCAN, ErrorMessage, Request};
    use serde_json::json;

    fn request(max_output_bytes: u64) -> Request {
        Request {
            protocol_version: dfir_protocol::PROTOCOL_VERSION.into(),
            schema_version: dfir_protocol::SCHEMA_VERSION.into(),
            message_type: dfir_protocol::REQUEST_MESSAGE.into(),
            request_id: "req".into(),
            job_id: "job".into(),
            input_reference: "evidence/sample.bin".into(),
            output_reference: None,
            max_output_bytes,
            max_items: 10,
            analysis_kind: ANALYSIS_KIND_YARA_SCAN.into(),
        }
    }

    #[test]
    fn result_batch_is_one_jsonl_line_and_matches_the_sized_length() {
        let request = request(1024 * 1024);
        let items = vec![json!({"a": 1}), json!({"b": 2})];
        let sized = result_line_size(&request, &items, true).expect("size computed");

        let mut out = Vec::new();
        let total = emit_result_batch(&request, &mut out, items, true, 0).expect("batch emitted");

        assert_eq!(out.last(), Some(&b'\n'));
        assert_eq!(out.iter().filter(|&&b| b == b'\n').count(), 1);
        assert_eq!(
            out.len() - 1,
            sized,
            "sizing must match what is really written"
        );
        assert_eq!(total, out.len() as u64);
    }

    #[test]
    fn result_batch_over_max_output_bytes_is_rejected_without_writing() {
        let request = request(8);
        let mut out = Vec::new();
        let error = emit_result_batch(&request, &mut out, vec![json!({"a": 1})], true, 0)
            .expect_err("limit must apply");
        assert_eq!(error.0, "OUTPUT_LIMIT");
        assert!(out.is_empty());
    }

    #[test]
    fn output_accounting_accumulates_across_batches() {
        let request = request(1024 * 1024);
        let mut out = Vec::new();
        let first = emit_result_batch(&request, &mut out, vec![json!(1)], false, 0).expect("first");
        let second =
            emit_result_batch(&request, &mut out, vec![json!(2)], true, first).expect("second");
        assert_eq!(second, out.len() as u64);
        assert!(second > first);
    }

    #[test]
    fn protocol_lines_end_with_a_newline() {
        let mut out = Vec::new();
        emit_protocol_line(&mut out, &ErrorMessage::new("r", "j", "CODE", "msg"))
            .expect("line emitted");
        assert_eq!(out.last(), Some(&b'\n'));
        assert!(String::from_utf8(out).expect("utf-8").contains("\"CODE\""));
    }
}
