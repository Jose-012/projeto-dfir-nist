//! F1-01 — modelo analítico comum e versionamento de resultados.
//!
//! Base persistente e versionada compartilhada por YARA-X (F1-02), NIST gap
//! (F1-03), ATT&CK/caminho crítico/impacto (F1-04) e IOCs/recomendações
//! (F1-05): identificadores, evidência/incidente de origem, confiança,
//! provenance e versão de regra/modelo — com histórico preservado em
//! reprocessamento (nunca sobrescrito silenciosamente).
//!
//! Segue o mesmo padrão append-only de [`crate::persistence::audit`]
//! (append-only, tamper-resistant via triggers `BEFORE UPDATE`/`BEFORE
//! DELETE`), mas sem cadeia de hash: a garantia exigida por F1-01 é
//! "histórico preservado", não "tamper-evident" — a audit chain já cobre
//! tamper-evidence para toda mutação do banco, incluindo os inserts feitos
//! aqui (cada insert grava um evento `ANALYTICAL_RESULT_RECORDED`).
//!
//! ## Decisão de design: sem UPDATE, nunca
//!
//! Reprocessamento não atualiza a linha anterior (nem mesmo um ponteiro
//! `superseded_by`): insere uma nova linha com `supersedes` apontando para a
//! anterior. O "resultado atual" de uma `correlation_key` é resolvido por
//! consulta (`current_for_correlation_key`), não por um campo mutável — isso
//! elimina qualquer necessidade de UPDATE na tabela, permitindo os mesmos
//! triggers append-only já usados em `audit_events`/`chain_of_custody`.
//!
//! ## Canonicalização compartilhada
//!
//! A validação e a canonicalização JCS vivem em `persistence::jcs`, também
//! usados pela audit chain e pela custódia. Vetores fixos em `audit.rs`
//! protegem a compatibilidade dos hashes v1/v2.
//!
//! ## Fora de escopo desta migration (decisão deliberada de F1-01)
//!
//! `CurrentProfile`/`TargetProfile`/`TierAssessment` (seção 13 da spec
//! canônica) não fazem parte deste envelope: são mais próximos de dados de
//! configuração/referência da organização do que de achados versionados com
//! confiança, e ficam a critério de F1-03 definir. `NistGap` (RF-010,
//! "reproduzível a partir dos dados") é o que claramente se encaixa no
//! contrato de resultado analítico e está incluído.

use sqlx::{SqliteConnection, SqlitePool};
use uuid::Uuid;

use super::jcs::canonicalize_json;
use super::{rollback_loudly, validation_error};

/// Versão do envelope (não confundir com `rule_or_model_version`, que é a
/// versão da regra/modelo que produziu o resultado).
pub const SCHEMA_VERSION: &str = "analytical-result-v1";

fn validate_correlation_key(correlation_key: &str) -> Result<(), sqlx::Error> {
    if correlation_key.is_empty() || correlation_key.chars().count() > 200 {
        return Err(validation_error(
            "correlation_key must be 1..=200 characters",
        ));
    }
    if correlation_key.chars().any(char::is_control) {
        return Err(validation_error(
            "correlation_key must not contain control characters",
        ));
    }
    Ok(())
}

// ---------------------------------------------------------------------
// Tipos do envelope
// ---------------------------------------------------------------------

/// Categorias de resultado analítico conhecidas na Fase 1. Variantes só são
/// adicionadas (nunca reinterpretadas) quando um novo RF/ticket exigir —
/// mesma disciplina de não reutilização de IDs normativos usada no resto do
/// projeto.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnalyticalResultType {
    /// RF-008 — match de regra YARA-X contra evidência.
    YaraMatch,
    /// RF-010 — gap entre Current Profile e Target Profile NIST CSF 2.0.
    NistGap,
    /// RF-011 — mapeamento de evento/achado para técnica ATT&CK.
    AttackMapping,
    /// RF-018 — transição/vínculo entre eventos no caminho crítico do incidente.
    CriticalPathLink,
    /// RF-020 — estimativa na matriz de impacto do incidente.
    ImpactAssessment,
    /// RF-019 — indicador de comprometimento extraído de evidência.
    Ioc,
    /// RF-022 — recomendação imediata/estratégica.
    Recommendation,
    /// RF-051 (F1-25, ADR-0023) — vulnerabilidade ou condição explorada
    /// no incidente, com evidência, severidade e confiança.
    Finding,
    /// RF-051 (F1-25, ADR-0023) — conclusão sobre a causa do incidente, em
    /// nível (causa-raiz, causa provável ou fator contribuinte), com a
    /// confiança e as lacunas que a sustentam.
    RootCause,
    /// RF-052 (F1-29, ADR-0023) — item do plano de fortalecimento (hardening,
    /// criptografia, chaves, resposta a incidentes).
    PlanItem,
    /// RF-054 (F1-31, ADR-0023) — lacuna de fonte declarada pelo analista:
    /// fonte esperada, período sem dados e motivo (RB-001).
    SourceGap,
}

