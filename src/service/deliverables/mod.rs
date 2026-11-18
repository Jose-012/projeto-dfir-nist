//! F1-07 (RF-013, ADR-0018) — os cinco documentos dos entregáveis, um por
//! entregável, gerados em Markdown determinístico a partir do que está
//! persistido, mais um `manifest.json` com hashes.
//!
//! - **Nada é inventado.** Cada documento só reorganiza dados registrados no
//!   incidente; seção sem dados diz "sem registro" em vez de sumir, e cada
//!   item cita o resultado de origem (id curto no texto, ids completos no
//!   manifest).
//! - **Determinístico.** A renderização é uma função pura de
//!   [`DeliverableData`]: mesmos dados, mesmos bytes (as listas são
//!   ordenadas por chave explícita, com o id como desempate). O único
//!   campo que muda entre execuções é `generated_at`, e fica só no
//!   manifest.
//! - **Seguro.** Texto de dados passa por `markdown::esc`/`markdown::code`;
//!   os nomes de arquivo são constantes; a pasta de saída é validada.
//! - **Só EVTX é analisado automaticamente** (escopo do protótipo): o
//!   Entregável 1 declara isso em "Escopo e limitações".

mod crypto;
mod hardening;
pub mod markdown;
mod maturity;
mod redaction;
mod report;
mod vulnerabilities;

pub use redaction::RedactionSummary;

use super::findings::{CriticalPathView, FindingsView};
use super::nist::NistFunctionView;
use super::plan::PlanView;
use super::{
    AnalyticalResultResponse, AppServices, EvidenceResponse, IncidentResponse, findings, gaps,
    nist, plan,
};
use crate::error::{AppError, AppResult};
use crate::infrastructure::storage::StoragePaths;
use crate::persistence::audit;
use markdown::{Doc, esc};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub const FILE_E1: &str = "01-relatorio-forense.md";
pub const FILE_E2: &str = "02-vulnerabilidades-attack.md";
pub const FILE_E3: &str = "03-nist-recomendacoes.md";
pub const FILE_E4: &str = "04-hardening.md";
pub const FILE_E5: &str = "05-criptografia.md";
pub const FILE_MANIFEST: &str = "manifest.json";

// ---------------------------------------------------------------------
// Dados
// ---------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct TimelineSummary {
    pub total_events: u64,
    pub first_event: Option<String>,
    pub last_event: Option<String>,
    pub warning_count: u64,
}

#[derive(Debug, Clone)]
pub enum TimelineState {
    /// Nenhum job de linha do tempo concluído para a evidência.
    NoCompletedJob,
    Summary(TimelineSummary),
    /// O índice não pôde ser lido; o motivo não vai para o documento.
    Unavailable,
}

#[derive(Debug, Clone)]
pub struct CustodyEntry {
    pub sequence: i64,
    pub action: String,
    pub actor: String,
    pub timestamp: String,
    pub reason: String,
}

/// Situação da detecção por regras numa evidência (F1-28).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DetectionSummary {
    /// A evidência não é EVTX: as regras não se aplicam.
    NotApplicable,
    /// A evidência foi descartada: os candidatos (que carregam contas, IPs e
    /// referências de evento) saíram com ela. Os achados, mapeamentos e IOCs
    /// que o analista confirmou antes continuam no incidente.
    Discarded,
    /// Nenhuma execução das regras sobre esta evidência.
    NotRun,
    Run {
        confirmed: u64,
        rejected: u64,
        pending: u64,
        /// Conjuntos de regras que produziram os candidatos.
        rule_sets: Vec<String>,
    },
}

#[derive(Debug, Clone)]
pub struct EvidenceSection {
    pub evidence: EvidenceResponse,
    pub timeline: TimelineState,
    pub detection: DetectionSummary,
    pub custody: Vec<CustodyEntry>,
    /// `None` quando a verificação não pôde ser feita.
    pub custody_chain_intact: Option<bool>,
}

impl EvidenceSection {
    /// A evidência foi descartada localmente: o arquivo e os artefatos de
    /// análise não existem mais; hash e custódia permanecem. Vem da própria
    /// custódia (ação `DESTROYED`), não de um campo à parte.
    pub fn is_destroyed(&self) -> bool {
        self.custody.iter().any(|entry| entry.action == "DESTROYED")
    }
}

/// Tudo que os cinco documentos usam. É a única entrada da renderização.
#[derive(Debug, Clone)]
pub struct DeliverableData {
    pub incident: IncidentResponse,
    /// Evidências citadas por algum resultado do incidente.
    pub evidence: Vec<EvidenceSection>,
    /// Evidências importadas que nenhum resultado do incidente cita.
    pub other_evidence: Vec<EvidenceResponse>,
    pub attack_mappings: Vec<AnalyticalResultResponse>,
    pub critical_path: CriticalPathView,
    pub impact: Option<AnalyticalResultResponse>,
    pub iocs: Vec<AnalyticalResultResponse>,
    pub recommendations: Vec<AnalyticalResultResponse>,
    pub findings: FindingsView,
    pub plan: PlanView,
    pub nist: NistFunctionView,
    pub gaps: Vec<AnalyticalResultResponse>,
    /// Lacunas de fonte declaradas pelo analista (F1-31).
    pub source_gaps: Vec<AnalyticalResultResponse>,
    /// `Some` quando os dados pessoais foram mascarados (F1-32).
    pub redaction: Option<RedactionSummary>,
}

/// Opções da geração.
#[derive(Debug, Clone, Copy)]
pub struct RenderOptions {
    /// Mascara IPs, e-mails, contas e nomes (F1-32). Padrão: ligado.
    pub mask_personal_data: bool,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            mask_personal_data: true,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RenderedDocument {
    pub file_name: &'static str,
    pub title: &'static str,
    pub content: String,
    /// Ids completos dos resultados citados no documento.
    pub source_result_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeliverableSet {
    pub incident_id: String,
    pub incident_name: String,
    pub documents: Vec<RenderedDocument>,
    /// `Some` quando os dados pessoais foram mascarados.
    pub redaction: Option<RedactionSummary>,
    /// Quantas evidências citadas o operador marcou como contendo dado pessoal.
    pub personal_data_evidence: usize,
}

// ---------------------------------------------------------------------
// Auxiliares compartilhados pelos renderizadores
// ---------------------------------------------------------------------

/// Texto de um campo do payload; vazio quando ausente ou não textual.
pub(super) fn field<'a>(result: &'a AnalyticalResultResponse, key: &str) -> &'a str {
    result
        .payload
        .get(key)
        .and_then(|value| value.as_str())
        .unwrap_or("")
}

/// Representação textual de um valor JSON (texto, número, lista, nulo).
pub(super) fn value_text(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => String::new(),
        serde_json::Value::String(text) => text.clone(),
        serde_json::Value::Array(items) => items
            .iter()
            .map(value_text)
            .filter(|item| !item.is_empty())
            .collect::<Vec<_>>()
            .join(", "),
        other => other.to_string(),
    }
}

pub(super) fn field_text(result: &AnalyticalResultResponse, key: &str) -> String {
    result.payload.get(key).map(value_text).unwrap_or_default()
}

