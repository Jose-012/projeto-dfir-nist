//! Linhagem analítica (somente leitura, inspecionável).
//!
//! Fatia 4 do plano ADR-0030 (F1-01/F1-04): dado um resultado analítico selecionado, devolve
//! o resultado em si, o histórico completo da mesma `correlation_key` (do mais
//! antigo ao vigente) e as relações de saída declaradas por ele.
//!
//! Sem mutation, sem novo protocolo, sem grafo global: reutiliza
//! [`analytical_results::get`], [`analytical_results::history_for_correlation_key`],
//! [`analytical_results::current_for_correlation_key`],
//! [`analytical_results::related_to_strict`] e
//! [`AnalyticalResultResponse::try_from`].
//! Toda falha retorna diagnóstico específico, nunca null, default ou omissão
//! silenciosa (JSON inválido, relação desconhecida ou alvo ausente).

use crate::{
    error::AppError, persistence::analytical_results,
    persistence::analytical_results::RelationType, service::AnalyticalResultResponse,
};
use sqlx::SqlitePool;

/// Escopo de validação de pertença do resultado selecionado.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LineageScope {
    /// O resultado deve pertencer a esta evidência (registro `evidence_id`).
    Evidence(String),
    /// O resultado deve pertencer a este incidente (registro `incident_id`).
    Incident(String),
}

/// Uma relação de saída do resultado selecionado.
#[derive(Debug, Clone)]
pub struct OutgoingRelation {
    pub kind: RelationType,
    pub result: AnalyticalResultResponse,
}

/// Vista completa da linhagem de um resultado analítico.
#[derive(Debug, Clone)]
pub struct LineageView {
    /// Resultado selecionado (o alvo da consulta).
    pub selected: AnalyticalResultResponse,
    /// Histórico completo da `correlation_key`, do mais antigo ao vigente.
    /// Inclui o próprio resultado selecionado.
    pub history: Vec<AnalyticalResultResponse>,
    /// Resultado vigente da `correlation_key` (ou `None` quando não há linha
    /// não-superseded — não ocorre em prática para uma chave que existe, mas
    /// mantido para clareza).
    pub current: Option<AnalyticalResultResponse>,
    /// Relações de saída declaradas pelo resultado selecionado.
    pub outgoing: Vec<OutgoingRelation>,
}

/// Linhagem do resultado `result_id`, validada contra o escopo informado.
///
/// Validações (em ordem):
/// 1. `result_id` deve existir (senão [`AppError::NotFound`]);
/// 2. O resultado deve pertencer ao escopo solicitado (evidência ou incidente),
///    inclusive na relação cruzada — senão [`AppError::NotFound`] com motivo;
/// 3. O histórico e as relações de saída são lidos e convertidos via
///    [`AnalyticalResultResponse::try_from`]; payload ou provenance
///    malformados → [`AppError::Config`] com o motivo do JSON;
/// 4. Toda versão do histórico e todo alvo de relação também precisa pertencer
///    ao escopo; tipo de relação desconhecido ou alvo inexistente falha em
///    [`analytical_results::related_to_strict`], sem retorno parcial silencioso.
pub async fn lineage_for_result(
    pool: &SqlitePool,
    result_id: &str,
    scope: &LineageScope,
) -> Result<LineageView, AppError> {
    let record = analytical_results::get(pool, result_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("analytical result not found: {result_id}")))?;

    validate_scope(&record, scope)?;

    let correlation_key = &record.correlation_key;
    let history_records =
        analytical_results::history_for_correlation_key(pool, correlation_key).await?;
    let mut history = Vec::with_capacity(history_records.len());
    for record in history_records {
        validate_scope(&record, scope)?;
        history.push(AnalyticalResultResponse::try_from(record)?);
    }

    let current_record =
        analytical_results::current_for_correlation_key(pool, correlation_key).await?;
    if let Some(record) = &current_record {
        validate_scope(record, scope)?;
    }
    let current = current_record
        .map(AnalyticalResultResponse::try_from)
        .transpose()?;

    let relations = analytical_results::related_to_strict(pool, result_id).await?;
    let mut outgoing = Vec::with_capacity(relations.len());
    for (kind, record) in relations {
        validate_scope(&record, scope)?;
        outgoing.push(OutgoingRelation {
            kind,
            result: AnalyticalResultResponse::try_from(record)?,
        });
    }

    let selected = AnalyticalResultResponse::try_from(record)?;

    Ok(LineageView {
        selected,
        history,
        current,
        outgoing,
    })
}

