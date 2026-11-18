use crate::{
    application::engine,
    config::AppConfig,
    error::{AppError, AppResult},
    infrastructure::storage::StoragePaths,
    persistence::{audit, db},
    service::{self, AppServices},
};
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Clone, Copy)]
pub enum CommandOutput {
    EngineSmoke { messages: usize },
}

/// Texto de saída dos subcomandos de CLI. Antes desta migração (v6.4.5) isto
/// era renderizado por um template Askama (`templates/cli/engine-smoke.txt`);
/// Askama só existia neste binário por causa do escaping automático em
/// contexto HTML que o template do frontend precisava (`templates/html/`), e
/// esse frontend não existe mais (ADR-0017). Manter uma dependência de
/// compilação de templates para uma única linha de texto sem HTML deixou de
/// ter a necessidade demonstrada que `AGENTS.md`/`CONVENTIONS.md` exigem —
/// por isso ela foi removida, não só desviada.
pub fn render_output(output: CommandOutput) -> AppResult<String> {
    match output {
        CommandOutput::EngineSmoke { messages } => Ok(format!(
            "Engine sandbox smoke: {messages} mensagens recebidas.\n"
        )),
    }
}

// ---------------------------------------------------------------------------
// Modo headless (F1-34, ADR-0019): adaptador FINO sobre `service::*`, usado
// para testes ponta a ponta e CI. Não duplica regra de negócio, não tem TUI e
// escreve JSON em stdout (logs vão para stderr).
// ---------------------------------------------------------------------------

/// Monta os mesmos `AppServices` que a GUI monta em `gui/backend.rs`.
pub async fn build_services(config: &AppConfig, paths: &StoragePaths) -> AppResult<AppServices> {
    let url = format!("sqlite://{}", paths.db.join("dfir.sqlite").display());
    let pool = db::connect(&url).await?;
    db::migrate_with_backup(&pool, &paths.backups, config.min_free_disk_bytes).await?;
    Ok(AppServices {
        db: pool,
        paths: paths.clone(),
        engine: engine::EngineRunnerConfig::from_config(config),
    })
}

fn to_json<T: Serialize>(value: &T) -> AppResult<String> {
    serde_json::to_string_pretty(value)
        .map_err(|error| AppError::Config(format!("could not serialize output: {error}")))
}

pub async fn headless_import(
    config: &AppConfig,
    paths: &StoragePaths,
    source: &Path,
    provenance: &service::provenance::EvidenceProvenance,
) -> AppResult<String> {
    let services = build_services(config, paths).await?;
    let evidence = service::import_evidence_with_provenance(
        &services,
        source,
        config.max_upload_bytes as u64,
        provenance,
    )
    .await?;
    to_json(&evidence)
}

/// Gera os entregáveis de um incidente. Sem `out`, usa a pasta de
/// exportações da aplicação.
pub async fn headless_deliverables(
    config: &AppConfig,
    paths: &StoragePaths,
    incident_id: &str,
    out: Option<&Path>,
    options: service::deliverables::RenderOptions,
) -> AppResult<String> {
    let services = build_services(config, paths).await?;
    let out_dir = out.unwrap_or(paths.exports.as_path());
    let generated =
        service::deliverables::generate(&services, incident_id, out_dir, options).await?;
    to_json(&generated)
}

/// Incidente de demonstração (F1-11): importa o `.evtx`, executa as análises
/// reais do engine e registra o cenário BEC sintético num incidente novo.
pub async fn headless_demo(
    config: &AppConfig,
    paths: &StoragePaths,
    evtx: &Path,
) -> AppResult<String> {
    let services = build_services(config, paths).await?;
    let report = service::demo::seed(&services, evtx, config.max_upload_bytes as u64, true).await?;
    to_json(&report)
}

pub async fn headless_evidence_list(config: &AppConfig, paths: &StoragePaths) -> AppResult<String> {
    let services = build_services(config, paths).await?;
    to_json(&service::list_evidence(&services).await?)
}

