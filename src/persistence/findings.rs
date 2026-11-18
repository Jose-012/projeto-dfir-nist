//! F1-25 (RF-051) — Finding (vulnerabilidade ou condição explorada) e
//! conclusão de causa (causa-raiz, causa provável, fator contribuinte).
//!
//! Como ATT&CK e impacto (F1-04), são julgamentos analíticos do operador
//! sobre a evidência e cabem no envelope de F1-01
//! (`AnalyticalResultType::Finding` / `RootCause`, reservados pela migration
//! 0010 — ADR-0023). Não há tabela nova.
//!
//! ## Regras que o código impõe (não só a interface)
//!
//! - **Finding exige evidência.** É preciso ao menos uma destas: uma
//!   evidência (`evidence_id`), uma base textual (`basis`) ou resultados de
//!   suporte (mapeamentos ATT&CK, IOCs, matches YARA...). Sem nada, a
//!   inserção é recusada — um achado sem lastro não vira entregável.
//! - **Severidade `NONE` não é achado.** Uma condição sem severidade não
//!   entra na matriz de risco.
//! - **Causa-raiz é definitiva; causa provável não é.** `ROOT_CAUSE` só é
//!   aceita com confiança HIGH ou CONFIRMED; com menos evidência o analista
//!   registra `PROBABLE_CAUSE` (ou `CONTRIBUTING_FACTOR`). A demanda separa
//!   os três níveis justamente para não apresentar hipótese como fato.
//! - **Toda conclusão de causa cita ao menos um Finding** do mesmo
//!   incidente, ligado por `DERIVED_FROM`.
//! - **Lacunas ficam explícitas.** O payload de uma causa traz `gaps` e
//!   `gap_status` (`DECLARED` ou `NONE_DECLARED`): a ausência de lacunas é
//!   uma declaração, não um campo vazio que a interface possa omitir.
//! - **Matriz de risco sem valores inventados.** [`risk_matrix`] só conta o
//!   que existe (severidade × confiança); não calcula pontuação.
//!
//! O catálogo NIST não é embutido (ADR-0011): `nist_subcategory_ids` são
//! validados só quanto à forma, como em F1-03.

use super::validation_error;
use crate::persistence::analytical_results::{
    self, AnalyticalResultRecord, AnalyticalResultType, Confidence, NewAnalyticalResult,
    RelationType,
};
use crate::persistence::attack_impact::{ImpactLevel, require_evidence_for_high_confidence};
use crate::persistence::nist_profiles::is_valid_subcategory_id;
use serde::Serialize;
use sqlx::SqlitePool;
use std::collections::BTreeSet;
use uuid::Uuid;

// ---------------------------------------------------------------------
// Vocabulário
// ---------------------------------------------------------------------

/// Natureza do achado. Lista curta de propósito: serve para agrupar a tabela
/// do Entregável 2, não para classificar CVEs (isso fica em `reference`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindingKind {
    /// Falha de software/firmware (CVE, versão sem correção).
    Vulnerability,
    /// Configuração insegura (serviço exposto, padrão inseguro).
    Misconfiguration,
    /// Controle ausente (MFA, segmentação, log, backup).
    MissingControl,
    /// Credencial fraca, reutilizada ou exposta.
    WeakCredential,
    Other,
}

impl FindingKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Vulnerability => "VULNERABILITY",
            Self::Misconfiguration => "MISCONFIGURATION",
            Self::MissingControl => "MISSING_CONTROL",
            Self::WeakCredential => "WEAK_CREDENTIAL",
            Self::Other => "OTHER",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "VULNERABILITY" => Some(Self::Vulnerability),
            "MISCONFIGURATION" => Some(Self::Misconfiguration),
            "MISSING_CONTROL" => Some(Self::MissingControl),
            "WEAK_CREDENTIAL" => Some(Self::WeakCredential),
            "OTHER" => Some(Self::Other),
            _ => None,
        }
    }
}

