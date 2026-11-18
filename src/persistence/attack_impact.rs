//! F1-04 — ATT&CK mapping, caminho crítico e matriz de impacto.
//!
//! Diferente de F1-03 (NIST), este ticket **não** introduz tabelas novas:
//! os três conceitos (ATT&CK Mapping, elo de caminho crítico, avaliação de
//! impacto) são julgamentos analíticos (achados de um analista sobre a
//! evidência), não fatos organizacionais declarados — cabem diretamente no
//! envelope de F1-01 (`AnalyticalResultType::AttackMapping` /
//! `CriticalPathLink` / `ImpactAssessment`, reservados desde a ADR-0008).
//! O encadeamento do caminho crítico ("eventos relacionados podem formar
//! uma sequência", RF-018) reusa `RelationType::SequenceNext`, também já
//! reservado em F1-01 justamente para este caso.
//!
//! ## Catálogo ATT&CK: mesma decisão de F1-03 (ADR-0011)
//!
//! As 14 táticas oficiais do MITRE ATT&CK (Enterprise) são uma estrutura
//! estável e pequena — hardcoded como enum, igual às 6 Functions do NIST
//! CSF em F1-03. O catálogo de **técnicas** (~600+, evolui a cada
//! atualização do MITRE) não é embutido — `technique_id` é validado só
//! quanto à forma (`T####` ou `T####.###`), pela mesma razão de risco de
//! transcrição documentada na ADR-0011: um catálogo de centenas de itens
//! transcrito de memória, usado num relatório de incidente real, é um risco
//! de erro silencioso que nenhum teste automatizado pegaria.
//!
//! ## "Incerteza, não fato" (requisito explícito do ticket F1-04)
//!
//! Um mapping ou elo de caminho crítico sem nenhuma referência de evidência
//! (`basis` vazio) não pode ser persistido com confiança HIGH/CONFIRMED —
//! [`require_evidence_for_high_confidence`] rejeita a inserção nesse caso,
//! forçando o analista a registrar LOW/MEDIUM quando a evidência é
//! insuficiente, em vez de deixar isso para uma camada de apresentação
//! decidir silenciosamente. O payload registra `evidence_status` (`GAP` ou
//! `DOCUMENTED`) explicitamente, para que a ausência de evidência apareça
//! como lacuna visível (RF-018: "ausência de evidência deve ser exibida
//! como lacuna, não preenchida por inferência silenciosa"), não como um
//! campo vazio que uma UI possa simplesmente omitir.

use super::validation_error;
use crate::persistence::analytical_results::{
    self, AnalyticalResultType, Confidence, NewAnalyticalResult, RelationType,
};
use sqlx::SqlitePool;

/// Impõe "relações não suportadas por evidência são marcadas como
/// incerteza, não como fato": sem uma `basis` não-vazia, a confiança não
/// pode ser HIGH nem CONFIRMED.
pub(crate) fn require_evidence_for_high_confidence(
    basis: &str,
    confidence: Confidence,
) -> Result<(), sqlx::Error> {
    if basis.trim().is_empty() && matches!(confidence, Confidence::High | Confidence::Confirmed) {
        return Err(validation_error(
            "confidence HIGH/CONFIRMED requires a non-empty evidence basis; use LOW/MEDIUM when evidence is insufficient",
        ));
    }
    Ok(())
}

fn evidence_status(basis: &str) -> &'static str {
    if basis.trim().is_empty() {
        "GAP"
    } else {
        "DOCUMENTED"
    }
}

// ---------------------------------------------------------------------
// ATT&CK
// ---------------------------------------------------------------------

/// As 14 táticas oficiais do MITRE ATT&CK (Enterprise) — estrutura estável;
/// ver docs do módulo sobre por que o catálogo de técnicas não é embutido.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttackTactic {
    Reconnaissance,
    ResourceDevelopment,
    InitialAccess,
    Execution,
    Persistence,
    PrivilegeEscalation,
    DefenseEvasion,
    CredentialAccess,
    Discovery,
    LateralMovement,
    Collection,
    CommandAndControl,
    Exfiltration,
    Impact,
}