/// Vincula uma evidência a um incidente (F1-19).
pub async fn headless_link_evidence(
    config: &AppConfig,
    paths: &StoragePaths,
    evidence_id: &str,
    incident_id: &str,
) -> AppResult<String> {
    let services = build_services(config, paths).await?;
    to_json(&service::link_evidence_to_incident(&services, evidence_id, incident_id).await?)
}

/// Lista as evidências vinculadas a um incidente (F1-19).
pub async fn headless_incident_evidence(
    config: &AppConfig,
    paths: &StoragePaths,
    incident_id: &str,
) -> AppResult<String> {
    let services = build_services(config, paths).await?;
    to_json(&service::evidence_for_incident(&services, incident_id).await?)
}

/// Cadeia de custódia de uma evidência (F1-24).
pub async fn headless_custody(
    config: &AppConfig,
    paths: &StoragePaths,
    evidence_id: &str,
) -> AppResult<String> {
    let services = build_services(config, paths).await?;
    sqlx::query_scalar::<_, String>("SELECT id FROM evidence WHERE id=?")
        .bind(evidence_id)
        .fetch_optional(&services.db)
        .await?
        .ok_or_else(|| AppError::NotFound("evidence not found".into()))?;
    to_json(&audit::list_custody(&services.db, evidence_id).await?)
}

/// Eventos de auditoria (F1-24), opcionalmente filtrados por incidente.
pub async fn headless_audit(
    config: &AppConfig,
    paths: &StoragePaths,
    incident_id: Option<&str>,
    limit: u64,
) -> AppResult<String> {
    let services = build_services(config, paths).await?;
    to_json(&audit::list_events(&services.db, incident_id, limit).await?)
}

pub async fn headless_analyze(
    config: &AppConfig,
    paths: &StoragePaths,
    evidence_id: &str,
    kind: &str,
    max_output_bytes: Option<u64>,
    max_items: Option<u64>,
) -> AppResult<String> {
    let services = build_services(config, paths).await?;
    let response =
        service::analyze(&services, evidence_id, kind, max_output_bytes, max_items).await?;
    to_json(&response)
}

/// Roda as regras de detecção e grava os candidatos pendentes.
pub async fn headless_detect(
    config: &AppConfig,
    paths: &StoragePaths,
    evidence_id: &str,
) -> AppResult<String> {
    let services = build_services(config, paths).await?;
    to_json(&service::detection::run_detection(&services, evidence_id).await?)
}

pub async fn headless_candidates(
    config: &AppConfig,
    paths: &StoragePaths,
    evidence_id: &str,
    status: Option<&str>,
) -> AppResult<String> {
    let services = build_services(config, paths).await?;
    to_json(&service::detection::list_candidates(&services, evidence_id, status).await?)
}

pub async fn headless_candidate_confirm(
    config: &AppConfig,
    paths: &StoragePaths,
    candidate_id: &str,
    incident_id: &str,
    note: Option<&str>,
) -> AppResult<String> {
    let services = build_services(config, paths).await?;
    to_json(
        &service::detection::confirm_candidate(&services, candidate_id, incident_id, note).await?,
    )
}

pub async fn headless_candidate_reject(
    config: &AppConfig,
    paths: &StoragePaths,
    candidate_id: &str,
    reason: &str,
) -> AppResult<String> {
    let services = build_services(config, paths).await?;
    service::detection::reject_candidate(&services, candidate_id, reason).await?;
    to_json(&serde_json::json!({ "candidate_id": candidate_id, "status": "REJECTED" }))
}

pub async fn headless_results(
    config: &AppConfig,
    paths: &StoragePaths,
    job_id: &str,
    offset: u64,
    limit: u64,
    filter: Option<&str>,
) -> AppResult<String> {
    let services = build_services(config, paths).await?;
    to_json(&service::timeline_page(&services, job_id, offset, limit, filter, true).await?)
}

