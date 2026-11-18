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
    attack_impact::link_critical_path_stages(&services.db, from_stage_id, to_stage_id)
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
    Ok(results.into_iter().next())
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

pub async fn list_iocs(
    services: &AppServices,
    incident_id: &str,
) -> AppResult<Vec<AnalyticalResultResponse>> {
    incident_exists(&services.db, incident_id).await?;
    current_results_for_incident(&services.db, incident_id, "IOC").await
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

pub async fn list_recommendations(
    services: &AppServices,
    incident_id: &str,
    category: Option<&str>,
) -> AppResult<Vec<AnalyticalResultResponse>> {
    incident_exists(&services.db, incident_id).await?;
    let mut recommendations =
        current_results_for_incident(&services.db, incident_id, "RECOMMENDATION").await?;
    if let Some(category) = category {
        let category = ioc_recommendations::RecommendationCategory::from_str(category)
            .ok_or_else(|| AppError::Config("category must be IMMEDIATE or STRATEGIC".into()))?
            .as_str();
        recommendations.retain(|record| {
            record
                .payload
                .get("category")
                .and_then(serde_json::Value::as_str)
                == Some(category)
        });
    }
    Ok(recommendations)
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
    let iocs = current_results_for_incident(&services.db, incident_id, "IOC").await?;
    let recommendations =
        current_results_for_incident(&services.db, incident_id, "RECOMMENDATION").await?;
    let (immediate, strategic): (Vec<_>, Vec<_>) =
        recommendations.into_iter().partition(|record| {
            record
                .payload
                .get("category")
                .and_then(serde_json::Value::as_str)
                == Some("IMMEDIATE")
        });
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

fn risk_matrix_of(findings: &[AnalyticalResultResponse]) -> Vec<finding_model::RiskCell> {
    let pairs: Vec<(attack_impact::ImpactLevel, analytical_results::Confidence)> = findings
        .iter()
        .filter_map(|finding| {
            let severity = finding
                .payload
                .get("severity")?
                .as_str()
                .and_then(attack_impact::ImpactLevel::from_str)?;
            let confidence = finding
                .confidence
                .as_deref()
                .and_then(analytical_results::Confidence::from_str)?;
            Some((severity, confidence))
        })
        .collect();
    finding_model::risk_matrix(&pairs)
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
    let risk_matrix = risk_matrix_of(&findings);
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
