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

fn validate_source_gap_payload(gap: &AnalyticalResultResponse) -> AppResult<()> {
    let invalid =
        |field| AppError::Config(format!("stored source gap {} has invalid {field}", gap.id));
    gap.payload
        .get("expected_source")
        .and_then(serde_json::Value::as_str)
        .filter(|source| !source.trim().is_empty())
        .ok_or_else(|| invalid("expected_source"))?;
    let period = |field| match gap.payload.get(field) {
        None | Some(serde_json::Value::Null) => Ok(None),
        Some(value) => value
            .as_str()
            .and_then(gap_model::parse_instant)
            .map(Some)
            .ok_or_else(|| invalid(field)),
    };
    let from = period("period_from")?;
    let to = period("period_to")?;
    if let (Some(from), Some(to)) = (from, to)
        && from > to
    {
        return Err(invalid("period"));
    }
    let reason = gap
        .payload
        .get("reason")
        .and_then(serde_json::Value::as_str)
        .and_then(gap_model::GapReason::from_str)
        .ok_or_else(|| invalid("reason"))?;
    if reason == gap_model::GapReason::Other
        && gap
            .payload
            .get("note")
            .and_then(serde_json::Value::as_str)
            .is_none_or(|note| note.trim().is_empty())
    {
        return Err(invalid("note"));
    }
    Ok(())
}

