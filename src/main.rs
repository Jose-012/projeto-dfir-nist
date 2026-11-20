mod application;
mod cli;
mod config;
mod error;
mod gui;
mod infrastructure;
mod persistence;
mod service;
mod text_safety;

use clap::{Parser, Subcommand};
use persistence::db;

#[derive(Parser, Debug)]
#[command(name = "dfir-app")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

/// `Gui` não é mais um `Command` explícito: a interface nativa é o
/// comportamento padrão do binário (sem subcomando), exatamente como
/// `Serve` era antes (ADR-0017). `AuditVerify`/`EngineSmoke` continuam
/// utilitários de linha de comando — não fazem sentido dentro da GUI, que é
/// para o fluxo interativo do dia a dia.
#[derive(Subcommand, Debug)]
enum Command {
    AuditVerify,
    EngineSmoke,
    /// Reverifica uma cópia do banco feita antes de migration (RF-032):
    /// SHA-256, integridade do SQLite, chaves estrangeiras e contagens.
    /// Aceita o nome da pasta em `backups/` ou o caminho dela.
    BackupVerify {
        folder: std::path::PathBuf,
    },
    /// Importa uma evidência (headless; JSON em stdout).
    Import {
        path: std::path::PathBuf,
        /// Origem da evidência (sistema ou área).
        #[arg(long)]
        source: Option<String>,
        /// Quem ou qual área forneceu.
        #[arg(long)]
        provided_by: Option<String>,
        #[arg(long)]
        collection_method: Option<String>,
        #[arg(long)]
        collection_tool: Option<String>,
        #[arg(long)]
        collection_tool_version: Option<String>,
        /// Data (AAAA-MM-DD) ou instante RFC 3339 da coleta.
        #[arg(long)]
        collected_at: Option<String>,
        /// Referência da autorização para coletar.
        #[arg(long)]
        authorization: Option<String>,
        /// Marca que a evidência contém dado pessoal.
        #[arg(long)]
        personal_data: bool,
    },
    /// Lista as evidências ativas (headless; JSON).
    EvidenceList,
    /// Vincula uma evidência a um incidente (F1-19; headless; JSON).
    LinkEvidence {
        evidence_id: String,
        #[arg(long)]
        incident: String,
    },
    /// Lista as evidências vinculadas a um incidente (F1-19; headless; JSON).
    IncidentEvidence {
        incident_id: String,
    },
    /// Cadeia de custódia de uma evidência (F1-24; headless; JSON).
    Custody {
        evidence_id: String,
    },
    /// Eventos de auditoria (F1-24; headless; JSON). Sem `--incident`, todos.
    Audit {
        #[arg(long)]
        incident: Option<String>,
        #[arg(long, default_value_t = 200)]
        limit: u64,
    },
    /// Relatório de diagnóstico para suporte: versões, contagens, falhas por
    /// código, espaço livre e arquivos de log (sem nomes de arquivo).
    Diagnostics,
    /// Executa uma análise sobre uma evidência (headless; JSON).
    Analyze {
        evidence_id: String,
        #[arg(long, default_value = "EVTX_TIMELINE")]
        kind: String,
        #[arg(long)]
        max_output_bytes: Option<u64>,
        #[arg(long)]
        max_items: Option<u64>,
    },
    /// Cancela a análise PENDING/PROCESSING mais recente da evidência.
    CancelAnalysis {
        evidence_id: String,
    },
    /// Uma página da timeline do job, em ordem cronológica (headless; JSON).
    Results {
        job_id: String,
        #[arg(long, default_value_t = 0)]
        offset: u64,
        #[arg(long, default_value_t = 200)]
        limit: u64,
        /// Texto livre (título, descrição, provedor, canal) e termos
        /// `campo:valor`: event, user, host, ip, src, dst, proc, action,
        /// outcome, provider, channel, from, to. Ex.: `event:4625 user:admin
        /// from:2026-03-01`. Ver docs/OPERATION.md.
        #[arg(long)]
        filter: Option<String>,
    },
    /// Roda as regras de detecção sobre uma evidência EVTX e grava os
    /// CANDIDATOS (hipóteses pendentes; nada vira achado sem confirmação).
    Detect {
        evidence_id: String,
    },
    /// Lista os candidatos de detecção de uma evidência (headless; JSON).
    Candidates {
        evidence_id: String,
        /// PENDING, CONFIRMED ou REJECTED.
        #[arg(long)]
        status: Option<String>,
    },
    /// Confirma um candidato: cria o mapeamento ATT&CK, o achado e os IOCs no
    /// incidente informado, com a referência dos eventos de origem.
    CandidateConfirm {
        candidate_id: String,
        #[arg(long)]
        incident: String,
        #[arg(long)]
        note: Option<String>,
    },
    /// Rejeita um candidato; o motivo é obrigatório e fica na auditoria.
    CandidateReject {
        candidate_id: String,
        #[arg(long)]
        reason: String,
    },
    /// Resultados analíticos da evidência, ex.: YARA (headless; JSON).
    Findings {
        evidence_id: String,
    },
    /// Gera os cinco documentos dos entregáveis e o manifest.json de um
    /// incidente (headless; JSON). Grava em `<out>/entregaveis-<incidente>/`;
    /// sem `--out`, usa a pasta de exportações da aplicação.
    Deliverables {
        incident_id: String,
        #[arg(long)]
        out: Option<std::path::PathBuf>,
        /// Não mascara IPs, e-mails, contas e nomes. Por padrão os dados
        /// pessoais são mascarados; esta escolha é explícita e fica na auditoria.
        #[arg(long)]
        no_redaction: bool,
    },
    /// Cria o incidente de DEMONSTRAÇÃO: importa o .evtx informado, executa
    /// as análises reais e registra o cenário BEC sintético (dados fictícios,
    /// rotulados DEMO) para exercitar os cinco documentos (headless; JSON).
    /// Cada execução cria um incidente novo.
    Demo {
        evtx: std::path::PathBuf,
    },
    /// Descarte local explícito; exige o nome exato do arquivo.
    Destroy {
        evidence_id: String,
        #[arg(long)]
        confirm_filename: String,
    },
}

