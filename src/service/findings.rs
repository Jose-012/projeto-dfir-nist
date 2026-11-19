//! F1-04/F1-05 — ATT&CK mapping, caminho crítico, matriz de impacto, IOCs e
//! recomendações, camada de serviço. Porta direta dos handlers homônimos de
//! `web/mod.rs`.

use super::{AnalyticalResultResponse, AppServices, current_results_for_incident, incident_exists};
use crate::error::{AppError, AppResult};
use crate::persistence::{
    analytical_results, attack_impact, findings as finding_model, ioc_recommendations,
};
use serde::Serialize;

// ---------------------------------------------------------------------
// ATT&CK mapping
// ---------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
pub async fn create_attack_mapping(
    services: &AppServices,
    incident_id: &str,
    evidence_id: Option<&str>,
    tactic: &str,
    technique_id: &str,
    sub_technique_id: Option<&str>,
    basis: &str,
    confidence: &str,
    analyst_note: Option<&str>,
) -> AppResult<String> {
    incident_exists(&services.db, incident_id).await?;
    let tactic = attack_impact::AttackTactic::from_str(tactic)
        .ok_or_else(|| AppError::Config("invalid tactic".into()))?;
    let confidence = analytical_results::Confidence::from_str(confidence)
        .ok_or_else(|| AppError::Config("invalid confidence".into()))?;
    let id = attack_impact::insert_attack_mapping(
        &services.db,
        attack_impact::NewAttackMapping {
            incident_id,
            evidence_id,
            tactic,
            technique_id,
            sub_technique_id,
            basis,
            confidence,
            analyst_note,
            actor_id: None,
        },
    )
    .await
    .map_err(AppError::Database)?;
    Ok(id)
}

pub async fn list_attack_mappings(
    services: &AppServices,
    incident_id: &str,
) -> AppResult<Vec<AnalyticalResultResponse>> {
    incident_exists(&services.db, incident_id).await?;
    current_results_for_incident(&services.db, incident_id, "ATTACK_MAPPING").await
}

// ---------------------------------------------------------------------
// Caminho crítico
// ---------------------------------------------------------------------

pub async fn create_critical_path_stage(
    services: &AppServices,
    incident_id: &str,
    evidence_id: Option<&str>,
    sequence_index: i64,
    stage_label: &str,
    basis: &str,
    confidence: &str,
) -> AppResult<String> {
    incident_exists(&services.db, incident_id).await?;
    let confidence = analytical_results::Confidence::from_str(confidence)
        .ok_or_else(|| AppError::Config("invalid confidence".into()))?;
    let id = attack_impact::insert_critical_path_stage(
        &services.db,
        attack_impact::NewCriticalPathStage {
            incident_id,
            evidence_id,
            sequence_index,
            stage_label,
            basis,
            confidence,
            actor_id: None,
        },
    )
    .await
    .map_err(AppError::Database)?;
    Ok(id)
}

pub async fn link_critical_path_stages(
    services: &AppServices,
    incident_id: &str,
    from_stage_id: &str,
    to_stage_id: &str,
) -> AppResult<()> {
    incident_exists(&services.db, incident_id).await?;
    attack_impact::link_critical_path_stages(&services.db, incident_id, from_stage_id, to_stage_id)
        .await
        .map_err(AppError::Database)
}