/// Nível de uma conclusão de causa (demanda: causa-raiz, causa provável,
/// fatores contribuintes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CauseLevel {
    RootCause,
    ProbableCause,
    ContributingFactor,
}

impl CauseLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::RootCause => "ROOT_CAUSE",
            Self::ProbableCause => "PROBABLE_CAUSE",
            Self::ContributingFactor => "CONTRIBUTING_FACTOR",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "ROOT_CAUSE" => Some(Self::RootCause),
            "PROBABLE_CAUSE" => Some(Self::ProbableCause),
            "CONTRIBUTING_FACTOR" => Some(Self::ContributingFactor),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------
// Finding
// ---------------------------------------------------------------------

pub struct NewFinding<'a> {
    pub incident_id: &'a str,
    /// Evidência principal (opcional; um achado pode se apoiar em várias).
    pub evidence_id: Option<&'a str>,
    pub kind: FindingKind,
    pub title: &'a str,
    pub description: &'a str,
    pub severity: ImpactLevel,
    /// Referência à evidência em texto (ex.: "EVTX Security, EventID 4625,
    /// 312 tentativas entre 02:10 e 02:41").
    pub basis: &'a str,
    /// Identificador externo opcional (CVE, CWE, boletim do fabricante).
    pub reference: Option<&'a str>,
    pub nist_subcategory_ids: &'a [&'a str],
    /// Mapeamentos ATT&CK (resultados `ATTACK_MAPPING` do mesmo incidente).
    pub attack_mapping_ids: &'a [&'a str],
    /// Outros resultados que sustentam o achado (IOC, YARA, caminho crítico).
    pub supporting_result_ids: &'a [&'a str],
    pub confidence: Confidence,
    pub analyst_note: Option<&'a str>,
    /// Achado anterior que este revisa (mesma `correlation_key`).
    pub supersedes: Option<&'a str>,
    pub actor_id: Option<&'a str>,
}

pub(crate) async fn require_result(
    pool: &SqlitePool,
    id: &str,
    expected: Option<AnalyticalResultType>,
    incident_id: Option<&str>,
    what: &str,
) -> Result<AnalyticalResultRecord, sqlx::Error> {
    let record = analytical_results::get(pool, id)
        .await?
        .ok_or_else(|| validation_error(format!("{what} not found: {id}")))?;
    if let Some(expected) = expected
        && record.result_type != expected.as_str()
    {
        return Err(validation_error(format!(
            "{id} is not a {} result",
            expected.as_str()
        )));
    }
    if let Some(incident_id) = incident_id
        && record.incident_id.as_deref() != Some(incident_id)
    {
        return Err(validation_error(format!(
            "{what} {id} belongs to a different incident"
        )));
    }
    Ok(record)
}

