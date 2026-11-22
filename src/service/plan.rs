//! F1-29 (RF-052) — itens do plano de fortalecimento, camada de serviço.
//!
//! As regras (prioridade, esforço, custo e fase obrigatórios; detalhes de
//! proteção só nos domínios de criptografia e chaves; recomendação geral
//! quando não há achado) vivem em `persistence/plan_items.rs`.

use super::findings::non_blank;
use super::{AnalyticalResultResponse, AppServices, current_results_for_incident, incident_exists};
use crate::error::{AppError, AppResult};
use crate::persistence::plan_items as plan_model;
use serde::Serialize;

/// Os cinco detalhes do Entregável 5, como a interface os recebe.
#[derive(Debug, Clone, Default)]
pub struct CryptoInput {
    pub data_class: String,
    pub location: String,
    pub mechanism: String,
    pub rotation: String,
    pub recovery: String,
}

/// Dados de um item de plano como a interface os recebe (tudo texto).
#[derive(Debug, Clone, Default)]
pub struct PlanItemInput {
    pub domain: String,
    pub title: String,
    pub rationale: String,
    pub priority: String,
    pub effort: String,
    pub cost_class: String,
    pub phase: String,
    pub precondition: String,
    pub postcondition: String,
    pub verification: String,
    pub framework_reference: Option<String>,
    pub regulatory_reference: Option<String>,
    pub nist_subcategory_ids: Vec<String>,
    pub finding_ids: Vec<String>,
    pub depends_on: Vec<String>,
    pub crypto: Option<CryptoInput>,
    pub supersedes: Option<String>,
    /// Fonte da recomendação: OBSERVED / INFERRED / NOT_VERIFIED / RECOMMENDED.
    pub source: Option<String>,
    /// Estado de confiança: LOW / MEDIUM / HIGH / CONFIRMED.
    pub confidence: Option<String>,
}

fn invalid(what: &str, value: &str) -> AppError {
    AppError::Config(format!("invalid {what}: {value}"))
}

pub async fn create_plan_item(
    services: &AppServices,
    incident_id: &str,
    input: &PlanItemInput,
) -> AppResult<String> {
    incident_exists(&services.db, incident_id).await?;
    let domain = plan_model::PlanDomain::from_str(&input.domain)
        .ok_or_else(|| invalid("plan domain", &input.domain))?;
    let priority = plan_model::PlanPriority::from_str(&input.priority)
        .ok_or_else(|| invalid("priority", &input.priority))?;
    let effort = plan_model::PlanEffort::from_str(&input.effort)
        .ok_or_else(|| invalid("effort", &input.effort))?;
    let cost_class = plan_model::CostClass::from_str(&input.cost_class)
        .ok_or_else(|| invalid("cost class", &input.cost_class))?;
    let phase = plan_model::PlanPhase::from_str(&input.phase)
        .ok_or_else(|| invalid("phase", &input.phase))?;
    let nist: Vec<&str> = input
        .nist_subcategory_ids
        .iter()
        .map(String::as_str)
        .collect();
    let findings: Vec<&str> = input.finding_ids.iter().map(String::as_str).collect();
    let depends_on: Vec<&str> = input.depends_on.iter().map(String::as_str).collect();
    let crypto = input
        .crypto
        .as_ref()
        .map(|crypto| plan_model::CryptoDetail {
            data_class: &crypto.data_class,
            location: &crypto.location,
            mechanism: &crypto.mechanism,
            rotation: &crypto.rotation,
            recovery: &crypto.recovery,
        });
    let id = plan_model::insert_plan_item(
        &services.db,
        plan_model::NewPlanItem {
            incident_id,
            domain,
            title: &input.title,
            rationale: &input.rationale,
            priority,
            effort,
            cost_class,
            phase,
            precondition: &input.precondition,
            postcondition: &input.postcondition,
            verification: &input.verification,
            framework_reference: non_blank(&input.framework_reference),
            regulatory_reference: non_blank(&input.regulatory_reference),
            nist_subcategory_ids: nist.as_slice(),
            finding_ids: findings.as_slice(),
            depends_on: depends_on.as_slice(),
            crypto,
            supersedes: non_blank(&input.supersedes),
            source: non_blank(&input.source),
            confidence: non_blank(&input.confidence),
            actor_id: None,
        },
    )
    .await
    .map_err(AppError::Database)?;
    Ok(id)
}

