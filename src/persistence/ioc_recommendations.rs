//! F1-05 — IOCs e recomendações operacionais.
//!
//! Como em F1-04, nenhuma tabela nova: IOCs e recomendações são achados
//! (`AnalyticalResultType::Ioc`/`Recommendation`, F1-01). A parte nova é a
//! **normalização com agregação de provenance** para IOCs — diferente de
//! Gap/ATT&CK mapping/impacto (onde uma nova versão *substitui* a
//! anterior), uma nova ocorrência do mesmo IOC não invalida a anterior:
//! ambas continuam sendo fatos verdadeiros (o indicador foi visto nos dois
//! lugares). Ver [`record_ioc_sighting`] e a ADR-0013 para o desenho
//! completo.
//!
//! ## Validação de IOC: estrutural, não um serviço de threat intel
//!
//! `normalize_ioc_value` valida a FORMA de cada tipo (IP via
//! `std::net::IpAddr` da std, hashes por comprimento hexadecimal,
//! domínio/hostname/email/URL por checagens estruturais leves). Não há
//! parser RFC-completo de e-mail/URL nem consulta a nenhum serviço externo
//! — o objetivo é rejeitar valores obviamente inválidos e normalizar
//! maiúsculas/minúsculas/espaços para deduplicação, não validar contra uma
//! lista de reputação. Mesma disciplina de "necessidade demonstrada" das
//! ADRs 0011/0012, aplicada aqui à profundidade de validação em vez de a um
//! catálogo de referência.

use super::validation_error;
use crate::persistence::analytical_results::{
    self, AnalyticalResultType, Confidence, NewAnalyticalResult, RelationType,
};
use sqlx::SqlitePool;
use uuid::Uuid;

// ---------------------------------------------------------------------
// IOCs
// ---------------------------------------------------------------------

/// Tipos mínimos exigidos pela seção 17 da spec canônica.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IocType {
    Ip,
    Domain,
    Url,
    Email,
    Sha256,
    Sha1,
    Md5,
    FilePath,
    Hostname,
}

impl IocType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ip => "IP",
            Self::Domain => "DOMAIN",
            Self::Url => "URL",
            Self::Email => "EMAIL",
            Self::Sha256 => "SHA256",
            Self::Sha1 => "SHA1",
            Self::Md5 => "MD5",
            Self::FilePath => "FILE_PATH",
            Self::Hostname => "HOSTNAME",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "IP" => Some(Self::Ip),
            "DOMAIN" => Some(Self::Domain),
            "URL" => Some(Self::Url),
            "EMAIL" => Some(Self::Email),
            "SHA256" => Some(Self::Sha256),
            "SHA1" => Some(Self::Sha1),
            "MD5" => Some(Self::Md5),
            "FILE_PATH" => Some(Self::FilePath),
            "HOSTNAME" => Some(Self::Hostname),
            _ => None,
        }
    }
}

fn validate_hex(value: &str, expected_len: usize, label: &str) -> Result<String, sqlx::Error> {
    let lower = value.to_lowercase();
    if lower.len() != expected_len || !lower.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(validation_error(format!(
            "invalid {label} hash (expected {expected_len} hex characters): {value}"
        )));
    }
    Ok(lower)
}

/// Normaliza e valida um valor de IOC quanto à forma (não contra uma lista
/// de reputação — ver docs do módulo). Retorna o valor normalizado, usado
/// tanto para exibição quanto como parte da `correlation_key` de
/// deduplicação.
pub fn normalize_ioc_value(kind: IocType, raw: &str) -> Result<String, sqlx::Error> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(validation_error("IOC value must not be empty"));
    }
    match kind {
        IocType::Ip => trimmed
            .parse::<std::net::IpAddr>()
            .map(|addr| addr.to_string())
            .map_err(|_| validation_error(format!("invalid IP address: {trimmed}"))),
        IocType::Sha256 => validate_hex(trimmed, 64, "SHA-256"),
        IocType::Sha1 => validate_hex(trimmed, 40, "SHA-1"),
        IocType::Md5 => validate_hex(trimmed, 32, "MD5"),
        IocType::Domain | IocType::Hostname => {
            let lower = trimmed.trim_end_matches('.').to_lowercase();
            if lower.is_empty() || lower.chars().any(char::is_whitespace) {
                return Err(validation_error(format!(
                    "invalid {}: {trimmed}",
                    kind.as_str()
                )));
            }
            Ok(lower)
        }
        IocType::Email => {
            let lower = trimmed.to_lowercase();
            let parts: Vec<&str> = lower.splitn(2, '@').collect();
            if parts.len() != 2
                || parts[0].is_empty()
                || parts[1].is_empty()
                || lower.chars().any(char::is_whitespace)
            {
                return Err(validation_error(format!("invalid email: {trimmed}")));
            }
            Ok(lower)
        }
        IocType::Url => {
            if trimmed.chars().any(char::is_whitespace) {
                return Err(validation_error("URL must not contain whitespace"));
            }
            let lower = trimmed.to_lowercase();
            let has_scheme = ["http://", "https://", "ftp://"]
                .iter()
                .any(|scheme| lower.starts_with(scheme));
            if !has_scheme {
                return Err(validation_error(format!(
                    "URL missing a recognized scheme (http/https/ftp): {trimmed}"
                )));
            }
            // Preserva o valor além do esquema como fornecido: normalização
            // completa de URL (RFC 3986) é maior do que o necessário para
            // deduplicação neste MVP — ver docs do módulo.
            Ok(trimmed.to_string())
        }
        IocType::FilePath => Ok(trimmed.to_string()),
    }
}

