//! F1-29 (RF-052) — item de plano de fortalecimento.
//!
//! Uma entidade só alimenta os Entregáveis 3, 4 e 5: hardening de SO,
//! redes e dispositivos, autenticação e acesso, monitoramento, criptografia
//! em repouso e em trânsito, gestão de chaves e políticas de resposta a
//! incidentes. Os oito domínios correspondem um a um às sete ações do plano
//! de fortalecimento da demanda (a criptografia se divide em repouso e
//! trânsito). F1-06 (plano de hardening) é o domínio `OS_HARDENING` desta
//! mesma entidade: não há código separado nem artefato executável.
//!
//! Cabe no envelope de F1-01 (`AnalyticalResultType::PlanItem`, reservado
//! pela migration 0010 — ADR-0023). Não há tabela nova.
//!
//! ## Regras que o código impõe
//!
//! - **RB-004 (orçamento):** prioridade, esforço, classe de custo e fase são
//!   obrigatórios e tipados. Não há valores em moeda: o plano diz *quanto
//!   pesa*, não *quanto custa*.
//! - **RB-005 (políticas e regulação):** `regulatory_reference` é campo
//!   próprio, separado da referência de framework (`framework_reference`:
//!   CIS, NIST).
//! - **Item sem achado é "recomendação geral".** Ligar o item a um
//!   `FINDING` do mesmo incidente é opcional, mas o payload declara
//!   `support_kind` (`FINDING_BACKED` ou `GENERAL_RECOMMENDATION`): um item
//!   sem lastro nunca se passa por achado.
//! - **Criptografia e chaves exigem os cinco detalhes** (classe do dado,
//!   local, mecanismo, rotação, recuperação) — são o Entregável 5 — e só
//!   esses domínios os aceitam.
//! - **Verificável:** "como verificar" é obrigatório.
//! - **Dependências sem ciclo:** uma revisão não pode passar a depender,
//!   direta ou indiretamente, dela mesma.
//!
//! O catálogo NIST não é embutido (ADR-0011): `nist_subcategory_ids` são
//! validados só quanto à forma.

use super::validation_error;
use crate::persistence::analytical_results::{
    self, AnalyticalResultType, NewAnalyticalResult, RelationType,
};
use crate::persistence::findings::require_result;
use crate::persistence::nist_profiles::is_valid_subcategory_id;
use serde::Serialize;
use sqlx::SqlitePool;
use std::collections::BTreeSet;
use uuid::Uuid;

// ---------------------------------------------------------------------
// Vocabulário
// ---------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanDomain {
    OsHardening,
    NetworkDevices,
    AccessAuthentication,
    Monitoring,
    CryptoAtRest,
    CryptoInTransit,
    KeyManagement,
    IncidentResponsePolicy,
}

impl PlanDomain {
    /// Ordem canônica (a das ações da demanda), usada na cobertura e na
    /// ordenação dos itens.
    pub const ALL: [PlanDomain; 8] = [
        Self::OsHardening,
        Self::NetworkDevices,
        Self::AccessAuthentication,
        Self::Monitoring,
        Self::CryptoAtRest,
        Self::CryptoInTransit,
        Self::KeyManagement,
        Self::IncidentResponsePolicy,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::OsHardening => "OS_HARDENING",
            Self::NetworkDevices => "REDE_DISPOSITIVOS",
            Self::AccessAuthentication => "AUTENTICACAO_ACESSO",
            Self::Monitoring => "MONITORAMENTO",
            Self::CryptoAtRest => "CRIPTO_REPOUSO",
            Self::CryptoInTransit => "CRIPTO_TRANSITO",
            Self::KeyManagement => "GESTAO_CHAVES",
            Self::IncidentResponsePolicy => "POLITICA_IR",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|domain| domain.as_str() == value)
    }

    /// Texto da ação correspondente na demanda.
    pub fn label_pt(self) -> &'static str {
        match self {
            Self::OsHardening => "Hardening de SO e servidores",
            Self::NetworkDevices => "Redes e dispositivos",
            Self::AccessAuthentication => "Autenticação e controle de acesso",
            Self::Monitoring => "Monitoramento de segurança",
            Self::CryptoAtRest => "Criptografia de dados em repouso",
            Self::CryptoInTransit => "Criptografia de dados em trânsito",
            Self::KeyManagement => "Gestão de chaves criptográficas",
            Self::IncidentResponsePolicy => "Políticas de resposta a incidentes",
        }
    }

    /// Domínios do Entregável 5, que exigem os cinco detalhes de proteção.
    pub fn needs_crypto_detail(self) -> bool {
        matches!(
            self,
            Self::CryptoAtRest | Self::CryptoInTransit | Self::KeyManagement
        )
    }

    pub fn position(self) -> usize {
        Self::ALL
            .iter()
            .position(|domain| *domain == self)
            .unwrap_or(Self::ALL.len())
    }
}