/// Tudo que a aba "Plano" mostra, numa só consulta: os itens vigentes (por
/// domínio e prioridade), os achados vigentes (para ligar a um item) e a
/// cobertura das ações da demanda.
#[derive(Debug, Clone, Serialize)]
pub struct PlanView {
    pub items: Vec<AnalyticalResultResponse>,
    pub findings: Vec<AnalyticalResultResponse>,
    pub coverage: Vec<plan_model::DomainCoverage>,
}

fn payload_str<'a>(item: &'a AnalyticalResultResponse, key: &str) -> Option<&'a str> {
    item.payload.get(key).and_then(|value| value.as_str())
}

fn validate_plan_payload(item: &AnalyticalResultResponse) -> AppResult<()> {
    let invalid =
        |field| AppError::Config(format!("stored plan item {} has invalid {field}", item.id));
    payload_str(item, "domain")
        .and_then(plan_model::PlanDomain::from_str)
        .ok_or_else(|| invalid("domain"))?;
    payload_str(item, "priority")
        .and_then(plan_model::PlanPriority::from_str)
        .ok_or_else(|| invalid("priority"))?;
    payload_str(item, "effort")
        .and_then(plan_model::PlanEffort::from_str)
        .ok_or_else(|| invalid("effort"))?;
    payload_str(item, "cost_class")
        .and_then(plan_model::CostClass::from_str)
        .ok_or_else(|| invalid("cost_class"))?;
    payload_str(item, "phase")
        .and_then(plan_model::PlanPhase::from_str)
        .ok_or_else(|| invalid("phase"))?;
    if !matches!(
        payload_str(item, "support_kind"),
        Some("FINDING_BACKED" | "GENERAL_RECOMMENDATION")
    ) {
        return Err(invalid("support_kind"));
    }
    for key in ["finding_ids", "nist_subcategories", "depends_on"] {
        let Some(value) = item.payload.get(key) else {
            continue;
        };
        let values = value
            .as_array()
            .ok_or_else(|| AppError::Config(format!("stored plan item {key} is not an array")))?;
        if values.iter().any(|value| !value.is_string()) {
            return Err(AppError::Config(format!(
                "stored plan item {key} contains a non-string id"
            )));
        }
    }
    // source e confiança são obrigatórios nos itens novos: rejeita ausência
    // ou valores inválidos no payload armazenado.
    let Some(source_value) = item.payload.get("source") else {
        return Err(AppError::Config(format!(
            "stored plan item {} is missing source",
            item.id
        )));
    };
    if source_value.is_null() || !source_value.is_string() {
        return Err(AppError::Config(format!(
            "stored plan item {} source is not a string",
            item.id
        )));
    }
    let Some(source) = source_value.as_str() else {
        return Err(AppError::Config(format!(
            "stored plan item {} source is not a string",
            item.id
        )));
    };
    if !matches!(source, "OBSERVED" | "INFERRED" | "NOT_VERIFIED" | "RECOMMENDED") {
        return Err(AppError::Config(format!(
            "stored plan item {} source is not valid: {source}",
            item.id
        )));
    }
    let Some(confidence_value) = item.payload.get("confidence") else {
        return Err(AppError::Config(format!(
            "stored plan item {} is missing confidence",
            item.id
        )));
    };
    if confidence_value.is_null() || !confidence_value.is_string() {
        return Err(AppError::Config(format!(
            "stored plan item {} confidence is not a string",
            item.id
        )));
    }
    let Some(confidence) = confidence_value.as_str() else {
        return Err(AppError::Config(format!(
            "stored plan item {} confidence is not a string",
            item.id
        )));
    };
    if !matches!(confidence, "LOW" | "MEDIUM" | "HIGH" | "CONFIRMED") {
        return Err(AppError::Config(format!(
            "stored plan item {} confidence is not valid: {confidence}",
            item.id
        )));
    }
    Ok(())
}

