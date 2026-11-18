//! Detecção por regras sobre EVTX e decisão do analista (F1-28, ADR-0025).
//!
//! Fluxo: `run_detection` roda o engine com `EVTX_DETECTION` e grava os
//! **candidatos** (pendentes). O analista os revisa: `confirm_candidate` cria o
//! mapeamento ATT&CK, o achado e os IOCs, com a referência dos eventos de
//! origem; `reject_candidate` guarda o motivo. Nada vira achado sem confirmação.

use crate::{
    error::{AppError, AppResult},
    persistence::detection::{self as store, CandidateCounts, CandidateRow},
    service::{self, AppServices, JobWarning, findings},
};
use dfir_protocol::CandidateIoc;
use serde::Serialize;

/// Resultado de uma execução da detecção.
#[derive(Debug, Clone, Serialize)]
pub struct DetectionRun {
    pub job_id: String,
    pub evidence_id: String,
    pub status: String,
    /// Candidatos que esta execução criou (os que já existiam não contam).
    pub new_candidates: u64,
    pub counts: CandidateCounts,
    pub warnings: Vec<JobWarning>,
}

/// Candidato como a interface e a CLI o recebem.
#[derive(Debug, Clone, Serialize)]
pub struct CandidateView {
    pub id: String,
    pub evidence_id: String,
    pub job_id: String,
    /// Chave estável (regra + versão + agrupamento): identifica o candidato
    /// entre execuções.
    pub candidate_key: String,
    pub created_at: String,
    pub rule_id: String,
    pub rule_version: String,
    pub rule_set_version: String,
    pub title: String,
    pub description: String,
    pub tactic: String,
    pub technique_id: String,
    pub sub_technique_id: Option<String>,
    pub severity: String,
    pub confidence: String,
    pub aggregation: String,
    pub event_count: u64,
    pub first_seen: String,
    pub last_seen: String,
    pub event_refs: Vec<String>,
    pub refs_truncated: bool,
    pub host: Option<String>,
    pub user: Option<String>,
    pub src_ip: Option<String>,
    pub iocs: Vec<CandidateIoc>,
    pub limitations: String,
    pub status: String,
    pub decided_at: Option<String>,
    pub decision_note: Option<String>,
    pub incident_id: Option<String>,
    pub attack_mapping_id: Option<String>,
    pub finding_id: Option<String>,
}

fn clean(value: String) -> String {
    crate::text_safety::neutralized(&value)
}

fn clean_option(value: Option<String>) -> Option<String> {
    value.map(clean)
}

impl From<CandidateRow> for CandidateView {
    fn from(row: CandidateRow) -> Self {
        let event_refs = row.event_refs();
        let iocs = row
            .iocs()
            .into_iter()
            .map(|ioc| CandidateIoc {
                ioc_type: ioc.ioc_type,
                value: clean(ioc.value),
                event_ref: ioc.event_ref,
            })
            .collect();
        Self {
            id: row.id,
            evidence_id: row.evidence_id,
            job_id: row.job_id,
            candidate_key: row.candidate_key,
            created_at: row.created_at,
            rule_id: row.rule_id,
            rule_version: row.rule_version,
            rule_set_version: row.rule_set_version,
            title: row.title,
            // Texto montado a partir de campos do log: exibição neutralizada.
            description: clean(row.description),
            tactic: row.tactic,
            technique_id: row.technique_id,
            sub_technique_id: row.sub_technique_id,
            severity: row.severity,
            confidence: row.confidence,
            aggregation: clean(row.aggregation),
            event_count: row.event_count.max(0) as u64,
            first_seen: row.first_seen,
            last_seen: row.last_seen,
            event_refs,
            refs_truncated: row.refs_truncated != 0,
            host: clean_option(row.host),
            user: clean_option(row.user_name),
            src_ip: clean_option(row.src_ip),
            iocs,
            limitations: row.limitations,
            status: row.status,
            decided_at: row.decided_at,
            decision_note: row.decision_note,
            incident_id: row.incident_id,
            attack_mapping_id: row.attack_mapping_id,
            finding_id: row.finding_id,
        }
    }
}

