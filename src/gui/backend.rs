//! Ponte entre a thread da interface (egui, síncrona, um `update()` por
//! quadro) e a camada de serviço (assíncrona, `src/service/`). Uma thread
//! dedicada roda um runtime Tokio e processa [`Command`]s recebidos por um
//! canal; cada resultado volta como um [`Event`] por outro canal, que a UI
//! drena a cada quadro sem nunca bloquear (`try_recv`).
//!
//! Antes desta migração (v6.4.5), este papel era do próprio Axum: cada
//! requisição HTTP já era, por natureza, uma chamada assíncrona isolada. A
//! interface nativa não tem esse ponto de entrada por requisição — daí a
//! fila explícita. Ver `docs/adrs/ADR-0017-interface-nativa-eframe-egui.md`.

use crate::{
    application::engine::EngineRunnerConfig,
    config::AppConfig,
    error::AppError,
    infrastructure::storage::StoragePaths,
    persistence::{db, nist_profiles::ProfileType},
    service::{
        self, AnalyticalResultResponse, AnalyzeResponse, AppServices, EvidenceResponse,
        IncidentResponse, TimelinePage, deliverables, detection, findings, gaps, nist, plan,
        provenance,
    },
};
use std::path::PathBuf;

#[derive(Debug)]
pub enum Command {
    ListEvidence,
    ImportEvidence {
        path: PathBuf,
        max_upload_bytes: u64,
        provenance: provenance::EvidenceProvenance,
    },
    DestroyEvidence(String),
    Analyze {
        evidence_id: String,
        kind: String,
    },
    /// Uma página da timeline. `job_id = None` pede o último job EVTX
    /// concluído da evidência.
    TimelinePage {
        evidence_id: String,
        job_id: Option<String>,
        filter: String,
        offset: u64,
        limit: u64,
        want_total: bool,
    },
    EvidenceAnalyticalResults(String),

    /// Vincula a evidência a um incidente (F1-19).
    LinkEvidenceToIncident {
        evidence_id: String,
        incident_id: String,
    },
    /// Cadeia de custódia da evidência (F1-24).
    LoadCustody(String),
    /// Eventos de auditoria do incidente (F1-24).
    LoadAudit {
        incident_id: String,
        limit: u64,
    },
    /// Evidências vinculadas ao incidente (F1-19).
    LoadIncidentEvidence(String),

    /// Roda as regras de detecção (F1-28) e grava candidatos pendentes.
    RunDetection(String),
    LoadCandidates(String),
    ConfirmCandidate {
        evidence_id: String,
        candidate_id: String,
        incident_id: String,
        note: Option<String>,
    },
    RejectCandidate {
        evidence_id: String,
        candidate_id: String,
        reason: String,
    },

    CreateIncident(String),
    ListIncidents,

    NistProfileHistory {
        incident_id: String,
        profile_type: ProfileType,
    },
    CreateNistProfile {
        incident_id: String,
        profile_type: String,
        version_label: String,
        source: String,
        tier: Option<String>,
        tier_justification: Option<String>,
        entries: Vec<nist::ProfileEntryInput>,
    },
    RunGapAnalysis(String),
    CurrentGapAnalysis(String),

    #[allow(clippy::type_complexity)]
    CreateAttackMapping {
        incident_id: String,
        evidence_id: Option<String>,
        tactic: String,
        technique_id: String,
        sub_technique_id: Option<String>,
        basis: String,
        confidence: String,
        analyst_note: Option<String>,
    },
    ListAttackMappings(String),

    CreateCriticalPathStage {
        incident_id: String,
        evidence_id: Option<String>,
        sequence_index: i64,
        stage_label: String,
        basis: String,
        confidence: String,
    },
    LinkCriticalPathStages {
        incident_id: String,
        from_stage_id: String,
        to_stage_id: String,
    },
    CriticalPath(String),

    CreateImpactAssessment {
        incident_id: String,
        availability_impact: String,
        confidentiality_impact: String,
        integrity_impact: String,
        affected_assets: String,
        affected_data: String,
        operational_disruption: String,
        financial_impact: Option<String>,
        confidence: String,
    },
    CurrentImpactAssessment(String),

