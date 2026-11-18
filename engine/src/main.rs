//! `dfir-engine`: processo separado que lê requests JSONL no stdin, executa a
//! análise pedida sobre a evidência e escreve mensagens JSONL no stdout.
//!
//! Este arquivo é só a camada de protocolo (leitura de linhas, limites de
//! tamanho/profundidade, validação, progresso e erros). O restante vive em:
//!
//! - `runner`: despacho por `analysis_kind` (`EngineRunner`);
//! - `input`: resolução segura de `input_reference`;
//! - `limits`: limites internos de execução;
//! - `output`: emissão de linhas/lotes JSONL com contabilização de tamanho;
//! - `evtx_timeline` e `yara_scan`: regras de cada tipo de análise;
//! - o contrato tipado do evento de timeline é `dfir_protocol::NormalizedEvent`.

use dfir_protocol::{
    ErrorMessage, MAX_JSON_DEPTH, MAX_LINE_BYTES, Progress, ResultMessage,
    json_depth_exceeds_limit, validate,
};
use futures_util::StreamExt;
use serde_json::Value;
use std::{io, path::PathBuf};
use tokio_util::codec::{FramedRead, LinesCodec, LinesCodecError};

mod detect;
mod error;
mod evtx_timeline;
mod input;
mod limits;
mod output;
mod runner;
mod sandbox;
mod yara_scan;

use output::emit_protocol_line;
use runner::EngineRunner;

const SMOKE_MARKER: &str = "process-smoke-ok.txt";

