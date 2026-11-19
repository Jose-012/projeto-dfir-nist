//! F1-03 — NIST Current/Target Profile e Gap Analysis reproduzível.
//!
//! Modelo (seção 13 da spec canônica):
//!
//! ```text
//! nist_profiles (CURRENT | TARGET, versionado, append-only)
//!   └── nist_profile_entries (uma por subcategoria avaliada)
//!            ↓ compute_gaps() — função pura, determinística
//!       analytical_results (F1-01, AnalyticalResultType::NistGap)
//! ```
//!
//! Current/Target Profile são dados **de entrada** (declarados por um
//! analista); o Gap é **derivado** deles por uma função pura
//! ([`compute_gaps`]) e persistido como resultado analítico versionado via
//! F1-01 — nunca calculado "ao vivo" sem persistência, para que RF-010
//! ("reproduzível a partir dos dados, sem Internet") valha literalmente: os
//! mesmos dois profiles sempre produzem os mesmos gaps.
//!
//! ## Por que o catálogo oficial de subcategorias do CSF 2.0 não está embutido
//!
//! Diferente do ruleset YARA (F1-02), este módulo **não** embute uma cópia
//! do catálogo oficial de ~106 subcategorias do NIST CSF 2.0. Transcrever
//! esse catálogo de memória para uma ferramenta usada em avaliações de
//! conformidade reais é um risco real de erro silencioso (um código ou
//! descrição levemente errado não seria pego por nenhum teste). Em vez
//! disso, `subcategory_id` é validado apenas quanto à **forma estrutural**
//! (`FUNCTION.CATEGORY-NN`, com FUNCTION em {GV,ID,PR,DE,RS,RC} — a
//! estrutura de 6 Functions é estável e está no próprio texto da seção 13),
//! e a descrição (`subcategory_label`) é fornecida pelo analista a partir da
//! publicação oficial do NIST. Ver ADR-0011 para a decisão completa.
//!
//! ## Tier é contexto do profile, não por subcategoria
//!
//! Seção 13.2 é explícita: "Não representar Tier como score oficial por
//! Function, Category ou Subcategory". `tier`/`tier_justification` vivem em
//! `nist_profiles` (um valor por versão de profile), nunca em
//! `nist_profile_entries`.
//!
use sqlx::{SqliteConnection, SqlitePool};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use super::{rollback_loudly, validation_error};
use crate::persistence::analytical_results::{
    self, AnalyticalResultType, Confidence, NewAnalyticalResult,
};

const VALID_FUNCTIONS: [&str; 6] = ["GV", "ID", "PR", "DE", "RS", "RC"];

/// Valida apenas a forma estrutural `FUNCTION.CATEGORY-NN` (ex.: `GV.OC-01`)
/// e o prefixo de Function conhecido — não valida contra um catálogo de
/// subcategorias oficiais (ver docs do módulo).
pub fn is_valid_subcategory_id(value: &str) -> bool {
    let Some((function, rest)) = value.split_once('.') else {
        return false;
    };
    if !VALID_FUNCTIONS.contains(&function) {
        return false;
    }
    let Some((category, number)) = rest.split_once('-') else {
        return false;
    };
    let category_ok = category.len() == 2 && category.chars().all(|c| c.is_ascii_uppercase());
    let number_ok = number.len() == 2 && number.chars().all(|c| c.is_ascii_digit());
    category_ok && number_ok
}

// ---------------------------------------------------------------------
// Tipos do envelope
// ---------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileType {
    Current,
    Target,
}

impl ProfileType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Current => "CURRENT",
            Self::Target => "TARGET",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "CURRENT" => Some(Self::Current),
            "TARGET" => Some(Self::Target),
            _ => None,
        }
    }
}

/// Convenção operacional do produto (seção 13.2), não um nível oficial do NIST.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImplementationStatus {
    Implemented,
    PartiallyImplemented,
    NotImplemented,
    NotAssessed,
    NotApplicable,
}

impl ImplementationStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Implemented => "IMPLEMENTED",
            Self::PartiallyImplemented => "PARTIALLY_IMPLEMENTED",
            Self::NotImplemented => "NOT_IMPLEMENTED",
            Self::NotAssessed => "NOT_ASSESSED",
            Self::NotApplicable => "NOT_APPLICABLE",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "IMPLEMENTED" => Some(Self::Implemented),
            "PARTIALLY_IMPLEMENTED" => Some(Self::PartiallyImplemented),
            "NOT_IMPLEMENTED" => Some(Self::NotImplemented),
            "NOT_ASSESSED" => Some(Self::NotAssessed),
            "NOT_APPLICABLE" => Some(Self::NotApplicable),
            _ => None,
        }
    }

    /// Posição ordinal para comparação current vs. target. `None` para
    /// NOT_ASSESSED/NOT_APPLICABLE — não são pontos numa escala.
    fn rank(self) -> Option<u8> {
        match self {
            Self::NotImplemented => Some(0),
            Self::PartiallyImplemented => Some(1),
            Self::Implemented => Some(2),
            Self::NotAssessed | Self::NotApplicable => None,
        }
    }
}

/// Tier (seção 13.2): contexto de governança/risco do profile inteiro,
/// nunca uma nota por subcategoria.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    Partial,
    RiskInformed,
    Repeatable,
    Adaptive,
}

impl Tier {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Partial => "TIER_1_PARTIAL",
            Self::RiskInformed => "TIER_2_RISK_INFORMED",
            Self::Repeatable => "TIER_3_REPEATABLE",
            Self::Adaptive => "TIER_4_ADAPTIVE",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "TIER_1_PARTIAL" => Some(Self::Partial),
            "TIER_2_RISK_INFORMED" => Some(Self::RiskInformed),
            "TIER_3_REPEATABLE" => Some(Self::Repeatable),
            "TIER_4_ADAPTIVE" => Some(Self::Adaptive),
            _ => None,
        }
    }
}

pub struct NewProfileEntry<'a> {
    pub subcategory_id: &'a str,
    pub subcategory_label: &'a str,
    pub implementation_status: ImplementationStatus,
    pub justification: Option<&'a str>,
    pub evidence_reference: Option<&'a str>,
    pub confidence: Option<Confidence>,
}