/// Roda as regras de detecção sobre a evidência e grava os candidatos.
pub async fn run_detection(services: &AppServices, evidence_id: &str) -> AppResult<DetectionRun> {
    let response = service::analyze(
        services,
        evidence_id,
        dfir_protocol::ANALYSIS_KIND_EVTX_DETECTION,
        None,
        None,
    )
    .await?;
    let new_candidates = store::count_for_job(&services.db, &response.job_id).await?;
    let counts = candidate_counts(services, evidence_id).await?;
    Ok(DetectionRun {
        job_id: response.job_id,
        evidence_id: response.evidence_id,
        status: response.status,
        new_candidates,
        counts,
        warnings: response.warnings,
    })
}

pub async fn list_candidates(
    services: &AppServices,
    evidence_id: &str,
    status: Option<&str>,
) -> AppResult<Vec<CandidateView>> {
    if let Some(status) = status
        && ![
            store::STATUS_PENDING,
            store::STATUS_CONFIRMED,
            store::STATUS_REJECTED,
        ]
        .contains(&status)
    {
        return Err(AppError::Config(
            "status must be PENDING, CONFIRMED or REJECTED".into(),
        ));
    }
    Ok(store::list(&services.db, evidence_id, status)
        .await?
        .into_iter()
        .map(Into::into)
        .collect())
}

pub async fn candidate_counts(
    services: &AppServices,
    evidence_id: &str,
) -> AppResult<CandidateCounts> {
    Ok(store::counts(&services.db, evidence_id).await?)
}

async fn load(services: &AppServices, candidate_id: &str) -> AppResult<CandidateRow> {
    store::get(&services.db, candidate_id)
        .await?
        .ok_or_else(|| AppError::NotFound("detection candidate not found".into()))
}

/// O que a confirmação criou.
#[derive(Debug, Clone, Serialize)]
pub struct ConfirmOutcome {
    pub candidate_id: String,
    pub incident_id: String,
    pub attack_mapping_id: String,
    pub finding_id: String,
    pub ioc_ids: Vec<String>,
}

/// Texto de base (proveniência) do mapeamento e do achado.
fn basis_text(row: &CandidateRow) -> String {
    let refs = row.event_refs();
    let shown: Vec<&str> = refs.iter().take(8).map(String::as_str).collect();
    let more = if row.refs_truncated != 0 || refs.len() > shown.len() {
        ", ..."
    } else {
        ""
    };
    format!(
        "Candidato automático {} v{} (conjunto de regras {}) confirmado pelo analista. {} Eventos de origem ({} no total): {}{}. Período: {} a {}.",
        row.rule_id,
        row.rule_version,
        row.rule_set_version,
        clean(row.aggregation.clone()),
        row.event_count,
        shown.join(", "),
        more,
        row.first_seen,
        row.last_seen,
    )
}

