//! F1-31 (RF-054, RB-001) — registro de lacunas de fonte.
//!
//! Uma lacuna é o que o analista **declara** que faltou: a fonte esperada
//! (ex.: "logs do firewall de borda"), o período sem dados e o motivo
//! (retenção insuficiente, não fornecido, não coletado, outro). Entra no
//! Entregável 1, em "Escopo e limitações" e na seção própria, para que o
//! relatório diga o que **não** pôde afirmar.
//!
//! Cabe no envelope de F1-01 (`AnalyticalResultType::SourceGap`, reservado
//! pela migration 0011 — ADR-0023): é append-only, versionado por
//! `supersedes` e ligado ao incidente. Não há tabela nova.
//!
//! ## Regras
//!
//! - A fonte esperada é obrigatória: lacuna sem dizer de que fonte é não
//!   informa nada.
//! - O período é opcional, mas, quando informado, é data (`AAAA-MM-DD`) ou
//!   instante RFC 3339, e o início não pode vir depois do fim.
//! - O motivo `OTHER` exige uma nota que o explique.

use super::validation_error;
use crate::persistence::analytical_results::{self, AnalyticalResultType, NewAnalyticalResult};
use crate::persistence::findings::require_result;
use chrono::{DateTime, NaiveDate, Utc};
use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GapReason {
    /// Os logs existiram, mas a retenção do cliente já os descartou.
    Retention,
    /// O cliente ou outra área não forneceu.
    NotProvided,
    /// Existe, mas não foi coletado.
    NotCollected,
    Other,
}

impl GapReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Retention => "RETENTION",
            Self::NotProvided => "NOT_PROVIDED",
            Self::NotCollected => "NOT_COLLECTED",
            Self::Other => "OTHER",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "RETENTION" => Some(Self::Retention),
            "NOT_PROVIDED" => Some(Self::NotProvided),
            "NOT_COLLECTED" => Some(Self::NotCollected),
            "OTHER" => Some(Self::Other),
            _ => None,
        }
    }

    /// Texto para os documentos.
    pub fn label_pt(self) -> &'static str {
        match self {
            Self::Retention => "retenção insuficiente de logs",
            Self::NotProvided => "não fornecido",
            Self::NotCollected => "não coletado",
            Self::Other => "outro motivo",
        }
    }
}

pub struct NewSourceGap<'a> {
    pub incident_id: &'a str,
    /// A fonte que se esperava ter.
    pub expected_source: &'a str,
    pub period_from: Option<&'a str>,
    pub period_to: Option<&'a str>,
    pub reason: GapReason,
    pub note: &'a str,
    /// Lacuna anterior que esta revisa (mesma `correlation_key`).
    pub supersedes: Option<&'a str>,
    pub actor_id: Option<&'a str>,
}

/// Data (`AAAA-MM-DD`, tomada como 00:00 UTC) ou instante RFC 3339.
pub(crate) fn parse_instant(raw: &str) -> Option<DateTime<Utc>> {
    let raw = raw.trim();
    if let Ok(instant) = DateTime::parse_from_rfc3339(raw) {
        return Some(instant.with_timezone(&Utc));
    }
    NaiveDate::parse_from_str(raw, "%Y-%m-%d")
        .ok()
        .and_then(|date| date.and_hms_opt(0, 0, 0))
        .map(|naive| naive.and_utc())
}

fn period_bound(
    raw: Option<&str>,
    which: &str,
) -> Result<Option<(String, DateTime<Utc>)>, sqlx::Error> {
    match raw.map(str::trim).filter(|value| !value.is_empty()) {
        None => Ok(None),
        Some(value) => parse_instant(value)
            .map(|instant| Some((value.to_string(), instant)))
            .ok_or_else(|| {
                validation_error(format!(
                    "period {which} must be a date (YYYY-MM-DD) or an RFC 3339 instant: {value}"
                ))
            }),
    }
}