pub struct NewProfile<'a> {
    pub incident_id: &'a str,
    pub profile_type: ProfileType,
    pub version_label: &'a str,
    pub source: &'a str,
    pub tier: Option<Tier>,
    pub tier_justification: Option<&'a str>,
    pub entries: &'a [NewProfileEntry<'a>],
    pub actor_id: Option<&'a str>,
}

#[derive(Debug, Clone)]
pub struct ProfileRecord {
    pub id: String,
    pub version_label: String,
    pub source: String,
    pub tier: Option<String>,
    pub tier_justification: Option<String>,
    #[cfg(test)]
    pub supersedes: Option<String>,
    pub created_at: String,
}

type ProfileRow = (
    i64,
    String,
    String,
    String,
    String,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    String,
    Option<String>,
);

impl From<ProfileRow> for ProfileRecord {
    fn from(row: ProfileRow) -> Self {
        Self {
            id: row.1,
            version_label: row.4,
            source: row.5,
            tier: row.6,
            tier_justification: row.7,
            #[cfg(test)]
            supersedes: row.8,
            created_at: row.9,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProfileEntryRecord {
    pub subcategory_id: String,
    pub subcategory_label: String,
    pub implementation_status: String,
    pub confidence: Option<String>,
}

type ProfileEntryRow = (
    i64,
    String,
    String,
    String,
    String,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    String,
);

impl From<ProfileEntryRow> for ProfileEntryRecord {
    fn from(row: ProfileEntryRow) -> Self {
        Self {
            subcategory_id: row.3,
            subcategory_label: row.4,
            implementation_status: row.5,
            confidence: row.8,
        }
    }
}

// ---------------------------------------------------------------------
// Escrita
// ---------------------------------------------------------------------

/// Insere uma nova versão de profile (com suas entradas) em uma única
/// transação. Se já existir uma versão atual para `(incident_id,
/// profile_type)`, a nova versão a substitui automaticamente via
/// `supersedes` — a anterior nunca é alterada ou apagada.
pub async fn insert_profile(
    pool: &SqlitePool,
    input: NewProfile<'_>,
) -> Result<String, sqlx::Error> {
    if input.entries.is_empty() {
        return Err(validation_error(
            "a profile must have at least one subcategory entry",
        ));
    }
    let mut seen = HashSet::with_capacity(input.entries.len());
    for entry in input.entries {
        if !is_valid_subcategory_id(entry.subcategory_id) {
            return Err(validation_error(format!(
                "invalid CSF subcategory id shape: {}",
                entry.subcategory_id
            )));
        }
        if !seen.insert(entry.subcategory_id) {
            return Err(validation_error(format!(
                "duplicate subcategory id in submission: {}",
                entry.subcategory_id
            )));
        }
    }

    let previous = current_profile_for(pool, input.incident_id, input.profile_type).await?;

    let mut connection = pool.acquire().await?;
    sqlx::query("BEGIN IMMEDIATE")
        .execute(&mut *connection)
        .await?;
    let outcome = insert_profile_on_connection(&mut connection, &input, previous.as_ref()).await;
    match outcome {
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

async fn insert_profile_on_connection(
    connection: &mut SqliteConnection,
    input: &NewProfile<'_>,
    previous: Option<&ProfileRecord>,
) -> Result<String, sqlx::Error> {
    let profile_id = Uuid::new_v4().to_string();
    let created_at = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true);

    sqlx::query(
        "INSERT INTO nist_profiles (id,incident_id,profile_type,version_label,source,tier,tier_justification,supersedes,created_at,actor_id) VALUES (?,?,?,?,?,?,?,?,?,?)",
    )
    .bind(&profile_id)
    .bind(input.incident_id)
    .bind(input.profile_type.as_str())
    .bind(input.version_label)
    .bind(input.source)
    .bind(input.tier.map(Tier::as_str))
    .bind(input.tier_justification)
    .bind(previous.map(|record| record.id.as_str()))
    .bind(&created_at)
    .bind(input.actor_id)
    .execute(&mut *connection)
    .await?;

    for entry in input.entries {
        let entry_id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO nist_profile_entries (id,profile_id,subcategory_id,subcategory_label,implementation_status,justification,evidence_reference,confidence,created_at) VALUES (?,?,?,?,?,?,?,?,?)",
        )
        .bind(&entry_id)
        .bind(&profile_id)
        .bind(entry.subcategory_id)
        .bind(entry.subcategory_label)
        .bind(entry.implementation_status.as_str())
        .bind(entry.justification)
        .bind(entry.evidence_reference)
        .bind(entry.confidence.map(Confidence::as_str))
        .bind(&created_at)
        .execute(&mut *connection)
        .await?;
    }

    let metadata = serde_json::json!({
        "profile_id": &profile_id,
        "incident_id": input.incident_id,
        "profile_type": input.profile_type.as_str(),
        "version_label": input.version_label,
        "source": input.source,
        "entry_count": input.entries.len(),
        "supersedes": previous.map(|record| record.id.as_str()),
    })
    .to_string();
    crate::persistence::audit::append_on_connection(
        connection,
        input.actor_id,
        "NIST_PROFILE_RECORDED",
        "nist_profile",
        &profile_id,
        Some(input.incident_id),
        &metadata,
    )
    .await?;

    Ok(profile_id)
}

// ---------------------------------------------------------------------
// Leitura
// ---------------------------------------------------------------------

/// Versão atual (não superseded) de um profile para um incidente.
pub async fn current_profile_for(
    pool: &SqlitePool,
    incident_id: &str,
    profile_type: ProfileType,
) -> Result<Option<ProfileRecord>, sqlx::Error> {
    let row = sqlx::query_as::<_, ProfileRow>(
        "SELECT p.sequence,p.id,p.incident_id,p.profile_type,p.version_label,p.source,p.tier,p.tier_justification,p.supersedes,p.created_at,p.actor_id \
         FROM nist_profiles p \
         WHERE p.incident_id = ? AND p.profile_type = ? \
         AND NOT EXISTS (SELECT 1 FROM nist_profiles newer WHERE newer.supersedes = p.id) \
         ORDER BY p.sequence DESC LIMIT 1",
    )
    .bind(incident_id)
    .bind(profile_type.as_str())
    .fetch_optional(pool)
    .await?;
    Ok(row.map(ProfileRecord::from))
}

/// Todas as versões (histórico completo) de um profile para um incidente,
/// em ordem de inserção.
pub async fn profile_history(
    pool: &SqlitePool,
    incident_id: &str,
    profile_type: ProfileType,
) -> Result<Vec<ProfileRecord>, sqlx::Error> {
    let rows = sqlx::query_as::<_, ProfileRow>(
        "SELECT sequence,id,incident_id,profile_type,version_label,source,tier,tier_justification,supersedes,created_at,actor_id \
         FROM nist_profiles WHERE incident_id = ? AND profile_type = ? ORDER BY sequence ASC",
    )
    .bind(incident_id)
    .bind(profile_type.as_str())
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(ProfileRecord::from).collect())
}

pub async fn profile_entries(
    pool: &SqlitePool,
    profile_id: &str,
) -> Result<Vec<ProfileEntryRecord>, sqlx::Error> {
    let rows = sqlx::query_as::<_, ProfileEntryRow>(
        "SELECT sequence,id,profile_id,subcategory_id,subcategory_label,implementation_status,justification,evidence_reference,confidence,created_at \
         FROM nist_profile_entries WHERE profile_id = ? ORDER BY subcategory_id ASC",
    )
    .bind(profile_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(ProfileEntryRecord::from).collect())
}

// ---------------------------------------------------------------------
// Gap analysis
// ---------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GapEntry {
    pub subcategory_id: String,
    pub subcategory_label: String,
    pub current_status: String,
    pub target_status: String,
    /// Derivado deterministicamente da diferença de rank entre current e
    /// target — não é uma metodologia de risco completa (fora de escopo de
    /// F1-03); serve tanto como prioridade quanto como nível de risco
    /// aproximado até uma avaliação de impacto mais rica (F1-04) existir.
    pub priority: &'static str,
    pub confidence: Confidence,
}

fn combined_confidence(current: Option<Confidence>, target: Option<Confidence>) -> Confidence {
    fn rank(value: Confidence) -> u8 {
        match value {
            Confidence::Low => 0,
            Confidence::Medium => 1,
            Confidence::High => 2,
            Confidence::Confirmed => 3,
        }
    }
    match (current, target) {
        (Some(a), Some(b)) => {
            if rank(a) <= rank(b) {
                a
            } else {
                b
            }
        }
        (Some(a), None) => a,
        (None, Some(b)) => b,
        // Nenhum dos dois lados registrou confiança explícita: MEDIUM é o
        // padrão documentado (nem otimista/CONFIRMED nem pessimista/LOW).
        (None, None) => Confidence::Medium,
    }
}

/// Função pura e determinística: compara as entradas de um profile CURRENT
/// contra um profile TARGET e retorna um [`GapEntry`] para cada
/// subcategoria em que o current não atinge o target. Não acessa o banco —
/// é o núcleo testável isoladamente de RF-010 ("reproduzível a partir dos
/// dados").
///
/// Regras:
/// - subcategorias `NOT_APPLICABLE` (no target OU no current) são
///   ignoradas — não geram gap;
/// - uma subcategoria presente no target mas ausente no current é tratada
///   como `NOT_ASSESSED` (gap de prioridade alta: estado desconhecido);
/// - subcategorias presentes apenas no current (sem entrada no target) não
///   geram gap — o target não as elegeu como relevantes.
pub fn compute_gaps(
    current_entries: &[ProfileEntryRecord],
    target_entries: &[ProfileEntryRecord],
) -> Vec<GapEntry> {
    let current_by_subcategory: HashMap<&str, &ProfileEntryRecord> = current_entries
        .iter()
        .map(|entry| (entry.subcategory_id.as_str(), entry))
        .collect();

    let mut gaps = Vec::new();
    for target in target_entries {
        if target.implementation_status == ImplementationStatus::NotApplicable.as_str() {
            continue;
        }
        let current = current_by_subcategory
            .get(target.subcategory_id.as_str())
            .copied();
        if let Some(current) = current
            && current.implementation_status == ImplementationStatus::NotApplicable.as_str()
        {
            continue;
        }

        let current_status = current
            .map(|entry| entry.implementation_status.as_str())
            .unwrap_or(ImplementationStatus::NotAssessed.as_str());
        let current_rank =
            ImplementationStatus::from_str(current_status).and_then(ImplementationStatus::rank);
        let target_rank = ImplementationStatus::from_str(&target.implementation_status)
            .and_then(ImplementationStatus::rank);

        let (has_gap, priority) = match (current_rank, target_rank) {
            (Some(current_rank), Some(target_rank)) if current_rank < target_rank => {
                let priority = if target_rank - current_rank >= 2 {
                    "HIGH"
                } else {
                    "MEDIUM"
                };
                (true, priority)
            }
            (None, Some(_)) => (true, "HIGH"),
            _ => (false, "LOW"),
        };
        if !has_gap {
            continue;
        }

        let current_confidence = current
            .and_then(|entry| entry.confidence.as_deref())
            .and_then(Confidence::from_str);
        let target_confidence = target.confidence.as_deref().and_then(Confidence::from_str);

        gaps.push(GapEntry {
            subcategory_id: target.subcategory_id.clone(),
            subcategory_label: target.subcategory_label.clone(),
            current_status: current_status.to_string(),
            target_status: target.implementation_status.clone(),
            priority,
            confidence: combined_confidence(current_confidence, target_confidence),
        });
    }
    gaps
}

/// Um `NIST_GAP` vigente (não superseded, e sem `status: "CLOSED"` no
/// payload) para um incidente, como visto *antes* da rodada de
/// `run_gap_analysis` em andamento.
struct OpenGapRecord {
    correlation_key: String,
    result_id: String,
    subcategory_id: String,
    subcategory_label: String,
}

/// Todo `NIST_GAP` que hoje está vigente para `incident_id`. Usa
/// [`analytical_results::distinct_correlation_keys`] (não
/// `list_by_incident`, que carrega todas as linhas do incidente — inadequado
/// aqui: custo proporcional ao histórico inteiro) para achar toda
/// `correlation_key` de `NIST_GAP` já vista, filtra pelo prefixo deste
/// incidente e resolve cada uma para seu registro vigente.
async fn previously_open_gaps(
    pool: &SqlitePool,
    incident_id: &str,
) -> Result<Vec<OpenGapRecord>, sqlx::Error> {
    let prefix = gap_correlation_key(incident_id, "");
    let all_keys =
        analytical_results::distinct_correlation_keys(pool, AnalyticalResultType::NistGap).await?;

    let mut open = Vec::new();
    for correlation_key in all_keys {
        // Copia o sufixo para uma `String` própria: o `&str` devolvido por
        // `strip_prefix` empresta `correlation_key`, que é movido mais abaixo
        // para `OpenGapRecord` (E0505 em rustc 1.91).
        let Some(subcategory_id) = correlation_key.strip_prefix(&prefix).map(str::to_string) else {
            continue;
        };
        let Some(current) =
            analytical_results::current_for_correlation_key(pool, &correlation_key).await?
        else {
            continue;
        };
        let payload: serde_json::Value =
            serde_json::from_str(&current.payload).map_err(|error| {
                validation_error(format!(
                    "stored NIST gap payload is malformed JSON: {error}"
                ))
            })?;
        let is_closed = payload
            .get("status")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|status| status == "CLOSED");
        if is_closed {
            continue;
        }
        let invalid = |field| {
            validation_error(format!(
                "stored NIST gap {} has invalid {field}",
                current.id
            ))
        };
        payload
            .get("subcategory_id")
            .and_then(serde_json::Value::as_str)
            .filter(|value| is_valid_subcategory_id(value))
            .ok_or_else(|| invalid("subcategory_id"))?;
        if !matches!(
            payload.get("priority").and_then(serde_json::Value::as_str),
            Some("HIGH" | "MEDIUM" | "LOW")
        ) {
            return Err(invalid("priority"));
        }
        let subcategory_label = payload
            .get("subcategory_label")
            .and_then(serde_json::Value::as_str)
            .unwrap_or(&subcategory_id)
            .to_string();
        open.push(OpenGapRecord {
            correlation_key,
            result_id: current.id,
            subcategory_id,
            subcategory_label,
        });
    }
    Ok(open)
}