/// P1 é o que se faz primeiro.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanPriority {
    P1,
    P2,
    P3,
}

impl PlanPriority {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::P1 => "P1",
            Self::P2 => "P2",
            Self::P3 => "P3",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "P1" => Some(Self::P1),
            "P2" => Some(Self::P2),
            "P3" => Some(Self::P3),
            _ => None,
        }
    }

    pub fn rank(self) -> u8 {
        match self {
            Self::P1 => 1,
            Self::P2 => 2,
            Self::P3 => 3,
        }
    }
}

/// Esforço relativo: S (horas a poucos dias), M (semanas), L (meses ou
/// projeto próprio).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanEffort {
    Small,
    Medium,
    Large,
}

impl PlanEffort {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Small => "S",
            Self::Medium => "M",
            Self::Large => "L",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "S" => Some(Self::Small),
            "M" => Some(Self::Medium),
            "L" => Some(Self::Large),
            _ => None,
        }
    }
}

/// Classe de custo relativa ao orçamento do cliente (RB-004). Sem valores
/// em moeda; `NONE` é "só configuração, sem gasto novo".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CostClass {
    NoExtraCost,
    Low,
    Medium,
    High,
}

impl CostClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NoExtraCost => "NONE",
            Self::Low => "LOW",
            Self::Medium => "MEDIUM",
            Self::High => "HIGH",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "NONE" => Some(Self::NoExtraCost),
            "LOW" => Some(Self::Low),
            "MEDIUM" => Some(Self::Medium),
            "HIGH" => Some(Self::High),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanPhase {
    Immediate,
    ShortTerm,
    Strategic,
}

impl PlanPhase {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Immediate => "IMMEDIATE",
            Self::ShortTerm => "SHORT_TERM",
            Self::Strategic => "STRATEGIC",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "IMMEDIATE" => Some(Self::Immediate),
            "SHORT_TERM" => Some(Self::ShortTerm),
            "STRATEGIC" => Some(Self::Strategic),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------
// Item de plano
// ---------------------------------------------------------------------

/// Os cinco detalhes do Entregável 5.
pub struct CryptoDetail<'a> {
    /// Classe do dado protegido (ex.: "dados pessoais de clientes").
    pub data_class: &'a str,
    /// Onde o dado vive ou trafega (ex.: "banco PostgreSQL em DB01").
    pub location: &'a str,
    /// Mecanismo (ex.: "AES-256 no volume", "TLS 1.3").
    pub mechanism: &'a str,
    /// Política de rotação de chaves.
    pub rotation: &'a str,
    /// Recuperação de chave (custódia, backup, quem pode restaurar).
    pub recovery: &'a str,
}

pub struct NewPlanItem<'a> {
    pub incident_id: &'a str,
    pub domain: PlanDomain,
    pub title: &'a str,
    /// Por que este item existe.
    pub rationale: &'a str,
    pub priority: PlanPriority,
    pub effort: PlanEffort,
    pub cost_class: CostClass,
    pub phase: PlanPhase,
    pub precondition: &'a str,
    pub postcondition: &'a str,
    /// Como verificar que foi feito (obrigatório).
    pub verification: &'a str,
    /// CIS, NIST ou similar (ex.: "CIS Control 5.2").
    pub framework_reference: Option<&'a str>,
    /// Norma, lei ou política interna (RB-005).
    pub regulatory_reference: Option<&'a str>,
    pub nist_subcategory_ids: &'a [&'a str],
    /// Achados do mesmo incidente que motivam o item (opcional).
    pub finding_ids: &'a [&'a str],
    /// Itens de plano do mesmo incidente que precisam vir antes.
    pub depends_on: &'a [&'a str],
    pub crypto: Option<CryptoDetail<'a>>,
    /// Item anterior que este revisa (mesma `correlation_key`).
    pub supersedes: Option<&'a str>,
    pub actor_id: Option<&'a str>,
}