fn main() -> Result<(), error::AppError> {
    // A configuração vem primeiro: ela diz onde ficam os logs (ADR-0022).
    // Erros de configuração saem pelo próprio retorno de `main`.
    let config = config::AppConfig::from_env()?;
    let paths = infrastructure::storage::StoragePaths::from_root(&config.data_root);
    paths.ensure()?;
    let log_file = infrastructure::logging::init(&paths.logs);
    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        os = std::env::consts::OS,
        arch = std::env::consts::ARCH,
        log_file = ?log_file,
        "dfir-app starting"
    );
    tracing::info!(
        data_root = %config.data_root.display(),
        "resolved evidence/data storage root"
    );
    // O engine é resolvido a partir de DFIR_ENGINE_PATH, do diretório do próprio
    // executável ou de /usr/local/bin (ADR-0015); logar o resultado evita que o
    // caminho fique implícito, como já era feito antes desta migração.
    tracing::info!(
        engine_path = %config.engine_path.display(),
        engine_present = config.engine_path.is_file(),
        "resolved analysis engine executable"
    );

    match Cli::parse().command {
        Some(Command::AuditVerify) => run_blocking(audit_verify(paths)),
        Some(Command::BackupVerify { folder }) => {
            let dir = if folder.is_absolute() || folder.exists() {
                folder
            } else {
                paths.backups.join(folder)
            };
            run_json(async move {
                let manifest = persistence::backup::verify_published(&dir)
                    .await
                    .map_err(error::AppError::Database)?;
                serde_json::to_string_pretty(&manifest)
                    .map_err(|error| error::AppError::Config(error.to_string()))
            })
        }
        Some(Command::EngineSmoke) => run_blocking(async {
            let output = cli::engine_smoke(&config, &paths).await?;
            print!("{output}");
            Ok(())
        }),
        Some(Command::Import {
            path,
            source,
            provided_by,
            collection_method,
            collection_tool,
            collection_tool_version,
            collected_at,
            authorization,
            personal_data,
        }) => {
            let provenance = service::provenance::EvidenceProvenance {
                source,
                provided_by,
                collection_method,
                collection_tool,
                collection_tool_version,
                collected_at,
                authorization_reference: authorization,
                contains_personal_data: personal_data,
            };
            run_json(async { cli::headless_import(&config, &paths, &path, &provenance).await })
        }
        Some(Command::Diagnostics) => {
            run_json(async { cli::headless_diagnostics(&config, &paths).await })
        }
        Some(Command::EvidenceList) => {
            run_json(async { cli::headless_evidence_list(&config, &paths).await })
        }
        Some(Command::LinkEvidence {
            evidence_id,
            incident,
        }) => run_json(async {
            cli::headless_link_evidence(&config, &paths, &evidence_id, &incident).await
        }),
        Some(Command::IncidentEvidence { incident_id }) => {
            run_json(async { cli::headless_incident_evidence(&config, &paths, &incident_id).await })
        }
        Some(Command::Custody { evidence_id }) => {
            run_json(async { cli::headless_custody(&config, &paths, &evidence_id).await })
        }
        Some(Command::Audit { incident, limit }) => run_json(async {
            cli::headless_audit(&config, &paths, incident.as_deref(), limit).await
        }),
        Some(Command::Analyze {
            evidence_id,
            kind,
            max_output_bytes,
            max_items,
        }) => run_json(async {
            cli::headless_analyze(
                &config,
                &paths,
                &evidence_id,
                &kind,
                max_output_bytes,
                max_items,
            )
            .await
        }),
        Some(Command::CancelAnalysis { evidence_id }) => {
            run_json(async { cli::headless_cancel_analysis(&config, &paths, &evidence_id).await })
        }
        Some(Command::Results {
            job_id,
            offset,
            limit,
            filter,
        }) => run_json(async {
            cli::headless_results(&config, &paths, &job_id, offset, limit, filter.as_deref()).await
        }),
        Some(Command::Detect { evidence_id }) => {
            run_json(async { cli::headless_detect(&config, &paths, &evidence_id).await })
        }
        Some(Command::Candidates {
            evidence_id,
            status,
        }) => run_json(async {
            cli::headless_candidates(&config, &paths, &evidence_id, status.as_deref()).await
        }),
        Some(Command::CandidateConfirm {
            candidate_id,
            incident,
            note,
        }) => run_json(async {
            cli::headless_candidate_confirm(
                &config,
                &paths,
                &candidate_id,
                &incident,
                note.as_deref(),
            )
            .await
        }),
        Some(Command::CandidateReject {
            candidate_id,
            reason,
        }) => run_json(async {
            cli::headless_candidate_reject(&config, &paths, &candidate_id, &reason).await
        }),
        Some(Command::Findings { evidence_id }) => {
            run_json(async { cli::headless_findings(&config, &paths, &evidence_id).await })
        }
        Some(Command::Deliverables {
            incident_id,
            out,
            no_redaction,
        }) => run_json(async {
            let options = service::deliverables::RenderOptions {
                mask_personal_data: !no_redaction,
            };
            cli::headless_deliverables(&config, &paths, &incident_id, out.as_deref(), options).await
        }),
        Some(Command::Demo { evtx }) => {
            run_json(async { cli::headless_demo(&config, &paths, &evtx).await })
        }
        Some(Command::Destroy {
            evidence_id,
            confirm_filename,
        }) => run_json(async {
            cli::headless_destroy(&config, &paths, &evidence_id, &confirm_filename).await
        }),
        // Default: nenhuma requisição HTTP inicia mais nada — a interface
        // nativa é a própria aplicação. `gui::launch` roda seu próprio
        // runtime Tokio numa thread dedicada (`src/gui/backend.rs`);
        // `eframe::run_native` precisa da thread principal só para si.
        None => gui::launch(config, paths),
    }
}