pub async fn headless_findings(
    config: &AppConfig,
    paths: &StoragePaths,
    evidence_id: &str,
) -> AppResult<String> {
    let services = build_services(config, paths).await?;
    to_json(&service::evidence_analytical_results(&services, evidence_id).await?)
}

/// Descarte local explícito. Como na GUI, exige o nome EXATO do arquivo.
pub async fn headless_destroy(
    config: &AppConfig,
    paths: &StoragePaths,
    evidence_id: &str,
    confirm_filename: &str,
) -> AppResult<String> {
    let services = build_services(config, paths).await?;
    let evidence = service::list_evidence(&services)
        .await?
        .into_iter()
        .find(|item| item.id == evidence_id)
        .ok_or_else(|| AppError::NotFound("evidence not found".into()))?;
    if evidence.original_filename != confirm_filename {
        return Err(AppError::Config(
            "--confirm-filename must be the exact original filename of the evidence".into(),
        ));
    }
    service::destroy_evidence(&services, evidence_id).await?;
    to_json(&serde_json::json!({ "evidence_id": evidence_id, "status": "DESTROYED" }))
}

/// Relatório de diagnóstico em JSON (ADR-0022). Sem nomes de arquivo nem
/// mensagens de erro: pode ser anexado a um pedido de suporte.
pub async fn headless_diagnostics(config: &AppConfig, paths: &StoragePaths) -> AppResult<String> {
    let services = build_services(config, paths).await?;
    let settings = serde_json::json!({
        "data_root": config.data_root,
        "engine_present": config.engine_path.is_file(),
        "max_upload_bytes": config.max_upload_bytes,
        "min_free_disk_bytes": config.min_free_disk_bytes,
        "engine_max_memory_bytes": config.engine_max_memory_bytes,
    });
    to_json(&service::diagnostics_report(&services.db, paths, settings).await?)
}