fn require_text(value: &str, what: &str) -> Result<(), sqlx::Error> {
    if value.trim().is_empty() {
        return Err(validation_error(format!("a plan item requires {what}")));
    }
    Ok(())
}

/// Dependências diretas de um item vigente, lidas do payload.
async fn depends_on_of(pool: &SqlitePool, id: &str) -> Result<(String, Vec<String>), sqlx::Error> {
    let record = analytical_results::get(pool, id)
        .await?
        .ok_or_else(|| validation_error(format!("plan item not found: {id}")))?;
    let payload: serde_json::Value = serde_json::from_str(&record.payload)
        .map_err(|err| validation_error(format!("plan item payload is not JSON: {err}")))?;
    let dependencies = match payload.get("depends_on") {
        None => Vec::new(),
        Some(value) => {
            let items = value
                .as_array()
                .ok_or_else(|| validation_error("stored plan item depends_on is not an array"))?;
            items
                .iter()
                .map(|item| {
                    item.as_str().map(str::to_string).ok_or_else(|| {
                        validation_error("stored plan item depends_on contains a non-string id")
                    })
                })
                .collect::<Result<Vec<_>, sqlx::Error>>()?
        }
    };
    Ok((record.correlation_key, dependencies))
}

/// Recusa uma revisão que passaria a depender, direta ou indiretamente, de
/// si mesma. A busca é limitada: o plano de um incidente tem dezenas de
/// itens, não milhares.
async fn reject_dependency_cycle(
    pool: &SqlitePool,
    own_correlation_key: &str,
    depends_on: &[&str],
) -> Result<(), sqlx::Error> {
    let mut pending: Vec<String> = depends_on.iter().map(|id| id.to_string()).collect();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    while let Some(id) = pending.pop() {
        if !seen.insert(id.clone()) {
            continue;
        }
        if seen.len() > 512 {
            return Err(validation_error("plan dependency chain is too long"));
        }
        let (correlation_key, dependencies) = depends_on_of(pool, &id).await?;
        if correlation_key == own_correlation_key {
            return Err(validation_error(
                "a plan item cannot depend on itself, directly or through other items",
            ));
        }
        pending.extend(dependencies);
    }
    Ok(())
}