pub struct NewIocSighting<'a> {
    pub incident_id: &'a str,
    /// Obrigatório — seção 17 lista "evidência" como campo mandatório de
    /// todo IOC (reforçado por `AnalyticalResultType::Ioc::requires_evidence`).
    pub evidence_id: &'a str,
    pub ioc_type: IocType,
    pub raw_value: &'a str,
    /// "Evento" (seção 17): contexto de onde/como o IOC foi observado.
    pub event_context: Option<&'a str>,
    pub confidence: Confidence,
    /// Timestamp da ocorrência observada; usa o momento atual se ausente.
    pub observed_at: Option<&'a str>,
    pub actor_id: Option<&'a str>,
}

/// Teto de sightings guardados no payload de **uma** versão do agregado.
///
/// Sem teto, um indicador visto centenas de vezes faria o array
/// `sightings[]` crescer sem limite a cada ocorrência, virando uma linha
/// gigante em `analytical_results`. Com o teto, o payload de cada versão
/// mantém as `MAX_STORED_SIGHTINGS` ocorrências mais recentes (por
/// `observed_at`) e um contador do total (`sighting_count`) que continua
/// contando as descartadas. As ocorrências antigas não se perdem no banco:
/// a tabela é append-only e cada versão anterior do agregado continua
/// acessível via `analytical_results::history_for_correlation_key`.
pub const MAX_STORED_SIGHTINGS: usize = 100;

/// Resultado da agregação de sightings para uma nova versão do IOC.
#[derive(Debug, PartialEq, Eq)]
struct SightingAggregate {
    /// No máximo `max_stored` ocorrências, na ordem em que foram registradas.
    sightings: Vec<serde_json::Value>,
    /// Total de ocorrências já registradas para este IOC, inclusive as que
    /// já saíram de `sightings` por causa do teto.
    total_count: u64,
    first_seen: String,
    last_seen: String,
}

fn sighting_observed_at(sighting: &serde_json::Value) -> &str {
    sighting
        .get("observed_at")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
}

/// Função pura (sem banco): acrescenta `new_sighting` ao agregado descrito
/// por `previous` (o payload da versão atual, se houver) e aplica o teto.
///
/// - `first_seen`/`last_seen` vêm do payload anterior e são estendidos pelo
///   novo `observed_at`; assim continuam corretos mesmo depois que a
///   ocorrência mais antiga foi descartada do array.
/// - Payloads gravados antes do teto existir (sem `sighting_count`, e
///   possivelmente com mais de `max_stored` itens) são aceitos: o total é
///   inferido do tamanho do array e o excesso é descartado agora.
fn aggregate_sightings(
    previous: Option<&serde_json::Value>,
    new_sighting: serde_json::Value,
    observed_at: &str,
    max_stored: usize,
) -> Result<SightingAggregate, sqlx::Error> {
    let mut sightings: Vec<serde_json::Value> = match previous {
        Some(payload) => payload
            .get("sightings")
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| validation_error("stored IOC payload sightings is not an array"))?
            .clone(),
        None => Vec::new(),
    };

    let previous_total = match previous.and_then(|payload| payload.get("sighting_count")) {
        Some(value) => {
            let count = value.as_u64().ok_or_else(|| {
                validation_error("stored IOC payload sighting_count is not a non-negative integer")
            })?;
            if count < sightings.len() as u64 {
                return Err(validation_error(
                    "stored IOC payload sighting_count is smaller than stored sightings",
                ));
            }
            count
        }
        // Formato legado anterior ao contador: o array é a única fonte disponível.
        None => sightings.len() as u64,
    };
    let total_count = previous_total
        .checked_add(1)
        .ok_or_else(|| validation_error("stored IOC sighting_count cannot be incremented"))?;

    let previous_first = previous
        .and_then(|payload| payload.get("first_seen"))
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .or_else(|| {
            sightings
                .iter()
                .map(sighting_observed_at)
                .filter(|value| !value.is_empty())
                .min()
                .map(str::to_string)
        });
    let previous_last = previous
        .and_then(|payload| payload.get("last_seen"))
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .or_else(|| {
            sightings
                .iter()
                .map(sighting_observed_at)
                .filter(|value| !value.is_empty())
                .max()
                .map(str::to_string)
        });
    let first_seen = match previous_first {
        Some(value) if value.as_str() <= observed_at => value,
        _ => observed_at.to_string(),
    };
    let last_seen = match previous_last {
        Some(value) if value.as_str() >= observed_at => value,
        _ => observed_at.to_string(),
    };

    sightings.push(new_sighting);
    if sightings.len() > max_stored {
        // Descarta as mais antigas por `observed_at` (empate: a registrada
        // primeiro), preservando a ordem de registro das que ficam.
        let excess = sightings.len() - max_stored;
        let mut order: Vec<usize> = (0..sightings.len()).collect();
        order.sort_by(|&a, &b| {
            sighting_observed_at(&sightings[a])
                .cmp(sighting_observed_at(&sightings[b]))
                .then(a.cmp(&b))
        });
        let dropped: std::collections::HashSet<usize> = order.into_iter().take(excess).collect();
        sightings = sightings
            .into_iter()
            .enumerate()
            .filter(|(index, _)| !dropped.contains(index))
            .map(|(_, sighting)| sighting)
            .collect();
    }

    Ok(SightingAggregate {
        sightings,
        total_count,
        first_seen,
        last_seen,
    })
}