impl AttackTactic {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Reconnaissance => "RECONNAISSANCE",
            Self::ResourceDevelopment => "RESOURCE_DEVELOPMENT",
            Self::InitialAccess => "INITIAL_ACCESS",
            Self::Execution => "EXECUTION",
            Self::Persistence => "PERSISTENCE",
            Self::PrivilegeEscalation => "PRIVILEGE_ESCALATION",
            Self::DefenseEvasion => "DEFENSE_EVASION",
            Self::CredentialAccess => "CREDENTIAL_ACCESS",
            Self::Discovery => "DISCOVERY",
            Self::LateralMovement => "LATERAL_MOVEMENT",
            Self::Collection => "COLLECTION",
            Self::CommandAndControl => "COMMAND_AND_CONTROL",
            Self::Exfiltration => "EXFILTRATION",
            Self::Impact => "IMPACT",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "RECONNAISSANCE" => Some(Self::Reconnaissance),
            "RESOURCE_DEVELOPMENT" => Some(Self::ResourceDevelopment),
            "INITIAL_ACCESS" => Some(Self::InitialAccess),
            "EXECUTION" => Some(Self::Execution),
            "PERSISTENCE" => Some(Self::Persistence),
            "PRIVILEGE_ESCALATION" => Some(Self::PrivilegeEscalation),
            "DEFENSE_EVASION" => Some(Self::DefenseEvasion),
            "CREDENTIAL_ACCESS" => Some(Self::CredentialAccess),
            "DISCOVERY" => Some(Self::Discovery),
            "LATERAL_MOVEMENT" => Some(Self::LateralMovement),
            "COLLECTION" => Some(Self::Collection),
            "COMMAND_AND_CONTROL" => Some(Self::CommandAndControl),
            "EXFILTRATION" => Some(Self::Exfiltration),
            "IMPACT" => Some(Self::Impact),
            _ => None,
        }
    }
}

/// Valida apenas a forma estrutural de um technique id do MITRE ATT&CK:
/// `T####` (técnica) ou `T####.###` (sub-técnica) — ex.: `T1078`,
/// `T1078.001`. Não valida contra o catálogo oficial (ver docs do módulo).
pub fn is_valid_technique_id(value: &str) -> bool {
    let (base, sub) = match value.split_once('.') {
        Some((base, sub)) => (base, Some(sub)),
        None => (value, None),
    };
    let base_ok =
        base.len() == 5 && base.starts_with('T') && base[1..].chars().all(|c| c.is_ascii_digit());
    let sub_ok = sub.is_none_or(|sub| sub.len() == 3 && sub.chars().all(|c| c.is_ascii_digit()));
    base_ok && sub_ok
}

pub struct NewAttackMapping<'a> {
    pub incident_id: &'a str,
    pub evidence_id: Option<&'a str>,
    pub tactic: AttackTactic,
    pub technique_id: &'a str,
    pub sub_technique_id: Option<&'a str>,
    /// Referência/raciocínio de evidência que sustenta o mapping. Vazio é
    /// permitido apenas com confidence LOW/MEDIUM (ver
    /// [`require_evidence_for_high_confidence`]).
    pub basis: &'a str,
    pub confidence: Confidence,
    pub analyst_note: Option<&'a str>,
    pub actor_id: Option<&'a str>,
}