/// Executa uma operação headless e imprime o JSON resultante em stdout.
fn run_json<F>(future: F) -> Result<(), error::AppError>
where
    F: std::future::Future<Output = error::AppResult<String>> + Send,
{
    run_blocking(async {
        println!("{}", future.await?);
        Ok(())
    })
}

/// Runtime Tokio de curta duração para os utilitários de CLI que
/// permanecem além da GUI. Antes desta migração, este mesmo runtime também
/// hospedava o servidor Axum (`Command::Serve`, removido — ver ADR-0017).
fn run_blocking<F>(future: F) -> Result<(), error::AppError>
where
    F: std::future::Future<Output = Result<(), error::AppError>> + Send,
{
    // Thread dedicada com pilha grande + future na heap. Em build debug, o
    // future de `import` estourava os 8 MiB da thread principal
    // ("thread 'main' has overflowed its stack"), porque `block_on` mantém o
    // future na pilha de quem o chama. Achado pelo teste headless
    // `scripts/e2e_headless.py` (F1-34).
    const CLI_STACK_BYTES: usize = 64 * 1024 * 1024;
    std::thread::scope(|scope| {
        std::thread::Builder::new()
            .name("dfir-cli".into())
            .stack_size(CLI_STACK_BYTES)
            .spawn_scoped(scope, move || {
                tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(4)
                    .thread_stack_size(CLI_STACK_BYTES)
                    .enable_all()
                    .build()
                    .map_err(error::AppError::Storage)?
                    .block_on(Box::pin(future))
            })
            .map_err(error::AppError::Storage)?
            .join()
            .unwrap_or_else(|_| Err(error::AppError::Config("cli worker thread panicked".into())))
    })
}