    CreateIoc {
        incident_id: String,
        evidence_id: String,
        ioc_type: String,
        value: String,
        event_context: Option<String>,
        confidence: String,
        observed_at: Option<String>,
    },
    ListIocs(String),

    CreateRecommendation {
        incident_id: String,
        category: String,
        title: String,
        description: String,
        confidence: Option<String>,
        supporting_result_ids: Vec<String>,
        supersedes: Option<String>,
    },
    ListRecommendations {
        incident_id: String,
        category: Option<String>,
    },

    CreateFinding {
        incident_id: String,
        input: findings::FindingInput,
    },
    CreateRootCause {
        incident_id: String,
        input: findings::RootCauseInput,
    },
    LoadFindingsView(String),
    LoadNistFunctionView(String),
    CreatePlanItem {
        incident_id: String,
        input: plan::PlanItemInput,
    },
    LoadPlanView(String),
    CreateSourceGap {
        incident_id: String,
        input: gaps::SourceGapInput,
    },
    ListSourceGaps(String),
    PreviewDeliverables {
        incident_id: String,
        mask_personal_data: bool,
    },
    GenerateDeliverables {
        incident_id: String,
        out_dir: PathBuf,
        mask_personal_data: bool,
    },

    ExportFindings {
        incident_id: String,
        save_path: PathBuf,
    },
}

#[derive(Debug)]
pub enum Event {
    EvidenceListed(Result<Vec<EvidenceResponse>, String>),
    EvidenceImported(Result<EvidenceResponse, String>),
    EvidenceDestroyed {
        evidence_id: String,
        result: Result<(), String>,
    },
    AnalyzeCompleted {
        evidence_id: String,
        result: Result<AnalyzeResponse, String>,
    },
    TimelinePageLoaded {
        evidence_id: String,
        filter: String,
        offset: u64,
        result: Result<Option<TimelinePage>, String>,
    },
    EvidenceAnalyticalResultsLoaded {
        evidence_id: String,
        result: Result<Vec<AnalyticalResultResponse>, String>,
    },
    EvidenceLinkedToIncident {
        evidence_id: String,
        result: Result<EvidenceResponse, String>,
    },
    CustodyLoaded {
        evidence_id: String,
        result: Result<Vec<crate::persistence::audit::CustodyEntry>, String>,
    },
    AuditLoaded {
        incident_id: String,
        result: Result<Vec<crate::persistence::audit::AuditEventView>, String>,
    },
    IncidentEvidenceLoaded {
        incident_id: String,
        result: Result<Vec<EvidenceResponse>, String>,
    },
    DetectionRun {
        evidence_id: String,
        result: Result<detection::DetectionRun, String>,
    },
    CandidatesLoaded {
        evidence_id: String,
        result: Result<Vec<detection::CandidateView>, String>,
    },
    /// Resultado de confirmar ou rejeitar (texto curto para a tela).
    CandidateDecided {
        evidence_id: String,
        result: Result<String, String>,
    },

    IncidentCreated(Result<IncidentResponse, String>),
    IncidentsListed(Result<Vec<IncidentResponse>, String>),

    NistProfileHistoryLoaded {
        incident_id: String,
        result: Result<nist::ProfileHistoryView, String>,
    },
    NistProfileCreated {
        incident_id: String,
        result: Result<String, String>,
    },
    GapAnalysisRun {
        incident_id: String,
        result: Result<Vec<String>, String>,
    },
    GapAnalysisLoaded {
        incident_id: String,
        result: Result<Vec<AnalyticalResultResponse>, String>,
    },