pub async fn engine_smoke(config: &AppConfig, paths: &StoragePaths) -> AppResult<String> {
    let input_dir = paths.logical_path("staging/engine-smoke/input")?;
    let output_dir = paths.logical_path("staging/engine-smoke/output")?;
    std::fs::create_dir_all(&input_dir)?;
    std::fs::create_dir_all(&output_dir)?;
    let request = engine::new_request(
        "probe.evtx".into(),
        65_536,
        100,
        dfir_protocol::ANALYSIS_KIND_EVTX_TIMELINE,
    );
    let runner = engine::EngineRunnerConfig::from_config(config);
    let messages = engine::run_process_smoke(request, &input_dir, &output_dir, &runner)
        .await
        .map_err(|error| AppError::Engine(error.to_string()))?;
    let marker = output_dir.join("process-smoke-ok.txt");
    if !marker.is_file() {
        return Err(AppError::Engine(
            "process smoke did not produce the expected output marker".into(),
        ));
    }
    render_output(CommandOutput::EngineSmoke {
        messages: messages.message_count,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        CommandOutput, build_services, headless_deliverables, headless_import, render_output,
    };
    use crate::service::deliverables::RenderOptions;
    use crate::{config::AppConfig, infrastructure::storage::StoragePaths, service};
    use std::{path::PathBuf, time::Duration};

    fn test_config(root: &std::path::Path) -> AppConfig {
        AppConfig {
            data_root: root.to_path_buf(),
            engine_path: PathBuf::from("dfir-engine"),
            engine_max_line_bytes: 1024 * 1024,
            engine_timeout: Duration::from_secs(5),
            engine_max_concurrent_jobs: 1,
            engine_max_memory_bytes: 0,
            max_upload_bytes: 1024 * 1024,
            min_free_disk_bytes: 0,
            engine_sandbox: None,
        }
    }

    #[tokio::test]
    async fn import_subcommand_records_the_declared_provenance_and_only_that() {
        let root = tempfile::tempdir().expect("temporary directory");
        let paths = StoragePaths::from_root(root.path());
        paths.ensure().expect("storage layout");
        let config = test_config(root.path());
        let input = tempfile::tempdir().expect("input directory");
        let file = input.path().join("Security.evtx");
        std::fs::write(&file, b"conteudo").expect("write sample");

        // Sem flags: nada é inventado.
        let json = headless_import(
            &config,
            &paths,
            &file,
            &service::provenance::EvidenceProvenance::default(),
        )
        .await
        .expect("import without provenance");
        let value: serde_json::Value = serde_json::from_str(&json).expect("json");
        assert_eq!(value["source"], "NOT_PROVIDED");
        assert!(value["collected_at"].is_null());
        assert_eq!(value["contains_personal_data"], false);

        // Com flags: o que foi declarado.
        let json = headless_import(
            &config,
            &paths,
            &file,
            &service::provenance::EvidenceProvenance {
                source: Some("servidor FS02".into()),
                provided_by: Some("TI do cliente".into()),
                collected_at: Some("2026-09-30".into()),
                authorization_reference: Some("Ofício 123/2026".into()),
                contains_personal_data: true,
                ..Default::default()
            },
        )
        .await
        .expect("import with provenance");
        let value: serde_json::Value = serde_json::from_str(&json).expect("json");
        assert_eq!(value["source"], "servidor FS02");
        assert_eq!(value["provided_by"], "TI do cliente");
        assert_eq!(value["collected_at"], "2026-09-30");
        assert_eq!(value["contains_personal_data"], true);

        // Data inválida falha em vez de gravar lixo.
        assert!(
            headless_import(
                &config,
                &paths,
                &file,
                &service::provenance::EvidenceProvenance {
                    collected_at: Some("ontem".into()),
                    ..Default::default()
                },
            )
            .await
            .is_err()
        );
    }

    #[tokio::test]
    async fn deliverables_subcommand_writes_the_files_and_defaults_to_the_exports_folder() {
        let root = tempfile::tempdir().expect("temporary directory");
        let paths = StoragePaths::from_root(root.path());
        paths.ensure().expect("storage layout");
        let config = test_config(root.path());
        let services = build_services(&config, &paths).await.expect("services");
        let incident = service::create_incident(&services, "Incidente CLI")
            .await
            .expect("incident");

        let out = tempfile::tempdir().expect("output directory");
        let json = headless_deliverables(
            &config,
            &paths,
            &incident.id,
            Some(out.path()),
            RenderOptions::default(),
        )
        .await
        .expect("deliverables");
        let value: serde_json::Value = serde_json::from_str(&json).expect("json output");
        assert_eq!(value["files"].as_array().map(Vec::len), Some(6));
        let dir = PathBuf::from(value["output_dir"].as_str().expect("output_dir"));
        assert!(dir.join("01-relatorio-forense.md").is_file());
        assert!(dir.join("manifest.json").is_file());

        // Sem --out: a pasta de exportações da aplicação.
        let json = headless_deliverables(
            &config,
            &paths,
            &incident.id,
            None,
            RenderOptions::default(),
        )
        .await
        .expect("deliverables in exports");
        let value: serde_json::Value = serde_json::from_str(&json).expect("json output");
        let dir = PathBuf::from(value["output_dir"].as_str().expect("output_dir"));
        assert_eq!(
            dir.parent()
                .map(|parent| parent.canonicalize().expect("canonical")),
            Some(paths.exports.canonicalize().expect("canonical"))
        );

        // Incidente inexistente falha em vez de gerar documentos vazios.
        assert!(
            headless_deliverables(
                &config,
                &paths,
                "nao-existe",
                Some(out.path()),
                RenderOptions::default(),
            )
            .await
            .is_err()
        );
    }

    #[test]
    fn engine_smoke_output_reports_the_message_count() {
        let output = render_output(CommandOutput::EngineSmoke { messages: 3 }).expect("renders");
        assert_eq!(output, "Engine sandbox smoke: 3 mensagens recebidas.\n");
    }
}