impl AnalyticalResultType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::YaraMatch => "YARA_MATCH",
            Self::NistGap => "NIST_GAP",
            Self::AttackMapping => "ATTACK_MAPPING",
            Self::CriticalPathLink => "CRITICAL_PATH_LINK",
            Self::ImpactAssessment => "IMPACT_ASSESSMENT",
            Self::Ioc => "IOC",
            Self::Recommendation => "RECOMMENDATION",
            Self::Finding => "FINDING",
            Self::RootCause => "ROOT_CAUSE",
            Self::PlanItem => "PLAN_ITEM",
            Self::SourceGap => "SOURCE_GAP",
        }
    }

    #[cfg(test)]
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "YARA_MATCH" => Some(Self::YaraMatch),
            "NIST_GAP" => Some(Self::NistGap),
            "ATTACK_MAPPING" => Some(Self::AttackMapping),
            "CRITICAL_PATH_LINK" => Some(Self::CriticalPathLink),
            "IMPACT_ASSESSMENT" => Some(Self::ImpactAssessment),
            "IOC" => Some(Self::Ioc),
            "RECOMMENDATION" => Some(Self::Recommendation),
            "FINDING" => Some(Self::Finding),
            "ROOT_CAUSE" => Some(Self::RootCause),
            "PLAN_ITEM" => Some(Self::PlanItem),
            "SOURCE_GAP" => Some(Self::SourceGap),
            _ => None,
        }
    }

    /// Política provisória estabelecida por F1-01 a partir das seções 13
    /// (NIST), 14 (ATT&CK), 15 (caminho crítico), 16 (impacto) e 17 (IOC) da
    /// spec canônica, que listam "confiança" como campo obrigatório. YARA e
    /// Recommendation ficam de fora do bloqueio: a spec usa "confiança/
    /// contexto" (RF-008) para YARA, e não define explicitamente o campo
    /// para recomendações. `Finding` e `RootCause` exigem confiança (spec
    /// §4.2: Entregável 2 lista "confiança"; Entregável 1, o nível de
    /// evidência da causa); `PlanItem` não — é uma decisão de plano, não uma
    /// inferência sobre evidência. Cada F1-0x pode refinar isso sem precisar
    /// de migration, já que a checagem é em Rust, não no schema. (Já o
    /// conjunto de `result_type` aceitos É uma constraint do schema: tipo
    /// novo exige migration — ver 0010.)
    pub fn requires_confidence(self) -> bool {
        matches!(
            self,
            Self::NistGap
                | Self::AttackMapping
                | Self::CriticalPathLink
                | Self::ImpactAssessment
                | Self::Ioc
                | Self::Finding
                | Self::RootCause
        )
    }

    /// Tipos cuja origem primária é uma evidência específica.
    pub fn requires_evidence(self) -> bool {
        matches!(self, Self::YaraMatch | Self::Ioc)
    }

    /// Tipos cuja origem primária é o incidente (não uma evidência específica).
    pub fn requires_incident(self) -> bool {
        matches!(
            self,
            Self::NistGap
                | Self::AttackMapping
                | Self::CriticalPathLink
                | Self::ImpactAssessment
                | Self::Recommendation
                | Self::Finding
                | Self::RootCause
                | Self::PlanItem
                | Self::SourceGap
        )
    }
}

/// Nível de confiança qualitativo, conforme o vocabulário usado em toda a
/// spec canônica (NIST, ATT&CK, caminho crítico, impacto, IOC): LOW/MEDIUM/
/// HIGH/CONFIRMED.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Confidence {
    Low,
    Medium,
    High,
    Confirmed,
}

impl Confidence {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Low => "LOW",
            Self::Medium => "MEDIUM",
            Self::High => "HIGH",
            Self::Confirmed => "CONFIRMED",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "LOW" => Some(Self::Low),
            "MEDIUM" => Some(Self::Medium),
            "HIGH" => Some(Self::High),
            "CONFIRMED" => Some(Self::Confirmed),
            _ => None,
        }
    }
}

/// Tipo de relação entre dois resultados analíticos.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationType {
    /// Uma recomendação (F1-05) referencia os achados que a sustentam.
    DerivedFrom,
    /// Um evento/achado do caminho crítico (F1-04) referencia o próximo na sequência.
    SequenceNext,
}

impl RelationType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DerivedFrom => "DERIVED_FROM",
            Self::SequenceNext => "SEQUENCE_NEXT",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "DERIVED_FROM" => Some(Self::DerivedFrom),
            "SEQUENCE_NEXT" => Some(Self::SequenceNext),
            _ => None,
        }
    }
}

/// Entrada para inserir um novo resultado analítico.
pub struct NewAnalyticalResult<'a> {
    pub result_type: AnalyticalResultType,
    pub correlation_key: &'a str,
    pub evidence_id: Option<&'a str>,
    pub incident_id: Option<&'a str>,
    /// Referência lógica (não FK) ao job de análise de origem — ver nota na migration.
    pub analysis_job_id: Option<&'a str>,
    pub rule_or_model_id: Option<&'a str>,
    pub rule_or_model_version: Option<&'a str>,
    pub confidence: Option<Confidence>,
    /// JSON bruto do payload específico do tipo (ex.: campos de um match YARA).
    pub payload_json: &'a str,
    /// JSON bruto de provenance (produtor, método, versão de ruleset/modelo, etc.).
    pub provenance_json: &'a str,
    /// Id de um resultado anterior que esta linha substitui (mesma `correlation_key`).
    pub supersedes: Option<&'a str>,
    /// Outros resultados relacionados a este (ex.: achados que sustentam uma recomendação).
    pub related: &'a [(&'a str, RelationType)],
    pub actor_id: Option<&'a str>,
}

/// Linha lida de `analytical_results`.
#[derive(Debug, Clone)]
pub struct AnalyticalResultRecord {
    pub id: String,
    pub result_type: String,
    pub correlation_key: String,
    pub evidence_id: Option<String>,
    pub incident_id: Option<String>,
    pub analysis_job_id: Option<String>,
    pub rule_or_model_id: Option<String>,
    pub rule_or_model_version: Option<String>,
    pub confidence: Option<String>,
    pub payload: String,
    pub provenance: String,
    pub supersedes: Option<String>,
    pub created_at: String,
}

type ResultRow = (
    i64,
    String,
    String,
    String,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    String,
    String,
    Option<String>,
    String,
);

impl From<ResultRow> for AnalyticalResultRecord {
    fn from(row: ResultRow) -> Self {
        Self {
            id: row.1,
            result_type: row.3,
            correlation_key: row.4,
            evidence_id: row.5,
            incident_id: row.6,
            analysis_job_id: row.7,
            rule_or_model_id: row.8,
            rule_or_model_version: row.9,
            confidence: row.10,
            payload: row.11,
            provenance: row.12,
            supersedes: row.13,
            created_at: row.14,
        }
    }
}

// ---------------------------------------------------------------------
// Escrita
// ---------------------------------------------------------------------