/// O que o sandbox precisa liberar: só o arquivo de evidência fixado (ou, no
/// uso manual sem supervisor, a raiz de entrada) e, no smoke de processo, a
/// pasta de saída. Todo o resto fica negado.
fn sandbox_plan(process_smoke: bool, pinned: &Option<PathBuf>) -> sandbox::Plan {
    if process_smoke {
        let write_dir = std::env::var_os("DFIR_ENGINE_OUTPUT_ROOT")
            .map(PathBuf::from)
            .filter(|path| path.is_absolute() && path.is_dir());
        return sandbox::Plan {
            write_dir,
            ..sandbox::Plan::default()
        };
    }
    match pinned {
        Some(file) => sandbox::Plan {
            read_file: Some(file.clone()),
            ..sandbox::Plan::default()
        },
        None => sandbox::Plan {
            read_dir: Some(input::input_root()).filter(|root| root.is_dir()),
            ..sandbox::Plan::default()
        },
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let process_smoke = std::env::args().any(|arg| arg == "--process-smoke");
    let selftest = std::env::args().any(|arg| arg == "--sandbox-selftest");
    if std::env::args().any(|arg| arg == "--list-rules") {
        // Catálogo das regras de detecção, para a documentação e para auditoria.
        let rules: Vec<serde_json::Value> = detect::catalogue()
            .iter()
            .map(|rule| {
                serde_json::json!({
                    "id": rule.id,
                    "version": rule.version,
                    "title": rule.title,
                    "tactic": rule.tactic,
                    "technique": rule.technique,
                    "sub_technique": rule.sub_technique,
                    "severity": rule.severity,
                    "confidence": rule.confidence,
                    "description": rule.description,
                    "limitations": rule.limitations,
                })
            })
            .collect();
        println!(
            "{}",
            serde_json::json!({ "rule_set_version": detect::RULE_SET_VERSION, "rules": rules })
        );
        return Ok(());
    }

    // O sandbox vem ANTES de qualquer thread e de qualquer leitura da
    // evidência (F1-22): Landlock e seccomp valem para a thread que chama e
    // para as que ela criar depois, inclusive os workers do tokio.
    let pinned = input::pinned_input_file();
    let plan = sandbox_plan(process_smoke, &pinned);
    let sandbox_state = sandbox::apply(sandbox::Policy::from_env(), &plan);

    if selftest {
        // Prova de dentro do processo contido (usada pelo teste de integração).
        let report = match &sandbox_state {
            Ok(status) => serde_json::json!({
                "sandbox": status.summary(),
                "fully_applied": status.fully_applied(),
                "probes": sandbox::selftest(pinned.as_deref()),
            }),
            Err(message) => serde_json::json!({ "sandbox_error": message }),
        };
        println!("{report}");
        return Ok(());
    }

    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .thread_stack_size(8 * 1024 * 1024)
        .enable_all()
        .build()?
        .block_on(async_main(process_smoke, pinned, sandbox_state))
}

async fn async_main(
    process_smoke: bool,
    pinned: Option<PathBuf>,
    sandbox_state: Result<sandbox::Status, String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Sem sandbox exigido aplicado, nem o probe de smoke escreve nada.
    if process_smoke && sandbox_state.is_ok() {
        run_process_smoke_probe()?;
    }
    let runner = EngineRunner::from_env().with_pinned_file(pinned);
    let stdin = tokio::io::stdin();
    let mut lines = FramedRead::new(stdin, LinesCodec::new_with_max_length(MAX_LINE_BYTES));
    let mut stdout = std::io::BufWriter::new(std::io::stdout().lock());

    while let Some(line) = lines.next().await {
        let line = match line {
            Ok(line) => line,
            Err(error) => {
                let message = match error {
                    LinesCodecError::MaxLineLengthExceeded => {
                        "protocol line exceeds configured limit".to_string()
                    }
                    other => format!("invalid JSONL input: {other}"),
                };
                emit_protocol_line(
                    &mut stdout,
                    &ErrorMessage::new("", "", "BAD_JSONL", &message),
                )?;
                continue;
            }
        };

        if json_depth_exceeds_limit(line.as_bytes(), MAX_JSON_DEPTH) {
            emit_protocol_line(
                &mut stdout,
                &ErrorMessage::new(
                    "",
                    "",
                    "JSON_DEPTH_LIMIT",
                    "JSON nesting depth exceeds the configured safety limit",
                ),
            )?;
            continue;
        }
        let value: Value = match serde_json::from_str(&line) {
            Ok(value) => value,
            Err(error) => {
                emit_protocol_line(
                    &mut stdout,
                    &ErrorMessage::new("", "", "BAD_JSON", &error.to_string()),
                )?;
                continue;
            }
        };

        let request = match validate(&value, MAX_LINE_BYTES) {
            Ok(request) => request,
            Err(error) => {
                let request_id = value
                    .get("request_id")
                    .and_then(Value::as_str)
                    .unwrap_or("");
                let job_id = value.get("job_id").and_then(Value::as_str).unwrap_or("");
                emit_protocol_line(
                    &mut stdout,
                    &ErrorMessage::new(request_id, job_id, "INVALID_REQUEST", &error),
                )?;
                continue;
            }
        };

        emit_protocol_line(&mut stdout, &Progress::accepted(&request))?;
        // Falha fechada (RF-036): sem o sandbox exigido, nenhuma evidência é lida.
        match &sandbox_state {
            Ok(status) => {
                emit_protocol_line(&mut stdout, &Progress::sandbox(&request, &status.summary()))?;
            }
            Err(message) => {
                emit_protocol_line(
                    &mut stdout,
                    &ErrorMessage::new(
                        &request.request_id,
                        &request.job_id,
                        "SANDBOX_UNAVAILABLE",
                        message,
                    ),
                )?;
                continue;
            }
        }
        emit_protocol_line(&mut stdout, &Progress::parsing(&request))?;

        if process_smoke {
            emit_protocol_line(
                &mut stdout,
                &ResultMessage::batch(&request, Vec::new(), true),
            )?;
            continue;
        }

        if let Err((code, message)) = runner.process_request(&request, &mut stdout) {
            emit_protocol_line(
                &mut stdout,
                &ErrorMessage::new(&request.request_id, &request.job_id, code, &message),
            )?;
        }
    }

    Ok(())
}

fn run_process_smoke_probe() -> Result<(), io::Error> {
    let output_root = std::env::var_os("DFIR_ENGINE_OUTPUT_ROOT")
        .map(PathBuf::from)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "DFIR_ENGINE_OUTPUT_ROOT is missing",
            )
        })?;
    if !output_root.is_absolute() || !output_root.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "DFIR_ENGINE_OUTPUT_ROOT must be an existing absolute directory",
        ));
    }

    let marker = output_root.join(SMOKE_MARKER);
    std::fs::write(&marker, b"process-ok\n")?;

    Ok(())
}