/// Insere uma lacuna (ou uma revisão, quando `supersedes` é informado).
pub async fn insert_source_gap(
    pool: &SqlitePool,
    input: NewSourceGap<'_>,
) -> Result<String, sqlx::Error> {
    if input.expected_source.trim().is_empty() {
        return Err(validation_error(
            "a source gap requires the expected source",
        ));
    }
    if input.reason == GapReason::Other && input.note.trim().is_empty() {
        return Err(validation_error(
            "reason OTHER requires a note that explains it",
        ));
    }
    let from = period_bound(input.period_from, "start")?;
    let to = period_bound(input.period_to, "end")?;
    if let (Some((_, start)), Some((_, end))) = (&from, &to)
        && start > end
    {
        return Err(validation_error("the period starts after it ends"));
    }

    let correlation_key = match input.supersedes {
        Some(previous_id) => {
            require_result(
                pool,
                previous_id,
                Some(AnalyticalResultType::SourceGap),
                Some(input.incident_id),
                "source gap to supersede",
            )
            .await?
            .correlation_key
        }
        None => format!("gap:{}:{}", input.incident_id, Uuid::new_v4()),
    };

    let payload_json = serde_json::json!({
        "kind": "SOURCE_GAP",
        "expected_source": input.expected_source.trim(),
        "period_from": from.as_ref().map(|(value, _)| value),
        "period_to": to.as_ref().map(|(value, _)| value),
        "reason": input.reason.as_str(),
        "note": input.note.trim(),
    })
    .to_string();
    let provenance_json = serde_json::json!({
        "producer": "dfir-app",
        "method": "analyst-declared source gap",
    })
    .to_string();

    analytical_results::insert(
        pool,
        NewAnalyticalResult {
            result_type: AnalyticalResultType::SourceGap,
            correlation_key: &correlation_key,
            evidence_id: None,
            incident_id: Some(input.incident_id),
            analysis_job_id: None,
            rule_or_model_id: None,
            rule_or_model_version: None,
            confidence: None,
            payload_json: &payload_json,
            provenance_json: &provenance_json,
            supersedes: input.supersedes,
            related: &[],
            actor_id: input.actor_id,
        },
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn test_pool() -> SqlitePool {
        let temp = tempfile::tempdir().expect("temporary directory");
        let db_path = temp.path().join("gaps.sqlite");
        let url = format!("sqlite://{}", db_path.display());
        std::mem::forget(temp);
        let pool = crate::persistence::db::connect(&url)
            .await
            .expect("database connection");
        crate::persistence::db::migrate(&pool)
            .await
            .expect("migrations");
        pool
    }

    async fn seed_incident(pool: &SqlitePool, id: &str) {
        sqlx::query("INSERT INTO incidents(id,name) VALUES(?,?)")
            .bind(id)
            .bind(format!("incident {id}"))
            .execute(pool)
            .await
            .expect("incident insert");
    }

    fn gap<'a>(incident_id: &'a str, source: &'a str) -> NewSourceGap<'a> {
        NewSourceGap {
            incident_id,
            expected_source: source,
            period_from: Some("2026-09-01"),
            period_to: Some("2026-09-15T12:00:00Z"),
            reason: GapReason::Retention,
            note: "retenção de 7 dias no firewall",
            supersedes: None,
            actor_id: None,
        }
    }

    #[test]
    fn reasons_round_trip_and_have_a_label() {
        for reason in [
            GapReason::Retention,
            GapReason::NotProvided,
            GapReason::NotCollected,
            GapReason::Other,
        ] {
            assert_eq!(GapReason::from_str(reason.as_str()), Some(reason));
            assert!(!reason.label_pt().is_empty());
        }
        assert_eq!(GapReason::from_str("BOGUS"), None);
        assert_eq!(AnalyticalResultType::SourceGap.as_str(), "SOURCE_GAP");
        assert!(AnalyticalResultType::SourceGap.requires_incident());
        assert!(!AnalyticalResultType::SourceGap.requires_evidence());
        assert!(!AnalyticalResultType::SourceGap.requires_confidence());
    }

    #[tokio::test]
    async fn a_gap_is_persisted_with_its_period_and_reason() {
        let pool = test_pool().await;
        seed_incident(&pool, "inc-1").await;
        let id = insert_source_gap(&pool, gap("inc-1", "  logs do firewall de borda "))
            .await
            .expect("gap insert");
        let record = analytical_results::get(&pool, &id)
            .await
            .expect("get")
            .expect("exists");
        assert_eq!(record.result_type, "SOURCE_GAP");
        let payload: serde_json::Value = serde_json::from_str(&record.payload).expect("json");
        assert_eq!(payload["expected_source"], "logs do firewall de borda");
        assert_eq!(payload["reason"], "RETENTION");
        assert_eq!(payload["period_from"], "2026-09-01");
        assert_eq!(payload["period_to"], "2026-09-15T12:00:00Z");

        // Sem período é válido: a fonte inteira faltou.
        let whole = insert_source_gap(
            &pool,
            NewSourceGap {
                period_from: None,
                period_to: Some("  "),
                reason: GapReason::NotProvided,
                note: "",
                ..gap("inc-1", "logs do servidor de e-mail")
            },
        )
        .await
        .expect("gap without period");
        let payload = {
            let record = analytical_results::get(&pool, &whole)
                .await
                .expect("get")
                .expect("exists");
            serde_json::from_str::<serde_json::Value>(&record.payload).expect("json")
        };
        assert!(payload["period_from"].is_null() && payload["period_to"].is_null());
    }

    #[tokio::test]
    async fn invalid_gaps_are_refused() {
        let pool = test_pool().await;
        seed_incident(&pool, "inc-2").await;
        // Sem fonte esperada.
        assert!(insert_source_gap(&pool, gap("inc-2", "   ")).await.is_err());
        // Datas que não são datas.
        for bad in ["ontem", "2026-13-40", "01/09/2026"] {
            assert!(
                insert_source_gap(
                    &pool,
                    NewSourceGap {
                        period_from: Some(bad),
                        ..gap("inc-2", "fonte")
                    },
                )
                .await
                .is_err(),
                "{bad}"
            );
        }
        // Período invertido.
        assert!(
            insert_source_gap(
                &pool,
                NewSourceGap {
                    period_from: Some("2026-10-02"),
                    period_to: Some("2026-10-01"),
                    ..gap("inc-2", "fonte")
                },
            )
            .await
            .is_err()
        );
        // `OTHER` sem nota.
        assert!(
            insert_source_gap(
                &pool,
                NewSourceGap {
                    reason: GapReason::Other,
                    note: "  ",
                    ..gap("inc-2", "fonte")
                },
            )
            .await
            .is_err()
        );
        // O mesmo `OTHER` com nota passa.
        assert!(
            insert_source_gap(
                &pool,
                NewSourceGap {
                    reason: GapReason::Other,
                    note: "equipamento descartado antes da coleta",
                    ..gap("inc-2", "fonte")
                },
            )
            .await
            .is_ok()
        );
    }

    #[tokio::test]
    async fn revising_a_gap_keeps_its_key_and_only_same_type_and_incident_are_accepted() {
        let pool = test_pool().await;
        seed_incident(&pool, "inc-3").await;
        seed_incident(&pool, "inc-3b").await;
        let first = insert_source_gap(&pool, gap("inc-3", "fonte"))
            .await
            .expect("first");
        let second = insert_source_gap(
            &pool,
            NewSourceGap {
                supersedes: Some(&first),
                reason: GapReason::NotProvided,
                ..gap("inc-3", "fonte")
            },
        )
        .await
        .expect("revision");
        let first_record = analytical_results::get(&pool, &first)
            .await
            .expect("get")
            .expect("exists");
        let current =
            analytical_results::current_for_correlation_key(&pool, &first_record.correlation_key)
                .await
                .expect("current")
                .expect("has current");
        assert_eq!(current.id, second);

        // Outro incidente.
        assert!(
            insert_source_gap(
                &pool,
                NewSourceGap {
                    supersedes: Some(&first),
                    ..gap("inc-3b", "fonte")
                },
            )
            .await
            .is_err()
        );
        // Outro tipo de resultado.
        let other_type = crate::persistence::findings::insert_finding(
            &pool,
            crate::persistence::findings::NewFinding {
                incident_id: "inc-3",
                evidence_id: None,
                kind: crate::persistence::findings::FindingKind::Other,
                title: "achado",
                description: "",
                severity: crate::persistence::attack_impact::ImpactLevel::Low,
                basis: "x",
                reference: None,
                nist_subcategory_ids: &[],
                attack_mapping_ids: &[],
                supporting_result_ids: &[],
                confidence: crate::persistence::analytical_results::Confidence::Low,
                analyst_note: None,
                supersedes: None,
                actor_id: None,
            },
        )
        .await
        .expect("finding");
        assert!(
            insert_source_gap(
                &pool,
                NewSourceGap {
                    supersedes: Some(&other_type),
                    ..gap("inc-3", "fonte")
                },
            )
            .await
            .is_err()
        );
    }
}