/// Insere (ou substitui, via `supersedes` automático) um mapping ATT&CK.
/// "Um mapping só existe quando existir evidência suficiente" (seção 14) é
/// imposto como: sem `basis`, a confidence não pode ser HIGH/CONFIRMED.
pub async fn insert_attack_mapping(
    pool: &SqlitePool,
    input: NewAttackMapping<'_>,
) -> Result<String, sqlx::Error> {
    if !is_valid_technique_id(input.technique_id) {
        return Err(validation_error(format!(
            "invalid ATT&CK technique id shape: {}",
            input.technique_id
        )));
    }
    if let Some(sub) = input.sub_technique_id
        && !is_valid_technique_id(sub)
    {
        return Err(validation_error(format!(
            "invalid ATT&CK sub-technique id shape: {sub}"
        )));
    }
    require_evidence_for_high_confidence(input.basis, input.confidence)?;

    let correlation_key = format!(
        "attack-mapping:{}:{}:{}",
        input.incident_id,
        input.evidence_id.unwrap_or("none"),
        input.technique_id
    );
    let previous = analytical_results::current_for_correlation_key(pool, &correlation_key).await?;

    let payload_json = serde_json::json!({
        "kind": "ATTACK_MAPPING",
        "tactic": input.tactic.as_str(),
        "technique_id": input.technique_id,
        "sub_technique_id": input.sub_technique_id,
        "basis": input.basis,
        "analyst_note": input.analyst_note,
        "evidence_status": evidence_status(input.basis),
    })
    .to_string();
    let provenance_json = serde_json::json!({
        "producer": "dfir-app",
        "method": "analyst-asserted ATT&CK mapping",
    })
    .to_string();

    analytical_results::insert(
        pool,
        NewAnalyticalResult {
            result_type: AnalyticalResultType::AttackMapping,
            correlation_key: &correlation_key,
            evidence_id: input.evidence_id,
            incident_id: Some(input.incident_id),
            analysis_job_id: None,
            rule_or_model_id: Some(input.technique_id),
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
// Caminho crítico
// ---------------------------------------------------------------------

pub struct NewCriticalPathStage<'a> {
    pub incident_id: &'a str,
    pub evidence_id: Option<&'a str>,
    /// Posição do estágio no incidente (0-based), usada para identidade de
    /// versionamento — descrever de novo o estágio N substitui a versão
    /// anterior desse mesmo N via `supersedes`. As relações
    /// `SEQUENCE_NEXT` entre estágios são feitas à parte, por
    /// [`link_critical_path_stages`], e não são reatribuídas
    /// automaticamente quando um estágio é substituído (ver docs do
    /// módulo em `docs/adrs/ADR-0012-attack-caminho-critico-impacto.md`).
    pub sequence_index: i64,
    pub stage_label: &'a str,
    pub basis: &'a str,
    pub confidence: Confidence,
    pub actor_id: Option<&'a str>,
}

/// Insere um estágio do caminho crítico. Vincule estágios em sequência com
/// [`link_critical_path_stages`] depois de inserir cada um.
pub async fn insert_critical_path_stage(
    pool: &SqlitePool,
    input: NewCriticalPathStage<'_>,
) -> Result<String, sqlx::Error> {
    require_evidence_for_high_confidence(input.basis, input.confidence)?;

    let correlation_key = format!(
        "critical-path-stage:{}:{}",
        input.incident_id, input.sequence_index
    );
    let previous = analytical_results::current_for_correlation_key(pool, &correlation_key).await?;

    let payload_json = serde_json::json!({
        "kind": "CRITICAL_PATH_STAGE",
        "sequence_index": input.sequence_index,
        "stage_label": input.stage_label,
        "basis": input.basis,
        "evidence_status": evidence_status(input.basis),
    })
    .to_string();
    let provenance_json = serde_json::json!({
        "producer": "dfir-app",
        "method": "analyst-asserted critical path stage",
    })
    .to_string();

    analytical_results::insert(
        pool,
        NewAnalyticalResult {
            result_type: AnalyticalResultType::CriticalPathLink,
            correlation_key: &correlation_key,
            evidence_id: input.evidence_id,
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

/// Liga dois estágios já existentes em sequência (`from` → `to`). Ambos
/// precisam ser resultados `CRITICAL_PATH_LINK` — permite grafos com
/// ramificação (mais de um `SEQUENCE_NEXT` saindo do mesmo estágio), não
/// só uma cadeia linear, conforme "grafo temporal/causal" da seção 15.
pub async fn link_critical_path_stages(
    pool: &SqlitePool,
    from_stage_id: &str,
    to_stage_id: &str,
) -> Result<(), sqlx::Error> {
    for stage_id in [from_stage_id, to_stage_id] {
        let record = analytical_results::get(pool, stage_id)
            .await?
            .ok_or_else(|| {
                validation_error(format!("critical path stage not found: {stage_id}"))
            })?;
        if record.result_type != AnalyticalResultType::CriticalPathLink.as_str() {
            return Err(validation_error(format!(
                "{stage_id} is not a CRITICAL_PATH_LINK result"
            )));
        }
    }
    analytical_results::insert_relation(
        pool,
        from_stage_id,
        to_stage_id,
        RelationType::SequenceNext,
    )
    .await
}

// ---------------------------------------------------------------------
// Matriz de impacto
// ---------------------------------------------------------------------

/// Escala qualitativa de impacto (convenção operacional do produto, não um
/// padrão oficial do NIST/MITRE) — usada para disponibilidade,
/// confidencialidade e integridade (seção 16).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImpactLevel {
    None,
    Low,
    Medium,
    High,
    Critical,
}

impl ImpactLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "NONE",
            Self::Low => "LOW",
            Self::Medium => "MEDIUM",
            Self::High => "HIGH",
            Self::Critical => "CRITICAL",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "NONE" => Some(Self::None),
            "LOW" => Some(Self::Low),
            "MEDIUM" => Some(Self::Medium),
            "HIGH" => Some(Self::High),
            "CRITICAL" => Some(Self::Critical),
            _ => None,
        }
    }
}

pub struct NewImpactAssessment<'a> {
    pub incident_id: &'a str,
    pub availability_impact: ImpactLevel,
    pub confidentiality_impact: ImpactLevel,
    pub integrity_impact: ImpactLevel,
    pub affected_assets: &'a str,
    pub affected_data: &'a str,
    pub operational_disruption: &'a str,
    /// Texto livre (ex.: "desconhecido", "estimativa de US$X-Y"). `None`
    /// quando não há dados — nunca fabricado (seção 16: "Não fabricar
    /// valores financeiros quando não houver dados").
    pub financial_impact: Option<&'a str>,
    pub confidence: Confidence,
    pub actor_id: Option<&'a str>,
}