fn validate_new_result(input: &NewAnalyticalResult<'_>) -> Result<(), sqlx::Error> {
    validate_correlation_key(input.correlation_key)?;

    if input.evidence_id.is_none() && input.incident_id.is_none() {
        return Err(validation_error(
            "analytical result requires evidence_id and/or incident_id",
        ));
    }
    if input.result_type.requires_evidence() && input.evidence_id.is_none() {
        return Err(validation_error(format!(
            "{} requires evidence_id",
            input.result_type.as_str()
        )));
    }
    if input.result_type.requires_incident() && input.incident_id.is_none() {
        return Err(validation_error(format!(
            "{} requires incident_id",
            input.result_type.as_str()
        )));
    }
    if input.result_type.requires_confidence() && input.confidence.is_none() {
        return Err(validation_error(format!(
            "{} requires confidence",
            input.result_type.as_str()
        )));
    }
    Ok(())
}

/// Insere um novo resultado analítico na conexão/transação fornecida,
/// registrando um evento de auditoria na mesma transação. Não abre nem
/// fecha transação própria — chamadores que precisam compor com outras
/// mutações usam esta função a partir de sua transação; para um insert
/// isolado, use [`insert`].
pub(crate) async fn insert_on_connection(
    connection: &mut SqliteConnection,
    input: NewAnalyticalResult<'_>,
) -> Result<String, sqlx::Error> {
    validate_new_result(&input)?;

    let payload = canonicalize_json(input.payload_json)?;
    let provenance = canonicalize_json(input.provenance_json)?;

    if let Some(supersedes_id) = input.supersedes {
        let superseded_correlation_key: Option<String> =
            sqlx::query_scalar("SELECT correlation_key FROM analytical_results WHERE id = ?")
                .bind(supersedes_id)
                .fetch_optional(&mut *connection)
                .await?;
        match superseded_correlation_key {
            None => {
                return Err(validation_error(format!(
                    "supersedes references unknown analytical result: {supersedes_id}"
                )));
            }
            Some(key) if key != input.correlation_key => {
                return Err(validation_error(
                    "supersedes must reference a result with the same correlation_key",
                ));
            }
            Some(_) => {}
        }
        let already_superseded: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM analytical_results WHERE supersedes = ?)",
        )
        .bind(supersedes_id)
        .fetch_one(&mut *connection)
        .await?;
        if already_superseded {
            return Err(validation_error(format!(
                "supersedes references an analytical result that is already superseded: {supersedes_id}"
            )));
        }
    } else {
        let correlation_key_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM analytical_results WHERE correlation_key = ?)",
        )
        .bind(input.correlation_key)
        .fetch_one(&mut *connection)
        .await?;
        if correlation_key_exists {
            return Err(validation_error(
                "correlation_key already exists; supersedes is required to record a revision",
            ));
        }
    }

    let id = Uuid::new_v4().to_string();
    let created_at = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true);
    let confidence = input.confidence.map(Confidence::as_str);

    sqlx::query(
        "INSERT INTO analytical_results (id,schema_version,result_type,correlation_key,evidence_id,incident_id,analysis_job_id,rule_or_model_id,rule_or_model_version,confidence,payload,provenance,supersedes,created_at) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
    )
    .bind(&id)
    .bind(SCHEMA_VERSION)
    .bind(input.result_type.as_str())
    .bind(input.correlation_key)
    .bind(input.evidence_id)
    .bind(input.incident_id)
    .bind(input.analysis_job_id)
    .bind(input.rule_or_model_id)
    .bind(input.rule_or_model_version)
    .bind(confidence)
    .bind(&payload)
    .bind(&provenance)
    .bind(input.supersedes)
    .bind(&created_at)
    .execute(&mut *connection)
    .await?;

    for (related_result_id, relation_type) in input.related {
        let relation_id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO analytical_result_relations (id,result_id,related_result_id,relation_type,created_at) VALUES (?,?,?,?,?)",
        )
        .bind(&relation_id)
        .bind(&id)
        .bind(*related_result_id)
        .bind(relation_type.as_str())
        .bind(&created_at)
        .execute(&mut *connection)
        .await?;
    }

    let audit_metadata = serde_json::json!({
        "result_type": input.result_type.as_str(),
        "correlation_key": input.correlation_key,
        "rule_or_model_id": input.rule_or_model_id,
        "rule_or_model_version": input.rule_or_model_version,
        "evidence_id": input.evidence_id,
        "supersedes": input.supersedes,
    })
    .to_string();

    crate::persistence::audit::append_on_connection(
        connection,
        input.actor_id,
        "ANALYTICAL_RESULT_RECORDED",
        "analytical_result",
        &id,
        input.incident_id,
        &audit_metadata,
    )
    .await?;

    Ok(id)
}

/// Insere um resultado analítico em sua própria transação serializada.
///
/// Chamadores que precisam de várias mutações no banco em uma única
/// transação atômica devem usar [`insert_on_connection`] a partir da sua
/// própria transação.
pub async fn insert(
    pool: &SqlitePool,
    input: NewAnalyticalResult<'_>,
) -> Result<String, sqlx::Error> {
    let mut connection = pool.acquire().await?;
    sqlx::query("BEGIN IMMEDIATE")
        .execute(&mut *connection)
        .await?;
    let result = insert_on_connection(&mut connection, input).await;
    match result {
        Ok(id) => {
            sqlx::query("COMMIT").execute(&mut *connection).await?;
            Ok(id)
        }
        Err(error) => {
            rollback_loudly(&mut connection).await;
            Err(error)
        }
    }
}

// ---------------------------------------------------------------------
// Leitura
// ---------------------------------------------------------------------