    FindingCreated {
        incident_id: String,
        result: Result<String, String>,
    },
    RootCauseCreated {
        incident_id: String,
        result: Result<String, String>,
    },
    FindingsViewLoaded {
        incident_id: String,
        result: Result<findings::FindingsView, String>,
    },
    NistFunctionViewLoaded {
        incident_id: String,
        result: Result<nist::NistFunctionView, String>,
    },
    SourceGapCreated {
        incident_id: String,
        result: Result<String, String>,
    },
    SourceGapsLoaded {
        incident_id: String,
        result: Result<Vec<AnalyticalResultResponse>, String>,
    },
    DeliverablesPreviewed {
        incident_id: String,
        result: Result<deliverables::DeliverableSet, String>,
    },
    DeliverablesGenerated {
        incident_id: String,
        result: Result<deliverables::GeneratedDeliverables, String>,
    },
    PlanItemCreated {
        incident_id: String,
        result: Result<String, String>,
    },
    PlanViewLoaded {
        incident_id: String,
        result: Result<plan::PlanView, String>,
    },

    AttackMappingCreated {
        incident_id: String,
        result: Result<String, String>,
    },
    AttackMappingsLoaded {
        incident_id: String,
        result: Result<Vec<AnalyticalResultResponse>, String>,
    },

    CriticalPathStageCreated {
        incident_id: String,
        result: Result<String, String>,
    },
    CriticalPathStagesLinked {
        incident_id: String,
        result: Result<(), String>,
    },
    CriticalPathLoaded {
        incident_id: String,
        result: Result<findings::CriticalPathView, String>,
    },

    ImpactAssessmentCreated {
        incident_id: String,
        result: Result<String, String>,
    },
    ImpactAssessmentLoaded {
        incident_id: String,
        result: Result<Option<AnalyticalResultResponse>, String>,
    },

    IocCreated {
        incident_id: String,
        result: Result<String, String>,
    },
    IocsLoaded {
        incident_id: String,
        result: Result<Vec<AnalyticalResultResponse>, String>,
    },

    RecommendationCreated {
        incident_id: String,
        result: Result<String, String>,
    },
    RecommendationsLoaded {
        incident_id: String,
        result: Result<Vec<AnalyticalResultResponse>, String>,
    },

    FindingsExported {
        incident_id: String,
        result: Result<PathBuf, String>,
    },
}

fn msg(error: AppError) -> String {
    error.user_message()
}

pub struct Backend {
    command_tx: tokio::sync::mpsc::UnboundedSender<Command>,
    event_rx: std::sync::mpsc::Receiver<Event>,
    max_upload_bytes: u64,
    _thread: std::thread::JoinHandle<()>,
}

impl Backend {
    /// Backend de mentira para testes de GUI: os comandos enviados chegam ao
    /// receptor devolvido, e nenhum evento é produzido.
    #[cfg(test)]
    pub fn for_test() -> (Self, tokio::sync::mpsc::UnboundedReceiver<Command>) {
        let (command_tx, command_rx) = tokio::sync::mpsc::unbounded_channel();
        let (_event_tx, event_rx) = std::sync::mpsc::channel();
        let backend = Self {
            command_tx,
            event_rx,
            max_upload_bytes: 0,
            _thread: std::thread::spawn(|| {}),
        };
        (backend, command_rx)
    }

    pub fn send(&self, command: Command) {
        // Um envio pode falhar apenas se a thread de backend já saiu (painic
        // durante a inicialização); nesse caso o próprio evento de erro
        // inicial já foi reportado por `spawn` (F1-23: registrar em vez de
        // engolir).
        if self.command_tx.send(command).is_err() {
            tracing::warn!("backend thread is gone; command dropped");
        }
    }

    pub fn max_upload_bytes(&self) -> u64 {
        self.max_upload_bytes
    }

    /// Drena todos os eventos pendentes sem bloquear — chamado uma vez por
    /// quadro em `App::update`.
    pub fn drain_events(&self) -> Vec<Event> {
        let mut events = Vec::new();
        while let Ok(event) = self.event_rx.try_recv() {
            events.push(event);
        }
        events
    }
}