/// Insere (ou substitui, via `supersedes` automático) a avaliação de
/// impacto do incidente. Uma `correlation_key` por incidente — a avaliação
/// evolui como uma única linha versionada ao longo do tempo, não várias em
/// paralelo.
pub async fn insert_impact_assessment(
    pool: &SqlitePool,
    input: NewImpactAssessment<'_>,
) -> Result<String, sqlx::Error> {
    let correlation_key = format!("impact-assessment:{}", input.incident_id);
    let previous = analytical_results::current_for_correlation_key(pool, &correlation_key).await?;

    let payload_json = serde_json::json!({
        "kind": "IMPACT_ASSESSMENT",
        "availability_impact": input.availability_impact.as_str(),
        "confidentiality_impact": input.confidentiality_impact.as_str(),
        "integrity_impact": input.integrity_impact.as_str(),
        "affected_assets": input.affected_assets,
        "affected_data": input.affected_data,
        "operational_disruption": input.operational_disruption,
        "financial_impact": input.financial_impact,
        "financial_impact_known": input.financial_impact.is_some(),
    })
    .to_string();
    let provenance_json = serde_json::json!({
        "producer": "dfir-app",
        "method": "analyst-asserted impact assessment",
    })
    .to_string();

    analytical_results::insert(
        pool,
        NewAnalyticalResult {
            result_type: AnalyticalResultType::ImpactAssessment,
            correlation_key: &correlation_key,
            evidence_id: None,
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

#[cfg(test)]
mod tests {
    use super::*;

    async fn test_pool() -> SqlitePool {
        let temp = tempfile::tempdir().expect("temporary directory");
        let db_path = temp.path().join("attack-impact.sqlite");
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

    #[test]
    fn technique_id_validation_accepts_known_shape_and_rejects_others() {
        assert!(is_valid_technique_id("T1078"));
        assert!(is_valid_technique_id("T1078.001"));
        assert!(!is_valid_technique_id("1078"), "must start with T");
        assert!(!is_valid_technique_id("T107"), "must be 4 digits");
        assert!(
            !is_valid_technique_id("T1078.01"),
            "sub-technique must be 3 digits"
        );
        assert!(!is_valid_technique_id("t1078"), "must be uppercase T");
    }

    #[test]
    fn attack_tactic_round_trips_as_str() {
        for tactic in [
            AttackTactic::Reconnaissance,
            AttackTactic::ResourceDevelopment,
            AttackTactic::InitialAccess,
            AttackTactic::Execution,
            AttackTactic::Persistence,
            AttackTactic::PrivilegeEscalation,
            AttackTactic::DefenseEvasion,
            AttackTactic::CredentialAccess,
            AttackTactic::Discovery,
            AttackTactic::LateralMovement,
            AttackTactic::Collection,
            AttackTactic::CommandAndControl,
            AttackTactic::Exfiltration,
            AttackTactic::Impact,
        ] {
            assert_eq!(AttackTactic::from_str(tactic.as_str()), Some(tactic));
        }
    }

    #[test]
    fn impact_level_round_trips_as_str() {
        for level in [
            ImpactLevel::None,
            ImpactLevel::Low,
            ImpactLevel::Medium,
            ImpactLevel::High,
            ImpactLevel::Critical,
        ] {
            assert_eq!(ImpactLevel::from_str(level.as_str()), Some(level));
        }
    }

    #[tokio::test]
    async fn insert_attack_mapping_rejects_invalid_technique_id() {
        let pool = test_pool().await;
        seed_incident(&pool, "inc-1").await;
        let result = insert_attack_mapping(
            &pool,
            NewAttackMapping {
                incident_id: "inc-1",
                evidence_id: None,
                tactic: AttackTactic::InitialAccess,
                technique_id: "not-a-technique",
                sub_technique_id: None,
                basis: "phishing email observed in evidence X",
                confidence: Confidence::Medium,
                analyst_note: None,
                actor_id: None,
            },
        )
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn insert_attack_mapping_rejects_high_confidence_without_evidence_basis() {
        let pool = test_pool().await;
        seed_incident(&pool, "inc-2").await;
        let result = insert_attack_mapping(
            &pool,
            NewAttackMapping {
                incident_id: "inc-2",
                evidence_id: None,
                tactic: AttackTactic::InitialAccess,
                technique_id: "T1566",
                sub_technique_id: None,
                basis: "   ",
                confidence: Confidence::Confirmed,
                analyst_note: None,
                actor_id: None,
            },
        )
        .await;
        assert!(
            result.is_err(),
            "CONFIRMED without a basis must be rejected"
        );
    }

    #[tokio::test]
    async fn insert_attack_mapping_allows_low_confidence_without_evidence_basis() {
        let pool = test_pool().await;
        seed_incident(&pool, "inc-3").await;
        let id = insert_attack_mapping(
            &pool,
            NewAttackMapping {
                incident_id: "inc-3",
                evidence_id: None,
                tactic: AttackTactic::Discovery,
                technique_id: "T1082",
                sub_technique_id: None,
                basis: "",
                confidence: Confidence::Low,
                analyst_note: Some("weak circumstantial indicator only"),
                actor_id: None,
            },
        )
        .await
        .expect("LOW confidence without basis is allowed");

        let record = analytical_results::get(&pool, &id)
            .await
            .expect("query")
            .expect("record exists");
        assert!(record.payload.contains("\"evidence_status\":\"GAP\""));
    }

    #[tokio::test]
    async fn resubmitting_an_attack_mapping_supersedes_the_previous_one() {
        let pool = test_pool().await;
        seed_incident(&pool, "inc-4").await;
        let mapping = |confidence| NewAttackMapping {
            incident_id: "inc-4",
            evidence_id: None,
            tactic: AttackTactic::Execution,
            technique_id: "T1059",
            sub_technique_id: Some("T1059.001"),
            basis: "PowerShell execution observed",
            confidence,
            analyst_note: None,
            actor_id: None,
        };
        let first = insert_attack_mapping(&pool, mapping(Confidence::Medium))
            .await
            .expect("first insert succeeds");
        let second = insert_attack_mapping(&pool, mapping(Confidence::High))
            .await
            .expect("second insert succeeds");

        let second_record = analytical_results::get(&pool, &second)
            .await
            .expect("query")
            .expect("record exists");
        assert_eq!(second_record.supersedes.as_deref(), Some(first.as_str()));
    }

    #[tokio::test]
    async fn critical_path_stages_can_be_linked_in_sequence() {
        let pool = test_pool().await;
        seed_incident(&pool, "inc-5").await;
        let stage_1 = insert_critical_path_stage(
            &pool,
            NewCriticalPathStage {
                incident_id: "inc-5",
                evidence_id: None,
                sequence_index: 0,
                stage_label: "Initial Access",
                basis: "phishing email, evidence ev-1",
                confidence: Confidence::Medium,
                actor_id: None,
            },
        )
        .await
        .expect("stage 1 insert succeeds");
        let stage_2 = insert_critical_path_stage(
            &pool,
            NewCriticalPathStage {
                incident_id: "inc-5",
                evidence_id: None,
                sequence_index: 1,
                stage_label: "Credential Compromise",
                basis: "",
                confidence: Confidence::Low,
                actor_id: None,
            },
        )
        .await
        .expect("stage 2 insert succeeds");

        link_critical_path_stages(&pool, &stage_1, &stage_2)
            .await
            .expect("link succeeds");

        let related = analytical_results::related_to(&pool, &stage_1)
            .await
            .expect("query relations");
        assert_eq!(related.len(), 1);
        assert_eq!(related[0].0, RelationType::SequenceNext);
        assert_eq!(related[0].1.id, stage_2);
    }

    #[tokio::test]
    async fn linking_a_non_critical_path_result_is_rejected() {
        let pool = test_pool().await;
        seed_incident(&pool, "inc-6").await;
        let stage = insert_critical_path_stage(
            &pool,
            NewCriticalPathStage {
                incident_id: "inc-6",
                evidence_id: None,
                sequence_index: 0,
                stage_label: "Initial Access",
                basis: "evidence ev-1",
                confidence: Confidence::Medium,
                actor_id: None,
            },
        )
        .await
        .expect("stage insert succeeds");
        let mapping = insert_attack_mapping(
            &pool,
            NewAttackMapping {
                incident_id: "inc-6",
                evidence_id: None,
                tactic: AttackTactic::InitialAccess,
                technique_id: "T1566",
                sub_technique_id: None,
                basis: "evidence ev-1",
                confidence: Confidence::Medium,
                analyst_note: None,
                actor_id: None,
            },
        )
        .await
        .expect("mapping insert succeeds");

        let result = link_critical_path_stages(&pool, &stage, &mapping).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn insert_impact_assessment_never_fabricates_financial_impact() {
        let pool = test_pool().await;
        seed_incident(&pool, "inc-7").await;
        let id = insert_impact_assessment(
            &pool,
            NewImpactAssessment {
                incident_id: "inc-7",
                availability_impact: ImpactLevel::High,
                confidentiality_impact: ImpactLevel::Medium,
                integrity_impact: ImpactLevel::Low,
                affected_assets: "3 domain controllers",
                affected_data: "employee credential hashes",
                operational_disruption: "authentication services degraded for 4 hours",
                financial_impact: None,
                confidence: Confidence::Medium,
                actor_id: None,
            },
        )
        .await
        .expect("insert succeeds");

        let record = analytical_results::get(&pool, &id)
            .await
            .expect("query")
            .expect("record exists");
        assert!(record.payload.contains("\"financial_impact\":null"));
        assert!(record.payload.contains("\"financial_impact_known\":false"));
    }

    #[tokio::test]
    async fn resubmitting_impact_assessment_supersedes_the_previous_one() {
        let pool = test_pool().await;
        seed_incident(&pool, "inc-8").await;
        let assessment = |disruption: &'static str| NewImpactAssessment {
            incident_id: "inc-8",
            availability_impact: ImpactLevel::Medium,
            confidentiality_impact: ImpactLevel::Low,
            integrity_impact: ImpactLevel::None,
            affected_assets: "1 file server",
            affected_data: "none confirmed",
            operational_disruption: disruption,
            financial_impact: None,
            confidence: Confidence::Low,
            actor_id: None,
        };
        let first = insert_impact_assessment(&pool, assessment("initial estimate"))
            .await
            .expect("first insert succeeds");
        let second = insert_impact_assessment(&pool, assessment("revised after investigation"))
            .await
            .expect("second insert succeeds");

        let second_record = analytical_results::get(&pool, &second)
            .await
            .expect("query")
            .expect("record exists");
        assert_eq!(second_record.supersedes.as_deref(), Some(first.as_str()));

        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM analytical_results WHERE result_type='IMPACT_ASSESSMENT' AND incident_id='inc-8'",
        )
        .fetch_one(&pool)
        .await
        .expect("count query");
        assert_eq!(total, 2, "history preserved, not overwritten");
    }
}