#[derive(Debug, Clone, Serialize)]
pub struct CriticalPathEdge {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CriticalPathView {
    pub stages: Vec<AnalyticalResultResponse>,
    pub edges: Vec<CriticalPathEdge>,
}

/// Grafo do caminho crítico: estágios vigentes + arestas `SEQUENCE_NEXT`
/// entre eles. Consulta relações estágio a estágio (N+1) — aceitável para a
/// quantidade de estágios esperada num incidente.
pub async fn critical_path(
    services: &AppServices,
    incident_id: &str,
) -> AppResult<CriticalPathView> {
    incident_exists(&services.db, incident_id).await?;
    let stages =
        current_results_for_incident(&services.db, incident_id, "CRITICAL_PATH_LINK").await?;
    let mut edges = Vec::new();
    for stage in &stages {
        let related = analytical_results::related_to(&services.db, &stage.id).await?;
        for (relation_type, target) in related {
            if relation_type == analytical_results::RelationType::SequenceNext {
                edges.push(CriticalPathEdge {
                    from: stage.id.clone(),
                    to: target.id,
                });
            }
        }
    }
    Ok(CriticalPathView { stages, edges })
}

// ---------------------------------------------------------------------
// Matriz de impacto
// ---------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
pub async fn create_impact_assessment(
    services: &AppServices,
    incident_id: &str,
    availability_impact: &str,
    confidentiality_impact: &str,
    integrity_impact: &str,
    affected_assets: &str,
    affected_data: &str,
    operational_disruption: &str,
    financial_impact: Option<&str>,
    confidence: &str,
) -> AppResult<String> {
    incident_exists(&services.db, incident_id).await?;
    let availability_impact = attack_impact::ImpactLevel::from_str(availability_impact)
        .ok_or_else(|| AppError::Config("invalid availability_impact".into()))?;
    let confidentiality_impact = attack_impact::ImpactLevel::from_str(confidentiality_impact)
        .ok_or_else(|| AppError::Config("invalid confidentiality_impact".into()))?;
    let integrity_impact = attack_impact::ImpactLevel::from_str(integrity_impact)
        .ok_or_else(|| AppError::Config("invalid integrity_impact".into()))?;
    let confidence = analytical_results::Confidence::from_str(confidence)
        .ok_or_else(|| AppError::Config("invalid confidence".into()))?;
    let id = attack_impact::insert_impact_assessment(
        &services.db,
        attack_impact::NewImpactAssessment {
            incident_id,
            availability_impact,
            confidentiality_impact,
            integrity_impact,
            affected_assets,
            affected_data,
            operational_disruption,
            financial_impact,
            confidence,
            actor_id: None,
        },
    )
    .await
    .map_err(AppError::Database)?;
    Ok(id)
}

pub async fn current_impact_assessment(
    services: &AppServices,
    incident_id: &str,
) -> AppResult<Option<AnalyticalResultResponse>> {
    incident_exists(&services.db, incident_id).await?;
    let results =
        current_results_for_incident(&services.db, incident_id, "IMPACT_ASSESSMENT").await?;
    let current = results.into_iter().next();
    if let Some(record) = &current {
        for field in [
            "availability_impact",
            "confidentiality_impact",
            "integrity_impact",
        ] {
            if record
                .payload
                .get(field)
                .and_then(serde_json::Value::as_str)
                .and_then(attack_impact::ImpactLevel::from_str)
                .is_none()
            {
                return Err(AppError::Config(format!(
                    "stored impact assessment {} has invalid {field}",
                    record.id
                )));
            }
        }
        if record
            .confidence
            .as_deref()
            .and_then(analytical_results::Confidence::from_str)
            .is_none()
        {
            return Err(AppError::Config(format!(
                "stored impact assessment {} has invalid confidence",
                record.id
            )));
        }
    }
    Ok(current)
}

// ---------------------------------------------------------------------
// IOCs
// ---------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
pub async fn create_ioc(
    services: &AppServices,
    incident_id: &str,
    evidence_id: &str,
    ioc_type: &str,
    value: &str,
    event_context: Option<&str>,
    confidence: &str,
    observed_at: Option<&str>,
) -> AppResult<String> {
    incident_exists(&services.db, incident_id).await?;
    let ioc_type = ioc_recommendations::IocType::from_str(ioc_type)
        .ok_or_else(|| AppError::Config("invalid ioc_type".into()))?;
    let confidence = analytical_results::Confidence::from_str(confidence)
        .ok_or_else(|| AppError::Config("invalid confidence".into()))?;
    let id = ioc_recommendations::record_ioc_sighting(
        &services.db,
        ioc_recommendations::NewIocSighting {
            incident_id,
            evidence_id,
            ioc_type,
            raw_value: value,
            event_context,
            confidence,
            observed_at,
            actor_id: None,
        },
    )
    .await
    .map_err(AppError::Database)?;
    Ok(id)
}

fn validate_stored_ioc(record: &AnalyticalResultResponse) -> AppResult<()> {
    let invalid = |field| AppError::Config(format!("stored IOC {} has invalid {field}", record.id));
    let ioc_type = record
        .payload
        .get("ioc_type")
        .and_then(serde_json::Value::as_str)
        .and_then(ioc_recommendations::IocType::from_str)
        .ok_or_else(|| invalid("ioc_type"))?;
    let normalized_value = record
        .payload
        .get("normalized_value")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| invalid("normalized_value"))?;
    let normalized = ioc_recommendations::normalize_ioc_value(ioc_type, normalized_value)
        .map_err(|_| invalid("normalized_value"))?;
    if normalized != normalized_value {
        return Err(invalid("normalized_value"));
    }
    let expected_key = format!(
        "ioc:{}:{}:{}",
        record.incident_id.as_deref().unwrap_or_default(),
        ioc_type.as_str(),
        normalized
    );
    if record.correlation_key != expected_key {
        return Err(invalid("correlation_key"));
    }
    Ok(())
}

async fn current_iocs(
    services: &AppServices,
    incident_id: &str,
) -> AppResult<Vec<AnalyticalResultResponse>> {
    let records = current_results_for_incident(&services.db, incident_id, "IOC").await?;
    for record in &records {
        validate_stored_ioc(record)?;
    }
    Ok(records)
}

pub async fn list_iocs(
    services: &AppServices,
    incident_id: &str,
) -> AppResult<Vec<AnalyticalResultResponse>> {
    incident_exists(&services.db, incident_id).await?;
    current_iocs(services, incident_id).await
}

// ---------------------------------------------------------------------
// Recomendações
// ---------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
pub async fn create_recommendation(
    services: &AppServices,
    incident_id: &str,
    category: &str,
    title: &str,
    description: &str,
    confidence: Option<&str>,
    supporting_result_ids: &[String],
    supersedes: Option<&str>,
) -> AppResult<String> {
    incident_exists(&services.db, incident_id).await?;
    let category = ioc_recommendations::RecommendationCategory::from_str(category)
        .ok_or_else(|| AppError::Config("category must be IMMEDIATE or STRATEGIC".into()))?;
    let confidence = match confidence {
        Some(raw) => Some(
            analytical_results::Confidence::from_str(raw)
                .ok_or_else(|| AppError::Config(format!("invalid confidence: {raw}")))?,
        ),
        None => None,
    };
    let supporting_result_ids: Vec<&str> =
        supporting_result_ids.iter().map(String::as_str).collect();
    let id = ioc_recommendations::insert_recommendation(
        &services.db,
        ioc_recommendations::NewRecommendation {
            incident_id,
            category,
            title,
            description,
            confidence,
            supporting_result_ids: supporting_result_ids.as_slice(),
            supersedes,
            actor_id: None,
        },
    )
    .await
    .map_err(AppError::Database)?;
    Ok(id)
}