/// Registra uma ocorrência (sighting) de um IOC. Duplicatas (mesmo tipo +
/// valor normalizado, no mesmo incidente) são agregadas em uma única linha
/// "atual" via `supersedes` — cada nova ocorrência gera uma nova versão
/// cujo payload traz as ocorrências acumuladas até aquele ponto, limitadas
/// às `MAX_STORED_SIGHTINGS` mais recentes (ver a constante). `sighting_count`
/// é o total real; `first_seen`/`last_seen` cobrem todas as ocorrências,
/// inclusive as que já saíram do array.
pub async fn record_ioc_sighting(
    pool: &SqlitePool,
    input: NewIocSighting<'_>,
) -> Result<String, sqlx::Error> {
    let normalized_value = normalize_ioc_value(input.ioc_type, input.raw_value)?;
    let correlation_key = format!(
        "ioc:{}:{}:{}",
        input.incident_id,
        input.ioc_type.as_str(),
        normalized_value
    );
    let observed_at = input
        .observed_at
        .map(str::to_string)
        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

    let previous = analytical_results::current_for_correlation_key(pool, &correlation_key).await?;
    let previous_payload: Option<serde_json::Value> = match &previous {
        Some(record) => Some(serde_json::from_str(&record.payload).map_err(|error| {
            validation_error(format!("stored IOC payload is malformed: {error}"))
        })?),
        None => None,
    };
    let aggregate = aggregate_sightings(
        previous_payload.as_ref(),
        serde_json::json!({
            "evidence_id": input.evidence_id,
            "event_context": input.event_context,
            "confidence": input.confidence.as_str(),
            "observed_at": observed_at,
        }),
        &observed_at,
        MAX_STORED_SIGHTINGS,
    )?;

    let stored_count = aggregate.sightings.len() as u64;
    let payload_json = serde_json::json!({
        "kind": "IOC",
        "ioc_type": input.ioc_type.as_str(),
        "normalized_value": normalized_value,
        "first_seen": aggregate.first_seen,
        "last_seen": aggregate.last_seen,
        "sighting_count": aggregate.total_count,
        "sightings_dropped": aggregate.total_count.saturating_sub(stored_count),
        "sightings": aggregate.sightings,
    })
    .to_string();
    let provenance_json = serde_json::json!({
        "producer": "dfir-app",
        "method": "analyst-recorded IOC sighting, normalized and deduplicated by (incident, type, normalized value)",
    })
    .to_string();

    analytical_results::insert(
        pool,
        NewAnalyticalResult {
            result_type: AnalyticalResultType::Ioc,
            correlation_key: &correlation_key,
            evidence_id: Some(input.evidence_id),
            incident_id: Some(input.incident_id),
            analysis_job_id: None,
            rule_or_model_id: None,
            rule_or_model_version: None,
            confidence: Some(input.confidence),
            payload_json: &payload_json,
            provenance_json: &provenance_json,
            supersedes: previous.as_ref().map(|record| record.id.as_str()),
            related: &[],
            actor_id: input.actor_id,
        },
    )
    .await
}

// ---------------------------------------------------------------------
// Recomendações
// ---------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecommendationCategory {
    Immediate,
    Strategic,
}

impl RecommendationCategory {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Immediate => "IMMEDIATE",
            Self::Strategic => "STRATEGIC",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "IMMEDIATE" => Some(Self::Immediate),
            "STRATEGIC" => Some(Self::Strategic),
            _ => None,
        }
    }
}

pub struct NewRecommendation<'a> {
    pub incident_id: &'a str,
    pub category: RecommendationCategory,
    pub title: &'a str,
    pub description: &'a str,
    pub confidence: Option<Confidence>,
    /// Ids de resultados analíticos existentes (achados) que sustentam esta
    /// recomendação — obrigatório ter ao menos um ("Cada recomendação
    /// referencia achados que a sustentam").
    pub supporting_result_ids: &'a [&'a str],
    /// Id de uma recomendação existente que esta revisa. Diferente de
    /// IOCs/gaps, não há uma chave natural de deduplicação para
    /// recomendações — quem decide se isto é uma revisão ou uma
    /// recomendação nova é o chamador, explicitamente.
    pub supersedes: Option<&'a str>,
    pub actor_id: Option<&'a str>,
}