pub async fn get(
    pool: &SqlitePool,
    id: &str,
) -> Result<Option<AnalyticalResultRecord>, sqlx::Error> {
    let row = sqlx::query_as::<_, ResultRow>(
        "SELECT sequence,id,schema_version,result_type,correlation_key,evidence_id,incident_id,analysis_job_id,rule_or_model_id,rule_or_model_version,confidence,payload,provenance,supersedes,created_at \
         FROM analytical_results WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(AnalyticalResultRecord::from))
}

/// Resultado "atual" de uma `correlation_key`: a linha mais recente que
/// nenhuma linha posterior referencia via `supersedes`.
pub async fn current_for_correlation_key(
    pool: &SqlitePool,
    correlation_key: &str,
) -> Result<Option<AnalyticalResultRecord>, sqlx::Error> {
    let row = sqlx::query_as::<_, ResultRow>(
        "SELECT ar.sequence,ar.id,ar.schema_version,ar.result_type,ar.correlation_key,ar.evidence_id,ar.incident_id,ar.analysis_job_id,ar.rule_or_model_id,ar.rule_or_model_version,ar.confidence,ar.payload,ar.provenance,ar.supersedes,ar.created_at \
         FROM analytical_results ar \
         WHERE ar.correlation_key = ? \
         AND NOT EXISTS (SELECT 1 FROM analytical_results newer WHERE newer.supersedes = ar.id) \
         ORDER BY ar.sequence DESC LIMIT 1",
    )
    .bind(correlation_key)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(AnalyticalResultRecord::from))
}

#[cfg(test)]
/// Histórico completo de uma `correlation_key`, em ordem de inserção.
pub async fn history_for_correlation_key(
    pool: &SqlitePool,
    correlation_key: &str,
) -> Result<Vec<AnalyticalResultRecord>, sqlx::Error> {
    let rows = sqlx::query_as::<_, ResultRow>(
        "SELECT sequence,id,schema_version,result_type,correlation_key,evidence_id,incident_id,analysis_job_id,rule_or_model_id,rule_or_model_version,confidence,payload,provenance,supersedes,created_at \
         FROM analytical_results WHERE correlation_key = ? ORDER BY sequence ASC",
    )
    .bind(correlation_key)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(AnalyticalResultRecord::from).collect())
}

#[cfg(test)]
pub async fn list_by_evidence(
    pool: &SqlitePool,
    evidence_id: &str,
) -> Result<Vec<AnalyticalResultRecord>, sqlx::Error> {
    let rows = sqlx::query_as::<_, ResultRow>(
        "SELECT sequence,id,schema_version,result_type,correlation_key,evidence_id,incident_id,analysis_job_id,rule_or_model_id,rule_or_model_version,confidence,payload,provenance,supersedes,created_at \
         FROM analytical_results WHERE evidence_id = ? ORDER BY created_at DESC",
    )
    .bind(evidence_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(AnalyticalResultRecord::from).collect())
}

/// Resultados **vigentes** da evidência: os que nenhuma linha posterior
/// substitui via `supersedes` (o histórico fica em
/// [`history_for_correlation_key`]). Sem corte de quantidade (ADR-0020).
pub async fn list_current_by_evidence(
    pool: &SqlitePool,
    evidence_id: &str,
) -> Result<Vec<AnalyticalResultRecord>, sqlx::Error> {
    let rows = sqlx::query_as::<_, ResultRow>(
        "SELECT ar.sequence,ar.id,ar.schema_version,ar.result_type,ar.correlation_key,ar.evidence_id,ar.incident_id,ar.analysis_job_id,ar.rule_or_model_id,ar.rule_or_model_version,ar.confidence,ar.payload,ar.provenance,ar.supersedes,ar.created_at \
         FROM analytical_results ar \
         WHERE ar.evidence_id = ? \
         AND NOT EXISTS (SELECT 1 FROM analytical_results newer WHERE newer.supersedes = ar.id) \
         ORDER BY ar.created_at DESC, ar.sequence DESC",
    )
    .bind(evidence_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(AnalyticalResultRecord::from).collect())
}

pub async fn list_by_incident(
    pool: &SqlitePool,
    incident_id: &str,
) -> Result<Vec<AnalyticalResultRecord>, sqlx::Error> {
    let rows = sqlx::query_as::<_, ResultRow>(
        "SELECT sequence,id,schema_version,result_type,correlation_key,evidence_id,incident_id,analysis_job_id,rule_or_model_id,rule_or_model_version,confidence,payload,provenance,supersedes,created_at \
         FROM analytical_results WHERE incident_id = ? ORDER BY created_at DESC",
    )
    .bind(incident_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(AnalyticalResultRecord::from).collect())
}

/// Todas as `correlation_key` distintas já usadas por um `result_type`, sem
/// a ordenação por `created_at` de [`list_by_incident`] (que não tem mais corte de quantidade)
/// (que existe para telas de "resultados recentes", não para uma varredura
/// que precisa ser completa). Usado por F1-03 para achar, com certeza,
/// todo `NIST_GAP` já visto para um incidente — inclusive um que não tenha
/// sido tocado em rodadas recentes o bastante para caber no corte acima —
/// ao decidir quais gaps de rodadas anteriores precisam ser fechados.
pub async fn distinct_correlation_keys(
    pool: &SqlitePool,
    result_type: AnalyticalResultType,
) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar::<_, String>(
        "SELECT DISTINCT correlation_key FROM analytical_results WHERE result_type = ?",
    )
    .bind(result_type.as_str())
    .fetch_all(pool)
    .await
}

/// Resultados relacionados a `result_id` (os alvos das relações que ele
/// declara), junto com o tipo de relação.
pub async fn related_to(
    pool: &SqlitePool,
    result_id: &str,
) -> Result<Vec<(RelationType, AnalyticalResultRecord)>, sqlx::Error> {
    let relations: Vec<(String, String)> = sqlx::query_as(
        "SELECT related_result_id,relation_type FROM analytical_result_relations WHERE result_id = ? ORDER BY created_at ASC",
    )
    .bind(result_id)
    .fetch_all(pool)
    .await?;

    let mut results = Vec::with_capacity(relations.len());
    for (related_result_id, relation_type) in relations {
        let Some(kind) = RelationType::from_str(&relation_type) else {
            continue;
        };
        if let Some(record) = get(pool, &related_result_id).await? {
            results.push((kind, record));
        }
    }
    Ok(results)
}