fn stored_recommendation_category(
    record: &AnalyticalResultResponse,
) -> AppResult<ioc_recommendations::RecommendationCategory> {
    record
        .payload
        .get("category")
        .and_then(serde_json::Value::as_str)
        .and_then(ioc_recommendations::RecommendationCategory::from_str)
        .ok_or_else(|| {
            AppError::Config(format!(
                "stored recommendation {} has invalid category",
                record.id
            ))
        })
}

pub async fn list_recommendations(
    services: &AppServices,
    incident_id: &str,
    category: Option<&str>,
) -> AppResult<Vec<AnalyticalResultResponse>> {
    incident_exists(&services.db, incident_id).await?;
    let recommendations =
        current_results_for_incident(&services.db, incident_id, "RECOMMENDATION").await?;
    let mut categorized = recommendations
        .into_iter()
        .map(|record| {
            let category = stored_recommendation_category(&record)?;
            Ok((record, category))
        })
        .collect::<AppResult<Vec<_>>>()?;
    if let Some(category) = category {
        let category = ioc_recommendations::RecommendationCategory::from_str(category)
            .ok_or_else(|| AppError::Config("category must be IMMEDIATE or STRATEGIC".into()))?;
        categorized.retain(|(_, current)| *current == category);
    }
    Ok(categorized.into_iter().map(|(record, _)| record).collect())
}

// ---------------------------------------------------------------------
// Exportação
// ---------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct ExportBundle {
    pub incident_id: String,
    pub iocs: Vec<AnalyticalResultResponse>,
    pub recommendations_immediate: Vec<AnalyticalResultResponse>,
    pub recommendations_strategic: Vec<AnalyticalResultResponse>,
}

/// Exporta achados de um incidente (IOCs + recomendações). Reusa
/// deliberadamente `current_results_for_incident` — a mesma função que
/// alimenta os painéis da interface — para que a exportação nunca divirja
/// da origem exibida na tela (RF-022: "Exportação preserva a mesma origem
/// usada na UI").
pub async fn export_incident_findings(
    services: &AppServices,
    incident_id: &str,
) -> AppResult<ExportBundle> {
    incident_exists(&services.db, incident_id).await?;
    let iocs = current_iocs(services, incident_id).await?;
    let recommendations =
        current_results_for_incident(&services.db, incident_id, "RECOMMENDATION").await?;
    let mut immediate = Vec::new();
    let mut strategic = Vec::new();
    for record in recommendations {
        match stored_recommendation_category(&record)? {
            ioc_recommendations::RecommendationCategory::Immediate => immediate.push(record),
            ioc_recommendations::RecommendationCategory::Strategic => strategic.push(record),
        }
    }
    Ok(ExportBundle {
        incident_id: incident_id.to_string(),
        iocs,
        recommendations_immediate: immediate,
        recommendations_strategic: strategic,
    })
}

// ---------------------------------------------------------------------
// Achados, conclusão de causa e matriz de risco (F1-25, RF-051)
// ---------------------------------------------------------------------

/// Dados de um achado como a interface os recebe (tudo texto).
#[derive(Debug, Clone, Default)]
pub struct FindingInput {
    pub evidence_id: Option<String>,
    pub kind: String,
    pub title: String,
    pub description: String,
    pub severity: String,
    pub basis: String,
    pub reference: Option<String>,
    pub nist_subcategory_ids: Vec<String>,
    pub attack_mapping_ids: Vec<String>,
    pub supporting_result_ids: Vec<String>,
    pub confidence: String,
    pub analyst_note: Option<String>,
    pub supersedes: Option<String>,
}

/// Dados de uma conclusão de causa como a interface os recebe.
#[derive(Debug, Clone, Default)]
pub struct RootCauseInput {
    pub level: String,
    pub statement: String,
    pub basis: String,
    pub gaps: Vec<String>,
    pub finding_ids: Vec<String>,
    pub confidence: String,
    pub supersedes: Option<String>,
}

/// Campo opcional em branco é ausência, não string vazia.
pub(super) fn non_blank(value: &Option<String>) -> Option<&str> {
    value.as_deref().map(str::trim).filter(|v| !v.is_empty())
}

fn parse_confidence(raw: &str) -> AppResult<analytical_results::Confidence> {
    analytical_results::Confidence::from_str(raw)
        .ok_or_else(|| AppError::Config(format!("invalid confidence: {raw}")))
}