/// Recalcula a Gap Analysis a partir dos profiles CURRENT/TARGET atuais de
/// um incidente e persiste cada gap como `AnalyticalResultType::NistGap`
/// (F1-01), preservando histórico via `supersedes` quando uma gap anterior
/// já existir para a mesma subcategoria. Também fecha (supersede com
/// `status: "CLOSED"` no payload, sem inventar um novo `AnalyticalResultType`
/// só para isto) qualquer gap vigente de uma rodada anterior cuja
/// subcategoria não apareça mais em `gaps` nesta rodada — sem isso, um gap
/// já remediado (current profile passou a atender o target) continuaria
/// aparecendo como vigente para sempre, porque nada o superseded. Retorna
/// os ids de todos os resultados analíticos escritos nesta rodada — gaps
/// abertos e fechamentos incluídos (pode ser uma lista vazia — nenhuma
/// mudança de estado é um resultado válido, não um erro).
pub async fn run_gap_analysis(
    pool: &SqlitePool,
    incident_id: &str,
    actor_id: Option<&str>,
) -> Result<Vec<String>, sqlx::Error> {
    let current_profile = current_profile_for(pool, incident_id, ProfileType::Current)
        .await?
        .ok_or_else(|| validation_error("no CURRENT profile exists yet for this incident"))?;
    let target_profile = current_profile_for(pool, incident_id, ProfileType::Target)
        .await?
        .ok_or_else(|| validation_error("no TARGET profile exists yet for this incident"))?;

    let current_entries = profile_entries(pool, &current_profile.id).await?;
    let target_entries = profile_entries(pool, &target_profile.id).await?;
    let gaps = compute_gaps(&current_entries, &target_entries);
    let gap_subcategories: HashSet<&str> =
        gaps.iter().map(|gap| gap.subcategory_id.as_str()).collect();

    // Lookups de histórico (para encadear supersedes) são somente-leitura e
    // acontecem antes de abrir a transação de escrita — cada correlation_key
    // é independente entre si e nada aqui ainda foi escrito nesta rodada.
    let mut previous_ids = Vec::with_capacity(gaps.len());
    for gap in &gaps {
        let correlation_key = gap_correlation_key(incident_id, &gap.subcategory_id);
        let previous =
            analytical_results::current_for_correlation_key(pool, &correlation_key).await?;
        previous_ids.push(previous.map(|record| record.id));
    }

    // Gaps vigentes de rodadas anteriores cuja subcategoria não é mais gap
    // nesta rodada — cada um vira um registro de fechamento.
    let resolved: Vec<OpenGapRecord> = previously_open_gaps(pool, incident_id)
        .await?
        .into_iter()
        .filter(|record| !gap_subcategories.contains(record.subcategory_id.as_str()))
        .collect();
    let current_by_subcategory: HashMap<&str, &ProfileEntryRecord> = current_entries
        .iter()
        .map(|entry| (entry.subcategory_id.as_str(), entry))
        .collect();
    let target_by_subcategory: HashMap<&str, &ProfileEntryRecord> = target_entries
        .iter()
        .map(|entry| (entry.subcategory_id.as_str(), entry))
        .collect();

    let mut connection = pool.acquire().await?;
    sqlx::query("BEGIN IMMEDIATE")
        .execute(&mut *connection)
        .await?;
    let outcome: Result<Vec<String>, sqlx::Error> = async {
        let mut ids = Vec::with_capacity(gaps.len() + resolved.len());
        for (gap, previous_id) in gaps.iter().zip(previous_ids.iter()) {
            let correlation_key = gap_correlation_key(incident_id, &gap.subcategory_id);
            let payload_json = serde_json::json!({
                "kind": "NIST_GAP",
                "subcategory_id": gap.subcategory_id,
                "subcategory_label": gap.subcategory_label,
                "current_status": gap.current_status,
                "target_status": gap.target_status,
                "priority": gap.priority,
                "risk_level": gap.priority,
            })
            .to_string();
            let provenance_json = serde_json::json!({
                "producer": "dfir-app",
                "method": "deterministic implementation_status rank comparison (current vs target)",
                "current_profile_id": current_profile.id,
                "current_profile_version": current_profile.version_label,
                "target_profile_id": target_profile.id,
                "target_profile_version": target_profile.version_label,
            })
            .to_string();
            let id = analytical_results::insert_on_connection(
                &mut connection,
                NewAnalyticalResult {
                    result_type: AnalyticalResultType::NistGap,
                    correlation_key: correlation_key.as_str(),
                    evidence_id: None,
                    incident_id: Some(incident_id),
                    analysis_job_id: None,
                    rule_or_model_id: Some("nist-csf-2.0-gap"),
                    rule_or_model_version: Some("1"),
                    confidence: Some(gap.confidence),
                    payload_json: payload_json.as_str(),
                    provenance_json: provenance_json.as_str(),
                    supersedes: previous_id.as_deref(),
                    related: &[],
                    actor_id,
                },
            )
            .await?;
            ids.push(id);
        }

        for record in &resolved {
            let current = current_by_subcategory.get(record.subcategory_id.as_str());
            let target = target_by_subcategory.get(record.subcategory_id.as_str());
            // A subcategoria pode ter deixado de ser gap por dois motivos
            // distintos: o current profile passou a atender o target
            // (`target_met`), ou o target profile vigente simplesmente não
            // declara mais essa subcategoria como relevante, ou passou a
            // marcá-la NOT_APPLICABLE (`no_longer_targeted`). Os dois
            // fecham o gap da mesma forma; só o motivo registrado difere.
            let closed_reason = match target {
                Some(target)
                    if target.implementation_status
                        != ImplementationStatus::NotApplicable.as_str() =>
                {
                    "target_met"
                }
                _ => "no_longer_targeted",
            };
            let subcategory_label = target
                .map(|entry| entry.subcategory_label.as_str())
                .or_else(|| current.map(|entry| entry.subcategory_label.as_str()))
                .unwrap_or(record.subcategory_label.as_str());
            let resolved_current_status = current
                .map(|entry| entry.implementation_status.as_str())
                .unwrap_or(ImplementationStatus::NotAssessed.as_str());
            let current_confidence = current
                .and_then(|entry| entry.confidence.as_deref())
                .and_then(Confidence::from_str);
            let target_confidence = target
                .and_then(|entry| entry.confidence.as_deref())
                .and_then(Confidence::from_str);

            let payload_json = serde_json::json!({
                "kind": "NIST_GAP",
                "status": "CLOSED",
                "closed_reason": closed_reason,
                "subcategory_id": record.subcategory_id,
                "subcategory_label": subcategory_label,
                "resolved_current_status": resolved_current_status,
            })
            .to_string();
            let provenance_json = serde_json::json!({
                "producer": "dfir-app",
                "method": "deterministic implementation_status rank comparison (current vs target)",
                "current_profile_id": current_profile.id,
                "current_profile_version": current_profile.version_label,
                "target_profile_id": target_profile.id,
                "target_profile_version": target_profile.version_label,
            })
            .to_string();
            let id = analytical_results::insert_on_connection(
                &mut connection,
                NewAnalyticalResult {
                    result_type: AnalyticalResultType::NistGap,
                    correlation_key: record.correlation_key.as_str(),
                    evidence_id: None,
                    incident_id: Some(incident_id),
                    analysis_job_id: None,
                    rule_or_model_id: Some("nist-csf-2.0-gap"),
                    rule_or_model_version: Some("1"),
                    confidence: Some(combined_confidence(current_confidence, target_confidence)),
                    payload_json: payload_json.as_str(),
                    provenance_json: provenance_json.as_str(),
                    supersedes: Some(record.result_id.as_str()),
                    related: &[],
                    actor_id,
                },
            )
            .await?;
            ids.push(id);
        }
        Ok(ids)
    }
    .await;

    match outcome {
        Ok(ids) => {
            sqlx::query("COMMIT").execute(&mut *connection).await?;
            Ok(ids)
        }
        Err(error) => {
            rollback_loudly(&mut connection).await;
            Err(error)
        }
    }
}