/// Insere um achado (ou uma revisão, quando `supersedes` é informado).
pub async fn insert_finding(
    pool: &SqlitePool,
    input: NewFinding<'_>,
) -> Result<String, sqlx::Error> {
    if input.title.trim().is_empty() {
        return Err(validation_error("a finding requires a title"));
    }
    if input.severity == ImpactLevel::None {
        return Err(validation_error(
            "a finding requires a severity above NONE; a condition with no severity is not a finding",
        ));
    }
    if input.evidence_id.is_none()
        && input.basis.trim().is_empty()
        && input.supporting_result_ids.is_empty()
        && input.attack_mapping_ids.is_empty()
    {
        return Err(validation_error(
            "a finding requires evidence: an evidence_id, a basis or supporting results",
        ));
    }
    // Regra de F1-04: HIGH/CONFIRMED exige uma base textual ou uma evidência
    // primária; resultados de suporte sozinhos sustentam no máximo MEDIUM.
    if input.evidence_id.is_none() {
        require_evidence_for_high_confidence(input.basis, input.confidence)?;
    }
    for subcategory in input.nist_subcategory_ids {
        if !is_valid_subcategory_id(subcategory) {
            return Err(validation_error(format!(
                "invalid NIST subcategory id shape: {subcategory}"
            )));
        }
    }
    for id in input.attack_mapping_ids {
        require_result(
            pool,
            id,
            Some(AnalyticalResultType::AttackMapping),
            Some(input.incident_id),
            "ATT&CK mapping",
        )
        .await?;
    }
    for id in input.supporting_result_ids {
        require_result(pool, id, None, None, "supporting result").await?;
    }

    let correlation_key = match input.supersedes {
        Some(previous_id) => {
            require_result(
                pool,
                previous_id,
                Some(AnalyticalResultType::Finding),
                Some(input.incident_id),
                "finding to supersede",
            )
            .await?
            .correlation_key
        }
        None => format!("finding:{}:{}", input.incident_id, Uuid::new_v4()),
    };

    let nist: Vec<&str> = input.nist_subcategory_ids.to_vec();
    let payload_json = serde_json::json!({
        "kind": "FINDING",
        "finding_kind": input.kind.as_str(),
        "title": input.title.trim(),
        "description": input.description,
        "severity": input.severity.as_str(),
        "basis": input.basis,
        "reference": input.reference,
        "nist_subcategories": nist,
        "attack_mapping_ids": input.attack_mapping_ids,
        "supporting_result_ids": input.supporting_result_ids,
        "analyst_note": input.analyst_note,
    })
    .to_string();
    let provenance_json = serde_json::json!({
        "producer": "dfir-app",
        "method": "analyst-asserted finding",
    })
    .to_string();

    let linked: BTreeSet<&str> = input
        .attack_mapping_ids
        .iter()
        .chain(input.supporting_result_ids.iter())
        .copied()
        .collect();
    let related: Vec<(&str, RelationType)> = linked
        .into_iter()
        .map(|id| (id, RelationType::DerivedFrom))
        .collect();

    analytical_results::insert(
        pool,
        NewAnalyticalResult {
            result_type: AnalyticalResultType::Finding,
            correlation_key: &correlation_key,
            evidence_id: input.evidence_id,
            incident_id: Some(input.incident_id),
            analysis_job_id: None,
            rule_or_model_id: None,
            rule_or_model_version: None,
            confidence: Some(input.confidence),
            payload_json: &payload_json,
            provenance_json: &provenance_json,
            supersedes: input.supersedes,
            related: related.as_slice(),
            actor_id: input.actor_id,
        },
    )
    .await
}

// ---------------------------------------------------------------------
// Conclusão de causa
// ---------------------------------------------------------------------

pub struct NewRootCause<'a> {
    pub incident_id: &'a str,
    pub level: CauseLevel,
    /// O que se conclui, em uma ou duas frases.
    pub statement: &'a str,
    pub basis: &'a str,
    /// O que não se sabe ou não pôde ser verificado (logs ausentes, retenção
    /// curta, host não coletado...).
    pub gaps: &'a [&'a str],
    /// Achados do mesmo incidente que sustentam a conclusão (≥ 1).
    pub finding_ids: &'a [&'a str],
    pub confidence: Confidence,
    /// Conclusão anterior que esta revisa (mesma `correlation_key`).
    pub supersedes: Option<&'a str>,
    pub actor_id: Option<&'a str>,
}

