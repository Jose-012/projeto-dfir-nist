//! F1-31 (RF-054, RB-001) — registro de lacunas de fonte, camada de serviço.
//!
//! As regras (fonte esperada obrigatória, período válido, motivo `OTHER`
//! com nota) vivem em `persistence/source_gaps.rs`.

use super::findings::non_blank;
use super::{AnalyticalResultResponse, AppServices, current_results_for_incident, incident_exists};
use crate::error::{AppError, AppResult};
use crate::persistence::source_gaps as gap_model;

/// Dados de uma lacuna como a interface os recebe (tudo texto).
#[derive(Debug, Clone, Default)]
pub struct SourceGapInput {
    pub expected_source: String,
    pub period_from: Option<String>,
    pub period_to: Option<String>,
    pub reason: String,
    pub note: String,
    pub supersedes: Option<String>,
}

pub async fn create_source_gap(
    services: &AppServices,
    incident_id: &str,
    input: &SourceGapInput,
) -> AppResult<String> {
    incident_exists(&services.db, incident_id).await?;
    let reason = gap_model::GapReason::from_str(&input.reason)
        .ok_or_else(|| AppError::Config(format!("invalid gap reason: {}", input.reason)))?;
    let id = gap_model::insert_source_gap(
        &services.db,
        gap_model::NewSourceGap {
            incident_id,
            expected_source: &input.expected_source,
            period_from: non_blank(&input.period_from),
            period_to: non_blank(&input.period_to),
            reason,
            note: &input.note,
            supersedes: non_blank(&input.supersedes),
            actor_id: None,
        },
    )
    .await
    .map_err(AppError::Database)?;
    Ok(id)
}

/// Lacunas vigentes do incidente, da fonte esperada e do período mais
/// antigos para os mais recentes (ordem estável: o id desempata).
pub async fn list_source_gaps(
    services: &AppServices,
    incident_id: &str,
) -> AppResult<Vec<AnalyticalResultResponse>> {
    incident_exists(&services.db, incident_id).await?;
    let mut gaps = current_results_for_incident(&services.db, incident_id, "SOURCE_GAP").await?;
    let text = |gap: &AnalyticalResultResponse, key: &str| -> String {
        gap.payload
            .get(key)
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .to_string()
    };
    gaps.sort_by(|a, b| {
        text(a, "expected_source")
            .cmp(&text(b, "expected_source"))
            .then_with(|| text(a, "period_from").cmp(&text(b, "period_from")))
            .then_with(|| a.id.cmp(&b.id))
    });
    Ok(gaps)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::test_support::fixture;

    fn input(source: &str, reason: &str) -> SourceGapInput {
        SourceGapInput {
            expected_source: source.into(),
            period_from: Some("2026-09-01".into()),
            period_to: Some("2026-09-15".into()),
            reason: reason.into(),
            note: "retenção de 7 dias".into(),
            supersedes: None,
        }
    }

    #[tokio::test]
    async fn gaps_round_trip_in_a_stable_order_and_only_the_current_revision_shows() {
        let fx = fixture().await;
        let incident = crate::service::create_incident(&fx.services, "incidente")
            .await
            .expect("incident");
        let firewall = create_source_gap(
            &fx.services,
            &incident.id,
            &input("logs do firewall", "RETENTION"),
        )
        .await
        .expect("firewall");
        create_source_gap(
            &fx.services,
            &incident.id,
            &SourceGapInput {
                period_from: Some("  ".into()),
                period_to: None,
                note: String::new(),
                ..input("logs de e-mail", "NOT_PROVIDED")
            },
        )
        .await
        .expect("mail");

        let listed = list_source_gaps(&fx.services, &incident.id)
            .await
            .expect("list");
        let sources: Vec<&str> = listed
            .iter()
            .map(|gap| gap.payload["expected_source"].as_str().unwrap_or(""))
            .collect();
        assert_eq!(sources, vec!["logs de e-mail", "logs do firewall"]);
        assert!(
            listed[0].payload["period_from"].is_null(),
            "em branco vale como ausente"
        );

        // Revisar mantém uma só lacuna vigente.
        create_source_gap(
            &fx.services,
            &incident.id,
            &SourceGapInput {
                supersedes: Some(firewall),
                ..input("logs do firewall", "NOT_COLLECTED")
            },
        )
        .await
        .expect("revision");
        let listed = list_source_gaps(&fx.services, &incident.id)
            .await
            .expect("list");
        assert_eq!(listed.len(), 2);
        let reasons: Vec<&str> = listed
            .iter()
            .filter_map(|gap| gap.payload["reason"].as_str())
            .collect();
        assert!(reasons.contains(&"NOT_COLLECTED") && !reasons.contains(&"RETENTION"));
    }

    #[tokio::test]
    async fn service_rejects_unknown_incident_reason_and_invalid_periods() {
        let fx = fixture().await;
        assert!(
            create_source_gap(&fx.services, "nope", &input("fonte", "RETENTION"))
                .await
                .is_err()
        );
        let incident = crate::service::create_incident(&fx.services, "incidente")
            .await
            .expect("incident");
        for bad in [
            input("fonte", "FORGOT"),
            SourceGapInput {
                period_from: Some("2026-10-02".into()),
                period_to: Some("2026-10-01".into()),
                ..input("fonte", "RETENTION")
            },
            input("   ", "RETENTION"),
            SourceGapInput {
                note: String::new(),
                ..input("fonte", "OTHER")
            },
        ] {
            assert!(
                create_source_gap(&fx.services, &incident.id, &bad)
                    .await
                    .is_err(),
                "{bad:?}"
            );
        }
        assert!(list_source_gaps(&fx.services, "nope").await.is_err());
    }
}