pub async fn create_finding(
    services: &AppServices,
    incident_id: &str,
    input: &FindingInput,
) -> AppResult<String> {
    incident_exists(&services.db, incident_id).await?;
    let kind = finding_model::FindingKind::from_str(&input.kind)
        .ok_or_else(|| AppError::Config(format!("invalid finding kind: {}", input.kind)))?;
    let severity = attack_impact::ImpactLevel::from_str(&input.severity)
        .ok_or_else(|| AppError::Config(format!("invalid severity: {}", input.severity)))?;
    let confidence = parse_confidence(&input.confidence)?;
    let nist: Vec<&str> = input
        .nist_subcategory_ids
        .iter()
        .map(String::as_str)
        .collect();
    let attack: Vec<&str> = input
        .attack_mapping_ids
        .iter()
        .map(String::as_str)
        .collect();
    let supporting: Vec<&str> = input
        .supporting_result_ids
        .iter()
        .map(String::as_str)
        .collect();
    let id = finding_model::insert_finding(
        &services.db,
        finding_model::NewFinding {
            incident_id,
            evidence_id: non_blank(&input.evidence_id),
            kind,
            title: &input.title,
            description: &input.description,
            severity,
            basis: &input.basis,
            reference: non_blank(&input.reference),
            nist_subcategory_ids: nist.as_slice(),
            attack_mapping_ids: attack.as_slice(),
            supporting_result_ids: supporting.as_slice(),
            confidence,
            analyst_note: non_blank(&input.analyst_note),
            supersedes: non_blank(&input.supersedes),
            actor_id: None,
        },
    )
    .await
    .map_err(AppError::Database)?;
    Ok(id)
}

pub async fn create_root_cause(
    services: &AppServices,
    incident_id: &str,
    input: &RootCauseInput,
) -> AppResult<String> {
    incident_exists(&services.db, incident_id).await?;
    let level = finding_model::CauseLevel::from_str(&input.level)
        .ok_or_else(|| AppError::Config(format!("invalid cause level: {}", input.level)))?;
    let confidence = parse_confidence(&input.confidence)?;
    let gaps: Vec<&str> = input.gaps.iter().map(String::as_str).collect();
    let finding_ids: Vec<&str> = input.finding_ids.iter().map(String::as_str).collect();
    let id = finding_model::insert_root_cause(
        &services.db,
        finding_model::NewRootCause {
            incident_id,
            level,
            statement: &input.statement,
            basis: &input.basis,
            gaps: gaps.as_slice(),
            finding_ids: finding_ids.as_slice(),
            confidence,
            supersedes: non_blank(&input.supersedes),
            actor_id: None,
        },
    )
    .await
    .map_err(AppError::Database)?;
    Ok(id)
}

fn risk_matrix_of(
    findings: &[AnalyticalResultResponse],
) -> AppResult<Vec<finding_model::RiskCell>> {
    let pairs = findings
        .iter()
        .map(|finding| {
            let invalid = |field| {
                AppError::Config(format!("stored finding {} has invalid {field}", finding.id))
            };
            let severity = finding
                .payload
                .get("severity")
                .and_then(serde_json::Value::as_str)
                .and_then(attack_impact::ImpactLevel::from_str)
                .filter(|severity| *severity != attack_impact::ImpactLevel::None)
                .ok_or_else(|| invalid("severity"))?;
            let confidence = finding
                .confidence
                .as_deref()
                .and_then(analytical_results::Confidence::from_str)
                .ok_or_else(|| invalid("confidence"))?;
            Ok((severity, confidence))
        })
        .collect::<AppResult<Vec<_>>>()?;
    Ok(finding_model::risk_matrix(&pairs))
}

/// Tudo que a aba "Achados" mostra, numa só consulta: achados e conclusões
/// de causa vigentes, a matriz de risco e os mapeamentos ATT&CK vigentes (a
/// interface os oferece para ligar a um achado).
#[derive(Debug, Clone, Serialize)]
pub struct FindingsView {
    pub findings: Vec<AnalyticalResultResponse>,
    pub root_causes: Vec<AnalyticalResultResponse>,
    pub attack_mappings: Vec<AnalyticalResultResponse>,
    pub risk_matrix: Vec<finding_model::RiskCell>,
}