async fn audit_verify(paths: infrastructure::storage::StoragePaths) -> Result<(), error::AppError> {
    let db_url = format!("sqlite://{}", paths.db.join("dfir.sqlite").display());
    let pool = db::connect(&db_url).await?;
    db::migrate(&pool).await?;
    let valid = persistence::audit::verify(&pool).await?;
    if valid {
        tracing::info!("audit chain verified");
    } else {
        return Err(error::AppError::Config(
            "audit chain verification failed".into(),
        ));
    }
    let evidence_ids: Vec<String> = sqlx::query_scalar("SELECT id FROM evidence ORDER BY id")
        .fetch_all(&pool)
        .await?;
    for evidence_id in evidence_ids {
        if !persistence::audit::verify_custody(&pool, &evidence_id).await? {
            return Err(error::AppError::Config(format!(
                "chain of custody verification failed for {evidence_id}"
            )));
        }
    }
    tracing::info!("chain of custody verified");
    Ok(())
}

#[cfg(test)]
mod cli_parse_tests {
    use super::{Cli, Command};
    use clap::Parser;

    /// RF-027 (protocolo do engine): produção usa subprocesso + JSONL e
    /// **nenhum endpoint TCP/Unix é exposto**. Guarda estática sobre o código
    /// do app, do engine e do protocol: nenhum listener de rede pode aparecer
    /// sem ADR. (O engine contido ainda prova em runtime que socket TCP/Unix
    /// é negado pelo seccomp: `engine/tests/sandbox.rs`.)
    ///
    /// O próprio teste é excluído da varredura (ele cita os identificadores
    /// como literais de string); o restante do arquivo `main.rs` é varrido.
    #[test]
    fn no_network_listener_exists_in_app_or_engine_source() {
        fn visit(dir: &std::path::Path, skip: &std::path::Path, offenders: &mut Vec<String>) {
            for entry in std::fs::read_dir(dir).expect("source dir") {
                let path = entry.expect("dir entry").path();
                if path.is_dir() {
                    visit(&path, skip, offenders);
                } else if path.extension().is_some_and(|ext| ext == "rs") && path != skip {
                    let source = std::fs::read_to_string(&path).expect("read source");
                    // Identificadores de rede da std/tokio (não `.bind(`/`.listen(`,
                    // que casariam com `sqlx::Query::bind` em qualquer repositório).
                    for needle in [
                        "TcpListener",
                        "TcpStream",
                        "UdpSocket",
                        "UnixListener",
                        "UnixStream",
                        "tokio::net::",
                        "std::net::Tcp",
                        "std::net::Udp",
                        "std::os::unix::net",
                    ] {
                        if source.contains(needle) {
                            offenders.push(format!("{}:{needle}", path.display()));
                        }
                    }
                }
            }
        }

        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut offenders = Vec::new();
        for crate_dir in ["src", "engine/src", "protocol/src"] {
            visit(
                &root.join(crate_dir),
                &root.join("src/main.rs"),
                &mut offenders,
            );
        }
        assert!(
            offenders.is_empty(),
            "endpoint de rede encontrado sem ADR (RF-027): {}",
            offenders.join(", ")
        );
    }

    #[test]
    fn analyze_defaults_to_the_evtx_timeline() {
        let cli = Cli::try_parse_from(["dfir-app", "analyze", "ev_1"]).expect("parses");
        match cli.command {
            Some(Command::Analyze {
                evidence_id,
                kind,
                max_items,
                ..
            }) => {
                assert_eq!(evidence_id, "ev_1");
                assert_eq!(kind, dfir_protocol::ANALYSIS_KIND_EVTX_TIMELINE);
                assert!(max_items.is_none());
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn destroy_requires_the_exact_filename_confirmation() {
        assert!(Cli::try_parse_from(["dfir-app", "destroy", "ev_1"]).is_err());
        let cli = Cli::try_parse_from([
            "dfir-app",
            "destroy",
            "ev_1",
            "--confirm-filename",
            "a.evtx",
        ])
        .expect("parses");
        assert!(matches!(cli.command, Some(Command::Destroy { .. })));
    }

    #[test]
    fn demo_takes_the_evtx_path_and_nothing_else() {
        assert!(Cli::try_parse_from(["dfir-app", "demo"]).is_err());
        let cli = Cli::try_parse_from(["dfir-app", "demo", "security.evtx"]).expect("parses");
        match cli.command {
            Some(Command::Demo { evtx }) => {
                assert_eq!(evtx, std::path::PathBuf::from("security.evtx"));
            }
            other => panic!("expected the demo command, got {other:?}"),
        }
    }

    #[test]
    fn no_subcommand_means_the_native_ui() {
        let cli = Cli::try_parse_from(["dfir-app"]).expect("parses");
        assert!(cli.command.is_none());
    }
}