/// Insere um item de plano (ou uma revisão, quando `supersedes` é informado).
pub async fn insert_plan_item(
    pool: &SqlitePool,
    input: NewPlanItem<'_>,
) -> Result<String, sqlx::Error> {
    require_text(input.title, "a title")?;
    require_text(input.rationale, "a rationale")?;
    require_text(input.verification, "a way to verify it (verification)")?;

    match (&input.crypto, input.domain.needs_crypto_detail()) {
        (None, true) => {
            return Err(validation_error(format!(
                "domain {} requires the five protection details (data class, location, mechanism, rotation, recovery)",
                input.domain.as_str()
            )));
        }
        (Some(_), false) => {
            return Err(validation_error(format!(
                "protection details only apply to CRIPTO_REPOUSO, CRIPTO_TRANSITO and GESTAO_CHAVES, not {}",
                input.domain.as_str()
            )));
        }
        (Some(crypto), true) => {
            for (value, what) in [
                (crypto.data_class, "data class"),
                (crypto.location, "location"),
                (crypto.mechanism, "mechanism"),
                (crypto.rotation, "key rotation"),
                (crypto.recovery, "key recovery"),
            ] {
                require_text(value, &format!("the protection detail: {what}"))?;
            }
        }
        (None, false) => {}
    }

    for subcategory in input.nist_subcategory_ids {
        if !is_valid_subcategory_id(subcategory) {
            return Err(validation_error(format!(
                "invalid NIST subcategory id shape: {subcategory}"
            )));
        }
    }
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
    for id in input.depends_on {
        require_result(
            pool,
            id,
            Some(AnalyticalResultType::PlanItem),
            Some(input.incident_id),
            "plan item dependency",
        )
        .await?;
    }

    let correlation_key = match input.supersedes {
        Some(previous_id) => {
            let previous = require_result(
                pool,
                previous_id,
                Some(AnalyticalResultType::PlanItem),
                Some(input.incident_id),
                "plan item to supersede",
            )
            .await?;
            reject_dependency_cycle(pool, &previous.correlation_key, input.depends_on).await?;
            previous.correlation_key
        }
        None => format!("plan:{}:{}", input.incident_id, Uuid::new_v4()),
    };

    let support_kind = if input.finding_ids.is_empty() {
        "GENERAL_RECOMMENDATION"
    } else {
        "FINDING_BACKED"
    };
    let crypto = input.crypto.as_ref().map(|crypto| {
        serde_json::json!({
            "data_class": crypto.data_class.trim(),
            "location": crypto.location.trim(),
            "mechanism": crypto.mechanism.trim(),
            "rotation": crypto.rotation.trim(),
            "recovery": crypto.recovery.trim(),
        })
    });
    let payload_json = serde_json::json!({
        "kind": "PLAN_ITEM",
        "domain": input.domain.as_str(),
        "title": input.title.trim(),
        "rationale": input.rationale.trim(),
        "priority": input.priority.as_str(),
        "effort": input.effort.as_str(),
        "cost_class": input.cost_class.as_str(),
        "phase": input.phase.as_str(),
        "precondition": input.precondition.trim(),
        "postcondition": input.postcondition.trim(),
        "verification": input.verification.trim(),
        "framework_reference": input.framework_reference,
        "regulatory_reference": input.regulatory_reference,
        "nist_subcategories": input.nist_subcategory_ids,
        "finding_ids": input.finding_ids,
        "depends_on": input.depends_on,
        "support_kind": support_kind,
        "crypto": crypto,
    })
    .to_string();
    let provenance_json = serde_json::json!({
        "producer": "dfir-app",
        "method": "analyst-authored plan item",
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
            result_type: AnalyticalResultType::PlanItem,
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
            related: related.as_slice(),
            actor_id: input.actor_id,
        },
    )
    .await
}

// ---------------------------------------------------------------------
// Cobertura das ações da demanda
// ---------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DomainCoverage {
    pub domain: String,
    pub label: String,
    pub items: usize,
    /// Itens do domínio sem achado de suporte (recomendações gerais).
    pub general_items: usize,
}