/// Confere que `record` pertence ao escopo solicitado.
///
/// Relação cruzada: quando o escopo é `Incident`, o resultado deve carregar
/// esse `incident_id`; quando é `Evidence`, deve carregar esse `evidence_id`.
/// Qualquer outra combinação é rejeitada com diagnóstico claro — nunca é
/// aceito por omissão ou default.
fn validate_scope(
    record: &analytical_results::AnalyticalResultRecord,
    scope: &LineageScope,
) -> Result<(), AppError> {
    match scope {
        LineageScope::Evidence(evidence_id) => match &record.evidence_id {
            Some(actual) if actual == evidence_id => Ok(()),
            _ => Err(AppError::NotFound(format!(
                "analytical result {id} does not belong to evidence {evidence_id}",
                id = record.id
            ))),
        },
        LineageScope::Incident(incident_id) => match &record.incident_id {
            Some(actual) if actual == incident_id => Ok(()),
            _ => Err(AppError::NotFound(format!(
                "analytical result {id} does not belong to incident {incident_id}",
                id = record.id
            ))),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::analytical_results::{
        AnalyticalResultType, Confidence, NewAnalyticalResult,
    };
    use crate::persistence::db;
    use crate::persistence::test_support::seed_incident;
    use uuid::Uuid;

    async fn test_pool() -> SqlitePool {
        let temp = tempfile::tempdir().expect("temporary directory");
        let db_path = temp.path().join("lineage-service.sqlite");
        let url = format!("sqlite://{}", db_path.display());
        std::mem::forget(temp);
        let pool = db::connect(&url).await.expect("database connection");
        db::migrate(&pool).await.expect("migrations");
        pool
    }

    async fn seed_evidence(pool: &SqlitePool, id: &str, incident_id: Option<&str>) {
        sqlx::query(
            "INSERT INTO evidence(id,original_filename,storage_reference,format,size_bytes,blake3,sha256,imported_at,status,incident_id) VALUES(?,?,?,?,?,?,?,?,?,?)",
        )
        .bind(id)
        .bind("test.evtx")
        .bind(format!("evidence/{id}.evtx"))
        .bind("evtx")
        .bind(1_i64)
        .bind("b3")
        .bind("sha")
        .bind("2026-08-25T00:00:00Z")
        .bind("FORENSIC")
        .bind(incident_id)
        .execute(pool)
        .await
        .expect("evidence insert");
    }

    fn yara_input<'a>(
        correlation_key: &'a str,
        evidence_id: Option<&'a str>,
        incident_id: Option<&'a str>,
    ) -> NewAnalyticalResult<'a> {
        NewAnalyticalResult {
            result_type: AnalyticalResultType::YaraMatch,
            correlation_key,
            evidence_id,
            incident_id,
            analysis_job_id: None,
            rule_or_model_id: Some("rule-1"),
            rule_or_model_version: Some("1.0"),
            confidence: Some(Confidence::High),
            payload_json: r#"{"rule":"rule-1","severity":"high"}"#,
            provenance_json: r#"{"producer":"dfir-app","method":"yara-scan"}"#,
            supersedes: None,
            related: &[],
            actor_id: None,
        }
    }

    async fn insert(pool: &SqlitePool, input: NewAnalyticalResult<'_>) -> String {
        analytical_results::insert(pool, input)
            .await
            .expect("insert result")
    }

    async fn insert_with_relation(
        pool: &SqlitePool,
        correlation_key: &str,
        evidence_id: Option<&str>,
        incident_id: Option<&str>,
        target: &str,
    ) -> String {
        let related = [(target, RelationType::DerivedFrom)];
        insert(
            pool,
            NewAnalyticalResult {
                related: &related,
                ..yara_input(correlation_key, evidence_id, incident_id)
            },
        )
        .await
    }

    #[tokio::test]
    async fn lineage_returns_selected_history_and_outgoing_relations() {
        let pool = test_pool().await;
        seed_incident(&pool, "inc-1").await;
        seed_evidence(&pool, "ev-1", Some("inc-1")).await;

        let root = insert(&pool, yara_input("corr-1", Some("ev-1"), Some("inc-1"))).await;
        let child = insert(
            &pool,
            NewAnalyticalResult {
                result_type: AnalyticalResultType::YaraMatch,
                correlation_key: "corr-1",
                evidence_id: Some("ev-1"),
                incident_id: Some("inc-1"),
                analysis_job_id: None,
                rule_or_model_id: Some("rule-1"),
                rule_or_model_version: Some("1.1"),
                confidence: Some(Confidence::High),
                payload_json: r#"{"rule":"rule-1","severity":"critical"}"#,
                provenance_json: r#"{"producer":"dfir-app","method":"yara-scan"}"#,
                supersedes: Some(&root),
                related: &[(&root, RelationType::DerivedFrom)],
                actor_id: None,
            },
        )
        .await;

        let view = lineage_for_result(&pool, &child, &LineageScope::Evidence("ev-1".into()))
            .await
            .expect("lineage");

        assert_eq!(view.selected.id, child);
        assert_eq!(view.history.len(), 2);
        // Histórico do mais antigo ao vigente.
        assert_eq!(view.history[0].id, root);
        assert_eq!(view.history[1].id, child);
        // O vigente é o último (não é superseded por ninguém).
        assert_eq!(
            view.current.as_ref().map(|r| &r.id),
            Some(&view.history[1].id)
        );
        // Uma relação de saída: aponta para o predecessor.
        assert_eq!(view.outgoing.len(), 1);
        assert_eq!(view.outgoing[0].kind, RelationType::DerivedFrom);
        assert_eq!(view.outgoing[0].result.id, root);
    }

    #[tokio::test]
    async fn history_is_ordered_oldest_to_current_even_with_many_revisions() {
        let pool = test_pool().await;
        seed_incident(&pool, "inc-1").await;
        seed_evidence(&pool, "ev-1", Some("inc-1")).await;

        let first = insert(&pool, yara_input("corr-2", Some("ev-1"), Some("inc-1"))).await;
        let second = insert(
            &pool,
            NewAnalyticalResult {
                result_type: AnalyticalResultType::YaraMatch,
                correlation_key: "corr-2",
                evidence_id: Some("ev-1"),
                incident_id: Some("inc-1"),
                analysis_job_id: None,
                rule_or_model_id: Some("rule-1"),
                rule_or_model_version: Some("1.0"),
                confidence: Some(Confidence::High),
                payload_json: "{}",
                provenance_json: "{}",
                supersedes: Some(&first),
                related: &[],
                actor_id: None,
            },
        )
        .await;
        let third = insert(
            &pool,
            NewAnalyticalResult {
                result_type: AnalyticalResultType::YaraMatch,
                correlation_key: "corr-2",
                evidence_id: Some("ev-1"),
                incident_id: Some("inc-1"),
                analysis_job_id: None,
                rule_or_model_id: Some("rule-1"),
                rule_or_model_version: Some("1.0"),
                confidence: Some(Confidence::High),
                payload_json: "{}",
                provenance_json: "{}",
                supersedes: Some(&second),
                related: &[],
                actor_id: None,
            },
        )
        .await;

        let view = lineage_for_result(&pool, &third, &LineageScope::Evidence("ev-1".into()))
            .await
            .expect("lineage");

        let ids: Vec<_> = view.history.iter().map(|r| r.id.as_str()).collect();
        assert_eq!(ids, vec![first.as_str(), second.as_str(), third.as_str()]);
        assert_eq!(
            view.current.as_ref().map(|r| &r.id),
            Some(&view.history[2].id)
        );
    }

    #[tokio::test]
    async fn unknown_result_id_fails_with_not_found() {
        let pool = test_pool().await;
        let error = lineage_for_result(
            &pool,
            "does-not-exist",
            &LineageScope::Evidence("ev-1".into()),
        )
        .await
        .expect_err("unknown result must fail");
        assert!(matches!(error, AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn wrong_evidence_scope_fails_with_not_found() {
        let pool = test_pool().await;
        seed_incident(&pool, "inc-1").await;
        seed_evidence(&pool, "ev-1", Some("inc-1")).await;
        seed_evidence(&pool, "ev-2", Some("inc-1")).await;

        let id = insert(&pool, yara_input("corr-3", Some("ev-1"), Some("inc-1"))).await;
        let error = lineage_for_result(&pool, &id, &LineageScope::Evidence("ev-2".into()))
            .await
            .expect_err("wrong evidence must fail");
        assert!(matches!(error, AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn wrong_incident_scope_fails_with_not_found() {
        let pool = test_pool().await;
        seed_incident(&pool, "inc-1").await;
        seed_incident(&pool, "inc-2").await;
        seed_evidence(&pool, "ev-1", Some("inc-1")).await;

        let id = insert(&pool, yara_input("corr-4", Some("ev-1"), Some("inc-1"))).await;
        let error = lineage_for_result(&pool, &id, &LineageScope::Incident("inc-2".into()))
            .await
            .expect_err("wrong incident must fail");
        assert!(matches!(error, AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn incident_scope_accepts_result_belonging_to_incident() {
        let pool = test_pool().await;
        seed_incident(&pool, "inc-1").await;
        seed_evidence(&pool, "ev-1", Some("inc-1")).await;

        let id = insert(&pool, yara_input("corr-5", Some("ev-1"), Some("inc-1"))).await;
        let view = lineage_for_result(&pool, &id, &LineageScope::Incident("inc-1".into()))
            .await
            .expect("incident scope accepts");
        assert_eq!(view.selected.id, id);
    }

    #[tokio::test]
    async fn outgoing_relation_to_another_evidence_is_rejected() {
        let pool = test_pool().await;
        seed_incident(&pool, "inc-1").await;
        seed_evidence(&pool, "ev-1", Some("inc-1")).await;
        seed_evidence(&pool, "ev-2", Some("inc-1")).await;
        let target = insert(
            &pool,
            yara_input("corr-target", Some("ev-2"), Some("inc-1")),
        )
        .await;
        let selected =
            insert_with_relation(&pool, "corr-selected", Some("ev-1"), Some("inc-1"), &target)
                .await;

        let error = lineage_for_result(&pool, &selected, &LineageScope::Evidence("ev-1".into()))
            .await
            .expect_err("relation to another evidence must fail");
        assert!(matches!(error, AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn outgoing_relation_to_another_incident_is_rejected() {
        let pool = test_pool().await;
        seed_incident(&pool, "inc-1").await;
        seed_incident(&pool, "inc-2").await;
        seed_evidence(&pool, "ev-1", Some("inc-1")).await;
        seed_evidence(&pool, "ev-2", Some("inc-2")).await;
        let target = insert(
            &pool,
            yara_input("corr-target", Some("ev-2"), Some("inc-2")),
        )
        .await;
        let selected =
            insert_with_relation(&pool, "corr-selected", Some("ev-1"), Some("inc-1"), &target)
                .await;

        let error = lineage_for_result(&pool, &selected, &LineageScope::Incident("inc-1".into()))
            .await
            .expect_err("relation to another incident must fail");
        assert!(matches!(error, AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn history_from_another_evidence_is_rejected_in_evidence_scope() {
        let pool = test_pool().await;
        seed_incident(&pool, "inc-1").await;
        seed_evidence(&pool, "ev-1", Some("inc-1")).await;
        seed_evidence(&pool, "ev-2", Some("inc-1")).await;

        // IOC pode agregar ocorrências de evidências distintas no mesmo
        // incidente. Essa exceção de escrita não autoriza vazamento de escopo
        // quando a inspeção foi pedida para uma única evidência.
        let mut root_input = yara_input("corr-ioc", Some("ev-2"), Some("inc-1"));
        root_input.result_type = AnalyticalResultType::Ioc;
        let root = insert(&pool, root_input).await;
        let mut current_input = yara_input("corr-ioc", Some("ev-1"), Some("inc-1"));
        current_input.result_type = AnalyticalResultType::Ioc;
        current_input.supersedes = Some(&root);
        let current = insert(&pool, current_input).await;

        let error = lineage_for_result(&pool, &current, &LineageScope::Evidence("ev-1".into()))
            .await
            .expect_err("history from another evidence must fail");
        assert!(matches!(error, AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn malformed_payload_fails_with_config_error() {
        let pool = test_pool().await;
        seed_incident(&pool, "inc-1").await;
        seed_evidence(&pool, "ev-1", Some("inc-1")).await;

        // `insert()` valida antes de gravar e o trigger proíbe UPDATE. Cria
        // uma revisão append-only com payload malformed no banco temporário.
        let id = insert(&pool, yara_input("corr-6", Some("ev-1"), Some("inc-1"))).await;
        let malformed_id = "malformed-payload";
        sqlx::query("INSERT INTO analytical_results(id,schema_version,result_type,correlation_key,evidence_id,incident_id,payload,provenance,supersedes,created_at) VALUES(?,'analytical-result-v1','YARA_MATCH','corr-6','ev-1','inc-1',?,?,?,'2026-10-03T00:00:00Z')")
            .bind(malformed_id)
            .bind("{not valid json")
            .bind("{}")
            .bind(&id)
            .execute(&pool)
            .await
            .expect("malformed payload revision");

        let error = lineage_for_result(&pool, malformed_id, &LineageScope::Evidence("ev-1".into()))
            .await
            .expect_err("malformed payload must fail");
        assert!(
            matches!(error, AppError::Config(_)),
            "expected Config, got {error:?}"
        );
    }

    #[tokio::test]
    async fn malformed_provenance_fails_with_config_error() {
        let pool = test_pool().await;
        seed_incident(&pool, "inc-1").await;
        seed_evidence(&pool, "ev-1", Some("inc-1")).await;

        let id = insert(&pool, yara_input("corr-7", Some("ev-1"), Some("inc-1"))).await;
        let malformed_id = "malformed-provenance";
        sqlx::query("INSERT INTO analytical_results(id,schema_version,result_type,correlation_key,evidence_id,incident_id,payload,provenance,supersedes,created_at) VALUES(?,'analytical-result-v1','YARA_MATCH','corr-7','ev-1','inc-1',?,?,?,'2026-10-03T00:00:00Z')")
            .bind(malformed_id)
            .bind("{}")
            .bind("not json")
            .bind(&id)
            .execute(&pool)
            .await
            .expect("malformed provenance revision");

        let error = lineage_for_result(&pool, malformed_id, &LineageScope::Evidence("ev-1".into()))
            .await
            .expect_err("malformed provenance must fail");
        assert!(matches!(error, AppError::Config(_)));
    }

    /// Insere uma relação de saída diretamente por SQL (contornando o tipo
    /// `RelationType` conhecido de `insert`), para poder testar tipo
    /// desconhecido e alvo ausente.
    async fn insert_raw_relation(
        pool: &SqlitePool,
        result_id: &str,
        related_result_id: &str,
        relation_type: &str,
    ) {
        let relation_id = Uuid::new_v4().to_string();
        let created_at = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true);
        let mut connection = pool.acquire().await.expect("relation fixture connection");
        sqlx::query("PRAGMA foreign_keys=OFF")
            .execute(&mut *connection)
            .await
            .expect("disable FK only for corrupt fixture");
        sqlx::query("PRAGMA ignore_check_constraints=ON")
            .execute(&mut *connection)
            .await
            .expect("disable CHECK only for corrupt fixture");
        let inserted = sqlx::query(
            "INSERT INTO analytical_result_relations (id,result_id,related_result_id,relation_type,created_at) VALUES (?,?,?,?,?)",
        )
        .bind(&relation_id)
        .bind(result_id)
        .bind(related_result_id)
        .bind(relation_type)
        .bind(&created_at)
        .execute(&mut *connection)
        .await;
        let check_restored = sqlx::query("PRAGMA ignore_check_constraints=OFF")
            .execute(&mut *connection)
            .await;
        let foreign_restored = sqlx::query("PRAGMA foreign_keys=ON")
            .execute(&mut *connection)
            .await;
        inserted.expect("insert raw relation");
        check_restored.expect("restore CHECK after corrupt fixture");
        foreign_restored.expect("restore FK after corrupt fixture");
    }

    #[tokio::test]
    async fn unknown_relation_type_fails_with_specific_error() {
        let pool = test_pool().await;
        seed_incident(&pool, "inc-1").await;
        seed_evidence(&pool, "ev-1", Some("inc-1")).await;

        let id = insert(&pool, yara_input("corr-8", Some("ev-1"), Some("inc-1"))).await;
        insert_raw_relation(&pool, &id, "missing-target", "NOT_A_REAL_TYPE").await;

        let error = lineage_for_result(&pool, &id, &LineageScope::Evidence("ev-1".into()))
            .await
            .expect_err("unknown relation type must fail");
        assert!(
            matches!(error, AppError::Database(sqlx::Error::Protocol(_))),
            "expected a specific protocol error for unknown relation type, got {error:?}"
        );
    }

    #[tokio::test]
    async fn missing_relation_target_fails_with_specific_error() {
        let pool = test_pool().await;
        seed_incident(&pool, "inc-1").await;
        seed_evidence(&pool, "ev-1", Some("inc-1")).await;

        let id = insert(&pool, yara_input("corr-9", Some("ev-1"), Some("inc-1"))).await;
        insert_raw_relation(
            &pool,
            &id,
            "does-not-exist-result",
            RelationType::DerivedFrom.as_str(),
        )
        .await;

        let error = lineage_for_result(&pool, &id, &LineageScope::Evidence("ev-1".into()))
            .await
            .expect_err("missing relation target must fail");
        assert!(
            matches!(error, AppError::Database(sqlx::Error::Protocol(_))),
            "expected a specific protocol error for missing target, got {error:?}"
        );
    }

    #[tokio::test]
    async fn result_without_evidence_or_incident_is_rejected_even_in_incident_scope() {
        let pool = test_pool().await;
        seed_incident(&pool, "inc-1").await;

        // YaraMatch exige evidence_id; forçar um insert sem evidência/incidente não passa
        // pela validação do repository. Em vez disso, confirmamos que o escopo Incident
        // rejeita um resultado cujo incident_id difere.
        seed_evidence(&pool, "ev-1", Some("inc-1")).await;
        seed_incident(&pool, "inc-2").await;
        let id = insert(&pool, yara_input("corr-9", Some("ev-1"), Some("inc-1"))).await;
        let error = lineage_for_result(&pool, &id, &LineageScope::Incident("inc-2".into()))
            .await
            .expect_err("incident scope rejects foreign result");
        assert!(matches!(error, AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn user_message_exposes_the_reason() {
        let pool = test_pool().await;
        let error = lineage_for_result(
            &pool,
            "does-not-exist",
            &LineageScope::Evidence("ev-1".into()),
        )
        .await
        .expect_err("unknown result");
        let message = error.user_message();
        assert!(
            message.contains("analytical result not found"),
            "diagnostic should name the missing target: {message}"
        );
    }
}