/// Insere uma recomendação, ligada via `RelationType::DerivedFrom` a cada
/// achado que a sustenta (validados como existentes antes do insert).
pub async fn insert_recommendation(
    pool: &SqlitePool,
    input: NewRecommendation<'_>,
) -> Result<String, sqlx::Error> {
    if input.supporting_result_ids.is_empty() {
        return Err(validation_error(
            "a recommendation must reference at least one supporting finding",
        ));
    }
    for result_id in input.supporting_result_ids {
        analytical_results::get(pool, result_id)
            .await?
            .ok_or_else(|| {
                validation_error(format!("supporting finding not found: {result_id}"))
            })?;
    }

    let correlation_key = match input.supersedes {
        Some(previous_id) => {
            let previous = analytical_results::get(pool, previous_id)
                .await?
                .ok_or_else(|| {
                    validation_error(format!(
                        "recommendation to supersede not found: {previous_id}"
                    ))
                })?;
            if previous.result_type != AnalyticalResultType::Recommendation.as_str() {
                return Err(validation_error(format!(
                    "{previous_id} is not a RECOMMENDATION result"
                )));
            }
            previous.correlation_key
        }
        None => format!("recommendation:{}:{}", input.incident_id, Uuid::new_v4()),
    };

    let payload_json = serde_json::json!({
        "kind": "RECOMMENDATION",
        "category": input.category.as_str(),
        "title": input.title,
        "description": input.description,
    })
    .to_string();
    let provenance_json = serde_json::json!({
        "producer": "dfir-app",
        "method": "analyst-authored recommendation",
        "supporting_result_ids": input.supporting_result_ids,
    })
    .to_string();

    let related: Vec<(&str, RelationType)> = input
        .supporting_result_ids
        .iter()
        .map(|id| (*id, RelationType::DerivedFrom))
        .collect();

    analytical_results::insert(
        pool,
        NewAnalyticalResult {
            result_type: AnalyticalResultType::Recommendation,
            correlation_key: &correlation_key,
            evidence_id: None,
            incident_id: Some(input.incident_id),
            analysis_job_id: None,
            rule_or_model_id: None,
            rule_or_model_version: None,
            confidence: input.confidence,
            payload_json: &payload_json,
            provenance_json: &provenance_json,
            supersedes: input.supersedes,
            related: related.as_slice(),
            actor_id: input.actor_id,
        },
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::test_support::seed_incident;

    async fn test_pool() -> SqlitePool {
        let temp = tempfile::tempdir().expect("temporary directory");
        let db_path = temp.path().join("ioc-recommendations.sqlite");
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

    async fn seed_evidence(pool: &SqlitePool, id: &str) {
        sqlx::query(
            "INSERT INTO evidence(id,original_filename,storage_reference,format,size_bytes,blake3,sha256,imported_at,status) VALUES(?,?,?,?,?,?,?,?,?)",
        )
        .bind(id)
        .bind("test.bin")
        .bind(format!("evidence/{id}.bin"))
        .bind("bin")
        .bind(1_i64)
        .bind("b3")
        .bind("sha")
        .bind("2026-08-28T00:00:00Z")
        .bind("FORENSIC")
        .execute(pool)
        .await
        .expect("evidence insert");
    }

    #[test]
    fn normalize_ioc_value_validates_ip_via_std() {
        assert_eq!(
            normalize_ioc_value(IocType::Ip, "192.168.1.1").unwrap(),
            "192.168.1.1"
        );
        assert!(normalize_ioc_value(IocType::Ip, "not-an-ip").is_err());
    }

    #[test]
    fn normalize_ioc_value_lowercases_and_trims_domain() {
        assert_eq!(
            normalize_ioc_value(IocType::Domain, "  Evil.Example.COM.  ").unwrap(),
            "evil.example.com"
        );
    }

    #[test]
    fn normalize_ioc_value_validates_hash_length() {
        assert!(normalize_ioc_value(IocType::Sha256, "abc").is_err());
        let valid_sha256 = "a".repeat(64);
        assert_eq!(
            normalize_ioc_value(IocType::Sha256, &valid_sha256).unwrap(),
            valid_sha256
        );
        assert!(normalize_ioc_value(IocType::Sha1, &"a".repeat(41)).is_err());
        assert!(
            normalize_ioc_value(IocType::Md5, &"g".repeat(32)).is_err(),
            "non-hex chars rejected"
        );
    }

    #[test]
    fn normalize_ioc_value_validates_email_shape() {
        assert_eq!(
            normalize_ioc_value(IocType::Email, "Attacker@Example.com").unwrap(),
            "attacker@example.com"
        );
        assert!(normalize_ioc_value(IocType::Email, "not-an-email").is_err());
    }

    #[test]
    fn normalize_ioc_value_requires_a_recognized_url_scheme() {
        assert!(normalize_ioc_value(IocType::Url, "http://evil.example.com/payload").is_ok());
        assert!(normalize_ioc_value(IocType::Url, "evil.example.com/payload").is_err());
    }

    #[test]
    fn normalize_ioc_value_rejects_empty_values() {
        assert!(normalize_ioc_value(IocType::Hostname, "   ").is_err());
    }

    fn sighting(evidence_id: &str, observed_at: &str) -> serde_json::Value {
        serde_json::json!({
            "evidence_id": evidence_id,
            "event_context": null,
            "confidence": "HIGH",
            "observed_at": observed_at,
        })
    }

    /// Reaplica `aggregate_sightings` em cadeia, como `record_ioc_sighting`
    /// faz ao ler o payload da versão anterior.
    fn fold_sightings(
        observed: &[&str],
        max_stored: usize,
    ) -> (serde_json::Value, SightingAggregate) {
        let mut payload: Option<serde_json::Value> = None;
        let mut last = None;
        for (index, at) in observed.iter().enumerate() {
            let aggregate = aggregate_sightings(
                payload.as_ref(),
                sighting(&format!("ev-{index}"), at),
                at,
                max_stored,
            )
            .expect("valid sighting aggregate");
            payload = Some(serde_json::json!({
                "first_seen": aggregate.first_seen,
                "last_seen": aggregate.last_seen,
                "sighting_count": aggregate.total_count,
                "sightings": aggregate.sightings,
            }));
            last = Some(aggregate);
        }
        (
            payload.expect("at least one sighting"),
            last.expect("at least one sighting"),
        )
    }

    #[test]
    fn aggregate_below_the_cap_keeps_every_sighting() {
        let (_, aggregate) = fold_sightings(&["2026-08-01T00:00:00Z", "2026-08-02T00:00:00Z"], 5);
        assert_eq!(aggregate.sightings.len(), 2);
        assert_eq!(aggregate.total_count, 2);
        assert_eq!(aggregate.first_seen, "2026-08-01T00:00:00Z");
        assert_eq!(aggregate.last_seen, "2026-08-02T00:00:00Z");
    }

    #[test]
    fn aggregate_over_the_cap_keeps_the_most_recent_and_counts_the_total() {
        let observed = [
            "2026-08-01T00:00:00Z",
            "2026-08-02T00:00:00Z",
            "2026-08-03T00:00:00Z",
            "2026-08-04T00:00:00Z",
            "2026-08-05T00:00:00Z",
        ];
        let (_, aggregate) = fold_sightings(&observed, 3);
        assert_eq!(aggregate.sightings.len(), 3);
        assert_eq!(
            aggregate.total_count, 5,
            "total keeps counting dropped sightings"
        );
        let kept: Vec<&str> = aggregate
            .sightings
            .iter()
            .map(sighting_observed_at)
            .collect();
        assert_eq!(
            kept,
            [
                "2026-08-03T00:00:00Z",
                "2026-08-04T00:00:00Z",
                "2026-08-05T00:00:00Z"
            ]
        );
    }

    #[test]
    fn aggregate_keeps_first_seen_after_the_oldest_sighting_is_dropped() {
        let observed = [
            "2026-08-01T00:00:00Z",
            "2026-08-02T00:00:00Z",
            "2026-08-03T00:00:00Z",
            "2026-08-04T00:00:00Z",
        ];
        let (_, aggregate) = fold_sightings(&observed, 2);
        assert!(
            aggregate
                .sightings
                .iter()
                .all(|s| sighting_observed_at(s) != "2026-08-01T00:00:00Z"),
            "oldest sighting no longer stored"
        );
        assert_eq!(aggregate.first_seen, "2026-08-01T00:00:00Z");
        assert_eq!(aggregate.last_seen, "2026-08-04T00:00:00Z");
    }

    #[test]
    fn aggregate_drops_by_observed_at_even_when_recorded_out_of_order() {
        // Uma ocorrência antiga registrada por último é a primeira a sair,
        // e não deve mexer em `last_seen`.
        let observed = [
            "2026-08-10T00:00:00Z",
            "2026-08-11T00:00:00Z",
            "2026-08-01T00:00:00Z",
        ];
        let (_, aggregate) = fold_sightings(&observed, 2);
        let kept: Vec<&str> = aggregate
            .sightings
            .iter()
            .map(sighting_observed_at)
            .collect();
        assert_eq!(kept, ["2026-08-10T00:00:00Z", "2026-08-11T00:00:00Z"]);
        assert_eq!(aggregate.first_seen, "2026-08-01T00:00:00Z");
        assert_eq!(aggregate.last_seen, "2026-08-11T00:00:00Z");
        assert_eq!(aggregate.total_count, 3);
    }

    #[test]
    fn aggregate_accepts_a_legacy_payload_without_count_and_trims_it() {
        // Formato anterior ao teto: sem `sighting_count`, sem limite de itens.
        let legacy_sightings: Vec<serde_json::Value> = (1..=6)
            .map(|day| sighting("ev-legacy", &format!("2026-07-{day:02}T00:00:00Z")))
            .collect();
        let legacy = serde_json::json!({
            "first_seen": "2026-07-01T00:00:00Z",
            "last_seen": "2026-07-06T00:00:00Z",
            "sightings": legacy_sightings,
        });
        let aggregate = aggregate_sightings(
            Some(&legacy),
            sighting("ev-new", "2026-08-01T00:00:00Z"),
            "2026-08-01T00:00:00Z",
            3,
        )
        .expect("legacy payload without count remains valid");
        assert_eq!(aggregate.sightings.len(), 3);
        assert_eq!(aggregate.total_count, 7, "6 legadas + 1 nova");
        assert_eq!(aggregate.first_seen, "2026-07-01T00:00:00Z");
        assert_eq!(aggregate.last_seen, "2026-08-01T00:00:00Z");
    }

    #[test]
    fn aggregate_of_the_first_sighting_starts_the_history() {
        let aggregate = aggregate_sightings(
            None,
            sighting("ev-1", "2026-08-01T00:00:00Z"),
            "2026-08-01T00:00:00Z",
            MAX_STORED_SIGHTINGS,
        )
        .expect("first sighting aggregate");
        assert_eq!(aggregate.sightings.len(), 1);
        assert_eq!(aggregate.total_count, 1);
        assert_eq!(aggregate.first_seen, aggregate.last_seen);
    }

    #[tokio::test]
    async fn record_ioc_sighting_rejects_malformed_current_ioc_history() {
        let pool = test_pool().await;
        let two_sightings = || {
            serde_json::json!([
                sighting("ev-old", "2026-08-01T00:00:00Z"),
                sighting("ev-old", "2026-08-02T00:00:00Z")
            ])
        };
        let cases = [
            (
                "shape",
                serde_json::json!({"sighting_count": 7, "sightings": {}}),
                "stored IOC payload sightings is not an array",
            ),
            (
                "count",
                serde_json::json!({"sighting_count": "seven", "sightings": two_sightings()}),
                "stored IOC payload sighting_count is not a non-negative integer",
            ),
            (
                "inconsistent",
                serde_json::json!({"sighting_count": 1, "sightings": two_sightings()}),
                "stored IOC payload sighting_count is smaller than stored sightings",
            ),
            (
                "overflow",
                serde_json::json!({"sighting_count": u64::MAX, "sightings": []}),
                "stored IOC sighting_count cannot be incremented",
            ),
        ];

        for (case, payload, expected) in cases {
            let incident_id = format!("inc-bad-{case}");
            let evidence_id = format!("ev-bad-{case}");
            seed_incident(&pool, &incident_id).await;
            seed_evidence(&pool, &evidence_id).await;
            let correlation_key = format!("ioc:{incident_id}:IP:203.0.113.5");
            let bad_id = format!("bad-ioc-{case}");
            sqlx::query("INSERT INTO analytical_results(id,schema_version,result_type,correlation_key,evidence_id,incident_id,payload,provenance,created_at) VALUES(?,'analytical-result-v1','IOC',?,?,?,?,'{}','2026-08-20T00:00:00Z')")
                .bind(&bad_id)
                .bind(&correlation_key)
                .bind(&evidence_id)
                .bind(&incident_id)
                .bind(payload.to_string())
                .execute(&pool)
                .await
                .expect("malformed current IOC fixture");

            let error = match record_ioc_sighting(
                &pool,
                NewIocSighting {
                    incident_id: &incident_id,
                    evidence_id: &evidence_id,
                    ioc_type: IocType::Ip,
                    raw_value: "203.0.113.5",
                    event_context: None,
                    confidence: Confidence::High,
                    observed_at: Some("2026-08-21T00:00:00Z"),
                    actor_id: None,
                },
            )
            .await
            {
                Ok(new_id) => {
                    let record = analytical_results::get(&pool, &new_id)
                        .await
                        .expect("query accepted successor")
                        .expect("accepted successor exists");
                    let payload: serde_json::Value =
                        serde_json::from_str(&record.payload).expect("accepted successor payload");
                    panic!(
                        "malformed current IOC was superseded for {case}: new sighting_count={}, stored_sightings={}",
                        payload["sighting_count"],
                        payload["sightings"].as_array().map_or(0, Vec::len)
                    );
                }
                Err(error) => error,
            };
            assert!(
                error.to_string().contains(expected),
                "unexpected error for {case}: {error:?}"
            );

            let current = analytical_results::current_for_correlation_key(&pool, &correlation_key)
                .await
                .expect("current IOC query")
                .expect("malformed IOC remains current");
            assert_eq!(current.id, bad_id, "no successor allowed for {case}");
            let rows: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM analytical_results WHERE correlation_key=?",
            )
            .bind(&correlation_key)
            .fetch_one(&pool)
            .await
            .expect("IOC history count");
            assert_eq!(rows, 1, "no new history row allowed for {case}");
        }
    }

    #[tokio::test]
    async fn recording_the_same_ioc_twice_aggregates_sightings_without_losing_provenance() {
        let pool = test_pool().await;
        seed_incident(&pool, "inc-1").await;
        seed_evidence(&pool, "ev-1").await;
        seed_evidence(&pool, "ev-2").await;

        let first_id = record_ioc_sighting(
            &pool,
            NewIocSighting {
                incident_id: "inc-1",
                evidence_id: "ev-1",
                ioc_type: IocType::Ip,
                raw_value: "203.0.113.5",
                event_context: Some("C2 beacon"),
                confidence: Confidence::High,
                observed_at: Some("2026-08-01T00:00:00Z"),
                actor_id: None,
            },
        )
        .await
        .expect("first sighting succeeds");

        let second_id = record_ioc_sighting(
            &pool,
            NewIocSighting {
                incident_id: "inc-1",
                evidence_id: "ev-2",
                ioc_type: IocType::Ip,
                raw_value: "203.0.113.5",
                event_context: Some("lateral movement"),
                confidence: Confidence::Medium,
                observed_at: Some("2026-08-05T00:00:00Z"),
                actor_id: None,
            },
        )
        .await
        .expect("second sighting succeeds");

        assert_ne!(first_id, second_id);

        let current = analytical_results::get(&pool, &second_id)
            .await
            .expect("query")
            .expect("record exists");
        assert_eq!(current.supersedes.as_deref(), Some(first_id.as_str()));
        assert!(current.payload.contains("\"sighting_count\":2"));
        assert!(
            current.payload.contains("ev-1"),
            "first sighting's provenance preserved"
        );
        assert!(current.payload.contains("ev-2"));
        assert!(
            current
                .payload
                .contains("\"first_seen\":\"2026-08-01T00:00:00Z\"")
        );
        assert!(
            current
                .payload
                .contains("\"last_seen\":\"2026-08-05T00:00:00Z\"")
        );

        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM analytical_results WHERE result_type='IOC' AND incident_id='inc-1'",
        )
        .fetch_one(&pool)
        .await
        .expect("count query");
        assert_eq!(total, 2, "both versions kept in history");
    }

    #[tokio::test]
    async fn sighting_payload_stays_bounded_while_history_and_total_keep_growing() {
        let pool = test_pool().await;
        seed_incident(&pool, "inc-cap").await;
        seed_evidence(&pool, "ev-cap").await;

        let total = MAX_STORED_SIGHTINGS + 3;
        let mut last_id = String::new();
        for index in 0..total {
            let observed_at = format!("2026-08-01T{:02}:{:02}:00Z", index / 60, index % 60);
            last_id = record_ioc_sighting(
                &pool,
                NewIocSighting {
                    incident_id: "inc-cap",
                    evidence_id: "ev-cap",
                    ioc_type: IocType::Ip,
                    raw_value: "198.51.100.7",
                    event_context: None,
                    confidence: Confidence::Medium,
                    observed_at: Some(observed_at.as_str()),
                    actor_id: None,
                },
            )
            .await
            .expect("sighting succeeds");
        }

        let current = analytical_results::get(&pool, &last_id)
            .await
            .expect("query")
            .expect("record exists");
        let payload: serde_json::Value =
            serde_json::from_str(&current.payload).expect("payload is JSON");
        assert_eq!(
            payload["sightings"].as_array().map(Vec::len),
            Some(MAX_STORED_SIGHTINGS)
        );
        assert_eq!(payload["sighting_count"], total as u64);
        assert_eq!(payload["sightings_dropped"], 3);
        assert_eq!(payload["first_seen"], "2026-08-01T00:00:00Z");
        let last_minute = total - 1;
        assert_eq!(
            payload["last_seen"],
            format!(
                "2026-08-01T{:02}:{:02}:00Z",
                last_minute / 60,
                last_minute % 60
            )
        );

        // Append-only: o teto limita o tamanho de cada linha, não o histórico.
        let rows: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM analytical_results WHERE result_type='IOC' AND incident_id='inc-cap'",
        )
        .fetch_one(&pool)
        .await
        .expect("count query");
        assert_eq!(rows, total as i64);
    }

    #[tokio::test]
    async fn different_incidents_do_not_share_ioc_aggregation() {
        let pool = test_pool().await;
        seed_incident(&pool, "inc-2a").await;
        seed_incident(&pool, "inc-2b").await;
        seed_evidence(&pool, "ev-3").await;
        seed_evidence(&pool, "ev-4").await;

        record_ioc_sighting(
            &pool,
            NewIocSighting {
                incident_id: "inc-2a",
                evidence_id: "ev-3",
                ioc_type: IocType::Domain,
                raw_value: "evil.example.com",
                event_context: None,
                confidence: Confidence::Low,
                observed_at: None,
                actor_id: None,
            },
        )
        .await
        .expect("first incident sighting succeeds");
        record_ioc_sighting(
            &pool,
            NewIocSighting {
                incident_id: "inc-2b",
                evidence_id: "ev-4",
                ioc_type: IocType::Domain,
                raw_value: "evil.example.com",
                event_context: None,
                confidence: Confidence::Low,
                observed_at: None,
                actor_id: None,
            },
        )
        .await
        .expect("second incident sighting succeeds");

        let total: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM analytical_results WHERE result_type='IOC'")
                .fetch_one(&pool)
                .await
                .expect("count query");
        assert_eq!(
            total, 2,
            "same IOC in different incidents is not aggregated together"
        );
    }

    #[tokio::test]
    async fn insert_recommendation_requires_at_least_one_supporting_finding() {
        let pool = test_pool().await;
        seed_incident(&pool, "inc-3").await;
        let result = insert_recommendation(
            &pool,
            NewRecommendation {
                incident_id: "inc-3",
                category: RecommendationCategory::Immediate,
                title: "Rotate credentials",
                description: "...",
                confidence: None,
                supporting_result_ids: &[],
                supersedes: None,
                actor_id: None,
            },
        )
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn insert_recommendation_rejects_a_nonexistent_supporting_finding() {
        let pool = test_pool().await;
        seed_incident(&pool, "inc-4").await;
        let result = insert_recommendation(
            &pool,
            NewRecommendation {
                incident_id: "inc-4",
                category: RecommendationCategory::Strategic,
                title: "Deploy EDR",
                description: "...",
                confidence: None,
                supporting_result_ids: &["does-not-exist"],
                supersedes: None,
                actor_id: None,
            },
        )
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn insert_recommendation_links_to_its_supporting_findings() {
        let pool = test_pool().await;
        seed_incident(&pool, "inc-5").await;
        let finding_id = analytical_results::insert(
            &pool,
            NewAnalyticalResult {
                result_type: AnalyticalResultType::AttackMapping,
                correlation_key: "corr-finding-1",
                evidence_id: None,
                incident_id: Some("inc-5"),
                analysis_job_id: None,
                rule_or_model_id: None,
                rule_or_model_version: None,
                confidence: Some(Confidence::High),
                payload_json: "{}",
                provenance_json: "{}",
                supersedes: None,
                related: &[],
                actor_id: None,
            },
        )
        .await
        .expect("finding insert succeeds");

        let recommendation_id = insert_recommendation(
            &pool,
            NewRecommendation {
                incident_id: "inc-5",
                category: RecommendationCategory::Immediate,
                title: "Isolate affected host",
                description: "...",
                confidence: Some(Confidence::High),
                supporting_result_ids: &[&finding_id],
                supersedes: None,
                actor_id: None,
            },
        )
        .await
        .expect("recommendation insert succeeds");

        let related = analytical_results::related_to(&pool, &recommendation_id)
            .await
            .expect("query relations");
        assert_eq!(related.len(), 1);
        assert_eq!(related[0].0, RelationType::DerivedFrom);
        assert_eq!(related[0].1.id, finding_id);
    }

    #[tokio::test]
    async fn superseding_a_recommendation_keeps_it_in_the_same_correlation_group() {
        let pool = test_pool().await;
        seed_incident(&pool, "inc-6").await;
        let finding_id = analytical_results::insert(
            &pool,
            NewAnalyticalResult {
                result_type: AnalyticalResultType::ImpactAssessment,
                correlation_key: "corr-finding-2",
                evidence_id: None,
                incident_id: Some("inc-6"),
                analysis_job_id: None,
                rule_or_model_id: None,
                rule_or_model_version: None,
                confidence: Some(Confidence::Medium),
                payload_json: "{}",
                provenance_json: "{}",
                supersedes: None,
                related: &[],
                actor_id: None,
            },
        )
        .await
        .expect("finding insert succeeds");

        let first = insert_recommendation(
            &pool,
            NewRecommendation {
                incident_id: "inc-6",
                category: RecommendationCategory::Strategic,
                title: "Improve logging",
                description: "initial draft",
                confidence: None,
                supporting_result_ids: &[&finding_id],
                supersedes: None,
                actor_id: None,
            },
        )
        .await
        .expect("first insert succeeds");

        let second = insert_recommendation(
            &pool,
            NewRecommendation {
                incident_id: "inc-6",
                category: RecommendationCategory::Strategic,
                title: "Improve logging (revised)",
                description: "revised after review",
                confidence: None,
                supporting_result_ids: &[&finding_id],
                supersedes: Some(&first),
                actor_id: None,
            },
        )
        .await
        .expect("second insert succeeds");

        let history = analytical_results::history_for_correlation_key(
            &pool,
            &analytical_results::get(&pool, &second)
                .await
                .expect("query")
                .expect("record exists")
                .correlation_key,
        )
        .await
        .expect("history query");
        assert_eq!(history.len(), 2);
    }
}