/// Insere uma conclusão de causa (ou uma revisão).
pub async fn insert_root_cause(
    pool: &SqlitePool,
    input: NewRootCause<'_>,
) -> Result<String, sqlx::Error> {
    if input.statement.trim().is_empty() {
        return Err(validation_error("a cause conclusion requires a statement"));
    }
    if input.finding_ids.is_empty() {
        return Err(validation_error(
            "a cause conclusion must reference at least one supporting finding",
        ));
    }
    if input.level == CauseLevel::RootCause
        && !matches!(input.confidence, Confidence::High | Confidence::Confirmed)
    {
        return Err(validation_error(
            "ROOT_CAUSE requires confidence HIGH or CONFIRMED; record it as PROBABLE_CAUSE when evidence is insufficient",
        ));
    }
    require_evidence_for_high_confidence(input.basis, input.confidence)?;
    for id in input.finding_ids {
        require_result(
            pool,
            id,
            Some(AnalyticalResultType::Finding),
            Some(input.incident_id),
            "supporting finding",
        )
        .await?;
    }

    let correlation_key = match input.supersedes {
        Some(previous_id) => {
            require_result(
                pool,
                previous_id,
                Some(AnalyticalResultType::RootCause),
                Some(input.incident_id),
                "cause conclusion to supersede",
            )
            .await?
            .correlation_key
        }
        None => format!("root-cause:{}:{}", input.incident_id, Uuid::new_v4()),
    };

    let gaps: Vec<&str> = input
        .gaps
        .iter()
        .map(|gap| gap.trim())
        .filter(|gap| !gap.is_empty())
        .collect();
    let gap_status = if gaps.is_empty() {
        "NONE_DECLARED"
    } else {
        "DECLARED"
    };
    let payload_json = serde_json::json!({
        "kind": "ROOT_CAUSE",
        "level": input.level.as_str(),
        "statement": input.statement.trim(),
        "basis": input.basis,
        "gaps": gaps,
        "gap_status": gap_status,
        "finding_ids": input.finding_ids,
    })
    .to_string();
    let provenance_json = serde_json::json!({
        "producer": "dfir-app",
        "method": "analyst-asserted cause conclusion",
    })
    .to_string();

    let linked: BTreeSet<&str> = input.finding_ids.iter().copied().collect();
    let related: Vec<(&str, RelationType)> = linked
        .into_iter()
        .map(|id| (id, RelationType::DerivedFrom))
        .collect();

    analytical_results::insert(
        pool,
        NewAnalyticalResult {
            result_type: AnalyticalResultType::RootCause,
            correlation_key: &correlation_key,
            evidence_id: None,
            incident_id: Some(input.incident_id),
            analysis_job_id: None,
            rule_or_model_id: None,
            rule_or_model_version: None,
            confidence: Some(input.confidence),
            payload_json: &payload_json,
            provenance_json: &provenance_json,
            supersedes: input.supersedes,
            related: related.as_slice(),
            actor_id: input.actor_id,
        },
    )
    .await
}

// ---------------------------------------------------------------------
// Matriz de risco (severidade × confiança)
// ---------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RiskCell {
    pub severity: String,
    pub confidence: String,
    pub count: usize,
}

fn severity_rank(level: ImpactLevel) -> u8 {
    match level {
        ImpactLevel::None => 0,
        ImpactLevel::Low => 1,
        ImpactLevel::Medium => 2,
        ImpactLevel::High => 3,
        ImpactLevel::Critical => 4,
    }
}

fn confidence_rank(level: Confidence) -> u8 {
    match level {
        Confidence::Low => 0,
        Confidence::Medium => 1,
        Confidence::High => 2,
        Confidence::Confirmed => 3,
    }
}