fn gap_correlation_key(incident_id: &str, subcategory_id: &str) -> String {
    format!("nist-gap:{incident_id}:{subcategory_id}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(
        subcategory_id: &str,
        status: ImplementationStatus,
        confidence: Option<Confidence>,
    ) -> ProfileEntryRecord {
        ProfileEntryRecord {
            subcategory_id: subcategory_id.into(),
            subcategory_label: format!("label for {subcategory_id}"),
            implementation_status: status.as_str().into(),
            confidence: confidence.map(|c| c.as_str().to_string()),
        }
    }

    #[test]
    fn subcategory_id_validation_accepts_known_shape_and_rejects_others() {
        assert!(is_valid_subcategory_id("GV.OC-01"));
        assert!(is_valid_subcategory_id("ID.AM-10"));
        assert!(
            !is_valid_subcategory_id("XX.OC-01"),
            "unknown function prefix"
        );
        assert!(!is_valid_subcategory_id("GV.OC01"), "missing dash");
        assert!(
            !is_valid_subcategory_id("GV.O-01"),
            "category must be 2 chars"
        );
        assert!(
            !is_valid_subcategory_id("GV.OC-1"),
            "number must be 2 digits"
        );
        assert!(!is_valid_subcategory_id("gv.oc-01"), "must be uppercase");
    }

    #[test]
    fn implementation_status_round_trips_as_str() {
        for status in [
            ImplementationStatus::Implemented,
            ImplementationStatus::PartiallyImplemented,
            ImplementationStatus::NotImplemented,
            ImplementationStatus::NotAssessed,
            ImplementationStatus::NotApplicable,
        ] {
            assert_eq!(
                ImplementationStatus::from_str(status.as_str()),
                Some(status)
            );
        }
    }

    #[test]
    fn compute_gaps_finds_a_gap_when_current_is_below_target() {
        let current = vec![entry(
            "GV.OC-01",
            ImplementationStatus::NotImplemented,
            None,
        )];
        let target = vec![entry("GV.OC-01", ImplementationStatus::Implemented, None)];
        let gaps = compute_gaps(&current, &target);
        assert_eq!(gaps.len(), 1);
        assert_eq!(gaps[0].subcategory_id, "GV.OC-01");
        assert_eq!(
            gaps[0].priority, "HIGH",
            "two-step rank jump is HIGH priority"
        );
    }

    #[test]
    fn compute_gaps_finds_no_gap_when_current_already_meets_target() {
        let current = vec![entry("GV.OC-01", ImplementationStatus::Implemented, None)];
        let target = vec![entry(
            "GV.OC-01",
            ImplementationStatus::PartiallyImplemented,
            None,
        )];
        assert!(compute_gaps(&current, &target).is_empty());
    }

    #[test]
    fn compute_gaps_treats_a_missing_current_entry_as_not_assessed_and_high_priority() {
        let current: Vec<ProfileEntryRecord> = vec![];
        let target = vec![entry(
            "ID.AM-01",
            ImplementationStatus::PartiallyImplemented,
            None,
        )];
        let gaps = compute_gaps(&current, &target);
        assert_eq!(gaps.len(), 1);
        assert_eq!(gaps[0].current_status, "NOT_ASSESSED");
        assert_eq!(gaps[0].priority, "HIGH");
    }

    #[test]
    fn compute_gaps_skips_not_applicable_on_either_side() {
        let current = vec![entry("GV.OC-01", ImplementationStatus::NotApplicable, None)];
        let target = vec![entry("GV.OC-01", ImplementationStatus::Implemented, None)];
        assert!(compute_gaps(&current, &target).is_empty());

        let current2 = vec![entry(
            "GV.OC-02",
            ImplementationStatus::NotImplemented,
            None,
        )];
        let target2 = vec![entry("GV.OC-02", ImplementationStatus::NotApplicable, None)];
        assert!(compute_gaps(&current2, &target2).is_empty());
    }

    #[test]
    fn compute_gaps_ignores_subcategories_only_present_in_current() {
        let current = vec![entry(
            "RC.RP-01",
            ImplementationStatus::NotImplemented,
            None,
        )];
        let target: Vec<ProfileEntryRecord> = vec![];
        assert!(compute_gaps(&current, &target).is_empty());
    }

    #[test]
    fn compute_gaps_is_deterministic_across_repeated_calls() {
        let current = vec![entry(
            "PR.AA-01",
            ImplementationStatus::PartiallyImplemented,
            Some(Confidence::High),
        )];
        let target = vec![entry(
            "PR.AA-01",
            ImplementationStatus::Implemented,
            Some(Confidence::Medium),
        )];
        let first = compute_gaps(&current, &target);
        let second = compute_gaps(&current, &target);
        assert_eq!(first, second);
    }

    #[test]
    fn combined_confidence_takes_the_more_conservative_side() {
        assert_eq!(
            combined_confidence(Some(Confidence::High), Some(Confidence::Low)),
            Confidence::Low
        );
        assert_eq!(
            combined_confidence(Some(Confidence::High), None),
            Confidence::High
        );
        assert_eq!(combined_confidence(None, None), Confidence::Medium);
    }

    async fn test_pool() -> SqlitePool {
        let temp = tempfile::tempdir().expect("temporary directory");
        let db_path = temp.path().join("nist-profiles.sqlite");
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

    #[tokio::test]
    async fn insert_profile_rejects_empty_entries() {
        let pool = test_pool().await;
        seed_incident(&pool, "inc-1").await;
        let result = insert_profile(
            &pool,
            NewProfile {
                incident_id: "inc-1",
                profile_type: ProfileType::Current,
                version_label: "v1",
                source: "test",
                tier: None,
                tier_justification: None,
                entries: &[],
                actor_id: None,
            },
        )
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn insert_profile_rejects_invalid_subcategory_id() {
        let pool = test_pool().await;
        seed_incident(&pool, "inc-2").await;
        let entries = [NewProfileEntry {
            subcategory_id: "NOT-VALID",
            subcategory_label: "bogus",
            implementation_status: ImplementationStatus::NotAssessed,
            justification: None,
            evidence_reference: None,
            confidence: None,
        }];
        let result = insert_profile(
            &pool,
            NewProfile {
                incident_id: "inc-2",
                profile_type: ProfileType::Current,
                version_label: "v1",
                source: "test",
                tier: None,
                tier_justification: None,
                entries: &entries,
                actor_id: None,
            },
        )
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn newer_profile_version_supersedes_the_previous_one_and_keeps_history() {
        let pool = test_pool().await;
        seed_incident(&pool, "inc-3").await;
        let entries_v1 = [NewProfileEntry {
            subcategory_id: "GV.OC-01",
            subcategory_label: "label",
            implementation_status: ImplementationStatus::NotImplemented,
            justification: None,
            evidence_reference: None,
            confidence: None,
        }];
        let v1_id = insert_profile(
            &pool,
            NewProfile {
                incident_id: "inc-3",
                profile_type: ProfileType::Current,
                version_label: "v1",
                source: "self-assessment",
                tier: Some(Tier::Partial),
                tier_justification: Some("initial"),
                entries: &entries_v1,
                actor_id: None,
            },
        )
        .await
        .expect("v1 insert succeeds");

        let entries_v2 = [NewProfileEntry {
            subcategory_id: "GV.OC-01",
            subcategory_label: "label",
            implementation_status: ImplementationStatus::Implemented,
            justification: None,
            evidence_reference: None,
            confidence: None,
        }];
        let v2_id = insert_profile(
            &pool,
            NewProfile {
                incident_id: "inc-3",
                profile_type: ProfileType::Current,
                version_label: "v2",
                source: "self-assessment",
                tier: Some(Tier::RiskInformed),
                tier_justification: Some("improved"),
                entries: &entries_v2,
                actor_id: None,
            },
        )
        .await
        .expect("v2 insert succeeds");

        let current = current_profile_for(&pool, "inc-3", ProfileType::Current)
            .await
            .expect("query current")
            .expect("current exists");
        assert_eq!(current.id, v2_id);
        assert_eq!(current.supersedes.as_deref(), Some(v1_id.as_str()));

        let history = profile_history(&pool, "inc-3", ProfileType::Current)
            .await
            .expect("query history");
        assert_eq!(
            history.len(),
            2,
            "superseding must not delete the prior version"
        );
    }

    #[tokio::test]
    async fn run_gap_analysis_requires_both_current_and_target_profiles() {
        let pool = test_pool().await;
        seed_incident(&pool, "inc-4").await;
        let result = run_gap_analysis(&pool, "inc-4", None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn run_gap_analysis_persists_gaps_as_analytical_results() {
        let pool = test_pool().await;
        seed_incident(&pool, "inc-5").await;

        let current_entries = [NewProfileEntry {
            subcategory_id: "ID.AM-01",
            subcategory_label: "Inventário de ativos",
            implementation_status: ImplementationStatus::NotImplemented,
            justification: None,
            evidence_reference: None,
            confidence: Some(Confidence::High),
        }];
        insert_profile(
            &pool,
            NewProfile {
                incident_id: "inc-5",
                profile_type: ProfileType::Current,
                version_label: "v1",
                source: "self-assessment",
                tier: None,
                tier_justification: None,
                entries: &current_entries,
                actor_id: None,
            },
        )
        .await
        .expect("current insert succeeds");

        let target_entries = [NewProfileEntry {
            subcategory_id: "ID.AM-01",
            subcategory_label: "Inventário de ativos",
            implementation_status: ImplementationStatus::Implemented,
            justification: None,
            evidence_reference: None,
            confidence: Some(Confidence::Medium),
        }];
        insert_profile(
            &pool,
            NewProfile {
                incident_id: "inc-5",
                profile_type: ProfileType::Target,
                version_label: "v1",
                source: "target framework",
                tier: None,
                tier_justification: None,
                entries: &target_entries,
                actor_id: None,
            },
        )
        .await
        .expect("target insert succeeds");

        let ids = run_gap_analysis(&pool, "inc-5", None)
            .await
            .expect("gap analysis succeeds");
        assert_eq!(ids.len(), 1);

        let gap_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM analytical_results WHERE result_type='NIST_GAP' AND incident_id='inc-5'",
        )
        .fetch_one(&pool)
        .await
        .expect("gap query");
        assert_eq!(gap_count, 1);

        // Re-running with unchanged profiles must supersede the previous gap
        // result, not duplicate it endlessly.
        let ids_again = run_gap_analysis(&pool, "inc-5", None)
            .await
            .expect("second gap analysis succeeds");
        assert_eq!(ids_again.len(), 1);
        assert_ne!(ids_again[0], ids[0]);

        let total_gap_rows: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM analytical_results WHERE result_type='NIST_GAP' AND incident_id='inc-5'",
        )
        .fetch_one(&pool)
        .await
        .expect("gap query");
        assert_eq!(total_gap_rows, 2, "history preserved, not overwritten");
    }

    // Regression test for the original bug: once a gap is fixed (current
    // profile improves to meet the target), `run_gap_analysis` must close
    // the old gap (supersede it with a `status: "CLOSED"` record) instead of
    // leaving it as the correlation_key's current record forever.
    #[tokio::test]
    async fn run_gap_analysis_rejects_malformed_stored_gap_before_writing() {
        let pool = test_pool().await;
        seed_incident(&pool, "inc-bad-gap").await;
        let entries = [NewProfileEntry {
            subcategory_id: "ID.AM-01",
            subcategory_label: "Inventário de ativos",
            implementation_status: ImplementationStatus::Implemented,
            justification: None,
            evidence_reference: None,
            confidence: Some(Confidence::High),
        }];
        for profile_type in [ProfileType::Current, ProfileType::Target] {
            insert_profile(
                &pool,
                NewProfile {
                    incident_id: "inc-bad-gap",
                    profile_type,
                    version_label: "v1",
                    source: "test",
                    tier: None,
                    tier_justification: None,
                    entries: &entries,
                    actor_id: None,
                },
            )
            .await
            .expect("profile insert succeeds");
        }
        let correlation_key = gap_correlation_key("inc-bad-gap", "ID.AM-01");
        sqlx::query("INSERT INTO analytical_results(id,schema_version,result_type,correlation_key,incident_id,rule_or_model_id,rule_or_model_version,confidence,payload,provenance,created_at) VALUES('bad-gap','analytical-result-v1','NIST_GAP',?,'inc-bad-gap','nist-csf-2.0-gap','1','MEDIUM','not-json','{}','2026-10-02T00:00:00Z')")
            .bind(&correlation_key)
            .execute(&pool)
            .await
            .expect("malformed gap fixture");

        let error = run_gap_analysis(&pool, "inc-bad-gap", None)
            .await
            .expect_err("gap inválido não pode ser fechado nem substituído");
        assert!(
            matches!(&error, sqlx::Error::Protocol(reason) if reason.contains("stored NIST gap payload is malformed JSON")),
            "{error:?}"
        );
        let rows: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM analytical_results WHERE incident_id='inc-bad-gap'",
        )
        .fetch_one(&pool)
        .await
        .expect("gap count");
        assert_eq!(rows, 1, "nenhuma escrita pode ocorrer antes do erro");
        let current = analytical_results::current_for_correlation_key(&pool, &correlation_key)
            .await
            .expect("current query")
            .expect("malformed current gap remains");
        assert_eq!(current.id, "bad-gap", "nenhum registro o supersede");
    }

    #[tokio::test]
    async fn run_gap_analysis_closes_a_gap_once_the_current_profile_meets_the_target() {
        let pool = test_pool().await;
        seed_incident(&pool, "inc-6").await;

        let target_entries = [NewProfileEntry {
            subcategory_id: "ID.AM-01",
            subcategory_label: "Inventário de ativos",
            implementation_status: ImplementationStatus::Implemented,
            justification: None,
            evidence_reference: None,
            confidence: Some(Confidence::Medium),
        }];
        insert_profile(
            &pool,
            NewProfile {
                incident_id: "inc-6",
                profile_type: ProfileType::Target,
                version_label: "v1",
                source: "target framework",
                tier: None,
                tier_justification: None,
                entries: &target_entries,
                actor_id: None,
            },
        )
        .await
        .expect("target insert succeeds");

        let current_v1 = [NewProfileEntry {
            subcategory_id: "ID.AM-01",
            subcategory_label: "Inventário de ativos",
            implementation_status: ImplementationStatus::NotImplemented,
            justification: None,
            evidence_reference: None,
            confidence: Some(Confidence::High),
        }];
        insert_profile(
            &pool,
            NewProfile {
                incident_id: "inc-6",
                profile_type: ProfileType::Current,
                version_label: "v1",
                source: "self-assessment",
                tier: None,
                tier_justification: None,
                entries: &current_v1,
                actor_id: None,
            },
        )
        .await
        .expect("current v1 insert succeeds");

        let opened = run_gap_analysis(&pool, "inc-6", None)
            .await
            .expect("first gap analysis succeeds");
        assert_eq!(opened.len(), 1, "one open gap");

        let correlation_key = gap_correlation_key("inc-6", "ID.AM-01");
        let open_record = analytical_results::current_for_correlation_key(&pool, &correlation_key)
            .await
            .expect("query current")
            .expect("open gap exists");
        assert_eq!(open_record.id, opened[0]);
        let open_payload: serde_json::Value =
            serde_json::from_str(&open_record.payload).expect("valid json");
        assert!(
            open_payload.get("status").is_none(),
            "an open gap has no status field"
        );

        // Current profile improves to meet the target.
        let current_v2 = [NewProfileEntry {
            subcategory_id: "ID.AM-01",
            subcategory_label: "Inventário de ativos",
            implementation_status: ImplementationStatus::Implemented,
            justification: None,
            evidence_reference: None,
            confidence: Some(Confidence::High),
        }];
        insert_profile(
            &pool,
            NewProfile {
                incident_id: "inc-6",
                profile_type: ProfileType::Current,
                version_label: "v2",
                source: "self-assessment",
                tier: None,
                tier_justification: None,
                entries: &current_v2,
                actor_id: None,
            },
        )
        .await
        .expect("current v2 insert succeeds");

        let closed = run_gap_analysis(&pool, "inc-6", None)
            .await
            .expect("second gap analysis succeeds");
        assert_eq!(closed.len(), 1, "one closing record, no new open gap");

        let closed_record =
            analytical_results::current_for_correlation_key(&pool, &correlation_key)
                .await
                .expect("query current")
                .expect("closed record exists");
        assert_eq!(closed_record.id, closed[0]);
        assert_eq!(
            closed_record.supersedes.as_deref(),
            Some(open_record.id.as_str())
        );
        let closed_payload: serde_json::Value =
            serde_json::from_str(&closed_record.payload).expect("valid json");
        assert_eq!(closed_payload["status"], "CLOSED");
        assert_eq!(closed_payload["closed_reason"], "target_met");
        assert_eq!(closed_payload["resolved_current_status"], "IMPLEMENTED");

        let total_gap_rows: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM analytical_results WHERE result_type='NIST_GAP' AND incident_id='inc-6'",
        )
        .fetch_one(&pool)
        .await
        .expect("gap query");
        assert_eq!(
            total_gap_rows, 2,
            "open gap kept in history, closing appended"
        );

        // Recalculating again with the same (already-met) profiles must not
        // keep re-closing the same gap on every run.
        let unchanged = run_gap_analysis(&pool, "inc-6", None)
            .await
            .expect("third gap analysis succeeds");
        assert!(
            unchanged.is_empty(),
            "an already-closed gap must not be closed again"
        );
    }

    // Same bug, different trigger: the target profile itself stops
    // targeting the subcategory (rather than the current profile improving).
    #[tokio::test]
    async fn run_gap_analysis_closes_a_gap_when_the_target_stops_requiring_it() {
        let pool = test_pool().await;
        seed_incident(&pool, "inc-7").await;

        let current_entries = [NewProfileEntry {
            subcategory_id: "ID.AM-01",
            subcategory_label: "Inventário de ativos",
            implementation_status: ImplementationStatus::NotImplemented,
            justification: None,
            evidence_reference: None,
            confidence: None,
        }];
        insert_profile(
            &pool,
            NewProfile {
                incident_id: "inc-7",
                profile_type: ProfileType::Current,
                version_label: "v1",
                source: "self-assessment",
                tier: None,
                tier_justification: None,
                entries: &current_entries,
                actor_id: None,
            },
        )
        .await
        .expect("current insert succeeds");

        let target_v1 = [NewProfileEntry {
            subcategory_id: "ID.AM-01",
            subcategory_label: "Inventário de ativos",
            implementation_status: ImplementationStatus::Implemented,
            justification: None,
            evidence_reference: None,
            confidence: None,
        }];
        insert_profile(
            &pool,
            NewProfile {
                incident_id: "inc-7",
                profile_type: ProfileType::Target,
                version_label: "v1",
                source: "target framework",
                tier: None,
                tier_justification: None,
                entries: &target_v1,
                actor_id: None,
            },
        )
        .await
        .expect("target v1 insert succeeds");

        run_gap_analysis(&pool, "inc-7", None)
            .await
            .expect("first gap analysis succeeds");

        // New target version drops the subcategory entirely (replaced by an
        // unrelated one — `insert_profile` rejects an empty entry list).
        let target_v2 = [NewProfileEntry {
            subcategory_id: "ID.AM-02",
            subcategory_label: "Outra subcategoria",
            implementation_status: ImplementationStatus::Implemented,
            justification: None,
            evidence_reference: None,
            confidence: None,
        }];
        insert_profile(
            &pool,
            NewProfile {
                incident_id: "inc-7",
                profile_type: ProfileType::Target,
                version_label: "v2",
                source: "target framework",
                tier: None,
                tier_justification: None,
                entries: &target_v2,
                actor_id: None,
            },
        )
        .await
        .expect("target v2 insert succeeds");

        let closed = run_gap_analysis(&pool, "inc-7", None)
            .await
            .expect("second gap analysis succeeds");
        // ID.AM-01 gets closed (no longer targeted); ID.AM-02 is a brand
        // new open gap (present in the target, absent from current) —
        // two ids, not one.
        assert_eq!(closed.len(), 2);

        let correlation_key = gap_correlation_key("inc-7", "ID.AM-01");
        let closed_record =
            analytical_results::current_for_correlation_key(&pool, &correlation_key)
                .await
                .expect("query current")
                .expect("closed record exists");
        let closed_payload: serde_json::Value =
            serde_json::from_str(&closed_record.payload).expect("valid json");
        assert_eq!(closed_payload["status"], "CLOSED");
        assert_eq!(closed_payload["closed_reason"], "no_longer_targeted");
    }
}