pub async fn plan_view(services: &AppServices, incident_id: &str) -> AppResult<PlanView> {
    incident_exists(&services.db, incident_id).await?;
    let mut items = current_results_for_incident(&services.db, incident_id, "PLAN_ITEM").await?;
    for item in &items {
        validate_plan_payload(item)?;
    }
    // Estável: dentro do mesmo domínio e prioridade, mantém a ordem recebida.
    items.sort_by_key(|item| {
        let domain = payload_str(item, "domain")
            .and_then(plan_model::PlanDomain::from_str)
            .map_or(usize::MAX, plan_model::PlanDomain::position);
        let priority = payload_str(item, "priority")
            .and_then(plan_model::PlanPriority::from_str)
            .map_or(u8::MAX, plan_model::PlanPriority::rank);
        (domain, priority)
    });
    let findings = current_results_for_incident(&services.db, incident_id, "FINDING").await?;
    let entries: Vec<(plan_model::PlanDomain, bool)> = items
        .iter()
        .filter_map(|item| {
            let domain = payload_str(item, "domain").and_then(plan_model::PlanDomain::from_str)?;
            let general = payload_str(item, "support_kind") == Some("GENERAL_RECOMMENDATION");
            Some((domain, general))
        })
        .collect();
    let coverage = plan_model::domain_coverage(&entries);
    Ok(PlanView {
        items,
        findings,
        coverage,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::findings::{FindingInput, create_finding};
    use crate::service::test_support::fixture;

    fn plan_input(domain: &str, title: &str, priority: &str) -> PlanItemInput {
        PlanItemInput {
            domain: domain.into(),
            title: title.into(),
            rationale: "motivo".into(),
            priority: priority.into(),
            effort: "M".into(),
            cost_class: "LOW".into(),
            phase: "SHORT_TERM".into(),
            precondition: "ambiente acessível para verificação".into(),
            postcondition: "controle aplicado e verificável".into(),
            verification: "checagem objetiva".into(),
            source: Some("OBSERVED".into()),
            confidence: Some("MEDIUM".into()),
            ..PlanItemInput::default()
        }
    }

    #[tokio::test]
    async fn malformed_stored_plan_vocabulary_fails_closed() {
        let cases = [
            (
                "unknown support kind",
                "support_kind",
                serde_json::json!("UNKNOWN"),
            ),
            (
                "missing support kind",
                "support_kind",
                serde_json::Value::Null,
            ),
            ("numeric support kind", "support_kind", serde_json::json!(1)),
            ("unknown effort", "effort", serde_json::json!("XL")),
            ("missing effort", "effort", serde_json::Value::Null),
            ("numeric effort", "effort", serde_json::json!(1)),
            (
                "unknown cost class",
                "cost_class",
                serde_json::json!("FREE"),
            ),
            ("missing cost class", "cost_class", serde_json::Value::Null),
            ("numeric cost class", "cost_class", serde_json::json!(1)),
            ("unknown phase", "phase", serde_json::json!("NOW")),
            ("missing phase", "phase", serde_json::Value::Null),
            ("numeric phase", "phase", serde_json::json!(1)),
            ("unknown domain", "domain", serde_json::json!("DATABASE")),
            ("missing domain", "domain", serde_json::Value::Null),
            ("numeric domain", "domain", serde_json::json!(1)),
            ("unknown priority", "priority", serde_json::json!("P9")),
            ("missing priority", "priority", serde_json::Value::Null),
            ("numeric priority", "priority", serde_json::json!(1)),
        ];
        for (name, field, replacement) in cases {
            let fx = fixture().await;
            let incident = crate::service::create_incident(&fx.services, name)
                .await
                .expect("incident");
            let item = create_plan_item(
                &fx.services,
                &incident.id,
                &plan_input("OS_HARDENING", "baseline", "P1"),
            )
            .await
            .expect("valid plan item");
            let record = crate::persistence::analytical_results::get(&fx.services.db, &item)
                .await
                .expect("get plan item")
                .expect("plan item exists");
            let mut payload: serde_json::Value =
                serde_json::from_str(&record.payload).expect("valid payload");
            if replacement.is_null() {
                payload
                    .as_object_mut()
                    .expect("payload object")
                    .remove(field);
            } else {
                payload[field] = replacement;
            }
            sqlx::query("INSERT INTO analytical_results(id,schema_version,result_type,correlation_key,incident_id,payload,provenance,supersedes,created_at) VALUES('bad-plan-vocabulary','analytical-result-v1','PLAN_ITEM',?,?,?,?,?,'2026-10-03T00:00:00Z')")
                .bind(&record.correlation_key)
                .bind(&incident.id)
                .bind(payload.to_string())
                .bind(record.provenance.to_string())
                .bind(&item)
                .execute(&fx.services.db)
                .await
                .expect("malformed current plan item");

            match plan_view(&fx.services, &incident.id).await {
                Ok(view) => panic!(
                    "{name} was accepted by plan_view: coverage={:?}, priorities={:?}",
                    view.coverage,
                    view.items
                        .iter()
                        .filter_map(|item| item.payload.get("priority"))
                        .collect::<Vec<_>>()
                ),
                Err(error) => {
                    assert!(
                        error.to_string().contains("bad-plan-vocabulary"),
                        "{name}: {error}"
                    );
                    assert!(error.to_string().contains(field), "{name}: {error}");
                }
            }
        }
    }

    #[tokio::test]
    async fn plan_view_rejects_invalid_source_and_confidence() {
        let fx = fixture().await;
        let incident = crate::service::create_incident(&fx.services, "incidente")
            .await
            .expect("incident");
        let item = create_plan_item(
            &fx.services,
            &incident.id,
            &plan_input("OS_HARDENING", "baseline", "P1"),
        )
        .await
        .expect("valid plan item");
        let record = crate::persistence::analytical_results::get(&fx.services.db, &item)
            .await
            .expect("get plan item")
            .expect("plan item exists");
        // source e confiança são obrigatórios: remover um deles falha.
        let mut payload: serde_json::Value =
            serde_json::from_str(&record.payload).expect("valid payload");
        payload.as_object_mut().expect("payload object").remove("source");
        sqlx::query("INSERT INTO analytical_results(id,schema_version,result_type,correlation_key,incident_id,payload,provenance,supersedes,created_at) VALUES('bad-plan-source-missing','analytical-result-v1','PLAN_ITEM',?,?,?,?,?,'2026-10-03T00:00:00Z')")
            .bind(&record.correlation_key)
            .bind(&incident.id)
            .bind(payload.to_string())
            .bind(record.provenance.to_string())
            .bind(&item)
            .execute(&fx.services.db)
            .await
            .expect("malformed source-missing plan item");
        match plan_view(&fx.services, &incident.id).await {
            Ok(view) => panic!(
                "missing source was accepted by plan_view: items={:?}",
                view.items.len()
            ),
            Err(error) => {
                assert!(
                    error.to_string().contains("bad-plan-source-missing"),
                    "{error}"
                );
                assert!(error.to_string().contains("source"), "{error}");
            }
        }

        // Confiança inválida é recusada.
        let mut payload: serde_json::Value =
            serde_json::from_str(&record.payload).expect("valid payload");
        payload["confidence"] = serde_json::json!("CRITICA");
        sqlx::query("INSERT INTO analytical_results(id,schema_version,result_type,correlation_key,incident_id,payload,provenance,supersedes,created_at) VALUES('bad-plan-confidence','analytical-result-v1','PLAN_ITEM',?,?,?,?,?,'2026-10-03T00:00:00Z')")
            .bind(&record.correlation_key)
            .bind(&incident.id)
            .bind(payload.to_string())
            .bind(record.provenance.to_string())
            .bind(&item)
            .execute(&fx.services.db)
            .await
            .expect("malformed confidence plan item");
        match plan_view(&fx.services, &incident.id).await {
            Ok(view) => panic!(
                "invalid confidence was accepted by plan_view: items={:?}",
                view.items.len()
            ),
            Err(error) => {
                assert!(
                    error.to_string().contains("bad-plan-confidence"),
                    "{error}"
                );
                assert!(error.to_string().contains("confidence"), "{error}");
            }
        }
        // Fonte inválida (fora do vocabulário inglês) é recusada.
        let mut payload: serde_json::Value =
            serde_json::from_str(&record.payload).expect("valid payload");
        payload["source"] = serde_json::json!("NAO_ESTA_NA_LISTA");
        sqlx::query("INSERT INTO analytical_results(id,schema_version,result_type,correlation_key,incident_id,payload,provenance,supersedes,created_at) VALUES('bad-plan-source','analytical-result-v1','PLAN_ITEM',?,?,?,?,?,'2026-10-03T00:00:00Z')")
            .bind(&record.correlation_key)
            .bind(&incident.id)
            .bind(payload.to_string())
            .bind(record.provenance.to_string())
            .bind(&item)
            .execute(&fx.services.db)
            .await
            .expect("malformed source plan item");
        match plan_view(&fx.services, &incident.id).await {
            Ok(view) => panic!(
                "invalid source was accepted by plan_view: items={:?}",
                view.items.len()
            ),
            Err(error) => {
                assert!(
                    error.to_string().contains("bad-plan-source"),
                    "{error}"
                );
                assert!(error.to_string().contains("source"), "{error}");
            }
        }
        // Fonte numérica é rejeitada.
        let mut payload: serde_json::Value =
            serde_json::from_str(&record.payload).expect("valid payload");
        payload["source"] = serde_json::json!(42);
        sqlx::query("INSERT INTO analytical_results(id,schema_version,result_type,correlation_key,incident_id,payload,provenance,supersedes,created_at) VALUES('bad-plan-source-numeric','analytical-result-v1','PLAN_ITEM',?,?,?,?,?,'2026-10-03T00:00:00Z')")
            .bind(&record.correlation_key)
            .bind(&incident.id)
            .bind(payload.to_string())
            .bind(record.provenance.to_string())
            .bind(&item)
            .execute(&fx.services.db)
            .await
            .expect("malformed numeric source plan item");
        match plan_view(&fx.services, &incident.id).await {
            Ok(view) => panic!(
                "numeric source was accepted by plan_view: items={:?}",
                view.items.len()
            ),
            Err(error) => {
                assert!(
                    error.to_string().contains("bad-plan-source-numeric"),
                    "{error}"
                );
                assert!(error.to_string().contains("source"), "{error}");
            }
        }
    }

    #[tokio::test]
    async fn plan_view_orders_items_and_reports_coverage_with_empty_domains() {
        let fx = fixture().await;
        let incident = crate::service::create_incident(&fx.services, "incidente")
            .await
            .expect("incident");
        let finding = create_finding(
            &fx.services,
            &incident.id,
            &FindingInput {
                kind: "MISSING_CONTROL".into(),
                title: "RDP sem MFA".into(),
                severity: "HIGH".into(),
                basis: "EVTX 4625".into(),
                confidence: "MEDIUM".into(),
                ..FindingInput::default()
            },
        )
        .await
        .expect("finding");

        // Criados fora de ordem de propósito.
        create_plan_item(
            &fx.services,
            &incident.id,
            &plan_input("MONITORAMENTO", "Alertar logons em massa", "P2"),
        )
        .await
        .expect("monitoring");
        create_plan_item(
            &fx.services,
            &incident.id,
            &PlanItemInput {
                finding_ids: vec![finding],
                // Campos opcionais em branco valem como ausentes.
                regulatory_reference: Some("   ".into()),
                ..plan_input("AUTENTICACAO_ACESSO", "MFA no acesso remoto", "P1")
            },
        )
        .await
        .expect("auth p1");
        create_plan_item(
            &fx.services,
            &incident.id,
            &plan_input("AUTENTICACAO_ACESSO", "Revisar política de senhas", "P3"),
        )
        .await
        .expect("auth p3");
        create_plan_item(
            &fx.services,
            &incident.id,
            &PlanItemInput {
                crypto: Some(CryptoInput {
                    data_class: "dados pessoais".into(),
                    location: "DB01".into(),
                    mechanism: "AES-256".into(),
                    rotation: "anual".into(),
                    recovery: "cofre".into(),
                }),
                ..plan_input("CRIPTO_REPOUSO", "Cifrar o banco", "P1")
            },
        )
        .await
        .expect("crypto");

        let view = plan_view(&fx.services, &incident.id).await.expect("view");
        let order: Vec<&str> = view
            .items
            .iter()
            .map(|item| item.payload["title"].as_str().unwrap_or(""))
            .collect();
        assert_eq!(
            order,
            vec![
                "MFA no acesso remoto",
                "Revisar política de senhas",
                "Alertar logons em massa",
                "Cifrar o banco",
            ],
            "por domínio na ordem da demanda e, dentro dele, por prioridade"
        );
        assert_eq!(view.findings.len(), 1);

        assert_eq!(view.coverage.len(), 8);
        let shape: Vec<(&str, usize, usize)> = view
            .coverage
            .iter()
            .map(|c| (c.domain.as_str(), c.items, c.general_items))
            .collect();
        assert_eq!(
            shape,
            vec![
                ("OS_HARDENING", 0, 0),
                ("REDE_DISPOSITIVOS", 0, 0),
                ("AUTENTICACAO_ACESSO", 2, 1),
                ("MONITORAMENTO", 1, 1),
                ("CRIPTO_REPOUSO", 1, 1),
                ("CRIPTO_TRANSITO", 0, 0),
                ("GESTAO_CHAVES", 0, 0),
                ("POLITICA_IR", 0, 0),
            ]
        );
    }

    #[tokio::test]
    async fn revising_an_item_shows_only_the_current_revision() {
        let fx = fixture().await;
        let incident = crate::service::create_incident(&fx.services, "incidente")
            .await
            .expect("incident");
        let first = create_plan_item(
            &fx.services,
            &incident.id,
            &plan_input("OS_HARDENING", "Aplicar baseline CIS", "P3"),
        )
        .await
        .expect("first");
        create_plan_item(
            &fx.services,
            &incident.id,
            &PlanItemInput {
                supersedes: Some(first),
                ..plan_input("OS_HARDENING", "Aplicar baseline CIS", "P1")
            },
        )
        .await
        .expect("revision");

        let view = plan_view(&fx.services, &incident.id).await.expect("view");
        assert_eq!(view.items.len(), 1);
        assert_eq!(view.items[0].payload["priority"], "P1");
    }

    #[tokio::test]
    async fn plan_view_accepts_legacy_missing_array_fields() {
        let fx = fixture().await;
        let incident = crate::service::create_incident(&fx.services, "legacy arrays")
            .await
            .expect("incident");
        let item = create_plan_item(
            &fx.services,
            &incident.id,
            &plan_input("OS_HARDENING", "legacy plan item", "P1"),
        )
        .await
        .expect("valid plan item");
        let record = crate::persistence::analytical_results::get(&fx.services.db, &item)
            .await
            .expect("get plan item")
            .expect("plan item exists");
        let mut payload: serde_json::Value =
            serde_json::from_str(&record.payload).expect("valid payload");
        for key in ["finding_ids", "nist_subcategories", "depends_on"] {
            payload.as_object_mut().expect("payload object").remove(key);
        }
        sqlx::query("INSERT INTO analytical_results(id,schema_version,result_type,correlation_key,incident_id,payload,provenance,supersedes,created_at) VALUES('legacy-plan-arrays','analytical-result-v1','PLAN_ITEM',?,?,?,?,?,'2026-10-03T00:00:00Z')")
            .bind(&record.correlation_key)
            .bind(&incident.id)
            .bind(payload.to_string())
            .bind(record.provenance.to_string())
            .bind(&item)
            .execute(&fx.services.db)
            .await
            .expect("legacy current plan item");

        let view = plan_view(&fx.services, &incident.id)
            .await
            .expect("legacy missing arrays remain valid");
        assert_eq!(view.items.len(), 1);
        assert_eq!(view.items[0].id, "legacy-plan-arrays");
    }

    #[tokio::test]
    async fn service_rejects_unknown_incident_vocabulary_and_missing_protection_details() {
        let fx = fixture().await;
        assert!(
            create_plan_item(&fx.services, "nope", &plan_input("OS_HARDENING", "x", "P1"))
                .await
                .is_err()
        );
        let incident = crate::service::create_incident(&fx.services, "incidente")
            .await
            .expect("incident");
        for input in [
            plan_input("BOGUS", "x", "P1"),
            plan_input("OS_HARDENING", "x", "P9"),
            PlanItemInput {
                effort: "XL".into(),
                ..plan_input("OS_HARDENING", "x", "P1")
            },
            PlanItemInput {
                cost_class: "FREE".into(),
                ..plan_input("OS_HARDENING", "x", "P1")
            },
            PlanItemInput {
                phase: "NOW".into(),
                ..plan_input("OS_HARDENING", "x", "P1")
            },
            // Domínio de criptografia sem os cinco detalhes.
            plan_input("GESTAO_CHAVES", "x", "P1"),
        ] {
            assert!(
                create_plan_item(&fx.services, &incident.id, &input)
                    .await
                    .is_err(),
                "{input:?}"
            );
        }
    }
}