/// Conta achados por (severidade, confiança), da severidade e confiança mais
/// altas para as mais baixas. Só devolve células com pelo menos um achado:
/// não há pontuação nem célula preenchida por inferência.
pub fn risk_matrix(findings: &[(ImpactLevel, Confidence)]) -> Vec<RiskCell> {
    let mut cells: Vec<(ImpactLevel, Confidence, usize)> = Vec::new();
    for (severity, confidence) in findings {
        match cells
            .iter_mut()
            .find(|(s, c, _)| s == severity && c == confidence)
        {
            Some(cell) => cell.2 += 1,
            None => cells.push((*severity, *confidence, 1)),
        }
    }
    cells.sort_by(|a, b| {
        severity_rank(b.0)
            .cmp(&severity_rank(a.0))
            .then(confidence_rank(b.1).cmp(&confidence_rank(a.1)))
    });
    cells
        .into_iter()
        .map(|(severity, confidence, count)| RiskCell {
            severity: severity.as_str().to_string(),
            confidence: confidence.as_str().to_string(),
            count,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::attack_impact::{
        AttackTactic, NewAttackMapping, insert_attack_mapping,
    };

    async fn test_pool() -> SqlitePool {
        let temp = tempfile::tempdir().expect("temporary directory");
        let db_path = temp.path().join("findings.sqlite");
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

    fn finding<'a>(incident_id: &'a str, title: &'a str) -> NewFinding<'a> {
        NewFinding {
            incident_id,
            evidence_id: None,
            kind: FindingKind::MissingControl,
            title,
            description: "RDP exposto sem MFA",
            severity: ImpactLevel::High,
            basis: "EVTX Security 4625 em massa seguido de 4624 tipo 10",
            reference: None,
            nist_subcategory_ids: &[],
            attack_mapping_ids: &[],
            supporting_result_ids: &[],
            confidence: Confidence::Medium,
            analyst_note: None,
            supersedes: None,
            actor_id: None,
        }
    }

    fn cause<'a>(incident_id: &'a str, finding_ids: &'a [&'a str]) -> NewRootCause<'a> {
        NewRootCause {
            incident_id,
            level: CauseLevel::ProbableCause,
            statement: "Acesso inicial por RDP exposto sem MFA",
            basis: "Finding f-1 e a sequência de logons",
            gaps: &[],
            finding_ids,
            confidence: Confidence::Medium,
            supersedes: None,
            actor_id: None,
        }
    }

    #[test]
    fn kind_and_level_round_trip_as_str() {
        for kind in [
            FindingKind::Vulnerability,
            FindingKind::Misconfiguration,
            FindingKind::MissingControl,
            FindingKind::WeakCredential,
            FindingKind::Other,
        ] {
            assert_eq!(FindingKind::from_str(kind.as_str()), Some(kind));
        }
        for level in [
            CauseLevel::RootCause,
            CauseLevel::ProbableCause,
            CauseLevel::ContributingFactor,
        ] {
            assert_eq!(CauseLevel::from_str(level.as_str()), Some(level));
        }
        assert_eq!(FindingKind::from_str("BOGUS"), None);
        assert_eq!(CauseLevel::from_str("BOGUS"), None);
    }

    #[tokio::test]
    async fn finding_is_persisted_with_links_and_payload() {
        let pool = test_pool().await;
        seed_incident(&pool, "inc-1").await;
        let mapping = insert_attack_mapping(
            &pool,
            NewAttackMapping {
                incident_id: "inc-1",
                evidence_id: None,
                tactic: AttackTactic::InitialAccess,
                technique_id: "T1133",
                sub_technique_id: None,
                basis: "logon tipo 10 de IP externo",
                confidence: Confidence::Medium,
                analyst_note: None,
                actor_id: None,
            },
        )
        .await
        .expect("attack mapping");

        let attack_ids = [mapping.as_str()];
        let nist_ids = ["PR.AA-03", "DE.CM-01"];
        let id = insert_finding(
            &pool,
            NewFinding {
                attack_mapping_ids: &attack_ids,
                nist_subcategory_ids: &nist_ids,
                reference: Some("CWE-308"),
                ..finding("inc-1", "RDP exposto sem MFA")
            },
        )
        .await
        .expect("finding insert");

        let record = analytical_results::get(&pool, &id)
            .await
            .expect("get")
            .expect("record exists");
        assert_eq!(record.result_type, "FINDING");
        assert_eq!(record.confidence.as_deref(), Some("MEDIUM"));
        let payload: serde_json::Value = serde_json::from_str(&record.payload).expect("json");
        assert_eq!(payload["severity"], "HIGH");
        assert_eq!(payload["finding_kind"], "MISSING_CONTROL");
        assert_eq!(payload["reference"], "CWE-308");
        assert_eq!(payload["nist_subcategories"][1], "DE.CM-01");

        let related = analytical_results::related_to(&pool, &id)
            .await
            .expect("relations");
        assert_eq!(related.len(), 1);
        assert_eq!(related[0].0, RelationType::DerivedFrom);
        assert_eq!(related[0].1.id, mapping);
    }

    #[tokio::test]
    async fn finding_rejects_missing_title_severity_none_and_missing_evidence() {
        let pool = test_pool().await;
        seed_incident(&pool, "inc-2").await;

        assert!(
            insert_finding(&pool, finding("inc-2", "   "))
                .await
                .is_err()
        );
        assert!(
            insert_finding(
                &pool,
                NewFinding {
                    severity: ImpactLevel::None,
                    ..finding("inc-2", "sem severidade")
                },
            )
            .await
            .is_err()
        );
        assert!(
            insert_finding(
                &pool,
                NewFinding {
                    basis: "  ",
                    ..finding("inc-2", "sem lastro")
                },
            )
            .await
            .is_err(),
            "no evidence_id, no basis, no supporting results"
        );
    }

    #[tokio::test]
    async fn finding_rejects_bad_nist_id_wrong_type_and_other_incident_links() {
        let pool = test_pool().await;
        seed_incident(&pool, "inc-3").await;
        seed_incident(&pool, "inc-3b").await;

        assert!(
            insert_finding(
                &pool,
                NewFinding {
                    nist_subcategory_ids: &["XX.YY-99"],
                    ..finding("inc-3", "NIST inválido")
                },
            )
            .await
            .is_err()
        );

        // Um Finding não é um ATTACK_MAPPING.
        let other = insert_finding(&pool, finding("inc-3", "base"))
            .await
            .expect("finding insert");
        let wrong_type = [other.as_str()];
        assert!(
            insert_finding(
                &pool,
                NewFinding {
                    attack_mapping_ids: &wrong_type,
                    ..finding("inc-3", "tipo errado")
                },
            )
            .await
            .is_err()
        );

        // Mapeamento ATT&CK de outro incidente.
        let foreign = insert_attack_mapping(
            &pool,
            NewAttackMapping {
                incident_id: "inc-3b",
                evidence_id: None,
                tactic: AttackTactic::Execution,
                technique_id: "T1059",
                sub_technique_id: None,
                basis: "powershell codificado",
                confidence: Confidence::Medium,
                analyst_note: None,
                actor_id: None,
            },
        )
        .await
        .expect("foreign mapping");
        let foreign_ids = [foreign.as_str()];
        assert!(
            insert_finding(
                &pool,
                NewFinding {
                    attack_mapping_ids: &foreign_ids,
                    ..finding("inc-3", "outro incidente")
                },
            )
            .await
            .is_err()
        );
    }

    #[tokio::test]
    async fn finding_revision_supersedes_and_keeps_the_same_correlation_key() {
        let pool = test_pool().await;
        seed_incident(&pool, "inc-4").await;
        seed_incident(&pool, "inc-4b").await;

        let first = insert_finding(&pool, finding("inc-4", "versão 1"))
            .await
            .expect("first");
        let second = insert_finding(
            &pool,
            NewFinding {
                supersedes: Some(&first),
                severity: ImpactLevel::Critical,
                ..finding("inc-4", "versão 2")
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

        // Revisar um achado de outro incidente é recusado.
        assert!(
            insert_finding(
                &pool,
                NewFinding {
                    supersedes: Some(&first),
                    ..finding("inc-4b", "intruso")
                },
            )
            .await
            .is_err()
        );
    }

    #[tokio::test]
    async fn cause_conclusion_enforces_levels_findings_and_explicit_gaps() {
        let pool = test_pool().await;
        seed_incident(&pool, "inc-5").await;
        seed_incident(&pool, "inc-5b").await;
        let f1 = insert_finding(&pool, finding("inc-5", "f1"))
            .await
            .expect("finding");
        let foreign = insert_finding(&pool, finding("inc-5b", "f-outro"))
            .await
            .expect("foreign finding");
        let ids = [f1.as_str()];

        // Sem achado de suporte, ou com achado de outro incidente.
        assert!(insert_root_cause(&pool, cause("inc-5", &[])).await.is_err());
        let foreign_ids = [foreign.as_str()];
        assert!(
            insert_root_cause(&pool, cause("inc-5", &foreign_ids))
                .await
                .is_err()
        );

        // Causa-raiz definitiva exige confiança HIGH/CONFIRMED.
        assert!(
            insert_root_cause(
                &pool,
                NewRootCause {
                    level: CauseLevel::RootCause,
                    confidence: Confidence::Medium,
                    ..cause("inc-5", &ids)
                },
            )
            .await
            .is_err()
        );
        // HIGH sem base é recusada (mesma regra de F1-04).
        assert!(
            insert_root_cause(
                &pool,
                NewRootCause {
                    level: CauseLevel::RootCause,
                    confidence: Confidence::High,
                    basis: "  ",
                    ..cause("inc-5", &ids)
                },
            )
            .await
            .is_err()
        );

        // Causa provável com lacunas: grava e registra as lacunas.
        let probable = insert_root_cause(
            &pool,
            NewRootCause {
                gaps: &["sem logs do firewall", "  ", "host DB01 não coletado"],
                ..cause("inc-5", &ids)
            },
        )
        .await
        .expect("probable cause");
        let record = analytical_results::get(&pool, &probable)
            .await
            .expect("get")
            .expect("exists");
        assert_eq!(record.result_type, "ROOT_CAUSE");
        let payload: serde_json::Value = serde_json::from_str(&record.payload).expect("json");
        assert_eq!(payload["level"], "PROBABLE_CAUSE");
        assert_eq!(payload["gap_status"], "DECLARED");
        assert_eq!(payload["gaps"].as_array().map(Vec::len), Some(2));

        // Sem lacunas: a ausência é uma declaração explícita.
        let confirmed = insert_root_cause(
            &pool,
            NewRootCause {
                level: CauseLevel::RootCause,
                confidence: Confidence::Confirmed,
                ..cause("inc-5", &ids)
            },
        )
        .await
        .expect("root cause");
        let record = analytical_results::get(&pool, &confirmed)
            .await
            .expect("get")
            .expect("exists");
        let payload: serde_json::Value = serde_json::from_str(&record.payload).expect("json");
        assert_eq!(payload["level"], "ROOT_CAUSE");
        assert_eq!(payload["gap_status"], "NONE_DECLARED");

        let related = analytical_results::related_to(&pool, &confirmed)
            .await
            .expect("relations");
        assert_eq!(related.len(), 1);
        assert_eq!(related[0].1.id, f1);
    }

    #[test]
    fn risk_matrix_counts_only_existing_cells_in_descending_order() {
        let cells = risk_matrix(&[
            (ImpactLevel::Low, Confidence::High),
            (ImpactLevel::Critical, Confidence::Medium),
            (ImpactLevel::Critical, Confidence::Medium),
            (ImpactLevel::Critical, Confidence::Confirmed),
        ]);
        let shape: Vec<(&str, &str, usize)> = cells
            .iter()
            .map(|c| (c.severity.as_str(), c.confidence.as_str(), c.count))
            .collect();
        assert_eq!(
            shape,
            vec![
                ("CRITICAL", "CONFIRMED", 1),
                ("CRITICAL", "MEDIUM", 2),
                ("LOW", "HIGH", 1),
            ]
        );
        assert!(risk_matrix(&[]).is_empty());
    }
}