/// Cria uma relação entre dois resultados analíticos já existentes. Ambos
/// devem existir (não valida o `result_type` de nenhum dos dois — quem
/// decide se a combinação faz sentido é o chamador, ex.: F1-04 exige que
/// ambos sejam `CRITICAL_PATH_LINK` antes de chamar isto).
pub async fn insert_relation(
    pool: &SqlitePool,
    result_id: &str,
    related_result_id: &str,
    relation_type: RelationType,
) -> Result<(), sqlx::Error> {
    let mut connection = pool.acquire().await?;
    sqlx::query("BEGIN IMMEDIATE")
        .execute(&mut *connection)
        .await?;
    let outcome: Result<(), sqlx::Error> = async {
        let relation_id = Uuid::new_v4().to_string();
        let created_at = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true);
        sqlx::query(
            "INSERT INTO analytical_result_relations (id,result_id,related_result_id,relation_type,created_at) VALUES (?,?,?,?,?)",
        )
        .bind(&relation_id)
        .bind(result_id)
        .bind(related_result_id)
        .bind(relation_type.as_str())
        .bind(&created_at)
        .execute(&mut *connection)
        .await?;
        Ok(())
    }
    .await;
    match outcome {
        Ok(()) => {
            sqlx::query("COMMIT").execute(&mut *connection).await?;
            Ok(())
        }
        Err(error) => {
            rollback_loudly(&mut connection).await;
            Err(error)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn test_pool() -> SqlitePool {
        let temp = tempfile::tempdir().expect("temporary directory");
        let db_path = temp.path().join("analytical-results.sqlite");
        let url = format!("sqlite://{}", db_path.display());
        // `temp` must outlive the pool for the file-backed database to stay
        // valid; leak the guard for the lifetime of the test process, mirroring
        // audit.rs's tempdir-per-test pattern (each test gets its own file).
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

    async fn seed_evidence(pool: &SqlitePool, id: &str) {
        sqlx::query(
            "INSERT INTO evidence(id,original_filename,storage_reference,format,size_bytes,blake3,sha256,imported_at,status) VALUES(?,?,?,?,?,?,?,?,?)",
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
        .execute(pool)
        .await
        .expect("evidence insert");
    }

    #[test]
    fn analytical_result_type_round_trips_as_str() {
        for kind in [
            AnalyticalResultType::YaraMatch,
            AnalyticalResultType::NistGap,
            AnalyticalResultType::AttackMapping,
            AnalyticalResultType::CriticalPathLink,
            AnalyticalResultType::ImpactAssessment,
            AnalyticalResultType::Ioc,
            AnalyticalResultType::Recommendation,
            AnalyticalResultType::Finding,
            AnalyticalResultType::RootCause,
            AnalyticalResultType::PlanItem,
            AnalyticalResultType::SourceGap,
        ] {
            assert_eq!(AnalyticalResultType::from_str(kind.as_str()), Some(kind));
        }
        assert_eq!(AnalyticalResultType::from_str("BOGUS"), None);
    }

    #[test]
    fn finding_root_cause_and_plan_item_have_stable_wire_names_and_requirements() {
        assert_eq!(AnalyticalResultType::Finding.as_str(), "FINDING");
        assert_eq!(AnalyticalResultType::RootCause.as_str(), "ROOT_CAUSE");
        assert_eq!(AnalyticalResultType::PlanItem.as_str(), "PLAN_ITEM");

        // Os três são julgamentos sobre o incidente, não sobre uma evidência.
        for kind in [
            AnalyticalResultType::Finding,
            AnalyticalResultType::RootCause,
            AnalyticalResultType::PlanItem,
        ] {
            assert!(kind.requires_incident());
            assert!(!kind.requires_evidence());
        }
        // Finding e RootCause são inferências sobre evidência → confiança
        // obrigatória. PlanItem é decisão de plano → confiança não é exigida.
        assert!(AnalyticalResultType::Finding.requires_confidence());
        assert!(AnalyticalResultType::RootCause.requires_confidence());
        assert!(!AnalyticalResultType::PlanItem.requires_confidence());
    }

    #[test]
    fn confidence_round_trips_as_str() {
        for level in [
            Confidence::Low,
            Confidence::Medium,
            Confidence::High,
            Confidence::Confirmed,
        ] {
            assert_eq!(Confidence::from_str(level.as_str()), Some(level));
        }
    }

    #[test]
    fn payload_canonicalization_is_key_order_independent() {
        let a = canonicalize_json(r#"{"z":1,"a":2}"#).expect("valid payload");
        let b = canonicalize_json(r#"{"a":2,"z":1}"#).expect("valid payload");
        assert_eq!(a, b);
    }

    #[test]
    fn canonicalization_rejects_unsafe_integers() {
        let result = canonicalize_json(r#"{"n":9007199254740993}"#);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn insert_requires_evidence_or_incident_link() {
        let pool = test_pool().await;
        let result = insert(
            &pool,
            NewAnalyticalResult {
                result_type: AnalyticalResultType::YaraMatch,
                correlation_key: "corr-1",
                evidence_id: None,
                incident_id: None,
                analysis_job_id: None,
                rule_or_model_id: None,
                rule_or_model_version: None,
                confidence: None,
                payload_json: "{}",
                provenance_json: "{}",
                supersedes: None,
                related: &[],
                actor_id: None,
            },
        )
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn insert_requires_confidence_when_type_demands_it() {
        let pool = test_pool().await;
        seed_incident(&pool, "inc-1").await;
        let result = insert(
            &pool,
            NewAnalyticalResult {
                result_type: AnalyticalResultType::AttackMapping,
                correlation_key: "corr-2",
                evidence_id: None,
                incident_id: Some("inc-1"),
                analysis_job_id: None,
                rule_or_model_id: None,
                rule_or_model_version: None,
                confidence: None,
                payload_json: "{}",
                provenance_json: "{}",
                supersedes: None,
                related: &[],
                actor_id: None,
            },
        )
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn insert_writes_audit_event() {
        let pool = test_pool().await;
        seed_evidence(&pool, "ev-1").await;
        let id = insert(
            &pool,
            NewAnalyticalResult {
                result_type: AnalyticalResultType::YaraMatch,
                correlation_key: "corr-3",
                evidence_id: Some("ev-1"),
                incident_id: None,
                analysis_job_id: None,
                rule_or_model_id: Some("rule-1"),
                rule_or_model_version: Some("1.0.0"),
                confidence: None,
                payload_json: r#"{"match":"true"}"#,
                provenance_json: r#"{"producer":"dfir-engine"}"#,
                supersedes: None,
                related: &[],
                actor_id: None,
            },
        )
        .await
        .expect("insert succeeds");

        let audit_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM audit_events WHERE action='ANALYTICAL_RESULT_RECORDED' AND object_id=?",
        )
        .bind(&id)
        .fetch_one(&pool)
        .await
        .expect("audit query");
        assert_eq!(audit_count, 1);
    }

    #[tokio::test]
    async fn reprocessing_preserves_history_via_supersedes() {
        let pool = test_pool().await;
        seed_evidence(&pool, "ev-2").await;
        let first_id = insert(
            &pool,
            NewAnalyticalResult {
                result_type: AnalyticalResultType::YaraMatch,
                correlation_key: "corr-4",
                evidence_id: Some("ev-2"),
                incident_id: None,
                analysis_job_id: None,
                rule_or_model_id: Some("rule-1"),
                rule_or_model_version: Some("1.0.0"),
                confidence: None,
                payload_json: r#"{"version":1}"#,
                provenance_json: "{}",
                supersedes: None,
                related: &[],
                actor_id: None,
            },
        )
        .await
        .expect("first insert succeeds");

        let second_id = insert(
            &pool,
            NewAnalyticalResult {
                result_type: AnalyticalResultType::YaraMatch,
                correlation_key: "corr-4",
                evidence_id: Some("ev-2"),
                incident_id: None,
                analysis_job_id: None,
                rule_or_model_id: Some("rule-1"),
                rule_or_model_version: Some("1.1.0"),
                confidence: None,
                payload_json: r#"{"version":2}"#,
                provenance_json: "{}",
                supersedes: Some(&first_id),
                related: &[],
                actor_id: None,
            },
        )
        .await
        .expect("second insert succeeds");

        let current = current_for_correlation_key(&pool, "corr-4")
            .await
            .expect("query current")
            .expect("current exists");
        assert_eq!(current.id, second_id);

        let history = history_for_correlation_key(&pool, "corr-4")
            .await
            .expect("query history");
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].id, first_id);
        assert_eq!(history[1].id, second_id);

        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM analytical_results")
            .fetch_one(&pool)
            .await
            .expect("count query");
        assert_eq!(total, 2, "superseding must not delete the prior row");
    }

    #[tokio::test]
    async fn a_superseded_result_cannot_gain_a_second_successor() {
        let pool = test_pool().await;
        seed_evidence(&pool, "ev-branch").await;
        fn new(supersedes: Option<&str>) -> NewAnalyticalResult<'_> {
            NewAnalyticalResult {
                result_type: AnalyticalResultType::YaraMatch,
                correlation_key: "corr-branch",
                evidence_id: Some("ev-branch"),
                incident_id: None,
                analysis_job_id: None,
                rule_or_model_id: Some("rule-1"),
                rule_or_model_version: Some("1.0.0"),
                confidence: None,
                payload_json: "{}",
                provenance_json: "{}",
                supersedes,
                related: &[],
                actor_id: None,
            }
        }
        let first = insert(&pool, new(None)).await.expect("first");
        let second = insert(&pool, new(Some(&first)))
            .await
            .expect("second supersedes first");

        let third = insert(&pool, new(Some(&first))).await;
        let current_rows = list_current_by_evidence(&pool, "ev-branch")
            .await
            .expect("current rows");
        let selected = current_for_correlation_key(&pool, "corr-branch")
            .await
            .expect("selected current")
            .expect("current exists");
        assert!(
            third.is_err(),
            "second successor accepted={} current_rows={} selected_is_third={}",
            third.is_ok(),
            current_rows.len(),
            third.as_ref().is_ok_and(|id| id == &selected.id)
        );
        let error = third.expect_err("second successor must fail");
        assert!(
            error.to_string().contains("already superseded"),
            "unexpected error: {error}"
        );
        assert_eq!(current_rows.len(), 1);
        assert_eq!(current_rows[0].id, second);
        assert_eq!(selected.id, second);
        assert_eq!(
            history_for_correlation_key(&pool, "corr-branch")
                .await
                .expect("history")
                .len(),
            2
        );
        let audit_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM audit_events WHERE action='ANALYTICAL_RESULT_RECORDED'",
        )
        .fetch_one(&pool)
        .await
        .expect("audit count");
        assert_eq!(audit_count, 2, "rejected successor must not write audit");
    }

    #[tokio::test]
    async fn current_listing_by_evidence_hides_superseded_rows_and_has_no_cap() {
        let pool = test_pool().await;
        seed_evidence(&pool, "ev-cur").await;
        let new = |key: &'static str, supersedes: Option<&'static str>| NewAnalyticalResult {
            result_type: AnalyticalResultType::YaraMatch,
            correlation_key: key,
            evidence_id: Some("ev-cur"),
            incident_id: None,
            analysis_job_id: None,
            rule_or_model_id: Some("rule-1"),
            rule_or_model_version: Some("1.0.0"),
            confidence: None,
            payload_json: "{}",
            provenance_json: "{}",
            supersedes,
            related: &[],
            actor_id: None,
        };
        let first = insert(&pool, new("yara:r:ev-cur", None))
            .await
            .expect("first");
        let first: &'static str = Box::leak(first.into_boxed_str());
        let second = insert(&pool, new("yara:r:ev-cur", Some(first)))
            .await
            .expect("second supersedes first");
        // Mais de 500 chaves distintas: a listagem antiga cortava em 500.
        for index in 0..600 {
            let key: &'static str = Box::leak(format!("yara:bulk-{index}:ev-cur").into_boxed_str());
            insert(&pool, new(key, None)).await.expect("bulk insert");
        }

        let current = list_current_by_evidence(&pool, "ev-cur")
            .await
            .expect("current");

        assert_eq!(
            current.len(),
            601,
            "600 avulsos + 1 vigente da chave substituída"
        );
        assert!(current.iter().any(|record| record.id == second));
        assert!(current.iter().all(|record| record.id != first));
        let everything = list_by_evidence(&pool, "ev-cur").await.expect("all rows");
        assert_eq!(
            everything.len(),
            602,
            "sem LIMIT: todas as linhas, inclusive o histórico"
        );
    }

    #[tokio::test]
    async fn a_correlation_key_cannot_have_two_root_results() {
        let pool = test_pool().await;
        seed_evidence(&pool, "ev-1").await;
        fn new() -> NewAnalyticalResult<'static> {
            NewAnalyticalResult {
                result_type: AnalyticalResultType::YaraMatch,
                correlation_key: "corr-root",
                evidence_id: Some("ev-1"),
                incident_id: None,
                analysis_job_id: None,
                rule_or_model_id: Some("rule-1"),
                rule_or_model_version: Some("1.0.0"),
                confidence: None,
                payload_json: "{}",
                provenance_json: "{}",
                supersedes: None,
                related: &[],
                actor_id: None,
            }
        }
        let first = insert(&pool, new()).await.expect("first root");
        let second = insert(&pool, new()).await;
        let current = list_current_by_evidence(&pool, "ev-1").await.unwrap();
        let selected = current_for_correlation_key(&pool, "corr-root")
            .await
            .unwrap()
            .unwrap();
        assert!(
            second.is_err(),
            "second root accepted=true current_rows={} selected_is_second={}",
            current.len(),
            second.as_ref().is_ok_and(|id| id == &selected.id)
        );
        let error = second.unwrap_err().to_string();
        assert!(
            error.contains("supersedes is required"),
            "unexpected error: {error}"
        );
        let current = list_current_by_evidence(&pool, "ev-1").await.unwrap();
        assert_eq!(current.len(), 1);
        assert_eq!(current[0].id, first);
        assert_eq!(
            current_for_correlation_key(&pool, "corr-root")
                .await
                .unwrap()
                .unwrap()
                .id,
            first
        );
        assert_eq!(
            history_for_correlation_key(&pool, "corr-root")
                .await
                .expect("history")
                .len(),
            1
        );
        let audit_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM audit_events WHERE action = 'ANALYTICAL_RESULT_RECORDED'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(audit_count, 1);
    }

    #[tokio::test]
    async fn supersedes_must_share_correlation_key() {
        let pool = test_pool().await;
        seed_evidence(&pool, "ev-3").await;
        let first_id = insert(
            &pool,
            NewAnalyticalResult {
                result_type: AnalyticalResultType::YaraMatch,
                correlation_key: "corr-5",
                evidence_id: Some("ev-3"),
                incident_id: None,
                analysis_job_id: None,
                rule_or_model_id: None,
                rule_or_model_version: None,
                confidence: None,
                payload_json: "{}",
                provenance_json: "{}",
                supersedes: None,
                related: &[],
                actor_id: None,
            },
        )
        .await
        .expect("first insert succeeds");

        let mismatched = insert(
            &pool,
            NewAnalyticalResult {
                result_type: AnalyticalResultType::YaraMatch,
                correlation_key: "corr-6",
                evidence_id: Some("ev-3"),
                incident_id: None,
                analysis_job_id: None,
                rule_or_model_id: None,
                rule_or_model_version: None,
                confidence: None,
                payload_json: "{}",
                provenance_json: "{}",
                supersedes: Some(&first_id),
                related: &[],
                actor_id: None,
            },
        )
        .await;
        assert!(mismatched.is_err());
    }

    #[tokio::test]
    async fn analytical_results_table_is_append_only() {
        let pool = test_pool().await;
        seed_evidence(&pool, "ev-4").await;
        let id = insert(
            &pool,
            NewAnalyticalResult {
                result_type: AnalyticalResultType::YaraMatch,
                correlation_key: "corr-7",
                evidence_id: Some("ev-4"),
                incident_id: None,
                analysis_job_id: None,
                rule_or_model_id: None,
                rule_or_model_version: None,
                confidence: None,
                payload_json: "{}",
                provenance_json: "{}",
                supersedes: None,
                related: &[],
                actor_id: None,
            },
        )
        .await
        .expect("insert succeeds");

        let update_result = sqlx::query("UPDATE analytical_results SET payload='{}' WHERE id=?")
            .bind(&id)
            .execute(&pool)
            .await;
        assert!(update_result.is_err(), "UPDATE must be rejected by trigger");

        let delete_result = sqlx::query("DELETE FROM analytical_results WHERE id=?")
            .bind(&id)
            .execute(&pool)
            .await;
        assert!(delete_result.is_err(), "DELETE must be rejected by trigger");
    }

    #[tokio::test]
    async fn relations_link_results_and_are_queryable() {
        let pool = test_pool().await;
        seed_incident(&pool, "inc-2").await;
        let finding_id = insert(
            &pool,
            NewAnalyticalResult {
                result_type: AnalyticalResultType::AttackMapping,
                correlation_key: "corr-8",
                evidence_id: None,
                incident_id: Some("inc-2"),
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

        let recommendation_id = insert(
            &pool,
            NewAnalyticalResult {
                result_type: AnalyticalResultType::Recommendation,
                correlation_key: "corr-9",
                evidence_id: None,
                incident_id: Some("inc-2"),
                analysis_job_id: None,
                rule_or_model_id: None,
                rule_or_model_version: None,
                confidence: None,
                payload_json: "{}",
                provenance_json: "{}",
                supersedes: None,
                related: &[(finding_id.as_str(), RelationType::DerivedFrom)],
                actor_id: None,
            },
        )
        .await
        .expect("recommendation insert succeeds");

        let related = related_to(&pool, &recommendation_id)
            .await
            .expect("query relations");
        assert_eq!(related.len(), 1);
        assert_eq!(related[0].0, RelationType::DerivedFrom);
        assert_eq!(related[0].1.id, finding_id);
    }

    #[tokio::test]
    async fn insert_relation_links_two_existing_results() {
        let pool = test_pool().await;
        seed_incident(&pool, "inc-3b").await;
        let stage_a = insert(
            &pool,
            NewAnalyticalResult {
                result_type: AnalyticalResultType::CriticalPathLink,
                correlation_key: "corr-10",
                evidence_id: None,
                incident_id: Some("inc-3b"),
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
        .expect("stage a insert succeeds");
        let stage_b = insert(
            &pool,
            NewAnalyticalResult {
                result_type: AnalyticalResultType::CriticalPathLink,
                correlation_key: "corr-11",
                evidence_id: None,
                incident_id: Some("inc-3b"),
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
        .expect("stage b insert succeeds");

        insert_relation(&pool, &stage_a, &stage_b, RelationType::SequenceNext)
            .await
            .expect("relation insert succeeds");

        let related = related_to(&pool, &stage_a).await.expect("query relations");
        assert_eq!(related.len(), 1);
        assert_eq!(related[0].0, RelationType::SequenceNext);
        assert_eq!(related[0].1.id, stage_b);
    }

    #[tokio::test]
    async fn finding_root_cause_and_plan_item_can_be_inserted_and_linked_after_migration_0010() {
        // `test_pool` roda o MIGRATOR embutido: se a 0010 não estiver no
        // binário ou o CHECK de `result_type` não tiver sido estendido, o
        // primeiro insert abaixo falha.
        let pool = test_pool().await;
        seed_incident(&pool, "inc-f1").await;

        let finding = insert(
            &pool,
            NewAnalyticalResult {
                result_type: AnalyticalResultType::Finding,
                correlation_key: "finding:inc-f1:f-1",
                evidence_id: None,
                incident_id: Some("inc-f1"),
                analysis_job_id: None,
                rule_or_model_id: None,
                rule_or_model_version: None,
                confidence: Some(Confidence::High),
                payload_json: r#"{"role":"EXPLOITED_CONDITION"}"#,
                provenance_json: "{}",
                supersedes: None,
                related: &[],
                actor_id: None,
            },
        )
        .await
        .expect("finding insert succeeds");

        let root_cause = insert(
            &pool,
            NewAnalyticalResult {
                result_type: AnalyticalResultType::RootCause,
                correlation_key: "root-cause:inc-f1",
                evidence_id: None,
                incident_id: Some("inc-f1"),
                analysis_job_id: None,
                rule_or_model_id: None,
                rule_or_model_version: None,
                confidence: Some(Confidence::Medium),
                payload_json: r#"{"level":"PROBABLE_CAUSE"}"#,
                provenance_json: "{}",
                supersedes: None,
                related: &[(finding.as_str(), RelationType::DerivedFrom)],
                actor_id: None,
            },
        )
        .await
        .expect("root cause insert succeeds");

        let plan_item = insert(
            &pool,
            NewAnalyticalResult {
                result_type: AnalyticalResultType::PlanItem,
                correlation_key: "plan:inc-f1:p-1",
                evidence_id: None,
                incident_id: Some("inc-f1"),
                analysis_job_id: None,
                rule_or_model_id: None,
                rule_or_model_version: None,
                confidence: None,
                payload_json: r#"{"domain":"OS_HARDENING"}"#,
                provenance_json: "{}",
                supersedes: None,
                related: &[(finding.as_str(), RelationType::DerivedFrom)],
                actor_id: None,
            },
        )
        .await
        .expect("plan item insert succeeds");

        let root_cause_links = related_to(&pool, &root_cause)
            .await
            .expect("query relations");
        assert_eq!(root_cause_links.len(), 1);
        assert_eq!(root_cause_links[0].1.id, finding);

        let related = related_to(&pool, &plan_item)
            .await
            .expect("query relations");
        assert_eq!(related.len(), 1);
        assert_eq!(related[0].0, RelationType::DerivedFrom);
        assert_eq!(related[0].1.id, finding);
        assert_eq!(
            related[0].1.result_type,
            AnalyticalResultType::Finding.as_str()
        );
    }

    #[tokio::test]
    async fn finding_and_root_cause_require_confidence_and_all_new_types_require_incident() {
        let pool = test_pool().await;
        seed_incident(&pool, "inc-f2").await;
        seed_evidence(&pool, "ev-f2").await;

        for (result_type, key) in [
            (AnalyticalResultType::Finding, "finding:inc-f2:f-1"),
            (AnalyticalResultType::RootCause, "root-cause:inc-f2"),
        ] {
            let result = insert(
                &pool,
                NewAnalyticalResult {
                    result_type,
                    correlation_key: key,
                    evidence_id: None,
                    incident_id: Some("inc-f2"),
                    analysis_job_id: None,
                    rule_or_model_id: None,
                    rule_or_model_version: None,
                    confidence: None,
                    payload_json: "{}",
                    provenance_json: "{}",
                    supersedes: None,
                    related: &[],
                    actor_id: None,
                },
            )
            .await;
            assert!(
                result.is_err(),
                "{} sem confiança deve falhar",
                result_type.as_str()
            );
        }

        // Só evidência, sem incidente: ambos os tipos são do incidente.
        for (result_type, key, confidence) in [
            (
                AnalyticalResultType::Finding,
                "finding:ev-f2:f-2",
                Some(Confidence::High),
            ),
            (
                AnalyticalResultType::RootCause,
                "root-cause:ev-f2",
                Some(Confidence::High),
            ),
            (AnalyticalResultType::PlanItem, "plan:ev-f2:p-2", None),
        ] {
            let result = insert(
                &pool,
                NewAnalyticalResult {
                    result_type,
                    correlation_key: key,
                    evidence_id: Some("ev-f2"),
                    incident_id: None,
                    analysis_job_id: None,
                    rule_or_model_id: None,
                    rule_or_model_version: None,
                    confidence,
                    payload_json: "{}",
                    provenance_json: "{}",
                    supersedes: None,
                    related: &[],
                    actor_id: None,
                },
            )
            .await;
            assert!(
                result.is_err(),
                "{} sem incidente deve falhar",
                result_type.as_str()
            );
        }
    }
}