/// Confirma o candidato: cria o mapeamento ATT&CK, o achado e os IOCs
/// ligados ao incidente escolhido, e marca o candidato como confirmado.
///
/// Os passos gravam seus vínculos no candidato à medida que terminam: se um
/// deles falhar, uma nova chamada **retoma** de onde parou, sem duplicar.
pub async fn confirm_candidate(
    services: &AppServices,
    candidate_id: &str,
    incident_id: &str,
    analyst_note: Option<&str>,
) -> AppResult<ConfirmOutcome> {
    let mut row = load(services, candidate_id).await?;
    if row.status != store::STATUS_PENDING {
        return Err(AppError::Config(format!(
            "detection candidate already decided ({})",
            row.status
        )));
    }
    let note = analyst_note.map(str::trim).filter(|text| !text.is_empty());
    let basis = basis_text(&row);

    if row.attack_mapping_id.is_none() {
        let mapping_id = findings::create_attack_mapping(
            services,
            incident_id,
            Some(&row.evidence_id),
            &row.tactic,
            &row.technique_id,
            row.sub_technique_id.as_deref(),
            &basis,
            &row.confidence,
            note,
        )
        .await?;
        store::link_attack_mapping(&services.db, &row.id, &mapping_id).await?;
        row.attack_mapping_id = Some(mapping_id);
    }
    let mapping_id = row.attack_mapping_id.clone().unwrap_or_default();

    if row.finding_id.is_none() {
        let input = findings::FindingInput {
            evidence_id: Some(row.evidence_id.clone()),
            kind: "OTHER".into(),
            title: format!("{} ({})", row.title, row.technique_id),
            description: format!(
                "{} {} Limite do método: {}",
                row.description, row.aggregation, row.limitations
            ),
            severity: row.severity.clone(),
            basis: basis.clone(),
            reference: None,
            nist_subcategory_ids: Vec::new(),
            attack_mapping_ids: vec![mapping_id.clone()],
            supporting_result_ids: Vec::new(),
            confidence: row.confidence.clone(),
            analyst_note: note.map(str::to_string),
            supersedes: None,
        };
        let finding_id = findings::create_finding(services, incident_id, &input).await?;
        store::link_finding(&services.db, &row.id, &finding_id).await?;
        row.finding_id = Some(finding_id);
    }
    let finding_id = row.finding_id.clone().unwrap_or_default();

    // IOCs: cada um com a referência do evento onde apareceu. `record_ioc_sighting`
    // já agrupa avistamentos do mesmo valor, então repetir não duplica o IOC.
    let mut ioc_ids = Vec::new();
    for ioc in row.iocs() {
        let context = format!("{} ({}) em {}", row.rule_id, row.tactic, ioc.event_ref);
        match findings::create_ioc(
            services,
            incident_id,
            &row.evidence_id,
            &ioc.ioc_type,
            &ioc.value,
            Some(&context),
            &row.confidence,
            Some(&row.first_seen),
        )
        .await
        {
            Ok(id) => ioc_ids.push(id),
            // Um valor que o normalizador de IOC recusa (ex.: caminho com
            // caractere inválido) não pode impedir a confirmação do candidato:
            // fica registrado em log, sem o valor.
            Err(AppError::Config(reason)) => {
                tracing::warn!(candidate = %row.id, ioc_type = %ioc.ioc_type, %reason, "candidate IOC was not recorded");
            }
            Err(other) => return Err(other),
        }
    }

    let decided = store::decide(
        &services.db,
        &row.id,
        store::STATUS_CONFIRMED,
        note,
        Some(incident_id),
    )
    .await?;
    if !decided {
        return Err(AppError::Config(
            "detection candidate already decided".into(),
        ));
    }
    // F1-19: confirmar um candidato vincula a evidência de origem ao
    // incidente escolhido (a coluna `evidence.incident_id` passa a valer,
    // com auditoria própria em `link_evidence_to_incident`).
    super::link_evidence_to_incident(services, &row.evidence_id, incident_id).await?;
    let metadata = serde_json::json!({
        "rule_id": row.rule_id,
        "rule_version": row.rule_version,
        "rule_set_version": row.rule_set_version,
        "attack_mapping_id": mapping_id,
        "finding_id": finding_id,
        "ioc_count": ioc_ids.len(),
        "event_count": row.event_count,
    });
    crate::persistence::audit::append(
        &services.db,
        None,
        "DETECTION_CANDIDATE_CONFIRMED",
        "detection_candidate",
        &row.id,
        Some(incident_id),
        &metadata.to_string(),
    )
    .await?;
    Ok(ConfirmOutcome {
        candidate_id: row.id,
        incident_id: incident_id.to_string(),
        attack_mapping_id: mapping_id,
        finding_id,
        ioc_ids,
    })
}

