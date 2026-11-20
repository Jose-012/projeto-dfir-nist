use crate::application::process::{ProcessHandle, ProcessInput, ProcessMode, launch_process};
use crate::config::AppConfig;
use dfir_protocol::{
    ERROR_MESSAGE, ErrorMessage, MAX_JSON_DEPTH, PROGRESS_MESSAGE, PROTOCOL_VERSION, Progress,
    RESULT_MESSAGE, Request, SANDBOX_PHASE_PREFIX, SCHEMA_VERSION, json_depth_exceeds_limit,
    validate,
};
use futures_util::StreamExt;
use serde_json::Value;

use std::{
    collections::HashMap,
    io,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
use thiserror::Error;

use tokio::{
    fs::File,
    io::{AsyncRead, AsyncReadExt, AsyncWriteExt},
    sync::{Notify, Semaphore},
    time,
};
use tokio_util::codec::{FramedRead, LinesCodec, LinesCodecError};
use uuid::Uuid;

/// Aviso NÃO fatal emitido pelo engine (ex.: `EVTX_TRUNCATED`,
/// `PARSER_RECORD_ERROR`), agregado por código: quantas vezes ocorreu e uma
/// amostra da mensagem. Antes eram lidos e descartados (AUD-04/AUD-36).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineDiagnostic {
    pub code: String,
    pub count: u64,
    pub sample: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineRunSummary {
    pub message_count: usize,
    pub item_count: u64,
    pub diagnostics: Vec<EngineDiagnostic>,
    /// Estado do sandbox informado pelo próprio engine (`policy=...;landlock=...`),
    /// sem caminhos. `None` se o engine não informou (F1-22).
    pub sandbox: Option<String>,
}

/// `true` quando o resumo do sandbox mostra todas as camadas aplicadas.
pub fn sandbox_fully_applied(summary: &str) -> bool {
    !summary.contains("unavailable") && !summary.contains("skipped")
}

fn diagnostic_sample(message: &str) -> String {
    const MAX_SAMPLE_CHARS: usize = 300;
    message.chars().take(MAX_SAMPLE_CHARS).collect()
}

#[derive(Debug, Error)]
pub enum EngineRunError {
    #[error("invalid request: {0}")]
    InvalidRequest(String),
    #[error("engine I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("engine execution timed out")]
    Timeout,
    #[error("analysis cancelled by operator")]
    Cancelled,
    #[error("engine output limit exceeded")]
    OutputLimitExceeded,
    #[error("analysis stopped: free disk space fell below the configured safety margin")]
    DiskSpaceLow,
    #[error("engine protocol error: {0}")]
    Protocol(String),
    #[error("engine process failed: {0}")]
    EngineFailed(String),
}

#[derive(Debug)]
struct CancellationInner {
    requested: AtomicBool,
    notify: Notify,
}

/// Sinal interno de um único job. A requisição também é persistida no banco;
/// este sinal apenas acorda imediatamente as esperas do processo atual.
#[derive(Debug, Clone)]
pub struct EngineCancellation {
    inner: Arc<CancellationInner>,
}

impl EngineCancellation {
    fn new() -> Self {
        Self {
            inner: Arc::new(CancellationInner {
                requested: AtomicBool::new(false),
                notify: Notify::new(),
            }),
        }
    }

    pub fn request(&self) {
        self.inner.requested.store(true, Ordering::Release);
        self.inner.notify.notify_waiters();
    }

    pub fn is_cancelled(&self) -> bool {
        self.inner.requested.load(Ordering::Acquire)
    }

    pub async fn cancelled(&self) {
        let notified = self.inner.notify.notified();
        if self.is_cancelled() {
            return;
        }
        notified.await;
    }
}

#[derive(Debug, Default)]
struct CancellationRegistry {
    active: Mutex<HashMap<String, EngineCancellation>>,
}

/// Mantém o registro somente enquanto a análise correspondente existe.
pub struct CancellationRegistration {
    job_id: String,
    registry: Arc<CancellationRegistry>,
    token: EngineCancellation,
}

impl CancellationRegistration {
    pub fn token(&self) -> EngineCancellation {
        self.token.clone()
    }
}

impl Drop for CancellationRegistration {
    fn drop(&mut self) {
        if let Ok(mut active) = self.registry.active.lock() {
            let same_registration = active
                .get(&self.job_id)
                .is_some_and(|current| Arc::ptr_eq(&current.inner, &self.token.inner));
            if same_registration {
                active.remove(&self.job_id);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct EngineRunnerConfig {
    pub engine_path: PathBuf,
    pub max_line_bytes: usize,
    pub default_timeout: Duration,
    pub trusted_data_root: PathBuf,
    pub max_concurrent_jobs: usize,
    pub max_memory_bytes: u64,
    /// Margem mínima de espaço livre; abaixo dela a análise é interrompida.
    pub min_free_disk_bytes: u64,
    /// Política do sandbox repassada ao engine (`DFIR_ENGINE_SANDBOX`).
    pub sandbox_policy: Option<String>,
    semaphore: Arc<Semaphore>,
    cancellations: Arc<CancellationRegistry>,
}

impl EngineRunnerConfig {
    pub fn from_config(config: &AppConfig) -> Self {
        Self {
            engine_path: config.engine_path.clone(),
            max_line_bytes: config.engine_max_line_bytes,
            default_timeout: config.engine_timeout,
            trusted_data_root: config.data_root.clone(),
            max_concurrent_jobs: config.engine_max_concurrent_jobs,
            max_memory_bytes: config.engine_max_memory_bytes,
            min_free_disk_bytes: config.min_free_disk_bytes,
            sandbox_policy: config.engine_sandbox.clone(),
            semaphore: Arc::new(Semaphore::new(config.engine_max_concurrent_jobs)),
            cancellations: Arc::new(CancellationRegistry::default()),
        }
    }

    pub fn register_cancellation(&self, job_id: &str) -> CancellationRegistration {
        let token = EngineCancellation::new();
        self.cancellations
            .active
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(job_id.to_string(), token.clone());
        CancellationRegistration {
            job_id: job_id.to_string(),
            registry: self.cancellations.clone(),
            token,
        }
    }

    /// Acorda a análise local depois que o cancelamento foi persistido.
    /// `false` significa que o job pertence a outro processo ou já terminou;
    /// nesse caso o observador do banco faz a propagação.
    pub fn cancel_registered_job(&self, job_id: &str) -> bool {
        let token = self
            .cancellations
            .active
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(job_id)
            .cloned();
        if let Some(token) = token {
            token.request();
            true
        } else {
            false
        }
    }
}

pub fn validate_envelope(value: &Value, max_line_bytes: usize) -> Result<Request, EngineRunError> {
    validate(value, max_line_bytes).map_err(EngineRunError::InvalidRequest)
}

pub async fn run_engine_cancellable(
    request: Request,
    input_root: &Path,
    input_file: &Path,
    output_dir: &Path,
    config: &EngineRunnerConfig,
    cancellation: EngineCancellation,
) -> Result<EngineRunSummary, EngineRunError> {
    run_engine_with_input(
        request,
        ProcessInput::File {
            root: input_root.to_path_buf(),
            file: input_file.to_path_buf(),
        },
        output_dir,
        config,
        ProcessMode::Stdio,
        cancellation,
    )
    .await
}

async fn run_engine_with_input(
    request: Request,
    input: ProcessInput,
    output_dir: &Path,
    config: &EngineRunnerConfig,
    mode: ProcessMode,
    cancellation: EngineCancellation,
) -> Result<EngineRunSummary, EngineRunError> {
    let _permit = tokio::select! {
        biased;
        _ = cancellation.cancelled() => return Err(EngineRunError::Cancelled),
        permit = config.semaphore.acquire() => permit.map_err(|_| {
            EngineRunError::InvalidRequest("engine concurrency limiter closed".into())
        })?,
    };
    tracing::debug!(
        max_concurrent_jobs = config.max_concurrent_jobs,
        ?mode,
        "engine job admitted"
    );
    let started = std::time::Instant::now();

    validate_envelope(
        &serde_json::to_value(&request)
            .map_err(|error| EngineRunError::Protocol(error.to_string()))?,
        config.max_line_bytes,
    )?;

    let deadline = Instant::now() + config.default_timeout;
    let mut child: ProcessHandle =
        launch_process(config, &input, output_dir, mode == ProcessMode::Smoke)?;
    tracing::debug!(?mode, "launching engine process");

    let Some(mut stdin) = child.take_stdin() else {
        child.kill().await;
        return Err(EngineRunError::Io(io::Error::new(
            io::ErrorKind::BrokenPipe,
            "engine stdin unavailable",
        )));
    };
    let Some(stdout) = child.take_stdout() else {
        child.kill().await;
        return Err(EngineRunError::Io(io::Error::new(
            io::ErrorKind::BrokenPipe,
            "engine stdout unavailable",
        )));
    };
    let Some(mut stderr) = child.take_stderr() else {
        child.kill().await;
        return Err(EngineRunError::Io(io::Error::new(
            io::ErrorKind::BrokenPipe,
            "engine stderr unavailable",
        )));
    };
    let stderr_task = tokio::spawn(async move { read_limited(&mut stderr, 4096).await });
    let results_path = output_dir.join("results.jsonl");
    let mut results_file = File::create(&results_path)
        .await
        .map_err(EngineRunError::Io)?;

    let encoded = serde_json::to_vec(&request)
        .map_err(|error| EngineRunError::Protocol(error.to_string()))?;
    if encoded.len() > config.max_line_bytes {
        abort_child(&mut child, stderr_task).await;
        return Err(EngineRunError::InvalidRequest(
            "request line exceeds maximum".into(),
        ));
    }

    let remaining = deadline.saturating_duration_since(Instant::now());
    if remaining.is_zero() {
        abort_child(&mut child, stderr_task).await;
        return Err(EngineRunError::Timeout);
    }
    let write_result = tokio::select! {
        biased;
        _ = cancellation.cancelled() => {
            abort_child(&mut child, stderr_task).await;
            return Err(EngineRunError::Cancelled);
        }
        result = time::timeout(remaining, async {
            stdin.write_all(&encoded).await?;
            stdin.write_all(b"\n").await?;
            stdin.shutdown().await
        }) => result,
    };
    match write_result {
        Ok(Ok(())) => {
            // The engine protocol is request-per-process for the current runtime path.
            // Closing the parent's stdin handle is therefore part of the protocol: without
            // it the child keeps waiting for EOF, while this side waits for the child to exit.
            drop(stdin);
        }
        Ok(Err(error)) => {
            abort_child(&mut child, stderr_task).await;
            return Err(EngineRunError::Io(error));
        }
        Err(_) => {
            abort_child(&mut child, stderr_task).await;
            return Err(EngineRunError::Timeout);
        }
    }

    let mut message_count = 0_usize;
    let mut total_result_bytes = 0_u64;
    let mut total_items = 0_u64;
    let mut sandbox: Option<String> = None;
    let mut saw_completed_result = false;
    let mut last_disk_check = Instant::now();
    let mut diagnostics: std::collections::BTreeMap<String, (u64, String)> =
        std::collections::BTreeMap::new();
    let mut lines = FramedRead::new(
        stdout,
        LinesCodec::new_with_max_length(config.max_line_bytes),
    );

    loop {
        if cancellation.is_cancelled() {
            abort_child(&mut child, stderr_task).await;
            return Err(EngineRunError::Cancelled);
        }
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            abort_child(&mut child, stderr_task).await;
            return Err(EngineRunError::Timeout);
        }

        // Sem teto de saída (ADR-0020), o freio é o espaço livre: a cada
        // segundo confere o volume e interrompe com erro explícito.
        if config.min_free_disk_bytes > 0 && last_disk_check.elapsed() >= Duration::from_secs(1) {
            last_disk_check = Instant::now();
            if matches!(
                crate::infrastructure::disk::check_path(output_dir, config.min_free_disk_bytes),
                crate::infrastructure::disk::SpaceCheck::Low { .. }
            ) {
                abort_child(&mut child, stderr_task).await;
                return Err(EngineRunError::DiskSpaceLow);
            }
        }

        let line_result = tokio::select! {
            biased;
            _ = cancellation.cancelled() => {
                abort_child(&mut child, stderr_task).await;
                return Err(EngineRunError::Cancelled);
            }
            result = time::timeout(remaining, lines.next()) => result,
        };
        let line = match line_result {
            Ok(Some(Ok(line))) => line.into_bytes(),
            Ok(Some(Err(error))) => {
                abort_child(&mut child, stderr_task).await;
                return Err(lines_codec_error(error, config.max_line_bytes));
            }
            Ok(None) => break,
            Err(_) => {
                abort_child(&mut child, stderr_task).await;
                return Err(EngineRunError::Timeout);
            }
        };

        if json_depth_exceeds_limit(&line, MAX_JSON_DEPTH) {
            abort_child(&mut child, stderr_task).await;
            return Err(EngineRunError::Protocol(
                "engine response exceeds configured JSON nesting depth".into(),
            ));
        }
        let value: Value = match serde_json::from_slice(&line) {
            Ok(value) => value,
            Err(error) => {
                abort_child(&mut child, stderr_task).await;
                return Err(EngineRunError::Protocol(error.to_string()));
            }
        };
        if value.get("protocol_version").and_then(Value::as_str) != Some(PROTOCOL_VERSION)
            || value.get("schema_version").and_then(Value::as_str) != Some(SCHEMA_VERSION)
            || value.get("request_id").and_then(Value::as_str) != Some(request.request_id.as_str())
            || value.get("job_id").and_then(Value::as_str) != Some(request.job_id.as_str())
        {
            abort_child(&mut child, stderr_task).await;
            return Err(EngineRunError::Protocol(
                "engine response does not match request contract".into(),
            ));
        }
        if let Some(items) = value.get("items").and_then(Value::as_array) {
            total_items = total_items.saturating_add(items.len() as u64);
            if total_items > request.max_items {
                abort_child(&mut child, stderr_task).await;
                return Err(EngineRunError::OutputLimitExceeded);
            }
        }

        let message_type = value
            .get("message_type")
            .and_then(Value::as_str)
            .unwrap_or("");
        match message_type {
            PROGRESS_MESSAGE => {
                let progress = serde_json::from_value::<Progress>(value)
                    .map_err(|error| EngineRunError::Protocol(error.to_string()))?;
                if let Some(state) = progress.phase.strip_prefix(SANDBOX_PHASE_PREFIX) {
                    sandbox = Some(state.chars().take(200).collect());
                }
            }
            RESULT_MESSAGE => {
                let status = value.get("status").and_then(Value::as_str).ok_or_else(|| {
                    EngineRunError::Protocol("engine RESULT missing status".into())
                })?;
                total_result_bytes = total_result_bytes.saturating_add(line.len() as u64 + 1);
                if total_result_bytes > request.max_output_bytes {
                    abort_child(&mut child, stderr_task).await;
                    return Err(EngineRunError::OutputLimitExceeded);
                }
                if let Err(error) =
                    write_result_part(&mut results_file, &line, deadline, &cancellation).await
                {
                    abort_child(&mut child, stderr_task).await;
                    return Err(error);
                }
                if let Err(error) =
                    write_result_part(&mut results_file, b"\n", deadline, &cancellation).await
                {
                    abort_child(&mut child, stderr_task).await;
                    return Err(error);
                }

                match status {
                    "PARTIAL" => {
                        if saw_completed_result {
                            abort_child(&mut child, stderr_task).await;
                            return Err(EngineRunError::Protocol(
                                "engine emitted PARTIAL after COMPLETED".into(),
                            ));
                        }
                    }
                    "COMPLETED" => {
                        if saw_completed_result {
                            abort_child(&mut child, stderr_task).await;
                            return Err(EngineRunError::Protocol(
                                "engine emitted multiple COMPLETED results".into(),
                            ));
                        }
                        saw_completed_result = true;
                    }
                    _ => {
                        abort_child(&mut child, stderr_task).await;
                        return Err(EngineRunError::Protocol(
                            "engine emitted RESULT with invalid status".into(),
                        ));
                    }
                }
            }
            ERROR_MESSAGE => {
                let error_message = serde_json::from_value::<ErrorMessage>(value)
                    .map_err(|error| EngineRunError::Protocol(error.to_string()))?;
                if is_fatal_engine_error(error_message.code.as_str()) {
                    abort_child(&mut child, stderr_task).await;
                    return Err(EngineRunError::EngineFailed(format!(
                        "{}: {}",
                        error_message.code, error_message.message
                    )));
                }
                let entry = diagnostics
                    .entry(error_message.code.clone())
                    .or_insert_with(|| (0, diagnostic_sample(&error_message.message)));
                entry.0 = entry.0.saturating_add(1);
            }
            _ => {
                abort_child(&mut child, stderr_task).await;
                return Err(EngineRunError::Protocol(
                    "unknown engine message_type".into(),
                ));
            }
        }
        message_count = message_count.saturating_add(1);
    }

    if !saw_completed_result {
        child.kill().await;
        abort_stderr_task(stderr_task).await;
        return Err(EngineRunError::Protocol(
            "engine exited without a COMPLETED RESULT message".into(),
        ));
    }

    let wait_result = tokio::select! {
        biased;
        _ = cancellation.cancelled() => Err(EngineRunError::Cancelled),
        result = child.wait(deadline.saturating_duration_since(Instant::now())) => result,
    };
    let status = match wait_result {
        Ok(status) => status,
        Err(EngineRunError::Cancelled) => {
            child.kill().await;
            abort_stderr_task(stderr_task).await;
            return Err(EngineRunError::Cancelled);
        }
        Err(EngineRunError::Timeout) => {
            child.kill().await;
            abort_stderr_task(stderr_task).await;
            return Err(EngineRunError::Timeout);
        }
        Err(error) => {
            child.kill().await;
            abort_stderr_task(stderr_task).await;
            return Err(error);
        }
    };

    let stderr = stderr_task
        .await
        .map_err(|error| EngineRunError::EngineFailed(format!("stderr task failed: {error}")))??;
    if !status.success {
        return Err(EngineRunError::EngineFailed(if stderr.is_empty() {
            format!("engine exited with code {}", status.code)
        } else {
            stderr
        }));
    }
    tracing::debug!(
        result_path = %results_path.display(),
        elapsed_ms = started.elapsed().as_millis(),
        item_count = total_items,
        output_bytes = total_result_bytes,
        "engine job completed",
    );

    Ok(EngineRunSummary {
        message_count,
        item_count: total_items,
        sandbox,
        diagnostics: diagnostics
            .into_iter()
            .map(|(code, (count, sample))| EngineDiagnostic {
                code,
                count,
                sample,
            })
            .collect(),
    })
}

pub async fn run_process_smoke(
    request: Request,
    input_dir: &Path,
    output_dir: &Path,
    config: &EngineRunnerConfig,
) -> Result<EngineRunSummary, EngineRunError> {
    run_engine_with_input(
        request,
        ProcessInput::Directory(input_dir.to_path_buf()),
        output_dir,
        config,
        ProcessMode::Smoke,
        EngineCancellation::new(),
    )
    .await
}

fn is_fatal_engine_error(code: &str) -> bool {
    matches!(
        code,
        "BAD_JSONL"
            | "BAD_JSON"
            | "INVALID_REQUEST"
            | "INPUT_UNAVAILABLE"
            | "INPUT_REJECTED"
            | "UNSUPPORTED_FORMAT"
            | "UNSUPPORTED_ANALYSIS_KIND"
            | "PARSER_ERROR"
            | "YARA_RULESET_ERROR"
            | "YARA_SCAN_ERROR"
            | "RESULT_LINE_TOO_LARGE"
            | "OUTPUT_LIMIT"
            | "OUTPUT_ERROR"
            | "SERIALIZATION_ERROR"
            | "SANDBOX_UNAVAILABLE"
    )
}

async fn write_result_part(
    file: &mut File,
    bytes: &[u8],
    deadline: Instant,
    cancellation: &EngineCancellation,
) -> Result<(), EngineRunError> {
    let remaining = deadline.saturating_duration_since(Instant::now());
    if remaining.is_zero() {
        return Err(EngineRunError::Timeout);
    }
    tokio::select! {
        biased;
        _ = cancellation.cancelled() => Err(EngineRunError::Cancelled),
        result = time::timeout(remaining, file.write_all(bytes)) => match result {
            Ok(Ok(())) => Ok(()),
            Ok(Err(error)) => Err(EngineRunError::Io(error)),
            Err(_) => Err(EngineRunError::Timeout),
        },
    }
}

async fn abort_child(
    child: &mut ProcessHandle,
    stderr_task: tokio::task::JoinHandle<Result<String, io::Error>>,
) {
    child.kill().await;
    abort_stderr_task(stderr_task).await;
}

async fn abort_stderr_task(stderr_task: tokio::task::JoinHandle<Result<String, io::Error>>) {
    stderr_task.abort();
    // F1-23: o JoinError de um abort é esperado aqui; registrar como debug em
    // vez de engolir em silêncio.
    if let Err(error) = stderr_task.await {
        tracing::debug!(%error, "stderr task aborted as expected");
    }
}

async fn read_limited<R: AsyncRead + Unpin>(
    reader: &mut R,
    limit: usize,
) -> Result<String, io::Error> {
    let mut collected = Vec::with_capacity(limit.min(4096));
    let mut buffer = [0_u8; 4096];
    let mut truncated = false;
    loop {
        let read = reader.read(&mut buffer).await?;
        if read == 0 {
            break;
        }
        if collected.len() < limit {
            let take = (limit - collected.len()).min(read);
            collected.extend_from_slice(&buffer[..take]);
            truncated |= take < read;
        } else {
            truncated = true;
        }
    }
    let mut text = String::from_utf8_lossy(&collected).into_owned();
    if truncated {
        text.push_str(" [truncated]");
    }
    Ok(text)
}

fn lines_codec_error(error: LinesCodecError, max_line_bytes: usize) -> EngineRunError {
    match error {
        LinesCodecError::MaxLineLengthExceeded => EngineRunError::Protocol(format!(
            "engine output line exceeds configured limit of {max_line_bytes} bytes"
        )),
        other => EngineRunError::Io(io::Error::new(
            io::ErrorKind::InvalidData,
            other.to_string(),
        )),
    }
}

pub fn new_request(
    input_reference: String,
    max_output_bytes: u64,
    max_items: u64,
    analysis_kind: &str,
) -> Request {
    Request {
        protocol_version: PROTOCOL_VERSION.into(),
        schema_version: SCHEMA_VERSION.into(),
        message_type: "REQUEST".into(),
        request_id: Uuid::new_v4().to_string(),
        job_id: Uuid::new_v4().to_string(),
        input_reference,
        output_reference: None,
        max_output_bytes,
        max_items,
        analysis_kind: analysis_kind.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        EngineCancellation, EngineRunError, EngineRunnerConfig, PROTOCOL_VERSION, SCHEMA_VERSION,
        new_request, run_engine_cancellable, validate_envelope,
    };
    use dfir_protocol::ANALYSIS_KIND_EVTX_TIMELINE;
    use futures_util::StreamExt;
    use serde_json::json;
    use tokio_util::codec::{FramedRead, LinesCodec, LinesCodecError};

    #[tokio::test]
    async fn bounded_jsonl_reader_accepts_exact_limit_but_rejects_overflow() {
        let (writer, reader) = tokio::io::duplex(128);
        tokio::spawn(async move {
            use tokio::io::AsyncWriteExt;
            let mut writer = writer;
            writer.write_all(b"12345\n").await.expect("write line");
        });
        let mut framed = FramedRead::new(reader, LinesCodec::new_with_max_length(5));
        let line = framed.next().await.expect("line").expect("valid line");
        assert_eq!(line, "12345");

        let (writer, reader) = tokio::io::duplex(128);
        tokio::spawn(async move {
            use tokio::io::AsyncWriteExt;
            let mut writer = writer;
            writer
                .write_all(b"123456")
                .await
                .expect("write oversized line");
        });
        let mut framed = FramedRead::new(reader, LinesCodec::new_with_max_length(5));
        let error = framed
            .next()
            .await
            .expect("codec result")
            .expect_err("overflow");
        assert!(matches!(error, LinesCodecError::MaxLineLengthExceeded));
    }

    #[test]
    fn validates_logical_request_and_rejects_traversal() {
        let request = new_request(
            "evidence/demo.evtx".into(),
            1024,
            10,
            ANALYSIS_KIND_EVTX_TIMELINE,
        );
        let value = serde_json::to_value(&request).expect("test serialization");
        assert!(validate_envelope(&value, 64 * 1024).is_ok());

        let bad = json!({
            "protocol_version": PROTOCOL_VERSION,
            "schema_version": SCHEMA_VERSION,
            "message_type": "REQUEST",
            "request_id": "req",
            "job_id": "job",
            "input_reference": "../secret",
            "output_reference": null,
            "max_output_bytes": 1024,
            "max_items": 10,
            "analysis_kind": ANALYSIS_KIND_EVTX_TIMELINE
        });
        assert!(matches!(
            validate_envelope(&bad, 64 * 1024),
            Err(EngineRunError::InvalidRequest(_))
        ));
    }

    #[tokio::test]
    async fn cancellation_wakes_a_job_waiting_for_the_semaphore() {
        let root = tempfile::tempdir().expect("root");
        let config = test_config(root.path(), &root.path().join("unused-engine"));
        let _permit = config.semaphore.acquire().await.expect("first permit");
        let cancellation = EngineCancellation::new();
        let signal = cancellation.clone();
        let request = new_request(
            "evidence/input.evtx".into(),
            1024,
            10,
            ANALYSIS_KIND_EVTX_TIMELINE,
        );
        let root_path = root.path().to_path_buf();
        let input_root = root_path.join("evidence");
        let input_file = input_root.join("input.evtx");
        let output = root_path.join("output");
        let runner_config = config.clone();
        let task = tokio::spawn(async move {
            run_engine_cancellable(
                request,
                &input_root,
                &input_file,
                &output,
                &runner_config,
                signal,
            )
            .await
        });
        cancellation.request();
        let result = tokio::time::timeout(std::time::Duration::from_secs(1), task)
            .await
            .expect("cancellation timeout")
            .expect("task join");
        assert!(matches!(result, Err(EngineRunError::Cancelled)));
    }

    fn test_config(root: &std::path::Path, engine: &std::path::Path) -> EngineRunnerConfig {
        let config = crate::config::AppConfig {
            data_root: root.to_path_buf(),
            engine_path: engine.to_path_buf(),
            engine_max_line_bytes: 1024 * 1024,
            engine_timeout: std::time::Duration::from_secs(60),
            engine_max_concurrent_jobs: 1,
            engine_max_memory_bytes: 1024 * 1024 * 1024,
            max_upload_bytes: 1024 * 1024,
            min_free_disk_bytes: 0,
            engine_sandbox: Some("best-effort".into()),
        };
        EngineRunnerConfig::from_config(&config)
    }

    #[cfg(unix)]
    fn write_script(path: &std::path::Path, body: &str) {
        use std::os::unix::fs::PermissionsExt;
        std::fs::write(path, format!("#!/bin/sh\n{body}\n")).expect("write script");
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).expect("chmod");
    }

    #[cfg(unix)]
    async fn run_cancellation_scenario(close_stdout: bool) {
        let root = tempfile::tempdir().expect("root");
        let root_path = root.path().canonicalize().expect("canonical root");
        let evidence = root_path.join("evidence");
        let output = root_path.join("staging/output");
        std::fs::create_dir_all(&evidence).expect("evidence dir");
        std::fs::create_dir_all(&output).expect("output dir");
        let input = evidence.join("input.evtx");
        std::fs::write(&input, b"synthetic input").expect("input");
        let script = root_path.join("cancellable-engine.sh");
        let output_result = if close_stdout {
            "printf '%s\\n' '{\"protocol_version\":\"1\",\"schema_version\":\"4\",\"request_id\":\"req-wait\",\"job_id\":\"job-wait\",\"message_type\":\"RESULT\",\"status\":\"COMPLETED\",\"items\":[]}'\nexec 1>&-"
        } else {
            ""
        };
        write_script(
            &script,
            &format!(
                "read _request\necho $$ > \"$DFIR_ENGINE_OUTPUT_ROOT/engine.pid\"\n{output_result}\nexec /bin/sleep 60"
            ),
        );
        let config = test_config(&root_path, &script);
        let mut request = new_request(
            "evidence/input.evtx".into(),
            1024 * 1024,
            10,
            ANALYSIS_KIND_EVTX_TIMELINE,
        );
        request.request_id = "req-wait".into();
        request.job_id = "job-wait".into();
        let cancellation = EngineCancellation::new();
        let signal = cancellation.clone();
        let task = tokio::spawn(async move {
            run_engine_cancellable(request, &evidence, &input, &output, &config, signal).await
        });
        let pid_file = root_path.join("staging/output/engine.pid");
        let mut pid = None;
        for _ in 0..100 {
            if let Ok(text) = std::fs::read_to_string(&pid_file) {
                pid = text.trim().parse::<libc::pid_t>().ok();
            }
            if pid.is_some() {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
        let pid = pid.expect("engine pid file");
        // SAFETY: sinal 0 apenas verifica que o processo sintético existe.
        assert_eq!(unsafe { libc::kill(pid, 0) }, 0);
        if close_stdout {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        cancellation.request();
        let result = tokio::time::timeout(std::time::Duration::from_secs(2), task)
            .await
            .expect("cancellation timeout")
            .expect("task join");
        assert!(
            matches!(result, Err(EngineRunError::Cancelled)),
            "{result:?}"
        );
        let mut gone = false;
        for _ in 0..100 {
            // SAFETY: ESRCH confirma que o processo foi encerrado e recolhido.
            if unsafe { libc::kill(pid, 0) } != 0 {
                gone = true;
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
        assert!(gone, "o processo continuou vivo após o cancelamento");
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn cancellation_kills_the_process_during_jsonl_read() {
        run_cancellation_scenario(false).await;
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn cancellation_kills_the_process_during_final_wait() {
        run_cancellation_scenario(true).await;
    }
}