pub fn spawn(config: AppConfig, paths: StoragePaths) -> Result<Backend, AppError> {
    let (command_tx, mut command_rx) = tokio::sync::mpsc::unbounded_channel::<Command>();
    let (event_tx, event_rx) = std::sync::mpsc::channel::<Event>();
    let (ready_tx, ready_rx) = std::sync::mpsc::channel::<Result<(), AppError>>();
    let max_upload_bytes = config.max_upload_bytes as u64;

    let thread = std::thread::Builder::new()
        .name("dfir-backend".into())
        .spawn(move || {
            let runtime = match tokio::runtime::Builder::new_multi_thread()
                .worker_threads(4)
                .thread_stack_size(8 * 1024 * 1024)
                .enable_all()
                .build()
            {
                Ok(runtime) => runtime,
                Err(error) => {
                    // F1-23: o erro original é devolvido no evento de
                    // inicialização; aqui registramos que o canal foi fechado.
                    if ready_tx.send(Err(AppError::Storage(error))).is_err() {
                        tracing::warn!("startup error not delivered: UI channel closed");
                    }
                    return;
                }
            };
            runtime.block_on(async move {
                let db_url = format!("sqlite://{}", paths.db.join("dfir.sqlite").display());
                let pool = match db::connect(&db_url).await {
                    Ok(pool) => pool,
                    Err(error) => {
                        if ready_tx.send(Err(AppError::Database(error))).is_err() {
                            tracing::warn!("startup error not delivered: UI channel closed");
                        }
                        return;
                    }
                };
                // RF-032: copia e verifica o banco antes de migrar; se a cópia
                // falhar, a migration não é aplicada e a janela mostra o erro.
                if let Err(error) =
                    db::migrate_with_backup(&pool, &paths.backups, config.min_free_disk_bytes).await
                {
                    if ready_tx.send(Err(AppError::Database(error))).is_err() {
                        tracing::warn!("startup error not delivered: UI channel closed");
                    }
                    return;
                }
                let services = AppServices {
                    db: pool,
                    paths,
                    engine: EngineRunnerConfig::from_config(&config),
                };
                // Jobs deixados `PROCESSING` por uma execução anterior que
                // fechou/morreu no meio da análise (F1-10). Só na partida.
                match crate::service::recover_orphaned_jobs(&services.db).await {
                    Ok(0) => {}
                    Ok(count) => tracing::warn!(count, "recovered orphaned analysis jobs"),
                    Err(error) => {
                        tracing::error!(%error, "could not recover orphaned analysis jobs")
                    }
                }
                if ready_tx.send(Ok(())).is_err() {
                    return;
                }
                while let Some(command) = command_rx.recv().await {
                    let services = services.clone();
                    let event_tx = event_tx.clone();
                    tokio::spawn(async move {
                        let event = handle_command(&services, command).await;
                        // F1-23: se o canal de eventos fechou (UI encerrando),
                        // registrar em vez de engolir.
                        if event_tx.send(event).is_err() {
                            tracing::debug!("event dropped: UI channel closed");
                        }
                    });
                }
            });
        })
        .map_err(AppError::Storage)?;

    match ready_rx.recv() {
        Ok(Ok(())) => {}
        Ok(Err(error)) => return Err(error),
        Err(_) => {
            return Err(AppError::Config(
                "backend thread exited before startup completed".into(),
            ));
        }
    }

    Ok(Backend {
        command_tx,
        event_rx,
        max_upload_bytes,
        _thread: thread,
    })
}