/// Rejeita o candidato. O motivo é obrigatório: uma rejeição sem explicação
/// não serve de rastro numa revisão posterior.
pub async fn reject_candidate(
    services: &AppServices,
    candidate_id: &str,
    reason: &str,
) -> AppResult<()> {
    let reason = reason.trim();
    if reason.is_empty() {
        return Err(AppError::Config(
            "a reason is required to reject a detection candidate".into(),
        ));
    }
    let row = load(services, candidate_id).await?;
    if row.status != store::STATUS_PENDING {
        return Err(AppError::Config(format!(
            "detection candidate already decided ({})",
            row.status
        )));
    }
    if !store::decide(
        &services.db,
        &row.id,
        store::STATUS_REJECTED,
        Some(reason),
        None,
    )
    .await?
    {
        return Err(AppError::Config(
            "detection candidate already decided".into(),
        ));
    }
    let metadata = serde_json::json!({
        "rule_id": row.rule_id,
        "rule_version": row.rule_version,
        "rule_set_version": row.rule_set_version,
        "event_count": row.event_count,
    });
    crate::persistence::audit::append(
        &services.db,
        None,
        "DETECTION_CANDIDATE_REJECTED",
        "detection_candidate",
        &row.id,
        None,
        &metadata.to_string(),
    )
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::test_support::{Fixture, fixture};
    use dfir_protocol::{DetectionCandidate, ITEM_KIND_DETECTION_CANDIDATE};

    async fn seed_evidence(fx: &Fixture, id: &str) {
        sqlx::query(
            "INSERT INTO evidence(id,original_filename,storage_reference,format,size_bytes,blake3,sha256,imported_at,status,mime_type,source,collection_method,collected_at,ingested_by,preservation_status) VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
        )
        .bind(id)
        .bind("sample.evtx")
        .bind(format!("evidence/{id}.evtx"))
        .bind("evtx")
        .bind(14_i64)
        .bind("b3")
        .bind("s256")
        .bind("2026-10-02T00:00:00Z")
        .bind("FORENSIC")
        .bind("application/octet-stream")
        .bind("LOCAL_OPERATOR")
        .bind("LOCAL_FILE_PICKER")
        .bind("2026-10-02T00:00:00Z")
        .bind("local-analyst")
        .bind("ACTIVE")
        .execute(&fx.services.db)
        .await
        .expect("evidence");
    }

    fn candidate(key: &str, rule: &str) -> DetectionCandidate {
        DetectionCandidate {
            kind: ITEM_KIND_DETECTION_CANDIDATE.into(),
            evidence_id: "ev1".into(),
            rule_id: rule.into(),
            rule_version: "1".into(),
            rule_set_version: "2026.10.1".into(),
            title: "Logon bem-sucedido depois de rajada de falhas".into(),
            description: "Um logon bem-sucedido depois de rajada.".into(),
            tactic: "INITIAL_ACCESS".into(),
            technique_id: "T1078".into(),
            sub_technique_id: None,
            severity: "HIGH".into(),
            confidence: "MEDIUM".into(),
            candidate_key: key.into(),
            aggregation: "conta=admin; origem=185.220.101.5".into(),
            count: 11,
            first_seen: "2026-03-01T10:00:00Z".into(),
            last_seen: "2026-03-01T10:10:00Z".into(),
            event_refs: vec!["record:5".into(), "record:9".into(), "record:12".into()],
            refs_truncated: false,
            host: Some("DC01".into()),
            user: Some("admin".into()),
            src_ip: Some("185.220.101.5".into()),
            iocs: vec![dfir_protocol::CandidateIoc {
                ioc_type: "IP".into(),
                value: "185.220.101.5".into(),
                event_ref: "record:12".into(),
            }],
            limitations: "Não prova que o logon foi do atacante.".into(),
        }
    }

    async fn insert(fx: &Fixture, candidate: &DetectionCandidate) -> bool {
        let mut connection = fx.services.db.acquire().await.expect("connection");
        store::insert_on_connection(&mut connection, "ev1", "job1", candidate)
            .await
            .expect("insert")
    }

    async fn only_candidate(fx: &Fixture) -> CandidateView {
        let mut all = list_candidates(&fx.services, "ev1", None)
            .await
            .expect("list");
        assert_eq!(all.len(), 1);
        all.remove(0)
    }

    async fn count_results(fx: &Fixture, result_type: &str) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*) FROM analytical_results WHERE result_type=?")
            .bind(result_type)
            .fetch_one(&fx.services.db)
            .await
            .expect("count")
    }

    #[tokio::test]
    async fn candidates_start_pending_and_a_rerun_does_not_duplicate_them() {
        let fx = fixture().await;
        seed_evidence(&fx, "ev1").await;
        let found = candidate("k1", "R-AUTH-002");
        assert!(insert(&fx, &found).await, "primeira vez cria");
        assert!(!insert(&fx, &found).await, "mesma chave não cria de novo");
        assert!(insert(&fx, &candidate("k2", "R-ACCESS-001")).await);
        let counts = candidate_counts(&fx.services, "ev1").await.expect("counts");
        assert_eq!(
            (counts.pending, counts.confirmed, counts.rejected),
            (2, 0, 0)
        );
        assert_eq!(counts.total(), 2);
        // Nada virou achado: a detecção sozinha não cria resultado analítico.
        assert_eq!(count_results(&fx, "FINDING").await, 0);
        assert_eq!(count_results(&fx, "ATTACK_MAPPING").await, 0);
    }

    #[tokio::test]
    async fn confirming_creates_the_mapping_the_finding_and_the_iocs_with_event_references() {
        let fx = fixture().await;
        seed_evidence(&fx, "ev1").await;
        insert(&fx, &candidate("k1", "R-AUTH-002")).await;
        let incident = service::create_incident(&fx.services, "Incidente de teste")
            .await
            .expect("incident");
        let id = only_candidate(&fx).await.id;

        let outcome = confirm_candidate(&fx.services, &id, &incident.id, Some("Conferi no AD."))
            .await
            .expect("confirm");
        assert_eq!(outcome.ioc_ids.len(), 1);
        assert_eq!(count_results(&fx, "ATTACK_MAPPING").await, 1);
        assert_eq!(count_results(&fx, "FINDING").await, 1);
        assert_eq!(count_results(&fx, "IOC").await, 1);

        // A base do mapeamento aponta a regra, a versão e os eventos de origem.
        let payload: String = sqlx::query_scalar(
            "SELECT payload FROM analytical_results WHERE result_type='ATTACK_MAPPING'",
        )
        .fetch_one(&fx.services.db)
        .await
        .expect("mapping payload");
        for needle in ["R-AUTH-002", "2026.10.1", "record:5", "record:12", "T1078"] {
            assert!(
                payload.contains(needle),
                "{needle} deveria constar: {payload}"
            );
        }

        let confirmed = only_candidate(&fx).await;
        assert_eq!(confirmed.status, "CONFIRMED");
        assert_eq!(confirmed.incident_id.as_deref(), Some(incident.id.as_str()));
        assert_eq!(
            confirmed.attack_mapping_id.as_deref(),
            Some(outcome.attack_mapping_id.as_str())
        );
        assert_eq!(
            confirmed.finding_id.as_deref(),
            Some(outcome.finding_id.as_str())
        );
        assert_eq!(confirmed.decision_note.as_deref(), Some("Conferi no AD."));

        let audited: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM audit_events WHERE action='DETECTION_CANDIDATE_CONFIRMED' AND object_id=?",
        )
        .bind(&id)
        .fetch_one(&fx.services.db)
        .await
        .expect("audit");
        assert_eq!(audited, 1);

        // F1-19: confirmar também vincula a evidência de origem ao incidente.
        let evidence_incident: Option<String> =
            sqlx::query_scalar("SELECT incident_id FROM evidence WHERE id='ev1'")
                .fetch_one(&fx.services.db)
                .await
                .expect("evidence incident");
        assert_eq!(evidence_incident.as_deref(), Some(incident.id.as_str()));
        let linked_evidence = service::evidence_for_incident(&fx.services, &incident.id)
            .await
            .expect("evidence for incident");
        assert_eq!(linked_evidence.len(), 1);
        assert_eq!(linked_evidence[0].id, "ev1");

        // A decisão é final.
        let again = confirm_candidate(&fx.services, &id, &incident.id, None)
            .await
            .expect_err("já decidido");
        assert!(matches!(again, AppError::Config(message) if message.contains("already decided")));
        assert_eq!(count_results(&fx, "FINDING").await, 1, "nada duplicado");
    }

    #[tokio::test]
    async fn confirming_into_a_missing_incident_creates_nothing_and_leaves_the_candidate_pending() {
        let fx = fixture().await;
        seed_evidence(&fx, "ev1").await;
        insert(&fx, &candidate("k1", "R-AUTH-002")).await;
        let id = only_candidate(&fx).await.id;
        let error = confirm_candidate(&fx.services, &id, "incidente-que-nao-existe", None)
            .await
            .expect_err("incident must exist");
        assert!(matches!(error, AppError::NotFound(_)), "{error:?}");
        assert_eq!(count_results(&fx, "ATTACK_MAPPING").await, 0);
        assert_eq!(count_results(&fx, "FINDING").await, 0);
        assert_eq!(only_candidate(&fx).await.status, "PENDING");
    }

    #[tokio::test]
    async fn an_interrupted_confirmation_resumes_without_duplicating_the_mapping() {
        let fx = fixture().await;
        seed_evidence(&fx, "ev1").await;
        insert(&fx, &candidate("k1", "R-AUTH-002")).await;
        let incident = service::create_incident(&fx.services, "Incidente")
            .await
            .expect("incident");
        let id = only_candidate(&fx).await.id;
        // Primeiro passo feito (mapeamento criado e vinculado), o resto não.
        let mapping = findings::create_attack_mapping(
            &fx.services,
            &incident.id,
            Some("ev1"),
            "INITIAL_ACCESS",
            "T1078",
            None,
            "base",
            "MEDIUM",
            None,
        )
        .await
        .expect("mapping");
        store::link_attack_mapping(&fx.services.db, &id, &mapping)
            .await
            .expect("link");

        let outcome = confirm_candidate(&fx.services, &id, &incident.id, None)
            .await
            .expect("resume");
        assert_eq!(
            outcome.attack_mapping_id, mapping,
            "reaproveita o mapeamento já criado"
        );
        assert_eq!(count_results(&fx, "ATTACK_MAPPING").await, 1);
        assert_eq!(count_results(&fx, "FINDING").await, 1);
    }

    #[tokio::test]
    async fn rejecting_needs_a_reason_keeps_the_candidate_and_is_final() {
        let fx = fixture().await;
        seed_evidence(&fx, "ev1").await;
        insert(&fx, &candidate("k1", "R-AUTH-002")).await;
        let id = only_candidate(&fx).await.id;

        assert!(
            reject_candidate(&fx.services, &id, "   ").await.is_err(),
            "sem motivo não rejeita"
        );
        assert_eq!(only_candidate(&fx).await.status, "PENDING");

        reject_candidate(
            &fx.services,
            &id,
            "Serviço de monitoramento com senha vencida.",
        )
        .await
        .expect("reject");
        let rejected = only_candidate(&fx).await;
        assert_eq!(rejected.status, "REJECTED");
        assert_eq!(
            rejected.decision_note.as_deref(),
            Some("Serviço de monitoramento com senha vencida.")
        );
        assert_eq!(
            count_results(&fx, "FINDING").await,
            0,
            "rejeitar não cria achado"
        );

        let incident = service::create_incident(&fx.services, "Incidente")
            .await
            .expect("incident");
        assert!(
            confirm_candidate(&fx.services, &id, &incident.id, None)
                .await
                .is_err()
        );
        assert!(
            reject_candidate(&fx.services, &id, "de novo")
                .await
                .is_err()
        );

        // Rodar a detecção de novo não ressuscita nem duplica o rejeitado.
        assert!(!insert(&fx, &candidate("k1", "R-AUTH-002")).await);
        assert_eq!(only_candidate(&fx).await.status, "REJECTED");
    }

    #[tokio::test]
    async fn the_database_itself_protects_what_the_rule_said_and_the_final_decision() {
        let fx = fixture().await;
        seed_evidence(&fx, "ev1").await;
        insert(&fx, &candidate("k1", "R-AUTH-002")).await;
        let id = only_candidate(&fx).await.id;

        let tamper = sqlx::query(
            "UPDATE detection_candidates SET title='outro', event_refs='[]' WHERE id=?",
        )
        .bind(&id)
        .execute(&fx.services.db)
        .await;
        assert!(tamper.is_err(), "o que a regra afirmou é imutável");

        reject_candidate(&fx.services, &id, "ruído")
            .await
            .expect("reject");
        let reopen = sqlx::query("UPDATE detection_candidates SET status='PENDING' WHERE id=?")
            .bind(&id)
            .execute(&fx.services.db)
            .await;
        assert!(reopen.is_err(), "decisão tomada não volta a pendente");
    }

    #[tokio::test]
    async fn the_view_neutralizes_hostile_text_that_came_from_the_log() {
        let fx = fixture().await;
        seed_evidence(&fx, "ev1").await;
        let mut hostile = candidate("k1", "R-AUTH-002");
        hostile.user = Some("admin\u{202E}gpj".into());
        hostile.aggregation = "conta=admin\u{202E}gpj".into();
        insert(&fx, &hostile).await;
        let view = only_candidate(&fx).await;
        assert!(!view.aggregation.contains('\u{202E}') && view.aggregation.contains("⟦U+202E⟧"));
        assert!(!view.user.as_deref().unwrap_or("").contains('\u{202E}'));
    }

    #[tokio::test]
    async fn invalid_status_filters_are_refused_and_destroying_the_evidence_removes_candidates() {
        let fx = fixture().await;
        seed_evidence(&fx, "ev1").await;
        insert(&fx, &candidate("k1", "R-AUTH-002")).await;
        assert!(
            list_candidates(&fx.services, "ev1", Some("QUALQUER"))
                .await
                .is_err()
        );
        assert_eq!(
            list_candidates(&fx.services, "ev1", Some("PENDING"))
                .await
                .unwrap()
                .len(),
            1
        );
        assert!(
            list_candidates(&fx.services, "ev1", Some("CONFIRMED"))
                .await
                .unwrap()
                .is_empty()
        );
        store::delete_for_evidence(&fx.services.db, "ev1")
            .await
            .expect("delete");
        assert!(
            list_candidates(&fx.services, "ev1", None)
                .await
                .unwrap()
                .is_empty()
        );
    }
}