pub(super) fn strings(result: &AnalyticalResultResponse, key: &str) -> Vec<String> {
    result
        .payload
        .get(key)
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

pub(super) fn severity_rank(token: &str) -> u8 {
    match token {
        "CRITICAL" => 4,
        "HIGH" => 3,
        "MEDIUM" => 2,
        "LOW" => 1,
        _ => 0,
    }
}

pub(super) fn confidence_rank(token: &str) -> u8 {
    match token {
        "CONFIRMED" => 4,
        "HIGH" => 3,
        "MEDIUM" => 2,
        "LOW" => 1,
        _ => 0,
    }
}

/// Nome do arquivo de evidência, ou o id curto quando não é conhecido.
pub(super) fn evidence_name(data: &DeliverableData, evidence_id: Option<&str>) -> String {
    let Some(id) = evidence_id else {
        return "—".to_string();
    };
    data.evidence
        .iter()
        .map(|section| &section.evidence)
        .chain(data.other_evidence.iter())
        .find(|evidence| evidence.id == id)
        .map_or_else(
            || markdown::code(&markdown::short(id)),
            |evidence| esc(&evidence.original_filename),
        )
}

/// Título do achado referenciado (com a severidade), ou uma nota quando o
/// achado já foi revisado (a referência guarda a versão anterior).
pub(super) fn finding_label(data: &DeliverableData, finding_id: &str) -> String {
    match data
        .findings
        .findings
        .iter()
        .find(|finding| finding.id == finding_id)
    {
        Some(finding) => format!(
            "{} ({})",
            esc(field(finding, "title")),
            markdown::code(field(finding, "severity"))
        ),
        None => format!(
            "achado {} (versão anterior)",
            markdown::code(&markdown::short(finding_id))
        ),
    }
}

/// Descreve um item de plano: prioridade, esforço, custo, fase, motivo,
/// referências, condições, verificação, dependências e, quando pedido, os
/// cinco detalhes de proteção. Usado pelos Entregáveis 4 e 5.
pub(super) fn write_plan_item(
    doc: &mut Doc,
    data: &DeliverableData,
    item: &AnalyticalResultResponse,
    with_protection_details: bool,
) {
    doc.h3(field(item, "title"));
    let origin = doc.cite(&item.id);
    let mut lines = vec![format!(
        "Prioridade {} · esforço {} · custo {} · fase {} · origem {origin}",
        markdown::code(field(item, "priority")),
        markdown::code(field(item, "effort")),
        markdown::code(field(item, "cost_class")),
        markdown::code(field(item, "phase")),
    )];
    let finding_ids = strings(item, "finding_ids");
    if finding_ids.is_empty() {
        lines.push("**Recomendação geral** (sem achado de suporte registrado).".to_string());
    } else {
        for id in &finding_ids {
            let cited = doc.cite(id);
            lines.push(format!(
                "Motivado pelo achado: {} · origem {cited}",
                finding_label(data, id)
            ));
        }
    }
    for (label, key) in [
        ("Motivo", "rationale"),
        ("Pré-condição", "precondition"),
        ("Pós-condição", "postcondition"),
        ("Como verificar", "verification"),
        ("Referência", "framework_reference"),
        ("Regulatório ou política interna", "regulatory_reference"),
    ] {
        let value = field(item, key);
        if !value.trim().is_empty() {
            lines.push(format!("{label}: {}", esc(value)));
        }
    }
    let nist = strings(item, "nist_subcategories");
    if !nist.is_empty() {
        lines.push(format!(
            "Subcategorias NIST: {}",
            nist.iter()
                .map(|id| markdown::code(id))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    let dependencies = strings(item, "depends_on");
    if !dependencies.is_empty() {
        let names: Vec<String> = dependencies
            .iter()
            .map(|id| {
                data.plan
                    .items
                    .iter()
                    .find(|other| other.id == *id)
                    .map_or_else(
                        || format!("item {}", markdown::code(&markdown::short(id))),
                        |other| esc(field(other, "title")),
                    )
            })
            .collect();
        lines.push(format!("Depende de: {}", names.join("; ")));
    }
    if with_protection_details && let Some(crypto) = item.payload.get("crypto") {
        for (label, key) in [
            ("Classe do dado", "data_class"),
            ("Local", "location"),
            ("Mecanismo", "mechanism"),
            ("Rotação de chave", "rotation"),
            ("Recuperação de chave", "recovery"),
        ] {
            let value = crypto.get(key).map(value_text).unwrap_or_default();
            if !value.trim().is_empty() {
                lines.push(format!("{label}: {}", esc(&value)));
            }
        }
    }
    doc.bullets(&lines);
}

/// Chave de ordenação dos itens de plano: prioridade, fase, título, id.
pub(super) fn plan_sort_key(item: &AnalyticalResultResponse) -> (u8, u8, String, String) {
    let priority = match field(item, "priority") {
        "P1" => 1,
        "P2" => 2,
        "P3" => 3,
        _ => 9,
    };
    let phase = match field(item, "phase") {
        "IMMEDIATE" => 0,
        "SHORT_TERM" => 1,
        "STRATEGIC" => 2,
        _ => 9,
    };
    (
        priority,
        phase,
        field(item, "title").to_string(),
        item.id.clone(),
    )
}

/// Texto da ação da demanda a que o domínio corresponde.
pub(super) fn domain_label(token: &str) -> String {
    match crate::persistence::plan_items::PlanDomain::from_str(token) {
        Some(domain) => domain.label_pt().to_string(),
        None => esc(token),
    }
}

pub(super) fn into_document(
    doc: Doc,
    file_name: &'static str,
    title: &'static str,
) -> RenderedDocument {
    let (content, source_result_ids) = doc.finish();
    RenderedDocument {
        file_name,
        title,
        content,
        source_result_ids,
    }
}

// ---------------------------------------------------------------------
// Renderização
// ---------------------------------------------------------------------

/// Ordena as listas que os renderizadores percorrem na ordem recebida
/// (evidências e eventos de custódia), para que a saída não dependa da
/// ordem de entrada. As demais listas são ordenadas dentro de cada
/// renderizador.
fn normalized(data: &DeliverableData) -> DeliverableData {
    let mut data = data.clone();
    data.evidence.sort_by(|a, b| {
        a.evidence
            .imported_at
            .cmp(&b.evidence.imported_at)
            .then_with(|| a.evidence.id.cmp(&b.evidence.id))
    });
    for section in &mut data.evidence {
        section.custody.sort_by_key(|entry| entry.sequence);
    }
    data.other_evidence.sort_by(|a, b| {
        a.imported_at
            .cmp(&b.imported_at)
            .then_with(|| a.id.cmp(&b.id))
    });
    data
}

/// Renderiza os cinco documentos. Função pura: não toca o banco nem o disco.
pub fn render_all(data: &DeliverableData) -> AppResult<DeliverableSet> {
    let data = &normalized(data);
    let documents = vec![
        report::render(data),
        vulnerabilities::render(data),
        maturity::render(data),
        hardening::render(data),
        crypto::render(data),
    ];
    // Falha de render falha o job (ticket F1-07): um documento vazio ou sem
    // título é bug, não conteúdo.
    for document in &documents {
        if !document.content.starts_with("# ") || document.content.trim().len() < 40 {
            return Err(AppError::Config(format!(
                "deliverable {} was rendered empty",
                document.file_name
            )));
        }
    }
    Ok(DeliverableSet {
        incident_id: data.incident.id.clone(),
        incident_name: data.incident.name.clone(),
        documents,
        redaction: data.redaction.clone(),
        personal_data_evidence: data
            .evidence
            .iter()
            .filter(|section| section.evidence.contains_personal_data)
            .count(),
    })
}

/// Renderiza aplicando as opções: com a máscara ligada, os dados pessoais
/// são trocados por marcadores antes de qualquer documento ser escrito.
pub fn render_with(data: &DeliverableData, options: RenderOptions) -> AppResult<DeliverableSet> {
    if options.mask_personal_data {
        let (masked, _summary) = redaction::redact(data);
        render_all(&masked)
    } else {
        render_all(data)
    }
}

// ---------------------------------------------------------------------
// Coleta
// ---------------------------------------------------------------------

fn referenced_evidence_ids(
    attack_mappings: &[AnalyticalResultResponse],
    critical_path: &CriticalPathView,
    impact: &Option<AnalyticalResultResponse>,
    iocs: &[AnalyticalResultResponse],
    recommendations: &[AnalyticalResultResponse],
    findings: &FindingsView,
) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    let all = attack_mappings
        .iter()
        .chain(critical_path.stages.iter())
        .chain(impact.iter())
        .chain(iocs.iter())
        .chain(recommendations.iter())
        .chain(findings.findings.iter())
        .chain(findings.root_causes.iter());
    for result in all {
        if let Some(id) = &result.evidence_id {
            ids.insert(id.clone());
        }
    }
    for ioc in iocs {
        if let Some(sightings) = ioc.payload.get("sightings").and_then(|s| s.as_array()) {
            for sighting in sightings {
                if let Some(id) = sighting.get("evidence_id").and_then(|v| v.as_str()) {
                    ids.insert(id.to_string());
                }
            }
        }
    }
    ids
}

async fn timeline_state(services: &AppServices, evidence_id: &str) -> TimelineState {
    let job_id = match super::latest_timeline_job(services, evidence_id).await {
        Ok(Some(job_id)) => job_id,
        Ok(None) => return TimelineState::NoCompletedJob,
        Err(_) => return TimelineState::Unavailable,
    };
    let Ok(first_page) = super::timeline_page(services, &job_id, 0, 1, None, true).await else {
        return TimelineState::Unavailable;
    };
    let total = first_page.total.unwrap_or(0);
    let first_event = first_page.events.first().map(|e| e.timestamp.clone());
    let last_event = if total > 1 {
        match super::timeline_page(services, &job_id, total - 1, 1, None, false).await {
            Ok(page) => page.events.first().map(|e| e.timestamp.clone()),
            Err(_) => return TimelineState::Unavailable,
        }
    } else {
        first_event.clone()
    };
    TimelineState::Summary(TimelineSummary {
        total_events: total,
        first_event,
        last_event,
        warning_count: first_page.job.warning_count,
    })
}

async fn custody_entries(
    services: &AppServices,
    evidence_id: &str,
) -> AppResult<Vec<CustodyEntry>> {
    let rows = sqlx::query_as::<_, (i64, String, String, String, String)>(
        "SELECT sequence, action, actor, timestamp, reason FROM chain_of_custody WHERE evidence_id=? ORDER BY sequence",
    )
    .bind(evidence_id)
    .fetch_all(&services.db)
    .await?;
    Ok(rows
        .into_iter()
        .map(
            |(sequence, action, actor, timestamp, reason)| CustodyEntry {
                sequence,
                action,
                actor,
                timestamp,
                reason,
            },
        )
        .collect())
}

/// Lê do banco tudo que os documentos usam. Não grava nada.
pub async fn collect(services: &AppServices, incident_id: &str) -> AppResult<DeliverableData> {
    let incident = super::list_incidents(services)
        .await?
        .into_iter()
        .find(|incident| incident.id == incident_id)
        .ok_or_else(|| AppError::NotFound("incident not found".into()))?;

    let attack_mappings = findings::list_attack_mappings(services, incident_id).await?;
    let critical_path = findings::critical_path(services, incident_id).await?;
    let impact = findings::current_impact_assessment(services, incident_id).await?;
    let iocs = findings::list_iocs(services, incident_id).await?;
    let recommendations = findings::list_recommendations(services, incident_id, None).await?;
    let findings_view = findings::findings_view(services, incident_id).await?;
    let plan_view = plan::plan_view(services, incident_id).await?;
    let nist_view = nist::function_view(services, incident_id).await?;
    let gaps = nist::current_gap_analysis(services, incident_id).await?;
    let source_gaps = gaps::list_source_gaps(services, incident_id).await?;

    let referenced = referenced_evidence_ids(
        &attack_mappings,
        &critical_path,
        &impact,
        &iocs,
        &recommendations,
        &findings_view,
    );
    let mut evidence = Vec::new();
    let mut other_evidence = Vec::new();
    for (item, destroyed) in super::list_evidence_including_destroyed(services).await? {
        if referenced.contains(&item.id) {
            let timeline = timeline_state(services, &item.id).await;
            let custody = custody_entries(services, &item.id).await?;
            let destroyed = custody.iter().any(|entry| entry.action == "DESTROYED");
            let detection = detection_summary(services, &item, destroyed).await?;
            let custody_chain_intact = audit::verify_custody(&services.db, &item.id).await.ok();
            evidence.push(EvidenceSection {
                evidence: item,
                timeline,
                detection,
                custody,
                custody_chain_intact,
            });
        } else if !destroyed {
            other_evidence.push(item);
        }
    }
    evidence.sort_by(|a, b| {
        a.evidence
            .imported_at
            .cmp(&b.evidence.imported_at)
            .then_with(|| a.evidence.id.cmp(&b.evidence.id))
    });
    other_evidence.sort_by(|a, b| {
        a.imported_at
            .cmp(&b.imported_at)
            .then_with(|| a.id.cmp(&b.id))
    });

    Ok(DeliverableData {
        incident,
        evidence,
        other_evidence,
        attack_mappings,
        critical_path,
        impact,
        iocs,
        recommendations,
        findings: findings_view,
        plan: plan_view,
        nist: nist_view,
        gaps,
        source_gaps,
        redaction: None,
    })
}

/// Resumo da detecção por regras de uma evidência citada.
async fn detection_summary(
    services: &AppServices,
    evidence: &EvidenceResponse,
    destroyed: bool,
) -> AppResult<DetectionSummary> {
    if destroyed {
        return Ok(DetectionSummary::Discarded);
    }
    if evidence.format != "evtx" {
        return Ok(DetectionSummary::NotApplicable);
    }
    let counts = crate::persistence::detection::counts(&services.db, &evidence.id).await?;
    if counts.total() == 0 {
        // Sem candidatos pode ser "não rodou" ou "rodou e nada disparou": só o
        // job concluído distingue os dois.
        let ran: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM analysis_jobs WHERE evidence_id=? AND kind='EVTX_DETECTION' AND status='COMPLETED'",
        )
        .bind(&evidence.id)
        .fetch_one(&services.db)
        .await?;
        return Ok(if ran > 0 {
            DetectionSummary::Run {
                confirmed: 0,
                rejected: 0,
                pending: 0,
                rule_sets: Vec::new(),
            }
        } else {
            DetectionSummary::NotRun
        });
    }
    let rule_sets: Vec<String> = sqlx::query_scalar(
        "SELECT DISTINCT rule_set_version FROM detection_candidates WHERE evidence_id=? ORDER BY rule_set_version",
    )
    .bind(&evidence.id)
    .fetch_all(&services.db)
    .await?;
    Ok(DetectionSummary::Run {
        confirmed: counts.confirmed,
        rejected: counts.rejected,
        pending: counts.pending,
        rule_sets,
    })
}

/// Pré-visualização: coleta e renderiza sem gravar nada.
pub async fn preview(
    services: &AppServices,
    incident_id: &str,
    options: RenderOptions,
) -> AppResult<DeliverableSet> {
    let data = collect(services, incident_id).await?;
    render_with(&data, options)
}

// ---------------------------------------------------------------------
// Gravação
// ---------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
struct ManifestDocument {
    file: String,
    title: String,
    bytes: u64,
    sha256: String,
    blake3: String,
    source_result_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct Manifest {
    schema: &'static str,
    incident_id: String,
    incident_name: String,
    generated_at: String,
    tool: &'static str,
    tool_version: &'static str,
    scope: &'static str,
    /// Só contagens do que foi mascarado, nunca os valores.
    redaction: ManifestRedaction,
    /// Evidências citadas que o operador marcou como contendo dado pessoal.
    personal_data_evidence: usize,
    documents: Vec<ManifestDocument>,
}

#[derive(Debug, Clone, Serialize)]
struct ManifestRedaction {
    masked: bool,
    ips: usize,
    emails: usize,
    accounts: usize,
    persons: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct GeneratedFile {
    pub file_name: String,
    pub bytes: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct GeneratedDeliverables {
    pub output_dir: PathBuf,
    pub generated_at: String,
    pub files: Vec<GeneratedFile>,
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// O id vira parte do nome da pasta: só caracteres seguros.
fn validate_incident_id(incident_id: &str) -> AppResult<()> {
    let safe = !incident_id.is_empty()
        && incident_id.len() <= 64
        && incident_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if safe {
        Ok(())
    } else {
        Err(AppError::Config(
            "incident id has characters that are not allowed in a folder name".into(),
        ))
    }
}

/// Pasta de saída: `<saída>/entregaveis-<id do incidente>`. Recusa links
/// simbólicos e qualquer pasta dentro do armazenamento interno da aplicação
/// (evidências, banco, quarentena...).
fn prepare_output_dir(
    out_dir: &Path,
    incident_id: &str,
    paths: &StoragePaths,
) -> AppResult<PathBuf> {
    validate_incident_id(incident_id)?;
    std::fs::create_dir_all(out_dir)?;
    let base = out_dir.canonicalize()?;
    for protected in [
        &paths.db,
        &paths.runtime,
        &paths.evidence,
        &paths.staging,
        &paths.sanitized,
        &paths.quarantine,
        &paths.logs,
        &paths.backups,
    ] {
        if let Ok(protected) = protected.canonicalize()
            && base.starts_with(&protected)
        {
            return Err(AppError::Config(
                "the output folder must be outside the application's internal storage".into(),
            ));
        }
    }
    let target = base.join(format!("entregaveis-{incident_id}"));
    match std::fs::symlink_metadata(&target) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            return Err(AppError::Config(
                "the deliverables folder is a symbolic link; refusing to write through it".into(),
            ));
        }
        Ok(metadata) if !metadata.is_dir() => {
            return Err(AppError::Config(
                "the deliverables folder path is occupied by a file".into(),
            ));
        }
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            std::fs::create_dir_all(&target)?;
        }
        Err(error) => return Err(error.into()),
    }
    Ok(target)
}

/// Grava o arquivo inteiro ou nada: escreve num temporário e renomeia.
fn write_atomically(dir: &Path, file_name: &str, bytes: &[u8]) -> AppResult<()> {
    let temp = dir.join(format!(".{file_name}.tmp"));
    let final_path = dir.join(file_name);
    if let Err(error) =
        std::fs::write(&temp, bytes).and_then(|()| std::fs::rename(&temp, &final_path))
    {
        // F1-23: registrar a falha de limpeza do temporário em vez de
        // engolir; o erro original continua sendo devolvido.
        if let Err(cleanup_error) = std::fs::remove_file(&temp)
            && cleanup_error.kind() != std::io::ErrorKind::NotFound
        {
            tracing::warn!(%cleanup_error, "failed to remove temporary deliverable file");
        }
        return Err(error.into());
    }
    Ok(())
}

/// Grava os documentos e o manifest. `generated_at` entra só no manifest.
pub fn write_set(
    set: &DeliverableSet,
    out_dir: &Path,
    paths: &StoragePaths,
    generated_at: &str,
) -> AppResult<GeneratedDeliverables> {
    let target = prepare_output_dir(out_dir, &set.incident_id, paths)?;
    let mut files = Vec::new();
    let mut manifest_documents = Vec::new();
    for document in &set.documents {
        let bytes = document.content.as_bytes();
        write_atomically(&target, document.file_name, bytes)?;
        let sha256 = sha256_hex(bytes);
        manifest_documents.push(ManifestDocument {
            file: document.file_name.to_string(),
            title: document.title.to_string(),
            bytes: bytes.len() as u64,
            sha256: sha256.clone(),
            blake3: blake3::hash(bytes).to_hex().to_string(),
            source_result_ids: document.source_result_ids.clone(),
        });
        files.push(GeneratedFile {
            file_name: document.file_name.to_string(),
            bytes: bytes.len() as u64,
            sha256,
        });
    }
    let manifest = Manifest {
        schema: "dfir-deliverables/1",
        incident_id: set.incident_id.clone(),
        incident_name: set.incident_name.clone(),
        generated_at: generated_at.to_string(),
        tool: "dfir-app",
        tool_version: env!("CARGO_PKG_VERSION"),
        scope: "Análise automática apenas de logs EVTX; demais fontes só por registro manual do analista.",
        redaction: match &set.redaction {
            Some(summary) => ManifestRedaction {
                masked: true,
                ips: summary.ips,
                emails: summary.emails,
                accounts: summary.accounts,
                persons: summary.persons,
            },
            None => ManifestRedaction {
                masked: false,
                ips: 0,
                emails: 0,
                accounts: 0,
                persons: 0,
            },
        },
        personal_data_evidence: set.personal_data_evidence,
        documents: manifest_documents,
    };
    let manifest_bytes = serde_json::to_vec_pretty(&manifest)
        .map_err(|error| AppError::Config(format!("could not serialize the manifest: {error}")))?;
    write_atomically(&target, FILE_MANIFEST, &manifest_bytes)?;
    files.push(GeneratedFile {
        file_name: FILE_MANIFEST.to_string(),
        bytes: manifest_bytes.len() as u64,
        sha256: sha256_hex(&manifest_bytes),
    });
    Ok(GeneratedDeliverables {
        output_dir: target,
        generated_at: generated_at.to_string(),
        files,
    })
}

/// Gera os cinco documentos e o manifest em `out_dir`, e registra o evento
/// na trilha de auditoria (só nomes de arquivo e hashes, sem caminhos).
pub async fn generate(
    services: &AppServices,
    incident_id: &str,
    out_dir: &Path,
    options: RenderOptions,
) -> AppResult<GeneratedDeliverables> {
    let data = collect(services, incident_id).await?;
    let set = render_with(&data, options)?;
    let generated_at = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let generated = write_set(&set, out_dir, &services.paths, &generated_at)?;

    let metadata = serde_json::json!({
        "redacted": set.redaction.is_some(),
        "personal_data_evidence": set.personal_data_evidence,
        "files": generated
            .files
            .iter()
            .map(|file| serde_json::json!({"file": file.file_name, "sha256": file.sha256}))
            .collect::<Vec<_>>(),
    })
    .to_string();
    audit::append(
        &services.db,
        None,
        "DELIVERABLES_GENERATED",
        "incident",
        incident_id,
        Some(incident_id),
        &metadata,
    )
    .await?;
    tracing::info!(
        incident_id,
        files = generated.files.len(),
        "deliverables generated"
    );
    Ok(generated)
}

#[cfg(test)]
mod tests {
    use super::markdown::esc;
    use super::*;
    use crate::persistence::findings::risk_matrix;
    use crate::persistence::nist_functions::{GapAnalysisState, summarize_by_function};
    use crate::persistence::plan_items::{PlanDomain, domain_coverage};
    use crate::service::findings::CriticalPathEdge;
    use crate::service::nist::{NistFunctionView, ProfileSummary};
    use crate::service::test_support::fixture;

    const HOSTILE: &str = "x|y\n<script>alert(1)</script> [l](javascript:alert(1)) `c` \u{1b}[31m";

    fn result(
        id: &str,
        result_type: &str,
        confidence: Option<&str>,
        evidence_id: Option<&str>,
        payload: serde_json::Value,
    ) -> AnalyticalResultResponse {
        AnalyticalResultResponse {
            id: id.into(),
            result_type: result_type.into(),
            correlation_key: format!("key-{id}"),
            evidence_id: evidence_id.map(str::to_string),
            incident_id: Some("inc-00000001".into()),
            analysis_job_id: None,
            rule_or_model_id: None,
            rule_or_model_version: None,
            confidence: confidence.map(str::to_string),
            payload,
            provenance: serde_json::Value::Null,
            supersedes: None,
            created_at: "2026-10-14T00:00:00Z".into(),
        }
    }

    fn empty_data() -> DeliverableData {
        DeliverableData {
            incident: IncidentResponse {
                id: "inc-00000001".into(),
                name: "Incidente de teste".into(),
                state: "OPEN".into(),
                created_at: "2026-10-14T00:00:00Z".into(),
            },
            evidence: vec![],
            other_evidence: vec![],
            attack_mappings: vec![],
            critical_path: CriticalPathView {
                stages: vec![],
                edges: vec![],
            },
            impact: None,
            iocs: vec![],
            recommendations: vec![],
            findings: FindingsView {
                findings: vec![],
                root_causes: vec![],
                attack_mappings: vec![],
                risk_matrix: vec![],
            },
            plan: PlanView {
                items: vec![],
                findings: vec![],
                coverage: domain_coverage(&[]),
            },
            nist: NistFunctionView {
                current_profile: None,
                target_profile: None,
                functions: summarize_by_function(&[], &[], &[]),
                gap_state: GapAnalysisState::MissingProfile,
            },
            gaps: vec![],
            source_gaps: vec![],
            redaction: None,
        }
    }

    fn evidence(id: &str, filename: &str) -> EvidenceResponse {
        EvidenceResponse {
            id: id.into(),
            original_filename: filename.into(),
            storage_reference: format!("evidence/{id}"),
            format: "evtx".into(),
            size_bytes: 4096,
            blake3: "b3".repeat(32),
            sha256: "5a".repeat(32),
            imported_at: "2026-10-01T10:00:00Z".into(),
            status: "FORENSIC".into(),
            source: "NOT_PROVIDED".into(),
            provided_by: None,
            collection_method: "NOT_PROVIDED".into(),
            collection_tool: None,
            collection_tool_version: None,
            collected_at: None,
            authorization_reference: None,
            contains_personal_data: false,
            incident_id: None,
        }
    }

    fn sample_data() -> DeliverableData {
        let mut data = empty_data();
        data.evidence = vec![EvidenceSection {
            evidence: evidence("ev-00000001", "Security.evtx"),
            timeline: TimelineState::Summary(TimelineSummary {
                total_events: 1200,
                first_event: Some("2026-09-30T22:00:00Z".into()),
                last_event: Some("2026-10-01T03:00:00Z".into()),
                warning_count: 2,
            }),
            custody: vec![CustodyEntry {
                sequence: 1,
                action: "IMPORTED".into(),
                actor: "operador".into(),
                timestamp: "2026-10-01T10:00:00Z".into(),
                reason: "recebido do cliente".into(),
            }],
            custody_chain_intact: Some(true),
            detection: DetectionSummary::Run {
                confirmed: 2,
                rejected: 1,
                pending: 3,
                rule_sets: vec!["2026.10.1".into()],
            },
        }];
        let mut pcap = evidence("ev-00000003", "captura.pcap");
        pcap.format = "pcap".into();
        pcap.imported_at = "2026-10-02T10:00:00Z".into();
        pcap.source = "sensor de rede SN01".into();
        pcap.provided_by = Some("Equipe de redes".into());
        pcap.collection_method = "captura com tcpdump".into();
        pcap.collection_tool = Some("tcpdump".into());
        pcap.collected_at = Some("2026-09-30".into());
        pcap.authorization_reference = Some("Ofício 123/2026".into());
        pcap.contains_personal_data = true;
        data.evidence.push(EvidenceSection {
            evidence: pcap,
            timeline: TimelineState::NoCompletedJob,
            detection: DetectionSummary::NotApplicable,
            custody: vec![],
            custody_chain_intact: Some(true),
        });
        data.source_gaps = vec![result(
            "sg-00000001",
            "SOURCE_GAP",
            None,
            None,
            serde_json::json!({"expected_source": "logs do firewall de borda",
                "period_from": "2026-09-01", "period_to": null,
                "reason": "RETENTION", "note": "retenção de 7 dias"}),
        )];
        data.other_evidence = vec![evidence("ev-00000002", "System.evtx")];
        data.attack_mappings = vec![
            result(
                "am-00000001",
                "ATTACK_MAPPING",
                Some("MEDIUM"),
                Some("ev-00000001"),
                serde_json::json!({"tactic": "INITIAL_ACCESS", "technique_id": "T1133",
                    "sub_technique_id": "", "basis": "logon tipo 10 de IP externo",
                    "evidence_status": "DOCUMENTED"}),
            ),
            result(
                "am-00000002",
                "ATTACK_MAPPING",
                Some("LOW"),
                None,
                serde_json::json!({"tactic": "LATERAL_MOVEMENT", "technique_id": "T1021",
                    "sub_technique_id": "T1021.001", "basis": "",
                    "evidence_status": "GAP"}),
            ),
            result(
                "am-00000003",
                "ATTACK_MAPPING",
                Some("MEDIUM"),
                None,
                serde_json::json!({"tactic": "COLLECTION", "technique_id": "T1005",
                    "sub_technique_id": "", "basis": "cópia de arquivos",
                    "evidence_status": "DOCUMENTED"}),
            ),
        ];
        data.critical_path = CriticalPathView {
            stages: vec![
                result(
                    "st-00000002",
                    "CRITICAL_PATH_LINK",
                    Some("MEDIUM"),
                    None,
                    serde_json::json!({"stage_label": "Movimento lateral", "sequence_index": 2,
                        "basis": "RDP interno", "evidence_status": "DOCUMENTED"}),
                ),
                result(
                    "st-00000001",
                    "CRITICAL_PATH_LINK",
                    Some("MEDIUM"),
                    None,
                    serde_json::json!({"stage_label": "Acesso inicial", "sequence_index": 1,
                        "basis": "RDP exposto", "evidence_status": "DOCUMENTED"}),
                ),
            ],
            edges: vec![CriticalPathEdge {
                from: "st-00000001".into(),
                to: "st-00000002".into(),
            }],
        };
        data.impact = Some(result(
            "im-00000001",
            "IMPACT_ASSESSMENT",
            Some("MEDIUM"),
            None,
            serde_json::json!({"confidentiality_impact": "HIGH", "integrity_impact": "LOW",
                "availability_impact": "NONE", "affected_assets": ["DB01", "FS02"],
                "affected_data": "dados de clientes", "operational_disruption": "parcial",
                "financial_impact": null}),
        ));
        data.iocs = vec![result(
            "io-00000001",
            "IOC",
            Some("HIGH"),
            Some("ev-00000001"),
            serde_json::json!({"ioc_type": "IPV4", "normalized_value": "203.0.113.7",
                "first_seen": "2026-09-30T22:01:00Z", "last_seen": "2026-10-01T02:00:00Z",
                "sighting_count": 12}),
        )];
        data.recommendations = vec![
            result(
                "rc-00000001",
                "RECOMMENDATION",
                None,
                None,
                serde_json::json!({"category": "IMMEDIATE", "title": "Fechar o RDP",
                    "description": "Bloquear 3389 na borda",
                    "supporting_result_ids": ["fd-00000001"]}),
            ),
            result(
                "rc-00000002",
                "RECOMMENDATION",
                None,
                None,
                serde_json::json!({"category": "STRATEGIC", "title": "Segmentar a rede",
                    "description": "", "supporting_result_ids": []}),
            ),
        ];
        let findings = vec![
            result(
                "fd-00000001",
                "FINDING",
                Some("HIGH"),
                Some("ev-00000001"),
                serde_json::json!({"title": "RDP exposto sem MFA", "severity": "CRITICAL",
                    "finding_kind": "MISSING_CONTROL", "description": "porta 3389 aberta",
                    "basis": "EVTX 4625 em massa", "reference": "CWE-308",
                    "nist_subcategories": ["PR.AA-03"],
                    "attack_mapping_ids": ["am-00000001"]}),
            ),
            result(
                "fd-00000002",
                "FINDING",
                Some("LOW"),
                None,
                serde_json::json!({"title": "Backup sem teste", "severity": "LOW",
                    "finding_kind": "OTHER", "description": "", "basis": "entrevista",
                    "reference": "", "nist_subcategories": [], "attack_mapping_ids": []}),
            ),
        ];
        data.findings = FindingsView {
            risk_matrix: risk_matrix(&[
                (
                    crate::persistence::attack_impact::ImpactLevel::Critical,
                    crate::persistence::analytical_results::Confidence::High,
                ),
                (
                    crate::persistence::attack_impact::ImpactLevel::Low,
                    crate::persistence::analytical_results::Confidence::Low,
                ),
            ]),
            findings,
            root_causes: vec![result(
                "rt-00000001",
                "ROOT_CAUSE",
                Some("MEDIUM"),
                None,
                serde_json::json!({"level": "PROBABLE_CAUSE",
                    "statement": "Acesso inicial por RDP exposto sem MFA",
                    "basis": "sequência de logons", "gaps": ["sem logs do firewall"],
                    "gap_status": "DECLARED", "finding_ids": ["fd-00000001"]}),
            )],
            attack_mappings: vec![],
        };
        let items = vec![
            result(
                "pl-00000001",
                "PLAN_ITEM",
                None,
                None,
                serde_json::json!({"domain": "AUTENTICACAO_ACESSO", "title": "MFA no acesso remoto",
                    "rationale": "acesso inicial sem segundo fator", "priority": "P1",
                    "effort": "S", "cost_class": "NONE", "phase": "IMMEDIATE",
                    "precondition": "RDP acessível", "postcondition": "RDP só via VPN com MFA",
                    "verification": "varredura externa sem 3389",
                    "framework_reference": "CIS 6.5", "regulatory_reference": "Política de acesso remoto",
                    "nist_subcategories": ["PR.AA-03"], "finding_ids": ["fd-00000001"],
                    "depends_on": [], "support_kind": "FINDING_BACKED", "crypto": null}),
            ),
            result(
                "pl-00000002",
                "PLAN_ITEM",
                None,
                None,
                serde_json::json!({"domain": "OS_HARDENING", "title": "Aplicar baseline CIS",
                    "rationale": "reduzir superfície", "priority": "P2", "effort": "M",
                    "cost_class": "LOW", "phase": "SHORT_TERM", "precondition": "",
                    "postcondition": "", "verification": "auditoria CIS-CAT",
                    "framework_reference": "CIS Benchmarks", "regulatory_reference": null,
                    "nist_subcategories": [], "finding_ids": [],
                    "depends_on": ["pl-00000001"], "support_kind": "GENERAL_RECOMMENDATION",
                    "crypto": null}),
            ),
            result(
                "pl-00000003",
                "PLAN_ITEM",
                None,
                None,
                serde_json::json!({"domain": "CRIPTO_REPOUSO", "title": "Cifrar o volume do banco",
                    "rationale": "dados pessoais em claro", "priority": "P2", "effort": "M",
                    "cost_class": "LOW", "phase": "SHORT_TERM", "precondition": "",
                    "postcondition": "", "verification": "volume reporta criptografia ativa",
                    "framework_reference": null, "regulatory_reference": "LGPD art. 46",
                    "nist_subcategories": [], "finding_ids": [], "depends_on": [],
                    "support_kind": "GENERAL_RECOMMENDATION",
                    "crypto": {"data_class": "Dados pessoais", "location": "DB01",
                        "mechanism": "AES-256", "rotation": "anual", "recovery": "cofre"}}),
            ),
        ];
        data.plan = PlanView {
            coverage: domain_coverage(&[
                (PlanDomain::AccessAuthentication, false),
                (PlanDomain::OsHardening, true),
                (PlanDomain::CryptoAtRest, true),
            ]),
            findings: data.findings.findings.clone(),
            items,
        };
        data.nist = NistFunctionView {
            current_profile: Some(ProfileSummary {
                version_label: "c1".into(),
                tier: Some("TIER_1_PARTIAL".into()),
                tier_justification: Some("governança informal".into()),
                entries: 3,
            }),
            target_profile: None,
            functions: summarize_by_function(&[], &[], &[("PR.AA-03", "HIGH")]),
            gap_state: GapAnalysisState::UpToDate,
        };
        data.gaps = vec![result(
            "gp-00000001",
            "NIST_GAP",
            Some("MEDIUM"),
            None,
            serde_json::json!({"subcategory_id": "PR.AA-03", "subcategory_label": "Autenticação",
                "current_status": "NOT_IMPLEMENTED", "target_status": "IMPLEMENTED",
                "priority": "HIGH"}),
        )];
        data
    }

    fn doc<'a>(set: &'a DeliverableSet, file: &str) -> &'a RenderedDocument {
        set.documents
            .iter()
            .find(|document| document.file_name == file)
            .expect("document exists")
    }

    /// O texto fora dos trechos de código (`` `...` ``). Dentro de um trecho
    /// o Markdown trata tudo como literal; o que se exige de `code()` é que
    /// o trecho não possa ser encerrado antes da hora e não quebre tabela.
    fn outside_code_spans(content: &str) -> String {
        let mut out = String::new();
        let mut in_span = false;
        let mut chars = content.chars();
        while let Some(ch) = chars.next() {
            match ch {
                '\\' if !in_span => {
                    out.push(ch);
                    if let Some(next) = chars.next() {
                        out.push(next);
                    }
                }
                '`' => in_span = !in_span,
                _ if !in_span => out.push(ch),
                _ => {}
            }
        }
        out
    }

    fn unescaped_pipes(line: &str) -> usize {
        let mut count = 0;
        let mut escaped = false;
        for ch in line.chars() {
            match ch {
                '\\' if !escaped => {
                    escaped = true;
                    continue;
                }
                '|' if !escaped => count += 1,
                _ => {}
            }
            escaped = false;
        }
        count
    }

    /// Cada bloco de linhas de tabela tem a mesma largura em todas as linhas.
    fn assert_tables_are_well_formed(content: &str) {
        let mut width: Option<usize> = None;
        for line in content.lines() {
            if line.starts_with('|') {
                let pipes = unescaped_pipes(line);
                match width {
                    None => width = Some(pipes),
                    Some(expected) => assert_eq!(pipes, expected, "linha desalinhada: {line}"),
                }
            } else {
                width = None;
            }
        }
    }

    #[test]
    fn the_report_states_what_the_automatic_detection_did_and_what_it_left_unreviewed() {
        let set = render_all(&sample_data()).expect("render");
        let e1 = &doc(&set, FILE_E1).content;
        assert!(e1.contains("4.3 Detecção automática por regras (EVTX)"));
        // Security.evtx: 2 confirmados, 1 rejeitado, 3 pendentes, 6 no total.
        let row = e1
            .lines()
            .find(|line| line.contains("Security.evtx") && line.contains("2026.10.1"))
            .expect("linha da evidência com o conjunto de regras");
        let cells: Vec<&str> = row.split('|').map(str::trim).collect();
        assert_eq!(&cells[3..7], ["2", "1", "3", "6"], "{row}");
        // O que não foi avaliado não vira achado, e o limite do método é dito.
        assert!(e1.contains("Os **pendentes** não foram avaliados e não constam como achado"));
        assert!(e1.contains("A ausência de candidato não prova ausência de ataque"));
        // Evidência que não é EVTX não finge ter sido avaliada.
        assert!(e1.contains("não se aplica (só EVTX)"));
    }

    #[test]
    fn a_discarded_evidence_is_reported_as_discarded_not_as_having_no_candidates() {
        let mut data = sample_data();
        data.evidence[0].detection = DetectionSummary::Discarded;
        let set = render_all(&data).expect("render");
        let e1 = &doc(&set, FILE_E1).content;
        assert!(e1.contains("evidência descartada; os candidatos saíram com ela"));
        assert!(!e1.contains("executadas; nenhum candidato"));
        // A seção não nega a evidência citada.
        assert!(!e1.contains("nenhuma evidência citada"));
    }

    #[test]
    fn an_evtx_that_never_had_the_rules_run_says_so_instead_of_implying_a_clean_result() {
        let mut data = sample_data();
        data.evidence[0].detection = DetectionSummary::NotRun;
        let set = render_all(&data).expect("render");
        let e1 = &doc(&set, FILE_E1).content;
        assert!(e1.contains("regras não executadas"));
        data.evidence[0].detection = DetectionSummary::Run {
            confirmed: 0,
            rejected: 0,
            pending: 0,
            rule_sets: Vec::new(),
        };
        let set = render_all(&data).expect("render");
        assert!(
            doc(&set, FILE_E1)
                .content
                .contains("executadas; nenhum candidato")
        );
    }

    #[test]
    fn five_documents_render_with_the_expected_files_titles_and_scope_statement() {
        let set = render_all(&sample_data()).expect("render");
        let files: Vec<&str> = set.documents.iter().map(|d| d.file_name).collect();
        assert_eq!(files, vec![FILE_E1, FILE_E2, FILE_E3, FILE_E4, FILE_E5]);
        for document in &set.documents {
            assert!(document.content.starts_with("# "), "{}", document.file_name);
            assert!(document.content.ends_with('\n'));
            assert_tables_are_well_formed(&document.content);
        }
        let e1 = &doc(&set, FILE_E1).content;
        assert!(e1.contains("somente logs EVTX do Windows"));
        assert!(e1.contains("Causa registrada: Causa provável"));
        assert!(e1.contains("Lacuna: sem logs do firewall"));
        assert!(e1.contains("Security.evtx"));
        assert!(e1.contains("Evidências importadas e não citadas por este incidente: System.evtx"));
        assert!(e1.contains("análise concluída com 2 aviso(s)"));
        assert!(e1.contains("RDP exposto") && e1.contains("Movimento lateral"));
        // Ordem dos marcos: 1 antes de 2, apesar da ordem de entrada.
        assert!(e1.find("Acesso inicial").unwrap() < e1.find("Movimento lateral").unwrap());
        // Mapeamento sem base é dito como tal.
        assert!(e1.contains("sem base documentada"));
        assert!(e1.contains("Impacto financeiro: não informado"));
        // Proveniência e lacunas (F1-31): o que não foi informado é dito.
        assert!(e1.contains("Proveniência e tratamento"));
        assert!(e1.contains("2.1 Lacunas de fonte declaradas"));
        assert!(e1.contains("logs do firewall de borda"));
        assert!(e1.contains("retenção insuficiente de logs"));
        assert!(e1.contains("a partir de `2026-09-01`"));
        assert!(e1.contains("Lacunas de fonte declaradas pelo analista: 1."));
        assert!(e1.contains("Security.evtx: proveniência não informada (origem, quem forneceu, método de coleta, data da coleta, autorização)"));
        assert!(e1.contains("captura.pcap: o formato `pcap` é apenas preservado"));
        assert!(e1.contains("captura.pcap: marcada pelo operador como contendo dado pessoal"));
        assert!(
            e1.contains("somente preservada") && e1.contains("analisada automaticamente (EVTX)")
        );
        assert!(e1.contains("sensor de rede SN01") && e1.contains("Ofício 123/2026"));
        assert!(
            !e1.contains("captura.pcap: proveniência não informada"),
            "a proveniência do pcap foi declarada"
        );
        assert!(e1.contains("Este documento **não** mascara dados pessoais"));
        // `affected_assets` é uma lista no payload: tem de aparecer, não ficar vazio.
        assert!(e1.contains("Ativos afetados: DB01, FS02"));
        let e2 = &doc(&set, FILE_E2).content;
        assert!(
            e2.contains("RDP exposto sem MFA") && e2.contains("CWE\\-308")
                || e2.contains("CWE-308")
        );
        assert!(e2.contains("Severidade × Confiança"));
        assert!(e2.contains("Não há pontuação nem média"));
        let e3 = &doc(&set, FILE_E3).content;
        assert!(e3.contains("não existe um Tier por Function"));
        assert!(e3.contains("Fechar o RDP") && e3.contains("Segmentar a rede"));
        assert!(e3.contains("Target: ainda não informado"));
        let e4 = &doc(&set, FILE_E4).content;
        assert!(e4.contains("Recomendação geral"));
        assert!(e4.contains("Depende de: MFA no acesso remoto"));
        assert!(e4.contains("Motivado pelo achado: RDP exposto sem MFA"));
        let e5 = &doc(&set, FILE_E5).content;
        assert!(e5.contains("AES\\-256") || e5.contains("AES-256"));
        // A classe de dado só tem item em repouso: trânsito e chaves são lacuna.
        assert!(e5.contains("— (sem item)"));
        for document in &set.documents {
            for forbidden in ["score oficial de", "pontuação de risco:", "nota final"] {
                assert!(!document.content.contains(forbidden));
            }
        }
    }

    #[test]
    fn an_empty_incident_renders_explicit_no_record_sections_and_claims_nothing() {
        let set = render_all(&empty_data()).expect("render");
        let e1 = &doc(&set, FILE_E1).content;
        assert!(e1.contains("Sem registro"));
        assert!(e1.contains("não afirma causa-raiz"));
        assert!(
            e1.contains("Achados (vulnerabilidades ou condições exploradas): nenhum registrado")
        );
        assert!(e1.contains("Impacto: não registrado"));
        assert!(e1.contains(
            "nenhuma lacuna de fonte declarada pelo analista; isso não prova que não houve lacunas"
        ));
        let e2 = &doc(&set, FILE_E2).content;
        assert!(e2.contains("Sem registro: nenhum achado registrado"));
        let e3 = &doc(&set, FILE_E3).content;
        assert!(e3.contains("Current: ainda não informado"));
        assert!(e3.contains("Falta o profile Current ou o Target"));
        // Os cinco domínios do Entregável 4 e os três do 5 aparecem, vazios.
        let e4 = &doc(&set, FILE_E4).content;
        assert_eq!(
            e4.matches("nenhum item de plano registrado para esta ação")
                .count(),
            5
        );
        let e5 = &doc(&set, FILE_E5).content;
        assert_eq!(
            e5.matches("nenhum item de plano registrado para esta ação")
                .count(),
            3
        );
        for document in &set.documents {
            assert!(
                document.source_result_ids.is_empty(),
                "{}",
                document.file_name
            );
        }
    }

    #[test]
    fn rendering_is_deterministic_and_independent_of_input_order() {
        let data = sample_data();
        let first = render_all(&data).expect("render");
        let second = render_all(&data).expect("render");
        let mut shuffled = data.clone();
        shuffled.attack_mappings.reverse();
        shuffled.critical_path.stages.reverse();
        shuffled.critical_path.edges.reverse();
        shuffled.iocs.reverse();
        shuffled.recommendations.reverse();
        shuffled.findings.findings.reverse();
        shuffled.plan.items.reverse();
        shuffled.gaps.reverse();
        let reordered = render_all(&shuffled).expect("render");
        for index in 0..5 {
            assert_eq!(
                first.documents[index].content,
                second.documents[index].content
            );
            assert_eq!(
                first.documents[index].content, reordered.documents[index].content,
                "{} depende da ordem de entrada",
                first.documents[index].file_name
            );
        }
    }

    #[test]
    fn hostile_text_cannot_inject_markup_or_break_tables() {
        let mut data = sample_data();
        data.incident.name = HOSTILE.into();
        data.evidence[0].evidence.original_filename = HOSTILE.into();
        data.evidence[0].custody[0].reason = HOSTILE.into();
        data.evidence[0].custody[0].actor = HOSTILE.into();
        data.iocs[0].payload["normalized_value"] = HOSTILE.into();
        data.findings.findings[0].payload["title"] = HOSTILE.into();
        data.findings.findings[0].payload["basis"] = HOSTILE.into();
        data.findings.findings[0].payload["description"] = HOSTILE.into();
        data.critical_path.stages[0].payload["stage_label"] = HOSTILE.into();
        data.plan.items[0].payload["title"] = HOSTILE.into();
        data.plan.items[0].payload["rationale"] = HOSTILE.into();
        data.plan.items[2].payload["crypto"]["mechanism"] = HOSTILE.into();
        data.recommendations[0].payload["description"] = HOSTILE.into();
        data.findings.root_causes[0].payload["statement"] = HOSTILE.into();

        let set = render_all(&data).expect("render");
        for document in &set.documents {
            let content = &document.content;
            let prose = outside_code_spans(content);
            assert!(!prose.contains("<script>"), "{}", document.file_name);
            // Colchete escapado (`\\]`, `\\[`) não forma link; só o cru formaria.
            let unescaped = prose.replace("\\]", "").replace("\\[", "");
            assert!(
                !unescaped.contains("](javascript"),
                "{}",
                document.file_name
            );
            assert!(!unescaped.contains("[l]("), "{}", document.file_name);
            assert!(!content.contains('\u{1b}'), "{}", document.file_name);
            assert_tables_are_well_formed(content);
        }
        // O texto hostil aparece neutralizado, não removido.
        let e1 = &doc(&set, FILE_E1).content;
        assert!(e1.contains(&esc(HOSTILE)));
    }

    #[test]
    fn cited_results_are_listed_with_their_full_ids_per_document() {
        let set = render_all(&sample_data()).expect("render");
        let e1_sources = &doc(&set, FILE_E1).source_result_ids;
        for id in [
            "st-00000001",
            "im-00000001",
            "rt-00000001",
            "am-00000001",
            "io-00000001",
        ] {
            assert!(e1_sources.contains(&id.to_string()), "{id} em E1");
        }
        let e2_sources = &doc(&set, FILE_E2).source_result_ids;
        assert!(e2_sources.contains(&"fd-00000001".to_string()));
        let e4_sources = &doc(&set, FILE_E4).source_result_ids;
        assert!(e4_sources.contains(&"pl-00000001".to_string()));
        assert!(
            !e4_sources.contains(&"pl-00000003".to_string()),
            "o item de criptografia fica só no Entregável 5"
        );
        let mut sorted = e1_sources.clone();
        sorted.sort();
        assert_eq!(&sorted, e1_sources, "ids em ordem estável");
    }

    /// Dados de exemplo com dados pessoais plantados em vários lugares.
    fn pii_data() -> DeliverableData {
        let mut data = sample_data();
        data.incident.name = "Incidente do cliente joao.silva@empresa.com.br".into();
        data.evidence[0].evidence.provided_by = Some("Maria Souza".into());
        data.evidence[0].custody[0].actor = "carlos.analista".into();
        data.evidence[0].custody[0].reason = "recebido de maria.souza@empresa.com.br".into();
        data.findings.findings[0].payload["basis"] =
            "logon de joao.silva@empresa.com.br a partir de 198.51.100.4 pela conta CORP\\joao"
                .into();
        data.plan.findings = data.findings.findings.clone();
        data.plan.items[0].payload["rationale"] = "acesso inicial a partir de 2001:db8::7".into();
        data.source_gaps[0].payload["note"] = "pedido a joao.silva@empresa.com.br".into();
        data
    }

    const PII_ORIGINALS: [&str; 7] = [
        "joao.silva@empresa.com.br",
        "maria.souza@empresa.com.br",
        "198.51.100.4",
        "203.0.113.7",
        "2001:db8::7",
        "Maria Souza",
        "carlos.analista",
    ];

    #[test]
    fn masked_documents_contain_none_of_the_original_personal_data_and_say_what_was_masked() {
        let data = pii_data();
        let masked = render_with(&data, RenderOptions::default()).expect("masked render");
        for document in &masked.documents {
            for original in PII_ORIGINALS {
                assert!(
                    !document.content.contains(original),
                    "{original} em {}",
                    document.file_name
                );
            }
            assert!(!document.content.contains("joao"), "{}", document.file_name);
            assert!(
                !document.content.contains("Souza"),
                "{}",
                document.file_name
            );
            assert_tables_are_well_formed(&document.content);
        }
        let summary = masked.redaction.clone().expect("summary");
        assert_eq!(summary.emails, 2);
        assert_eq!(summary.ips, 3);
        assert_eq!(summary.accounts, 1);
        // Maria Souza, carlos.analista e "Equipe de redes" (quem forneceu o pcap).
        assert_eq!(summary.persons, 3);

        let e1 = &doc(&masked, FILE_E1).content;
        assert!(
            e1.contains(
                "Dados pessoais mascarados: 3 IP(s), 2 e-mail(s), 1 conta(s) e 3 pessoa(s)"
            )
        );
        assert!(!e1.contains("**não** mascara"));
        // O mesmo valor recebe o mesmo marcador em documentos diferentes.
        let e2 = &doc(&masked, FILE_E2).content;
        assert!(e1.contains("{IP-2}") || e1.contains("{IP-1}"));
        // O e-mail do nome do incidente aparece nos dois; a conta, só na base
        // do achado (Entregável 2).
        assert!(e1.contains("{EMAIL-1}") && e2.contains("{EMAIL-1}"));
        assert!(e2.contains("{CONTA-1}") && !e1.contains("{CONTA-1}"));
        // Texto que não é dado pessoal fica intacto.
        assert!(e1.contains("RDP exposto") && e1.contains("2026-09-30T22:00:00Z"));
        assert!(e1.contains("Security.evtx"));

        // Sem a máscara os valores aparecem e o documento avisa.
        let plain = render_with(
            &data,
            RenderOptions {
                mask_personal_data: false,
            },
        )
        .expect("plain render");
        assert!(plain.redaction.is_none());
        let plain_text: String = plain.documents.iter().map(|d| d.content.as_str()).collect();
        for original in [
            "joao.silva@empresa.com.br",
            "198.51.100.4",
            "Maria Souza",
            "203.0.113.7",
        ] {
            assert!(plain_text.contains(original), "{original}");
        }
        assert!(doc(&plain, FILE_E1).content.contains("**não** mascara"));
        // A máscara não altera os dados de entrada.
        assert_eq!(
            data.incident.name,
            "Incidente do cliente joao.silva@empresa.com.br"
        );
    }

    #[test]
    fn masking_is_deterministic_and_independent_of_input_order() {
        let data = pii_data();
        let first = render_with(&data, RenderOptions::default()).expect("render");
        let second = render_with(&data, RenderOptions::default()).expect("render");
        let mut shuffled = data.clone();
        shuffled.attack_mappings.reverse();
        shuffled.iocs.reverse();
        shuffled.findings.findings.reverse();
        shuffled.plan.items.reverse();
        shuffled.evidence.reverse();
        shuffled.source_gaps.reverse();
        let reordered = render_with(&shuffled, RenderOptions::default()).expect("render");
        for index in 0..5 {
            assert_eq!(
                first.documents[index].content,
                second.documents[index].content
            );
            assert_eq!(
                first.documents[index].content, reordered.documents[index].content,
                "{} depende da ordem de entrada",
                first.documents[index].file_name
            );
        }
        assert_eq!(first.redaction, reordered.redaction);
    }

    #[tokio::test]
    async fn collect_rejects_malformed_plan_arrays_before_rendering() {
        let fx = fixture().await;
        for (key, bad_value, expected) in [
            (
                "finding_ids",
                serde_json::json!("not-an-array"),
                "stored plan item finding_ids is not an array",
            ),
            (
                "nist_subcategories",
                serde_json::json!([42]),
                "stored plan item nist_subcategories contains a non-string id",
            ),
            (
                "depends_on",
                serde_json::json!({}),
                "stored plan item depends_on is not an array",
            ),
        ] {
            let incident =
                crate::service::create_incident(&fx.services, &format!("malformed {key}"))
                    .await
                    .expect("incident");
            let finding = crate::service::findings::create_finding(
                &fx.services,
                &incident.id,
                &crate::service::findings::FindingInput {
                    kind: "MISSING_CONTROL".into(),
                    title: "RDP sem MFA".into(),
                    severity: "HIGH".into(),
                    basis: "EVTX 4625".into(),
                    confidence: "MEDIUM".into(),
                    ..crate::service::findings::FindingInput::default()
                },
            )
            .await
            .expect("finding");
            let item = crate::service::plan::create_plan_item(
                &fx.services,
                &incident.id,
                &crate::service::plan::PlanItemInput {
                    domain: "AUTENTICACAO_ACESSO".into(),
                    title: "MFA no acesso remoto".into(),
                    rationale: "motivo".into(),
                    priority: "P1".into(),
                    effort: "M".into(),
                    cost_class: "LOW".into(),
                    phase: "SHORT_TERM".into(),
                    verification: "checagem objetiva".into(),
                    finding_ids: vec![finding],
                    nist_subcategory_ids: vec!["PR.AA-03".into()],
                    ..crate::service::plan::PlanItemInput::default()
                },
            )
            .await
            .expect("valid plan item");
            let record = crate::persistence::analytical_results::get(&fx.services.db, &item)
                .await
                .expect("get valid plan item")
                .expect("valid plan item exists");
            let mut payload: serde_json::Value =
                serde_json::from_str(&record.payload).expect("valid plan payload");
            payload[key] = bad_value;
            let bad_id = format!("bad-plan-array-{key}");
            sqlx::query("INSERT INTO analytical_results(id,schema_version,result_type,correlation_key,incident_id,payload,provenance,supersedes,created_at) VALUES(?,'analytical-result-v1','PLAN_ITEM',?,?,?,?,?,'2026-10-03T00:00:00Z')")
                .bind(&bad_id)
                .bind(&record.correlation_key)
                .bind(&incident.id)
                .bind(payload.to_string())
                .bind(record.provenance.to_string())
                .bind(&item)
                .execute(&fx.services.db)
                .await
                .expect("malformed current plan item");

            let error = match collect(&fx.services, &incident.id).await {
                Ok(data) => {
                    let rendered = render_all(&data).expect("pre-fix render");
                    let false_general = doc(&rendered, FILE_E4)
                        .content
                        .contains("**Recomendação geral**");
                    panic!(
                        "malformed {key} reached Entregável 4; false general recommendation={false_general}"
                    );
                }
                Err(error) => error,
            };
            assert!(
                matches!(&error, AppError::Config(message) if message.contains(expected)),
                "unexpected error for {key}: {error:?}"
            );
        }
    }

    #[tokio::test]
    async fn collect_rejects_malformed_finding_risk_fields() {
        let fx = fixture().await;
        for (label, severity, confidence, expected) in [
            (
                "severity-unknown",
                serde_json::json!("SEVERE"),
                Some("MEDIUM"),
                "invalid severity",
            ),
            (
                "severity-none",
                serde_json::json!("NONE"),
                Some("MEDIUM"),
                "invalid severity",
            ),
            (
                "confidence-missing",
                serde_json::json!("HIGH"),
                None,
                "invalid confidence",
            ),
        ] {
            let incident = crate::service::create_incident(&fx.services, label)
                .await
                .expect("incident");
            let finding = crate::service::findings::create_finding(
                &fx.services,
                &incident.id,
                &crate::service::findings::FindingInput {
                    kind: "MISSING_CONTROL".into(),
                    title: "RDP sem MFA".into(),
                    severity: "HIGH".into(),
                    basis: "EVTX 4625".into(),
                    confidence: "MEDIUM".into(),
                    ..crate::service::findings::FindingInput::default()
                },
            )
            .await
            .expect("valid finding");
            let record = crate::persistence::analytical_results::get(&fx.services.db, &finding)
                .await
                .expect("get valid finding")
                .expect("valid finding exists");
            let mut payload: serde_json::Value =
                serde_json::from_str(&record.payload).expect("valid finding payload");
            payload["severity"] = severity;
            let bad_id = format!("bad-finding-risk-{label}");
            sqlx::query("INSERT INTO analytical_results(id,schema_version,result_type,correlation_key,incident_id,confidence,payload,provenance,supersedes,created_at) VALUES(?,'analytical-result-v1','FINDING',?,?,?,?,?,?,'2026-10-03T00:00:00Z')")
                .bind(&bad_id)
                .bind(&record.correlation_key)
                .bind(&incident.id)
                .bind(confidence)
                .bind(payload.to_string())
                .bind(record.provenance.to_string())
                .bind(&finding)
                .execute(&fx.services.db)
                .await
                .expect("malformed current finding");

            let error = match collect(&fx.services, &incident.id).await {
                Ok(data) => {
                    let finding_count = data.findings.findings.len();
                    let risk_cells = data.findings.risk_matrix.len();
                    let rendered = render_all(&data).expect("pre-fix render");
                    let zero_critical_row = doc(&rendered, FILE_E2)
                        .content
                        .contains("| `CRITICAL` | 0 | 0 | 0 | 0 |");
                    panic!(
                        "malformed {label} reached Entregável 2; findings={finding_count} risk_cells={risk_cells} zero_critical_row={zero_critical_row}"
                    );
                }
                Err(error) => error,
            };
            assert!(
                matches!(&error, AppError::Config(message) if message.contains(expected) && message.contains(&bad_id)),
                "unexpected error for {label}: {error:?}"
            );
        }
    }

    #[test]
    fn the_manifest_records_what_was_masked_by_count_and_never_by_value() {
        let (_root, paths) = storage_for_tests();
        let out = tempfile::tempdir().expect("output directory");
        let masked = render_with(&pii_data(), RenderOptions::default()).expect("render");
        let written =
            write_set(&masked, out.path(), &paths, "2026-10-16T12:00:00Z").expect("write");
        let manifest_text =
            std::fs::read_to_string(written.output_dir.join(FILE_MANIFEST)).expect("manifest");
        let manifest: serde_json::Value = serde_json::from_str(&manifest_text).expect("json");
        assert_eq!(manifest["redaction"]["masked"], true);
        assert_eq!(manifest["redaction"]["emails"], 2);
        assert_eq!(manifest["redaction"]["ips"], 3);
        assert_eq!(manifest["personal_data_evidence"], 1);
        for original in PII_ORIGINALS {
            assert!(!manifest_text.contains(original), "{original} no manifest");
        }
        assert!(
            !manifest_text.contains("joao"),
            "o nome do incidente também é mascarado"
        );

        let plain = render_with(
            &pii_data(),
            RenderOptions {
                mask_personal_data: false,
            },
        )
        .expect("render");
        let out2 = tempfile::tempdir().expect("output directory");
        let written = write_set(&plain, out2.path(), &paths, "t").expect("write");
        let manifest: serde_json::Value = serde_json::from_slice(
            &std::fs::read(written.output_dir.join(FILE_MANIFEST)).expect("manifest"),
        )
        .expect("json");
        assert_eq!(manifest["redaction"]["masked"], false);
    }

    fn storage_for_tests() -> (tempfile::TempDir, StoragePaths) {
        let root = tempfile::tempdir().expect("temporary directory");
        let paths = StoragePaths::from_root(root.path());
        paths.ensure().expect("storage layout");
        (root, paths)
    }

    #[test]
    fn write_set_writes_the_six_files_with_matching_hashes_and_is_idempotent() {
        let (_root, paths) = storage_for_tests();
        let out = tempfile::tempdir().expect("output directory");
        let set = render_all(&sample_data()).expect("render");

        let first = write_set(&set, out.path(), &paths, "2026-10-14T12:00:00Z").expect("write");
        let names: Vec<&str> = first.files.iter().map(|f| f.file_name.as_str()).collect();
        assert_eq!(
            names,
            vec![FILE_E1, FILE_E2, FILE_E3, FILE_E4, FILE_E5, FILE_MANIFEST]
        );
        assert!(first.output_dir.ends_with("entregaveis-inc-00000001"));
        for file in &first.files {
            let bytes = std::fs::read(first.output_dir.join(&file.file_name)).expect("file exists");
            assert_eq!(bytes.len() as u64, file.bytes);
            assert_eq!(sha256_hex(&bytes), file.sha256, "{}", file.file_name);
        }
        let manifest: serde_json::Value = serde_json::from_slice(
            &std::fs::read(first.output_dir.join(FILE_MANIFEST)).expect("manifest"),
        )
        .expect("manifest json");
        assert_eq!(manifest["schema"], "dfir-deliverables/1");
        assert_eq!(manifest["incident_id"], "inc-00000001");
        assert_eq!(manifest["generated_at"], "2026-10-14T12:00:00Z");
        assert_eq!(manifest["documents"].as_array().map(Vec::len), Some(5));
        assert!(
            manifest["documents"][0]["source_result_ids"]
                .as_array()
                .is_some_and(|ids| !ids.is_empty())
        );
        assert_eq!(
            manifest["documents"][0]["blake3"].as_str().map(str::len),
            Some(64)
        );

        // Regravar com os mesmos dados mantém os hashes dos documentos; só o
        // manifest (que carrega `generated_at`) muda.
        let second = write_set(&set, out.path(), &paths, "2026-10-14T13:00:00Z").expect("rewrite");
        for index in 0..5 {
            assert_eq!(first.files[index].sha256, second.files[index].sha256);
        }
        assert_ne!(first.files[5].sha256, second.files[5].sha256);
        let leftovers: Vec<String> = std::fs::read_dir(&first.output_dir)
            .expect("read dir")
            .filter_map(Result::ok)
            .map(|entry| entry.file_name().to_string_lossy().to_string())
            .filter(|name| name.ends_with(".tmp"))
            .collect();
        assert!(leftovers.is_empty(), "temporários sobrando: {leftovers:?}");
    }

    #[test]
    fn write_set_refuses_internal_storage_unsafe_ids_and_symbolic_links() {
        let (_root, paths) = storage_for_tests();
        let set = render_all(&sample_data()).expect("render");

        // Dentro do armazenamento interno (evidências).
        let inside = paths.evidence.join("saida");
        assert!(write_set(&set, &inside, &paths, "t").is_err());

        // Id do incidente que viraria caminho.
        let mut unsafe_set = set.clone();
        unsafe_set.incident_id = "../fora".into();
        let out = tempfile::tempdir().expect("output directory");
        assert!(write_set(&unsafe_set, out.path(), &paths, "t").is_err());
        unsafe_set.incident_id = "a/b".into();
        assert!(write_set(&unsafe_set, out.path(), &paths, "t").is_err());
        assert!(
            std::fs::read_dir(out.path())
                .expect("read dir")
                .next()
                .is_none()
        );

        // A pasta de entregáveis é um link simbólico: não se escreve através dele.
        #[cfg(unix)]
        {
            let elsewhere = tempfile::tempdir().expect("elsewhere");
            std::os::unix::fs::symlink(
                elsewhere.path(),
                out.path().join("entregaveis-inc-00000001"),
            )
            .expect("symlink");
            assert!(write_set(&set, out.path(), &paths, "t").is_err());
            assert!(
                std::fs::read_dir(elsewhere.path())
                    .expect("read dir")
                    .next()
                    .is_none(),
                "nada foi escrito através do link"
            );
        }

        // O caminho da pasta ocupado por um arquivo.
        let file_out = tempfile::tempdir().expect("output directory");
        std::fs::write(file_out.path().join("entregaveis-inc-00000001"), b"x").expect("file");
        assert!(write_set(&set, file_out.path(), &paths, "t").is_err());
    }

    #[tokio::test]
    async fn generate_reads_the_database_writes_the_files_and_audits_without_paths() {
        let fx = fixture().await;
        let incident = crate::service::create_incident(&fx.services, "Incidente real")
            .await
            .expect("incident");
        let finding = crate::service::findings::create_finding(
            &fx.services,
            &incident.id,
            &crate::service::findings::FindingInput {
                kind: "MISSING_CONTROL".into(),
                title: "RDP sem MFA".into(),
                severity: "HIGH".into(),
                basis: "EVTX 4625".into(),
                confidence: "MEDIUM".into(),
                ..Default::default()
            },
        )
        .await
        .expect("finding");
        crate::service::plan::create_plan_item(
            &fx.services,
            &incident.id,
            &crate::service::plan::PlanItemInput {
                domain: "AUTENTICACAO_ACESSO".into(),
                title: "MFA no acesso remoto".into(),
                rationale: "acesso inicial sem segundo fator".into(),
                priority: "P1".into(),
                effort: "S".into(),
                cost_class: "NONE".into(),
                phase: "IMMEDIATE".into(),
                verification: "varredura externa".into(),
                finding_ids: vec![finding.clone()],
                ..Default::default()
            },
        )
        .await
        .expect("plan item");

        // A pré-visualização não grava nada.
        let before = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM audit_events WHERE action = 'DELIVERABLES_GENERATED'",
        )
        .fetch_one(&fx.services.db)
        .await
        .expect("count");
        let preview = preview(&fx.services, &incident.id, RenderOptions::default())
            .await
            .expect("preview");
        assert_eq!(preview.documents.len(), 5);
        assert!(doc(&preview, FILE_E2).content.contains("RDP sem MFA"));
        assert!(
            doc(&preview, FILE_E4)
                .content
                .contains("MFA no acesso remoto")
        );
        let still = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM audit_events WHERE action = 'DELIVERABLES_GENERATED'",
        )
        .fetch_one(&fx.services.db)
        .await
        .expect("count");
        assert_eq!(before, still);

        let out = tempfile::tempdir().expect("output directory");
        let generated = generate(
            &fx.services,
            &incident.id,
            out.path(),
            RenderOptions::default(),
        )
        .await
        .expect("generate");
        assert_eq!(generated.files.len(), 6);
        let e4 = std::fs::read_to_string(generated.output_dir.join(FILE_E4)).expect("e4");
        assert!(e4.contains("MFA no acesso remoto"));

        let metadata = sqlx::query_scalar::<_, String>(
            "SELECT metadata FROM audit_events WHERE action = 'DELIVERABLES_GENERATED' ORDER BY sequence DESC LIMIT 1",
        )
        .fetch_one(&fx.services.db)
        .await
        .expect("audit metadata");
        assert!(metadata.contains(&generated.files[0].sha256));
        assert!(
            !metadata.contains(&out.path().display().to_string()),
            "a auditoria guarda nomes e hashes, não caminhos"
        );

        // Com e sem máscara: o conteúdo muda e a auditoria registra a escolha.
        let with_pii = crate::service::findings::create_finding(
            &fx.services,
            &incident.id,
            &crate::service::findings::FindingInput {
                kind: "OTHER".into(),
                title: "Conta comprometida".into(),
                severity: "HIGH".into(),
                basis: "logon de joao.silva@empresa.com.br de 198.51.100.4".into(),
                confidence: "MEDIUM".into(),
                ..Default::default()
            },
        )
        .await;
        assert!(with_pii.is_ok());
        let masked_out = tempfile::tempdir().expect("output directory");
        let masked = generate(
            &fx.services,
            &incident.id,
            masked_out.path(),
            RenderOptions::default(),
        )
        .await
        .expect("masked");
        let e2_masked = std::fs::read_to_string(masked.output_dir.join(FILE_E2)).expect("e2");
        assert!(!e2_masked.contains("joao.silva") && !e2_masked.contains("198.51.100.4"));
        assert!(e2_masked.contains("{EMAIL-1}") && e2_masked.contains("{IP-1}"));
        let plain_out = tempfile::tempdir().expect("output directory");
        let plain = generate(
            &fx.services,
            &incident.id,
            plain_out.path(),
            RenderOptions {
                mask_personal_data: false,
            },
        )
        .await
        .expect("plain");
        let e2_plain = std::fs::read_to_string(plain.output_dir.join(FILE_E2)).expect("e2");
        assert!(
            e2_plain.contains("joao.silva@empresa.com.br") && e2_plain.contains("198.51.100.4")
        );
        let redacted_flags: Vec<i64> = sqlx::query_scalar(
            "SELECT json_extract(metadata, '$.redacted') FROM audit_events WHERE action = 'DELIVERABLES_GENERATED' ORDER BY sequence",
        )
        .fetch_all(&fx.services.db)
        .await
        .expect("audit flags");
        assert_eq!(
            redacted_flags,
            vec![1, 1, 0],
            "gerado: padrão, máscara, sem máscara"
        );

        // Incidente inexistente.
        assert!(collect(&fx.services, "nao-existe").await.is_err());
    }
}