async fn handle_command(services: &AppServices, command: Command) -> Event {
    match command {
        Command::ListEvidence => {
            Event::EvidenceListed(service::list_evidence(services).await.map_err(msg))
        }
        Command::ImportEvidence {
            path,
            max_upload_bytes,
            provenance,
        } => Event::EvidenceImported(
            service::import_evidence_with_provenance(
                services,
                &path,
                max_upload_bytes,
                &provenance,
            )
            .await
            .map_err(msg),
        ),
        Command::DestroyEvidence(evidence_id) => Event::EvidenceDestroyed {
            result: service::destroy_evidence(services, &evidence_id)
                .await
                .map_err(msg),
            evidence_id,
        },
        Command::RunDetection(evidence_id) => {
            let result = detection::run_detection(services, &evidence_id)
                .await
                .map_err(msg);
            Event::DetectionRun {
                evidence_id,
                result,
            }
        }
        Command::LoadCandidates(evidence_id) => {
            let result = detection::list_candidates(services, &evidence_id, None)
                .await
                .map_err(msg);
            Event::CandidatesLoaded {
                evidence_id,
                result,
            }
        }
        Command::ConfirmCandidate {
            evidence_id,
            candidate_id,
            incident_id,
            note,
        } => {
            let result = detection::confirm_candidate(
                services,
                &candidate_id,
                &incident_id,
                note.as_deref(),
            )
            .await
            .map(|outcome| {
                format!(
                    "Candidato confirmado: mapeamento ATT&CK, achado e {} IOC(s) criados no incidente.",
                    outcome.ioc_ids.len()
                )
            })
            .map_err(msg);
            Event::CandidateDecided {
                evidence_id,
                result,
            }
        }
        Command::RejectCandidate {
            evidence_id,
            candidate_id,
            reason,
        } => {
            let result = detection::reject_candidate(services, &candidate_id, &reason)
                .await
                .map(|()| "Candidato rejeitado; o motivo ficou registrado.".to_string())
                .map_err(msg);
            Event::CandidateDecided {
                evidence_id,
                result,
            }
        }
        Command::Analyze { evidence_id, kind } => {
            let result = service::analyze(services, &evidence_id, &kind, None, None)
                .await
                .map_err(msg);
            Event::AnalyzeCompleted {
                evidence_id,
                result,
            }
        }
        Command::TimelinePage {
            evidence_id,
            job_id,
            filter,
            offset,
            limit,
            want_total,
        } => {
            let result = async {
                let job_id = match job_id {
                    Some(job_id) => Some(job_id),
                    None => service::latest_timeline_job(services, &evidence_id).await?,
                };
                match job_id {
                    Some(job_id) => service::timeline_page(
                        services,
                        &job_id,
                        offset,
                        limit,
                        Some(&filter),
                        want_total,
                    )
                    .await
                    .map(Some),
                    None => Ok(None),
                }
            }
            .await
            .map_err(msg);
            Event::TimelinePageLoaded {
                evidence_id,
                filter,
                offset,
                result,
            }
        }
        Command::EvidenceAnalyticalResults(evidence_id) => {
            let result = service::evidence_analytical_results(services, &evidence_id)
                .await
                .map_err(msg);
            Event::EvidenceAnalyticalResultsLoaded {
                evidence_id,
                result,
            }
        }
        Command::LinkEvidenceToIncident {
            evidence_id,
            incident_id,
        } => {
            let result = service::link_evidence_to_incident(services, &evidence_id, &incident_id)
                .await
                .map_err(msg);
            Event::EvidenceLinkedToIncident {
                evidence_id,
                result,
            }
        }
        Command::LoadCustody(evidence_id) => {
            let result = crate::persistence::audit::list_custody(&services.db, &evidence_id)
                .await
                .map_err(|error| format!("custódia indisponível: {error}"));
            Event::CustodyLoaded {
                evidence_id,
                result,
            }
        }
        Command::LoadAudit { incident_id, limit } => {
            let result =
                crate::persistence::audit::list_events(&services.db, Some(&incident_id), limit)
                    .await
                    .map_err(|error| format!("auditoria indisponível: {error}"));
            Event::AuditLoaded {
                incident_id,
                result,
            }
        }
        Command::LoadIncidentEvidence(incident_id) => {
            let result = service::evidence_for_incident(services, &incident_id)
                .await
                .map_err(msg);
            Event::IncidentEvidenceLoaded {
                incident_id,
                result,
            }
        }
        Command::CreateIncident(name) => {
            Event::IncidentCreated(service::create_incident(services, &name).await.map_err(msg))
        }
        Command::ListIncidents => {
            Event::IncidentsListed(service::list_incidents(services).await.map_err(msg))
        }
        Command::NistProfileHistory {
            incident_id,
            profile_type,
        } => {
            let result = nist::profile_history(services, &incident_id, profile_type)
                .await
                .map_err(msg);
            Event::NistProfileHistoryLoaded {
                incident_id,
                result,
            }
        }
        Command::CreateNistProfile {
            incident_id,
            profile_type,
            version_label,
            source,
            tier,
            tier_justification,
            entries,
        } => {
            let result = nist::create_profile(
                services,
                &incident_id,
                &profile_type,
                &version_label,
                &source,
                tier.as_deref(),
                tier_justification.as_deref(),
                &entries,
            )
            .await
            .map_err(msg);
            Event::NistProfileCreated {
                incident_id,
                result,
            }
        }
        Command::RunGapAnalysis(incident_id) => {
            let result = nist::run_gap_analysis(services, &incident_id)
                .await
                .map_err(msg);
            Event::GapAnalysisRun {
                incident_id,
                result,
            }
        }
        Command::CurrentGapAnalysis(incident_id) => {
            let result = nist::current_gap_analysis(services, &incident_id)
                .await
                .map_err(msg);
            Event::GapAnalysisLoaded {
                incident_id,
                result,
            }
        }
        Command::CreateAttackMapping {
            incident_id,
            evidence_id,
            tactic,
            technique_id,
            sub_technique_id,
            basis,
            confidence,
            analyst_note,
        } => {
            let result = findings::create_attack_mapping(
                services,
                &incident_id,
                evidence_id.as_deref(),
                &tactic,
                &technique_id,
                sub_technique_id.as_deref(),
                &basis,
                &confidence,
                analyst_note.as_deref(),
            )
            .await
            .map_err(msg);
            Event::AttackMappingCreated {
                incident_id,
                result,
            }
        }
        Command::CreateFinding { incident_id, input } => {
            let result = findings::create_finding(services, &incident_id, &input)
                .await
                .map_err(msg);
            Event::FindingCreated {
                incident_id,
                result,
            }
        }
        Command::CreateRootCause { incident_id, input } => {
            let result = findings::create_root_cause(services, &incident_id, &input)
                .await
                .map_err(msg);
            Event::RootCauseCreated {
                incident_id,
                result,
            }
        }
        Command::LoadFindingsView(incident_id) => {
            let result = findings::findings_view(services, &incident_id)
                .await
                .map_err(msg);
            Event::FindingsViewLoaded {
                incident_id,
                result,
            }
        }
        Command::LoadNistFunctionView(incident_id) => {
            let result = nist::function_view(services, &incident_id)
                .await
                .map_err(msg);
            Event::NistFunctionViewLoaded {
                incident_id,
                result,
            }
        }
        Command::CreateSourceGap { incident_id, input } => {
            let result = gaps::create_source_gap(services, &incident_id, &input)
                .await
                .map_err(msg);
            Event::SourceGapCreated {
                incident_id,
                result,
            }
        }
        Command::ListSourceGaps(incident_id) => {
            let result = gaps::list_source_gaps(services, &incident_id)
                .await
                .map_err(msg);
            Event::SourceGapsLoaded {
                incident_id,
                result,
            }
        }
        Command::PreviewDeliverables {
            incident_id,
            mask_personal_data,
        } => {
            let options = deliverables::RenderOptions { mask_personal_data };
            let result = deliverables::preview(services, &incident_id, options)
                .await
                .map_err(msg);
            Event::DeliverablesPreviewed {
                incident_id,
                result,
            }
        }
        Command::GenerateDeliverables {
            incident_id,
            out_dir,
            mask_personal_data,
        } => {
            let options = deliverables::RenderOptions { mask_personal_data };
            let result = deliverables::generate(services, &incident_id, &out_dir, options)
                .await
                .map_err(msg);
            Event::DeliverablesGenerated {
                incident_id,
                result,
            }
        }
        Command::CreatePlanItem { incident_id, input } => {
            let result = plan::create_plan_item(services, &incident_id, &input)
                .await
                .map_err(msg);
            Event::PlanItemCreated {
                incident_id,
                result,
            }
        }
        Command::LoadPlanView(incident_id) => {
            let result = plan::plan_view(services, &incident_id).await.map_err(msg);
            Event::PlanViewLoaded {
                incident_id,
                result,
            }
        }
        Command::ListAttackMappings(incident_id) => {
            let result = findings::list_attack_mappings(services, &incident_id)
                .await
                .map_err(msg);
            Event::AttackMappingsLoaded {
                incident_id,
                result,
            }
        }
        Command::CreateCriticalPathStage {
            incident_id,
            evidence_id,
            sequence_index,
            stage_label,
            basis,
            confidence,
        } => {
            let result = findings::create_critical_path_stage(
                services,
                &incident_id,
                evidence_id.as_deref(),
                sequence_index,
                &stage_label,
                &basis,
                &confidence,
            )
            .await
            .map_err(msg);
            Event::CriticalPathStageCreated {
                incident_id,
                result,
            }
        }
        Command::LinkCriticalPathStages {
            incident_id,
            from_stage_id,
            to_stage_id,
        } => {
            let result = findings::link_critical_path_stages(
                services,
                &incident_id,
                &from_stage_id,
                &to_stage_id,
            )
            .await
            .map_err(msg);
            Event::CriticalPathStagesLinked {
                incident_id,
                result,
            }
        }
        Command::CriticalPath(incident_id) => {
            let result = findings::critical_path(services, &incident_id)
                .await
                .map_err(msg);
            Event::CriticalPathLoaded {
                incident_id,
                result,
            }
        }
        Command::CreateImpactAssessment {
            incident_id,
            availability_impact,
            confidentiality_impact,
            integrity_impact,
            affected_assets,
            affected_data,
            operational_disruption,
            financial_impact,
            confidence,
        } => {
            let result = findings::create_impact_assessment(
                services,
                &incident_id,
                &availability_impact,
                &confidentiality_impact,
                &integrity_impact,
                &affected_assets,
                &affected_data,
                &operational_disruption,
                financial_impact.as_deref(),
                &confidence,
            )
            .await
            .map_err(msg);
            Event::ImpactAssessmentCreated {
                incident_id,
                result,
            }
        }
        Command::CurrentImpactAssessment(incident_id) => {
            let result = findings::current_impact_assessment(services, &incident_id)
                .await
                .map_err(msg);
            Event::ImpactAssessmentLoaded {
                incident_id,
                result,
            }
        }
        Command::CreateIoc {
            incident_id,
            evidence_id,
            ioc_type,
            value,
            event_context,
            confidence,
            observed_at,
        } => {
            let result = findings::create_ioc(
                services,
                &incident_id,
                &evidence_id,
                &ioc_type,
                &value,
                event_context.as_deref(),
                &confidence,
                observed_at.as_deref(),
            )
            .await
            .map_err(msg);
            Event::IocCreated {
                incident_id,
                result,
            }
        }
        Command::ListIocs(incident_id) => {
            let result = findings::list_iocs(services, &incident_id)
                .await
                .map_err(msg);
            Event::IocsLoaded {
                incident_id,
                result,
            }
        }
        Command::CreateRecommendation {
            incident_id,
            category,
            title,
            description,
            confidence,
            supporting_result_ids,
            supersedes,
        } => {
            let result = findings::create_recommendation(
                services,
                &incident_id,
                &category,
                &title,
                &description,
                confidence.as_deref(),
                &supporting_result_ids,
                supersedes.as_deref(),
            )
            .await
            .map_err(msg);
            Event::RecommendationCreated {
                incident_id,
                result,
            }
        }
        Command::ListRecommendations {
            incident_id,
            category,
        } => {
            let result =
                findings::list_recommendations(services, &incident_id, category.as_deref())
                    .await
                    .map_err(msg);
            Event::RecommendationsLoaded {
                incident_id,
                result,
            }
        }
        Command::ExportFindings {
            incident_id,
            save_path,
        } => {
            let result = export_findings_to_file(services, &incident_id, &save_path)
                .await
                .map_err(msg)
                .map(|()| save_path);
            Event::FindingsExported {
                incident_id,
                result,
            }
        }
    }
}

async fn export_findings_to_file(
    services: &AppServices,
    incident_id: &str,
    save_path: &std::path::Path,
) -> Result<(), AppError> {
    let bundle = findings::export_incident_findings(services, incident_id).await?;
    let json = serde_json::to_string_pretty(&bundle)
        .map_err(|error| AppError::Config(format!("failed to serialize export bundle: {error}")))?;
    tokio::fs::write(save_path, json)
        .await
        .map_err(AppError::Storage)
}