/// Uma linha por domínio, na ordem canônica, **inclusive os vazios**: que
/// uma das ações da demanda não tenha nenhum item é uma informação, não um
/// campo a omitir. Cada entrada de `items` é (domínio, é recomendação geral).
pub fn domain_coverage(items: &[(PlanDomain, bool)]) -> Vec<DomainCoverage> {
    PlanDomain::ALL
        .into_iter()
        .map(|domain| {
            let of_domain = items.iter().filter(|(d, _)| *d == domain);
            DomainCoverage {
                domain: domain.as_str().to_string(),
                label: domain.label_pt().to_string(),
                items: of_domain.clone().count(),
                general_items: of_domain.filter(|(_, general)| *general).count(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::analytical_results::Confidence;
    use crate::persistence::attack_impact::ImpactLevel;
    use crate::persistence::findings::{FindingKind, NewFinding, insert_finding};

    async fn test_pool() -> SqlitePool {
        let temp = tempfile::tempdir().expect("temporary directory");
        let db_path = temp.path().join("plan.sqlite");
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

    async fn seed_finding(pool: &SqlitePool, incident_id: &str) -> String {
        insert_finding(
            pool,
            NewFinding {
                incident_id,
                evidence_id: None,
                kind: FindingKind::MissingControl,
                title: "RDP exposto sem MFA",
                description: "",
                severity: ImpactLevel::High,
                basis: "EVTX Security 4625 em massa",
                reference: None,
                nist_subcategory_ids: &[],
                attack_mapping_ids: &[],
                supporting_result_ids: &[],
                confidence: Confidence::Medium,
                analyst_note: None,
                supersedes: None,
                actor_id: None,
            },
        )
        .await
        .expect("finding")
    }

    fn item<'a>(incident_id: &'a str, title: &'a str) -> NewPlanItem<'a> {
        NewPlanItem {
            incident_id,
            domain: PlanDomain::AccessAuthentication,
            title,
            rationale: "o acesso inicial foi por RDP sem segundo fator",
            priority: PlanPriority::P1,
            effort: PlanEffort::Small,
            cost_class: CostClass::NoExtraCost,
            phase: PlanPhase::Immediate,
            precondition: "RDP acessível pela internet",
            postcondition: "RDP só via VPN com MFA",
            verification: "varredura externa não encontra 3389 aberta",
            framework_reference: Some("CIS Control 6.5"),
            regulatory_reference: None,
            nist_subcategory_ids: &[],
            finding_ids: &[],
            depends_on: &[],
            crypto: None,
            supersedes: None,
            actor_id: None,
        }
    }

    fn crypto() -> CryptoDetail<'static> {
        CryptoDetail {
            data_class: "dados pessoais de clientes",
            location: "banco PostgreSQL em DB01",
            mechanism: "AES-256 no volume",
            rotation: "anual",
            recovery: "chave em cofre, duas pessoas para restaurar",
        }
    }

    async fn payload_of(pool: &SqlitePool, id: &str) -> serde_json::Value {
        let record = analytical_results::get(pool, id)
            .await
            .expect("get")
            .expect("exists");
        serde_json::from_str(&record.payload).expect("json payload")
    }

    #[test]
    fn vocabulary_round_trips_and_only_three_domains_need_crypto_detail() {
        assert_eq!(PlanDomain::ALL.len(), 8);
        for domain in PlanDomain::ALL {
            assert_eq!(PlanDomain::from_str(domain.as_str()), Some(domain));
            assert!(!domain.label_pt().is_empty());
        }
        let crypto_domains: Vec<&str> = PlanDomain::ALL
            .into_iter()
            .filter(|d| d.needs_crypto_detail())
            .map(PlanDomain::as_str)
            .collect();
        assert_eq!(
            crypto_domains,
            vec!["CRIPTO_REPOUSO", "CRIPTO_TRANSITO", "GESTAO_CHAVES"]
        );
        for priority in [PlanPriority::P1, PlanPriority::P2, PlanPriority::P3] {
            assert_eq!(PlanPriority::from_str(priority.as_str()), Some(priority));
        }
        for effort in [PlanEffort::Small, PlanEffort::Medium, PlanEffort::Large] {
            assert_eq!(PlanEffort::from_str(effort.as_str()), Some(effort));
        }
        for cost in [
            CostClass::NoExtraCost,
            CostClass::Low,
            CostClass::Medium,
            CostClass::High,
        ] {
            assert_eq!(CostClass::from_str(cost.as_str()), Some(cost));
        }
        for phase in [
            PlanPhase::Immediate,
            PlanPhase::ShortTerm,
            PlanPhase::Strategic,
        ] {
            assert_eq!(PlanPhase::from_str(phase.as_str()), Some(phase));
        }
        assert_eq!(PlanDomain::from_str("BOGUS"), None);
        assert_eq!(PlanPriority::from_str("P9"), None);
    }

    #[tokio::test]
    async fn item_with_a_finding_is_backed_and_linked_and_without_one_is_a_general_recommendation()
    {
        let pool = test_pool().await;
        seed_incident(&pool, "inc-1").await;
        let finding = seed_finding(&pool, "inc-1").await;

        let finding_ids = [finding.as_str()];
        let backed = insert_plan_item(
            &pool,
            NewPlanItem {
                finding_ids: &finding_ids,
                nist_subcategory_ids: &["PR.AA-03"],
                regulatory_reference: Some("Política interna de acesso remoto"),
                ..item("inc-1", "MFA no acesso remoto")
            },
        )
        .await
        .expect("backed item");
        let payload = payload_of(&pool, &backed).await;
        assert_eq!(payload["support_kind"], "FINDING_BACKED");
        assert_eq!(payload["priority"], "P1");
        assert_eq!(payload["effort"], "S");
        assert_eq!(payload["cost_class"], "NONE");
        assert_eq!(payload["phase"], "IMMEDIATE");
        assert_eq!(payload["framework_reference"], "CIS Control 6.5");
        assert_eq!(
            payload["regulatory_reference"],
            "Política interna de acesso remoto"
        );
        assert!(payload["crypto"].is_null());
        let related = analytical_results::related_to(&pool, &backed)
            .await
            .expect("relations");
        assert_eq!(related.len(), 1);
        assert_eq!(related[0].0, RelationType::DerivedFrom);
        assert_eq!(related[0].1.id, finding);

        let general = insert_plan_item(&pool, item("inc-1", "Revisar política de senhas"))
            .await
            .expect("general item");
        assert_eq!(
            payload_of(&pool, &general).await["support_kind"],
            "GENERAL_RECOMMENDATION"
        );
        assert!(
            analytical_results::related_to(&pool, &general)
                .await
                .expect("relations")
                .is_empty()
        );
    }

    #[tokio::test]
    async fn item_requires_title_rationale_and_verification() {
        let pool = test_pool().await;
        seed_incident(&pool, "inc-2").await;
        assert!(insert_plan_item(&pool, item("inc-2", "  ")).await.is_err());
        assert!(
            insert_plan_item(
                &pool,
                NewPlanItem {
                    rationale: " ",
                    ..item("inc-2", "sem motivo")
                },
            )
            .await
            .is_err()
        );
        assert!(
            insert_plan_item(
                &pool,
                NewPlanItem {
                    verification: "",
                    ..item("inc-2", "sem verificação")
                },
            )
            .await
            .is_err()
        );
    }

    #[tokio::test]
    async fn crypto_domains_require_all_five_details_and_other_domains_reject_them() {
        let pool = test_pool().await;
        seed_incident(&pool, "inc-3").await;

        for domain in [
            PlanDomain::CryptoAtRest,
            PlanDomain::CryptoInTransit,
            PlanDomain::KeyManagement,
        ] {
            assert!(
                insert_plan_item(
                    &pool,
                    NewPlanItem {
                        domain,
                        ..item("inc-3", "sem detalhes")
                    },
                )
                .await
                .is_err(),
                "{} sem detalhes de proteção",
                domain.as_str()
            );
        }
        assert!(
            insert_plan_item(
                &pool,
                NewPlanItem {
                    domain: PlanDomain::CryptoAtRest,
                    crypto: Some(CryptoDetail {
                        recovery: "  ",
                        ..crypto()
                    }),
                    ..item("inc-3", "sem recuperação")
                },
            )
            .await
            .is_err()
        );
        assert!(
            insert_plan_item(
                &pool,
                NewPlanItem {
                    crypto: Some(crypto()),
                    ..item("inc-3", "detalhe no domínio errado")
                },
            )
            .await
            .is_err()
        );

        let id = insert_plan_item(
            &pool,
            NewPlanItem {
                domain: PlanDomain::CryptoAtRest,
                crypto: Some(crypto()),
                ..item("inc-3", "Cifrar o volume do banco")
            },
        )
        .await
        .expect("crypto item");
        let payload = payload_of(&pool, &id).await;
        assert_eq!(payload["domain"], "CRIPTO_REPOUSO");
        assert_eq!(payload["crypto"]["mechanism"], "AES-256 no volume");
        assert_eq!(payload["crypto"]["rotation"], "anual");
    }

    #[tokio::test]
    async fn item_rejects_bad_nist_ids_and_links_to_other_incidents_or_types() {
        let pool = test_pool().await;
        seed_incident(&pool, "inc-4").await;
        seed_incident(&pool, "inc-4b").await;
        let finding = seed_finding(&pool, "inc-4").await;
        let foreign_finding = seed_finding(&pool, "inc-4b").await;

        assert!(
            insert_plan_item(
                &pool,
                NewPlanItem {
                    nist_subcategory_ids: &["XX.YY-99"],
                    ..item("inc-4", "NIST inválido")
                },
            )
            .await
            .is_err()
        );
        let foreign_ids = [foreign_finding.as_str()];
        assert!(
            insert_plan_item(
                &pool,
                NewPlanItem {
                    finding_ids: &foreign_ids,
                    ..item("inc-4", "achado de outro incidente")
                },
            )
            .await
            .is_err()
        );
        // Dependência precisa ser item de plano: um Finding não serve.
        let not_a_plan_item = [finding.as_str()];
        assert!(
            insert_plan_item(
                &pool,
                NewPlanItem {
                    depends_on: &not_a_plan_item,
                    ..item("inc-4", "dependência de tipo errado")
                },
            )
            .await
            .is_err()
        );
    }

    #[tokio::test]
    async fn revision_rejects_malformed_stored_plan_dependencies() {
        let pool = test_pool().await;

        for (case, expected) in [
            ("shape", "stored plan item depends_on is not an array"),
            (
                "non_string",
                "stored plan item depends_on contains a non-string id",
            ),
        ] {
            let incident_id = format!("inc-bad-dependency-{case}");
            seed_incident(&pool, &incident_id).await;
            let a = insert_plan_item(&pool, item(&incident_id, "A"))
                .await
                .expect("A");
            let a_ids = [a.as_str()];
            let b = insert_plan_item(
                &pool,
                NewPlanItem {
                    depends_on: &a_ids,
                    ..item(&incident_id, "B depends on A")
                },
            )
            .await
            .expect("B");
            let b_record = analytical_results::get(&pool, &b)
                .await
                .expect("get B")
                .expect("B exists");
            let bad_id = format!("bad-dependency-{case}");
            let bad_payload = if case == "shape" {
                serde_json::json!({"depends_on": a})
            } else {
                serde_json::json!({"depends_on": [42]})
            };
            sqlx::query("INSERT INTO analytical_results(id,schema_version,result_type,correlation_key,incident_id,payload,provenance,supersedes,created_at) VALUES(?,'analytical-result-v1','PLAN_ITEM',?,?,?,?,?,'2026-10-03T00:00:00Z')")
                .bind(&bad_id)
                .bind(&b_record.correlation_key)
                .bind(&incident_id)
                .bind(bad_payload.to_string())
                .bind("{}")
                .bind(&b)
                .execute(&pool)
                .await
                .expect("malformed current dependency fixture");

            let bad_ids = [bad_id.as_str()];
            let error = insert_plan_item(
                &pool,
                NewPlanItem {
                    supersedes: Some(&a),
                    depends_on: &bad_ids,
                    ..item(&incident_id, "A revised through malformed B")
                },
            )
            .await
            .expect_err("malformed dependency allowed a cycle-forming revision");
            assert!(
                error.to_string().contains(expected),
                "unexpected error for {case}: {error:?}"
            );

            let a_record = analytical_results::get(&pool, &a)
                .await
                .expect("get A")
                .expect("A exists");
            let current_a =
                analytical_results::current_for_correlation_key(&pool, &a_record.correlation_key)
                    .await
                    .expect("current A query")
                    .expect("A remains current");
            assert_eq!(current_a.id, a, "no A successor allowed for {case}");
            let current_b =
                analytical_results::current_for_correlation_key(&pool, &b_record.correlation_key)
                    .await
                    .expect("current B query")
                    .expect("malformed B remains current");
            assert_eq!(current_b.id, bad_id);
            let a_versions: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM analytical_results WHERE correlation_key=?",
            )
            .bind(&a_record.correlation_key)
            .fetch_one(&pool)
            .await
            .expect("A history count");
            assert_eq!(a_versions, 1, "no A revision allowed for {case}");
        }
    }

    #[tokio::test]
    async fn depends_on_of_accepts_legacy_missing_field() {
        let pool = test_pool().await;
        seed_incident(&pool, "inc-legacy-dependency").await;
        sqlx::query("INSERT INTO analytical_results(id,schema_version,result_type,correlation_key,incident_id,payload,provenance,created_at) VALUES('legacy-dependency','analytical-result-v1','PLAN_ITEM','plan:legacy','inc-legacy-dependency','{}','{}','2026-10-03T00:00:00Z')")
            .execute(&pool)
            .await
            .expect("legacy dependency fixture");

        let (correlation_key, dependencies) = depends_on_of(&pool, "legacy-dependency")
            .await
            .expect("missing depends_on is a legacy empty list");
        assert_eq!(correlation_key, "plan:legacy");
        assert!(dependencies.is_empty());
    }

    #[tokio::test]
    async fn revision_keeps_the_correlation_key_and_dependency_cycles_are_refused() {
        let pool = test_pool().await;
        seed_incident(&pool, "inc-5").await;
        seed_incident(&pool, "inc-5b").await;

        let a = insert_plan_item(&pool, item("inc-5", "A"))
            .await
            .expect("a");
        let a_ids = [a.as_str()];
        let b = insert_plan_item(
            &pool,
            NewPlanItem {
                depends_on: &a_ids,
                ..item("inc-5", "B depende de A")
            },
        )
        .await
        .expect("b");

        // Revisar A mudando a prioridade mantém a chave e vira o item vigente.
        let a2 = insert_plan_item(
            &pool,
            NewPlanItem {
                supersedes: Some(&a),
                priority: PlanPriority::P2,
                ..item("inc-5", "A (revisado)")
            },
        )
        .await
        .expect("revision");
        let a_record = analytical_results::get(&pool, &a)
            .await
            .expect("get")
            .expect("exists");
        let current =
            analytical_results::current_for_correlation_key(&pool, &a_record.correlation_key)
                .await
                .expect("current")
                .expect("has current");
        assert_eq!(current.id, a2);

        // Revisar A para depender de B fecha um ciclo A -> B -> A.
        let b_ids = [b.as_str()];
        assert!(
            insert_plan_item(
                &pool,
                NewPlanItem {
                    supersedes: Some(&a2),
                    depends_on: &b_ids,
                    ..item("inc-5", "A depende de B")
                },
            )
            .await
            .is_err(),
            "ciclo A -> B -> A"
        );
        // E depender de si mesmo, diretamente, também.
        let a2_ids = [a2.as_str()];
        assert!(
            insert_plan_item(
                &pool,
                NewPlanItem {
                    supersedes: Some(&a2),
                    depends_on: &a2_ids,
                    ..item("inc-5", "A depende de A")
                },
            )
            .await
            .is_err()
        );
        // Revisar item de outro incidente é recusado.
        assert!(
            insert_plan_item(
                &pool,
                NewPlanItem {
                    supersedes: Some(&a),
                    ..item("inc-5b", "intruso")
                },
            )
            .await
            .is_err()
        );
    }

    #[test]
    fn coverage_lists_every_domain_including_empty_ones() {
        let coverage = domain_coverage(&[
            (PlanDomain::AccessAuthentication, false),
            (PlanDomain::AccessAuthentication, true),
            (PlanDomain::CryptoAtRest, true),
        ]);
        assert_eq!(coverage.len(), 8);
        let shape: Vec<(&str, usize, usize)> = coverage
            .iter()
            .map(|c| (c.domain.as_str(), c.items, c.general_items))
            .collect();
        assert_eq!(
            shape,
            vec![
                ("OS_HARDENING", 0, 0),
                ("REDE_DISPOSITIVOS", 0, 0),
                ("AUTENTICACAO_ACESSO", 2, 1),
                ("MONITORAMENTO", 0, 0),
                ("CRIPTO_REPOUSO", 1, 1),
                ("CRIPTO_TRANSITO", 0, 0),
                ("GESTAO_CHAVES", 0, 0),
                ("POLITICA_IR", 0, 0),
            ]
        );
        assert_eq!(domain_coverage(&[]).len(), 8);
    }
}