/// Lacunas vigentes do incidente, da fonte esperada e do período mais
/// antigos para os mais recentes (ordem estável: o id desempata).
pub async fn list_source_gaps(
    services: &AppServices,
    incident_id: &str,
) -> AppResult<Vec<AnalyticalResultResponse>> {
    incident_exists(&services.db, incident_id).await?;
    let mut gaps = current_results_for_incident(&services.db, incident_id, "SOURCE_GAP").await?;
    for gap in &gaps {
        validate_source_gap_payload(gap)?;
    }
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
    async fn malformed_current_source_gap_missing_expected_source_fails_closed() {
        let fx = fixture().await;
        let incident = crate::service::create_incident(&fx.services, "incidente")
            .await
            .expect("incident");
        let valid = create_source_gap(
            &fx.services,
            &incident.id,
            &input("logs do firewall", "RETENTION"),
        )
        .await
        .expect("valid source gap");
        create_source_gap(
            &fx.services,
            &incident.id,
            &input("logs de e-mail", "NOT_PROVIDED"),
        )
        .await
        .expect("independent valid source gap");
        let record = crate::persistence::analytical_results::get(&fx.services.db, &valid)
            .await
            .expect("get source gap")
            .expect("source gap exists");
        let mut payload: serde_json::Value =
            serde_json::from_str(&record.payload).expect("valid payload");
        payload
            .as_object_mut()
            .expect("payload object")
            .remove("expected_source");
        let bad_id = "bad-source-gap-missing-expected-source";
        sqlx::query("INSERT INTO analytical_results(id,schema_version,result_type,correlation_key,incident_id,payload,provenance,supersedes,created_at) VALUES(?,'analytical-result-v1','SOURCE_GAP',?,?,?,?,?,'2026-11-19T00:00:00Z')")
            .bind(bad_id)
            .bind(&record.correlation_key)
            .bind(&incident.id)
            .bind(payload.to_string())
            .bind(record.provenance.to_string())
            .bind(&valid)
            .execute(&fx.services.db)
            .await
            .expect("malformed current source gap");

        match list_source_gaps(&fx.services, &incident.id).await {
            Ok(gaps) => {
                let shape: Vec<(&str, Option<&serde_json::Value>)> = gaps
                    .iter()
                    .map(|gap| (gap.id.as_str(), gap.payload.get("expected_source")))
                    .collect();
                assert_eq!(
                    shape.first().map(|(id, _)| *id),
                    Some(bad_id),
                    "the missing source should receive the empty sort key: {shape:?}"
                );
                panic!("missing expected_source was accepted and sorted first: {shape:?}");
            }
            Err(error) => assert!(
                matches!(&error, AppError::Config(message) if message.contains(bad_id) && message.contains("expected_source")),
                "unexpected error: {error:?}"
            ),
        }
    }

    #[tokio::test]
    async fn malformed_current_source_gap_fields_fail_before_listing() {
        let cases = [
            (
                "non-string-source",
                "RETENTION",
                "expected_source",
                Some(serde_json::json!(7)),
                "expected_source",
            ),
            (
                "blank-source",
                "RETENTION",
                "expected_source",
                Some(serde_json::json!(" \t\n")),
                "expected_source",
            ),
            (
                "invalid-period-from",
                "RETENTION",
                "period_from",
                Some(serde_json::json!("not-a-date")),
                "period_from",
            ),
            (
                "non-string-period-from",
                "RETENTION",
                "period_from",
                Some(serde_json::json!(7)),
                "period_from",
            ),
            (
                "invalid-period-to",
                "RETENTION",
                "period_to",
                Some(serde_json::json!("2026-02-30")),
                "period_to",
            ),
            (
                "non-string-period-to",
                "RETENTION",
                "period_to",
                Some(serde_json::json!(false)),
                "period_to",
            ),
            (
                "reversed-period",
                "RETENTION",
                "period_from",
                Some(serde_json::json!("2026-10-01")),
                "period",
            ),
            (
                "unknown-reason",
                "RETENTION",
                "reason",
                Some(serde_json::json!("UNKNOWN")),
                "reason",
            ),
            ("missing-reason", "RETENTION", "reason", None, "reason"),
            (
                "non-string-reason",
                "RETENTION",
                "reason",
                Some(serde_json::json!(7)),
                "reason",
            ),
            ("other-missing-note", "OTHER", "note", None, "note"),
            (
                "other-non-string-note",
                "OTHER",
                "note",
                Some(serde_json::json!(7)),
                "note",
            ),
            (
                "other-blank-note",
                "OTHER",
                "note",
                Some(serde_json::json!("  \n")),
                "note",
            ),
        ];
        for (label, reason, field, replacement, error_field) in cases {
            let fx = fixture().await;
            let incident = crate::service::create_incident(&fx.services, label)
                .await
                .expect("incident");
            let valid = create_source_gap(
                &fx.services,
                &incident.id,
                &input("logs do firewall", reason),
            )
            .await
            .expect("valid source gap");
            let record = crate::persistence::analytical_results::get(&fx.services.db, &valid)
                .await
                .expect("get source gap")
                .expect("source gap exists");
            let mut payload: serde_json::Value =
                serde_json::from_str(&record.payload).expect("valid payload");
            if let Some(replacement) = replacement {
                payload[field] = replacement;
            } else {
                payload
                    .as_object_mut()
                    .expect("payload object")
                    .remove(field);
            }
            let bad_id = format!("bad-source-gap-{label}");
            sqlx::query("INSERT INTO analytical_results(id,schema_version,result_type,correlation_key,incident_id,payload,provenance,supersedes,created_at) VALUES(?,'analytical-result-v1','SOURCE_GAP',?,?,?,?,?,'2026-11-19T00:00:00Z')")
                .bind(&bad_id)
                .bind(&record.correlation_key)
                .bind(&incident.id)
                .bind(payload.to_string())
                .bind(record.provenance.to_string())
                .bind(&valid)
                .execute(&fx.services.db)
                .await
                .expect("malformed current source gap");

            let result = list_source_gaps(&fx.services, &incident.id).await;
            let shape = match &result {
                Ok(gaps) => format!(
                    "Ok({:?})",
                    gaps.iter().map(|gap| &gap.id).collect::<Vec<_>>()
                ),
                Err(error) => format!("Err({error})"),
            };
            let Err(error) = result else {
                panic!("{label} was accepted: {shape}");
            };
            assert!(
                matches!(&error, AppError::Config(message) if message.contains(&bad_id) && message.contains(error_field)),
                "{label}: unexpected error: {error:?}"
            );
        }
    }

    #[tokio::test]
    async fn valid_source_gaps_keep_missing_or_null_periods_and_other_note() {
        let fx = fixture().await;
        let incident = crate::service::create_incident(&fx.services, "incidente")
            .await
            .expect("incident");
        let null_periods = create_source_gap(
            &fx.services,
            &incident.id,
            &SourceGapInput {
                period_from: None,
                period_to: None,
                ..input("z-null-periods", "OTHER")
            },
        )
        .await
        .expect("valid OTHER source gap with null periods");
        let legacy = create_source_gap(
            &fx.services,
            &incident.id,
            &input("a-legacy-missing-periods", "RETENTION"),
        )
        .await
        .expect("valid source gap to revise");
        let record = crate::persistence::analytical_results::get(&fx.services.db, &legacy)
            .await
            .expect("get source gap")
            .expect("source gap exists");
        let mut payload: serde_json::Value =
            serde_json::from_str(&record.payload).expect("valid payload");
        let object = payload.as_object_mut().expect("payload object");
        object.remove("period_from");
        object.remove("period_to");
        let legacy_id = "legacy-source-gap-without-period-keys";
        sqlx::query("INSERT INTO analytical_results(id,schema_version,result_type,correlation_key,incident_id,payload,provenance,supersedes,created_at) VALUES(?,'analytical-result-v1','SOURCE_GAP',?,?,?,?,?,'2026-11-19T00:00:00Z')")
            .bind(legacy_id)
            .bind(&record.correlation_key)
            .bind(&incident.id)
            .bind(payload.to_string())
            .bind(record.provenance.to_string())
            .bind(&legacy)
            .execute(&fx.services.db)
            .await
            .expect("valid legacy current source gap");

        let listed = list_source_gaps(&fx.services, &incident.id)
            .await
            .expect("valid legacy and null-period gaps");
        let ids: Vec<&str> = listed.iter().map(|gap| gap.id.as_str()).collect();
        assert_eq!(ids, vec![legacy_id, null_periods.as_str()]);
        let legacy = &listed[0];
        assert!(legacy.payload.get("period_from").is_none());
        assert!(legacy.payload.get("period_to").is_none());
        let null_periods = &listed[1];
        assert!(null_periods.payload["period_from"].is_null());
        assert!(null_periods.payload["period_to"].is_null());
        assert_eq!(null_periods.payload["reason"], "OTHER");
        assert_eq!(null_periods.payload["note"], "retenção de 7 dias");
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