pub async fn findings_view(services: &AppServices, incident_id: &str) -> AppResult<FindingsView> {
    incident_exists(&services.db, incident_id).await?;
    let findings = current_results_for_incident(&services.db, incident_id, "FINDING").await?;
    let root_causes = current_results_for_incident(&services.db, incident_id, "ROOT_CAUSE").await?;
    let attack_mappings =
        current_results_for_incident(&services.db, incident_id, "ATTACK_MAPPING").await?;
    let risk_matrix = risk_matrix_of(&findings)?;
    Ok(FindingsView {
        findings,
        root_causes,
        attack_mappings,
        risk_matrix,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::test_support::fixture;

    fn finding_input(title: &str, severity: &str, confidence: &str) -> FindingInput {
        FindingInput {
            kind: "MISSING_CONTROL".into(),
            title: title.into(),
            description: "RDP exposto sem MFA".into(),
            severity: severity.into(),
            basis: "EVTX Security 4625 em massa seguido de 4624 tipo 10".into(),
            confidence: confidence.into(),
            ..FindingInput::default()
        }
    }

    async fn critical_path_stage(
        services: &AppServices,
        incident_id: &str,
        sequence_index: i64,
    ) -> String {
        create_critical_path_stage(
            services,
            incident_id,
            None,
            sequence_index,
            &format!("stage {sequence_index}"),
            "documented event sequence",
            "MEDIUM",
        )
        .await
        .expect("critical path stage")
    }

    #[tokio::test]
    async fn findings_causes_and_risk_matrix_round_trip_through_the_service() {
        let fx = fixture().await;
        let incident = crate::service::create_incident(&fx.services, "incidente")
            .await
            .expect("incident");

        let high = create_finding(
            &fx.services,
            &incident.id,
            &FindingInput {
                // Campos opcionais em branco valem como ausentes.
                evidence_id: Some("   ".into()),
                reference: Some("".into()),
                nist_subcategory_ids: vec!["PR.AA-03".into()],
                ..finding_input("RDP sem MFA", "CRITICAL", "HIGH")
            },
        )
        .await
        .expect("finding");
        create_finding(
            &fx.services,
            &incident.id,
            &finding_input("Backup exposto", "LOW", "LOW"),
        )
        .await
        .expect("second finding");

        let view = findings_view(&fx.services, &incident.id)
            .await
            .expect("view");
        assert_eq!(view.findings.len(), 2);
        let shape: Vec<(&str, &str, usize)> = view
            .risk_matrix
            .iter()
            .map(|c| (c.severity.as_str(), c.confidence.as_str(), c.count))
            .collect();
        assert_eq!(shape, vec![("CRITICAL", "HIGH", 1), ("LOW", "LOW", 1)]);

        let cause = create_root_cause(
            &fx.services,
            &incident.id,
            &RootCauseInput {
                level: "ROOT_CAUSE".into(),
                statement: "Acesso inicial por RDP exposto sem MFA".into(),
                basis: "Finding de RDP e a sequência de logons".into(),
                gaps: vec!["sem logs do firewall".into()],
                finding_ids: vec![high],
                confidence: "HIGH".into(),
                supersedes: None,
            },
        )
        .await
        .expect("cause");
        let view = findings_view(&fx.services, &incident.id)
            .await
            .expect("view");
        assert_eq!(view.root_causes.len(), 1);
        assert_eq!(view.root_causes[0].id, cause);
        assert_eq!(view.root_causes[0].payload["gap_status"], "DECLARED");
        assert_eq!(view.findings.len(), 2);
        assert_eq!(view.risk_matrix.len(), 2);
        assert!(view.attack_mappings.is_empty());
    }

    #[tokio::test]
    async fn critical_path_links_stay_inside_requested_incident() {
        let fx = fixture().await;
        let incident_a = crate::service::create_incident(&fx.services, "incident A")
            .await
            .expect("incident A");
        let incident_b = crate::service::create_incident(&fx.services, "incident B")
            .await
            .expect("incident B");
        let a0 = critical_path_stage(&fx.services, &incident_a.id, 0).await;
        let a1 = critical_path_stage(&fx.services, &incident_a.id, 1).await;
        let b0 = critical_path_stage(&fx.services, &incident_b.id, 0).await;
        let b1 = critical_path_stage(&fx.services, &incident_b.id, 1).await;

        let cross_incident =
            link_critical_path_stages(&fx.services, &incident_a.id, &a0, &b0).await;
        let foreign_pair = link_critical_path_stages(&fx.services, &incident_a.id, &b0, &b1).await;
        let graph = critical_path(&fx.services, &incident_a.id)
            .await
            .expect("incident A graph");
        let foreign_targets = graph
            .edges
            .iter()
            .filter(|edge| {
                let from_is_local = graph.stages.iter().any(|stage| stage.id == edge.from);
                let to_is_local = graph.stages.iter().any(|stage| stage.id == edge.to);
                !from_is_local || !to_is_local
            })
            .count();
        let foreign_relations = analytical_results::related_to(&fx.services.db, &b0)
            .await
            .expect("foreign relations")
            .len();
        assert!(
            cross_incident.is_err() && foreign_pair.is_err(),
            "cross_incident_accepted={} foreign_pair_accepted={} graph_edges={} foreign_targets={} foreign_relations={}",
            cross_incident.is_ok(),
            foreign_pair.is_ok(),
            graph.edges.len(),
            foreign_targets,
            foreign_relations
        );
        assert!(graph.edges.is_empty());
        assert_eq!(foreign_relations, 0);
        for result in [&cross_incident, &foreign_pair] {
            let message = result.as_ref().expect_err("invalid link").user_message();
            assert!(
                message.contains("different incident"),
                "unexpected diagnostic: {message}"
            );
        }

        link_critical_path_stages(&fx.services, &incident_a.id, &a0, &a1)
            .await
            .expect("same-incident link");
        let graph = critical_path(&fx.services, &incident_a.id)
            .await
            .expect("valid incident A graph");
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.edges[0].from, a0);
        assert_eq!(graph.edges[0].to, a1);
    }

    async fn insert_current_impact_successor(
        services: &AppServices,
        label: &str,
        payload: serde_json::Value,
        confidence: Option<&str>,
    ) -> (String, String) {
        let incident = crate::service::create_incident(services, label)
            .await
            .expect("incident");
        let original = create_impact_assessment(
            services,
            &incident.id,
            "LOW",
            "MEDIUM",
            "HIGH",
            "assets",
            "data",
            "disruption",
            None,
            "HIGH",
        )
        .await
        .expect("original impact assessment");
        let bad_id = format!("bad-impact-{label}");
        sqlx::query("INSERT INTO analytical_results(id,schema_version,result_type,correlation_key,incident_id,confidence,payload,provenance,supersedes,created_at) VALUES(?,'analytical-result-v1','IMPACT_ASSESSMENT',?,?,?,?,'{}',?,'2026-11-19T00:00:00Z')")
            .bind(&bad_id)
            .bind(format!("impact-assessment:{}", incident.id))
            .bind(&incident.id)
            .bind(confidence)
            .bind(payload.to_string())
            .bind(&original)
            .execute(&services.db)
            .await
            .expect("malformed current impact assessment");
        (incident.id, bad_id)
    }

    fn valid_stored_ioc_payload() -> serde_json::Value {
        serde_json::json!({
            "kind": "IOC",
            "ioc_type": "IP",
            "normalized_value": "203.0.113.5",
        })
    }

    async fn assert_stored_ioc_rejected(
        services: &AppServices,
        incident_id: &str,
        bad_id: &str,
        field: &str,
    ) {
        let errors = [
            list_iocs(services, incident_id)
                .await
                .expect_err("list accepted malformed IOC"),
            export_incident_findings(services, incident_id)
                .await
                .expect_err("export accepted malformed IOC"),
        ];
        for error in errors {
            assert!(
                matches!(&error, AppError::Config(message) if message.contains(bad_id) && message.contains(field)),
                "unexpected error for {field} on {bad_id}: {error:?}"
            );
        }
    }

    fn assert_stored_impact_error(error: &AppError, id: &str, field: &str) {
        assert!(
            matches!(error, AppError::Config(message) if message.contains(id) && message.contains(field)),
            "unexpected error for {field} on {id}: {error:?}"
        );
    }

    #[tokio::test]
    async fn stored_impact_assessment_rejects_invalid_levels() {
        let fx = fixture().await;
        for field in [
            "availability_impact",
            "confidentiality_impact",
            "integrity_impact",
        ] {
            for (form, value) in [
                ("unknown", serde_json::json!("SEVERE")),
                ("missing", serde_json::Value::Null),
                ("non-string", serde_json::json!(7)),
            ] {
                let label = format!("{}-{form}", field.replace('_', "-"));
                let mut payload = serde_json::json!({
                    "kind": "IMPACT_ASSESSMENT",
                    "availability_impact": "LOW",
                    "confidentiality_impact": "MEDIUM",
                    "integrity_impact": "HIGH",
                    "affected_assets": "assets",
                    "affected_data": "data",
                    "operational_disruption": "disruption",
                    "financial_impact": null,
                    "financial_impact_known": false,
                });
                if form == "missing" {
                    payload
                        .as_object_mut()
                        .expect("payload object")
                        .remove(field);
                } else {
                    payload[field] = value;
                }
                let (incident_id, bad_id) =
                    insert_current_impact_successor(&fx.services, &label, payload, Some("HIGH"))
                        .await;

                let result = current_impact_assessment(&fx.services, &incident_id).await;
                let Err(error) = result else {
                    panic!("{form} {field} was returned by the reader: {result:?}");
                };
                assert_stored_impact_error(&error, &bad_id, field);
                let delivered =
                    crate::service::deliverables::collect(&fx.services, &incident_id).await;
                let Err(error) = delivered else {
                    panic!("{form} {field} reached deliverables");
                };
                assert_stored_impact_error(&error, &bad_id, field);
            }
        }
    }

    #[tokio::test]
    async fn stored_impact_assessment_requires_confidence() {
        let fx = fixture().await;
        let payload = serde_json::json!({
            "kind": "IMPACT_ASSESSMENT",
            "availability_impact": "LOW",
            "confidentiality_impact": "MEDIUM",
            "integrity_impact": "HIGH",
        });
        let (incident_id, bad_id) =
            insert_current_impact_successor(&fx.services, "missing-confidence", payload, None)
                .await;

        let result = current_impact_assessment(&fx.services, &incident_id).await;
        let Err(error) = result else {
            panic!("missing confidence was returned by the reader: {result:?}");
        };
        assert_stored_impact_error(&error, &bad_id, "confidence");
    }

    #[tokio::test]
    async fn stored_impact_assessment_accepts_none_levels_and_null_financial_impact() {
        let fx = fixture().await;
        let incident = crate::service::create_incident(&fx.services, "valid NONE control")
            .await
            .expect("incident");
        create_impact_assessment(
            &fx.services,
            &incident.id,
            "NONE",
            "NONE",
            "NONE",
            "assets",
            "data",
            "disruption",
            None,
            "CONFIRMED",
        )
        .await
        .expect("valid impact assessment");

        let current = current_impact_assessment(&fx.services, &incident.id)
            .await
            .expect("valid current impact assessment")
            .expect("impact assessment exists");
        for field in [
            "availability_impact",
            "confidentiality_impact",
            "integrity_impact",
        ] {
            assert_eq!(current.payload[field], "NONE");
        }
        assert_eq!(current.confidence.as_deref(), Some("CONFIRMED"));
        assert!(current.payload["financial_impact"].is_null());
        assert_eq!(current.payload["financial_impact_known"], false);
        let delivered = crate::service::deliverables::collect(&fx.services, &incident.id)
            .await
            .expect("valid deliverables");
        assert_eq!(delivered.impact.expect("deliverable impact").id, current.id);
    }

    async fn insert_current_ioc_successor(
        services: &AppServices,
        label: &str,
        payload: serde_json::Value,
        correlation_key: Option<&str>,
    ) -> (String, String) {
        let incident = crate::service::create_incident(services, label)
            .await
            .expect("incident");
        let evidence_id = format!("evidence-ioc-{label}");
        sqlx::query("INSERT INTO evidence(id,original_filename,storage_reference,format,size_bytes,blake3,sha256,imported_at,status) VALUES(?,?,?,?,?,?,?,?,?)")
            .bind(&evidence_id)
            .bind("test.bin")
            .bind(format!("evidence/{evidence_id}.bin"))
            .bind("bin")
            .bind(1_i64)
            .bind("b3")
            .bind("sha")
            .bind("2026-11-19T00:00:00Z")
            .bind("FORENSIC")
            .execute(&services.db)
            .await
            .expect("evidence");
        let original = create_ioc(
            services,
            &incident.id,
            &evidence_id,
            "IP",
            "203.0.113.5",
            None,
            "HIGH",
            Some("2026-11-19T00:00:00Z"),
        )
        .await
        .expect("original IOC");
        let bad_id = format!("bad-ioc-{label}");
        let correlation_key = correlation_key
            .map(str::to_string)
            .unwrap_or_else(|| format!("ioc:{}:IP:203.0.113.5", incident.id));
        sqlx::query("INSERT INTO analytical_results(id,schema_version,result_type,correlation_key,evidence_id,incident_id,confidence,payload,provenance,supersedes,created_at) VALUES(?,'analytical-result-v1','IOC',?,?,?,?,?,'{}',?,'2026-11-19T00:00:00Z')")
            .bind(&bad_id)
            .bind(correlation_key)
            .bind(&evidence_id)
            .bind(&incident.id)
            .bind("HIGH")
            .bind(payload.to_string())
            .bind(&original)
            .execute(&services.db)
            .await
            .expect("malformed current IOC");
        (incident.id, bad_id)
    }

    #[tokio::test]
    async fn stored_ioc_rejects_divergent_payload_identity() {
        let fx = fixture().await;
        let (incident_id, bad_id) = insert_current_ioc_successor(
            &fx.services,
            "divergent-identity",
            serde_json::json!({
                "kind": "IOC",
                "ioc_type": "DOMAIN",
                "normalized_value": "evil.example",
            }),
            None,
        )
        .await;

        let listed = list_iocs(&fx.services, &incident_id).await;
        let exported = export_incident_findings(&fx.services, &incident_id).await;
        let listed_shape = match &listed {
            Ok(rows) => format!(
                "Ok({:?})",
                rows.iter().map(|row| row.id.as_str()).collect::<Vec<_>>()
            ),
            Err(error) => format!("Err({error})"),
        };
        let export_shape = match &exported {
            Ok(bundle) => format!(
                "Ok({:?})",
                bundle
                    .iocs
                    .iter()
                    .map(|row| row.id.as_str())
                    .collect::<Vec<_>>()
            ),
            Err(error) => format!("Err({error})"),
        };
        let (Err(list_error), Err(export_error)) = (listed, exported) else {
            panic!("divergent IOC identity accepted: list={listed_shape} export={export_shape}");
        };
        for error in [list_error, export_error] {
            assert!(
                matches!(&error, AppError::Config(message) if message.contains(&bad_id) && message.contains("correlation_key")),
                "unexpected divergent identity error: {error:?}"
            );
        }
    }

    #[tokio::test]
    async fn stored_ioc_rejects_invalid_ioc_type() {
        let fx = fixture().await;
        for (label, value) in [
            ("unknown", serde_json::json!("ASN")),
            ("missing", serde_json::Value::Null),
            ("non-string", serde_json::json!(7)),
        ] {
            let mut payload = valid_stored_ioc_payload();
            if label == "missing" {
                payload
                    .as_object_mut()
                    .expect("payload object")
                    .remove("ioc_type");
            } else {
                payload["ioc_type"] = value;
            }
            let (incident_id, bad_id) =
                insert_current_ioc_successor(&fx.services, label, payload, None).await;
            assert_stored_ioc_rejected(&fx.services, &incident_id, &bad_id, "ioc_type").await;
        }
    }

    #[tokio::test]
    async fn stored_ioc_rejects_invalid_normalized_value() {
        let fx = fixture().await;
        for (label, value) in [
            ("missing", serde_json::Value::Null),
            ("non-string", serde_json::json!(7)),
            ("structural", serde_json::json!("not-an-ip")),
            ("non-normalized", serde_json::json!(" 203.0.113.5 ")),
        ] {
            let mut payload = valid_stored_ioc_payload();
            if label == "missing" {
                payload
                    .as_object_mut()
                    .expect("payload object")
                    .remove("normalized_value");
            } else {
                payload["normalized_value"] = value;
            }
            let bad_label = format!("normalized-{label}");
            let (incident_id, bad_id) =
                insert_current_ioc_successor(&fx.services, &bad_label, payload, None).await;
            assert_stored_ioc_rejected(&fx.services, &incident_id, &bad_id, "normalized_value")
                .await;
        }
    }

    #[tokio::test]
    async fn stored_ioc_rejects_mismatched_correlation_key() {
        let fx = fixture().await;
        let (incident_id, bad_id) = insert_current_ioc_successor(
            &fx.services,
            "wrong-key",
            valid_stored_ioc_payload(),
            Some("ioc:wrong:IP:203.0.113.5"),
        )
        .await;
        assert_stored_ioc_rejected(&fx.services, &incident_id, &bad_id, "correlation_key").await;
    }

    #[tokio::test]
    async fn stored_ioc_accepts_valid_legacy_payload() {
        let fx = fixture().await;
        let (incident_id, current_id) = insert_current_ioc_successor(
            &fx.services,
            "valid-legacy",
            valid_stored_ioc_payload(),
            None,
        )
        .await;

        let listed = list_iocs(&fx.services, &incident_id)
            .await
            .expect("valid legacy IOC list");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, current_id);
        assert!(listed[0].payload.get("sighting_count").is_none());
        let exported = export_incident_findings(&fx.services, &incident_id)
            .await
            .expect("valid legacy IOC export");
        assert_eq!(exported.iocs.len(), 1);
        assert_eq!(exported.iocs[0].id, current_id);
        assert!(exported.iocs[0].payload.get("sighting_count").is_none());
    }

    #[tokio::test]
    async fn malformed_stored_recommendation_categories_fail_before_listing_or_export() {
        let fx = fixture().await;
        for (label, category) in [
            ("unknown", serde_json::json!("URGENT")),
            ("missing", serde_json::Value::Null),
            ("non-string", serde_json::json!(7)),
        ] {
            let incident = crate::service::create_incident(&fx.services, label)
                .await
                .expect("incident");
            let bad_id = format!("bad-recommendation-category-{label}");
            let mut payload = serde_json::json!({
                "kind": "RECOMMENDATION",
                "category": "IMMEDIATE",
                "title": "Ação com categoria inválida",
                "description": "fixture de dado persistido inválido",
            });
            if label == "missing" {
                payload
                    .as_object_mut()
                    .expect("payload object")
                    .remove("category");
            } else {
                payload["category"] = category;
            }
            sqlx::query("INSERT INTO analytical_results(id,schema_version,result_type,correlation_key,incident_id,payload,provenance,created_at) VALUES(?,'analytical-result-v1','RECOMMENDATION',?,?,?,?,'2026-11-19T00:00:00Z')")
                .bind(&bad_id)
                .bind(format!("recommendation:{}:{label}", incident.id))
                .bind(&incident.id)
                .bind(payload.to_string())
                .bind("{}")
                .execute(&fx.services.db)
                .await
                .expect("malformed current recommendation");

            let unfiltered = list_recommendations(&fx.services, &incident.id, None).await;
            let filtered =
                list_recommendations(&fx.services, &incident.id, Some("IMMEDIATE")).await;
            let exported = export_incident_findings(&fx.services, &incident.id).await;
            let unfiltered_shape = match &unfiltered {
                Ok(rows) => format!("Ok({})", rows.len()),
                Err(error) => format!("Err({error})"),
            };
            let filtered_shape = match &filtered {
                Ok(rows) => format!("Ok({})", rows.len()),
                Err(error) => format!("Err({error})"),
            };
            let export_shape = match &exported {
                Ok(bundle) => format!(
                    "Ok(immediate={}, strategic={})",
                    bundle.recommendations_immediate.len(),
                    bundle.recommendations_strategic.len()
                ),
                Err(error) => format!("Err({error})"),
            };
            let (Err(unfiltered_error), Err(filtered_error), Err(export_error)) =
                (unfiltered, filtered, exported)
            else {
                panic!(
                    "malformed category accepted for {label}: unfiltered={unfiltered_shape} filtered={filtered_shape} export={export_shape}"
                );
            };
            for error in [unfiltered_error, filtered_error, export_error] {
                assert!(
                    matches!(&error, AppError::Config(message) if message.contains("category") && message.contains(&bad_id)),
                    "unexpected error for {label}: {error:?}"
                );
            }
        }
    }

    #[tokio::test]
    async fn valid_recommendations_keep_their_filter_and_export_partition() {
        let fx = fixture().await;
        let incident = crate::service::create_incident(&fx.services, "categorias válidas")
            .await
            .expect("incident");
        let finding = create_finding(
            &fx.services,
            &incident.id,
            &finding_input("RDP sem MFA", "HIGH", "MEDIUM"),
        )
        .await
        .expect("finding");
        for (category, title) in [("IMMEDIATE", "Isolar host"), ("STRATEGIC", "Implantar EDR")] {
            create_recommendation(
                &fx.services,
                &incident.id,
                category,
                title,
                "ação motivada pelo achado",
                Some("MEDIUM"),
                std::slice::from_ref(&finding),
                None,
            )
            .await
            .expect("valid recommendation");
        }

        assert_eq!(
            list_recommendations(&fx.services, &incident.id, Some("IMMEDIATE"))
                .await
                .expect("immediate filter")
                .len(),
            1
        );
        assert_eq!(
            list_recommendations(&fx.services, &incident.id, Some("STRATEGIC"))
                .await
                .expect("strategic filter")
                .len(),
            1
        );
        let exported = export_incident_findings(&fx.services, &incident.id)
            .await
            .expect("valid export");
        assert_eq!(exported.recommendations_immediate.len(), 1);
        assert_eq!(exported.recommendations_strategic.len(), 1);
    }

    #[tokio::test]
    async fn service_rejects_unknown_incident_and_invalid_vocabulary() {
        let fx = fixture().await;
        assert!(
            create_finding(&fx.services, "nope", &finding_input("x", "HIGH", "HIGH"))
                .await
                .is_err()
        );
        let incident = crate::service::create_incident(&fx.services, "incidente")
            .await
            .expect("incident");
        for input in [
            FindingInput {
                kind: "BOGUS".into(),
                ..finding_input("x", "HIGH", "HIGH")
            },
            finding_input("x", "SEVERE", "HIGH"),
            finding_input("x", "HIGH", "CERTAIN"),
        ] {
            assert!(
                create_finding(&fx.services, &incident.id, &input)
                    .await
                    .is_err()
            );
        }
        assert!(
            create_root_cause(
                &fx.services,
                &incident.id,
                &RootCauseInput {
                    level: "BOGUS".into(),
                    ..RootCauseInput::default()
                },
            )
            .await
            .is_err()
        );
    }
}
