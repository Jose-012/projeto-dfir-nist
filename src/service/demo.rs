//! F1-11 (Etapa 7, ADR-0023) — incidente de demonstração: o cenário BEC da
//! seção 42 da especificação, registrado pelas mesmas funções de serviço que a
//! interface usa (nenhuma regra é contornada).
//!
//! ## O que é real e o que é sintético
//!
//! - **Real:** o arquivo `.evtx` que o operador informa é importado de
//!   verdade (hash, custódia, proveniência) e analisado de verdade pelo
//!   engine (timeline e YARA-X). Nada dele é simulado.
//! - **Sintético:** tudo o que um *analista* registraria — mapeamentos
//!   ATT&CK, caminho crítico, impacto, IOCs, achados, causas, plano,
//!   recomendações, lacunas de fonte e profiles NIST — vem deste arquivo. O
//!   cenário **não decorre do `.evtx`** (a amostra pública é um log comum,
//!   sem ataque) e cada texto diz isso. Os IOCs usam só valores de
//!   documentação (RFC 5737 e TLD `.example`), sem relação com alvos reais.
//!
//! O rótulo `DEMO` (seção 21.3) aparece na classe da evidência (`status =
//! DEMO`), no nome do incidente — portanto no título dos cinco documentos —,
//! na proveniência, na base de cada registro e na auditoria
//! (`DEMO_DATASET_SEEDED`).
//!
//! O `.evtx` **não** é distribuído com o projeto: o operador o informa
//! (`dfir-app demo <arquivo.evtx>`; origem e hash em `tests/fixtures/README.md`).
//!
//! Determinismo: o conteúdo é constante. O que muda entre execuções são os
//! ids (UUID) e os instantes de registro, que ficam fora do texto dos
//! documentos, exceto onde o aplicativo já os mostra (custódia).

use super::findings::{FindingInput, RootCauseInput};
use super::gaps::SourceGapInput;
use super::nist::ProfileEntryInput;
use super::plan::{CryptoInput, PlanItemInput};
use super::provenance::EvidenceProvenance;
use super::{
    AppServices, DATA_CLASS_DEMO, analyze, create_incident, findings, gaps,
    import_evidence_classified, link_evidence_to_incident, nist, plan,
};
use crate::error::AppResult;
use crate::persistence::audit;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::path::Path;

/// O nome começa com `[DEMO]`: tudo que começa assim é demonstração.
pub const DEMO_INCIDENT_NAME: &str = "[DEMO] Phishing/BEC — cenário sintético (dados fictícios)";
/// Versão do conteúdo; muda quando o texto abaixo muda.
pub const DEMO_DATASET_VERSION: &str = "bec-demo-1";

const SYNTHETIC: &str = "DEMO (sintético)";

/// A matriz de proteção do Entregável 5 agrupa por este texto: os itens do
/// mesmo dado o repetem igual para fechar a linha da matriz.
const FINANCIAL_DATA: &str = "Dados financeiros e cadastrais de fornecedores";

/// Contagens do que foi registrado.
#[derive(Debug, Clone, Default, Serialize)]
pub struct DemoCounts {
    pub attack_mappings: usize,
    pub critical_path_stages: usize,
    pub iocs: usize,
    pub findings: usize,
    pub root_causes: usize,
    pub recommendations: usize,
    pub plan_items: usize,
    pub source_gaps: usize,
    pub nist_gaps: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct DemoSeedReport {
    pub dataset_version: &'static str,
    pub incident_id: String,
    pub incident_name: String,
    pub evidence_id: String,
    /// Jobs de análise reais executados sobre o `.evtx` (vazio quando a
    /// análise foi dispensada, só nos testes unitários, que não têm engine).
    pub analysis_job_ids: Vec<String>,
    pub counts: DemoCounts,
}

fn own(items: &[&str]) -> Vec<String> {
    items.iter().map(|item| (*item).to_string()).collect()
}

fn pick(ids: &[String], positions: &[usize]) -> Vec<String> {
    positions
        .iter()
        .map(|&position| ids[position].clone())
        .collect()
}

/// Hash de um texto fixo: um IOC `SHA256` que não corresponde a nenhum
/// arquivo real.
fn synthetic_sha256() -> String {
    let digest =
        Sha256::digest(b"DEMO sintetico: arquivo inexistente usado so como exemplo de IOC");
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Importa o `.evtx`, executa as análises reais (se pedido) e registra o
/// cenário sintético num incidente novo. Cada chamada cria um incidente
/// novo e importa a evidência de novo.
pub async fn seed(
    services: &AppServices,
    evtx: &Path,
    max_upload_bytes: u64,
    run_analysis: bool,
) -> AppResult<DemoSeedReport> {
    // 1. Evidência real, com proveniência declarada como amostra pública.
    let evidence = import_evidence_classified(
        services,
        evtx,
        max_upload_bytes,
        &EvidenceProvenance {
            source: Some(
                "DEMO — amostra pública de teste (security.evtx, repositório da crate evtx)".into(),
            ),
            provided_by: Some("Ana Exemplo (pessoa fictícia, DEMO)".into()),
            collection_method: Some(
                "cópia de arquivo de amostra; não é coleta forense de um sistema".into(),
            ),
            collection_tool: None,
            collection_tool_version: None,
            collected_at: None,
            authorization_reference: Some(
                "DEMO — amostra pública; não há autorização de coleta a registrar".into(),
            ),
            contains_personal_data: false,
        },
        DATA_CLASS_DEMO,
    )
    .await?;

    // 2. Análises reais do engine sobre a evidência.
    let mut analysis_job_ids = Vec::new();
    if run_analysis {
        for kind in [
            dfir_protocol::ANALYSIS_KIND_EVTX_TIMELINE,
            dfir_protocol::ANALYSIS_KIND_YARA_SCAN,
            // As regras de detecção rodam de verdade sobre o EVTX real. Os
            // candidatos ficam PENDENTES: o cenário BEC da demonstração segue
            // sendo registrado à mão e rotulado DEMO; confirmar ou rejeitar os
            // candidatos é o passo do analista (e do e2e).
            dfir_protocol::ANALYSIS_KIND_EVTX_DETECTION,
        ] {
            let done = analyze(services, &evidence.id, kind, None, None).await?;
            analysis_job_ids.push(done.job_id);
        }
    }

    // 3. Incidente e cenário sintético.
    let incident = create_incident(services, DEMO_INCIDENT_NAME).await?;
    let incident_id = incident.id.as_str();
    let mut counts = DemoCounts::default();

    // F1-19: a evidência importada é vinculada ao incidente da demonstração.
    link_evidence_to_incident(services, &evidence.id, incident_id).await?;

    seed_nist_profiles(services, incident_id).await?;
    counts.nist_gaps = nist::run_gap_analysis(services, incident_id).await?.len();

    let mappings = seed_attack_mappings(services, incident_id).await?;
    counts.attack_mappings = mappings.len();
    counts.critical_path_stages = seed_critical_path(services, incident_id).await?;
    seed_impact(services, incident_id).await?;
    counts.iocs = seed_iocs(services, incident_id, &evidence.id).await?;
    let source_gaps = seed_source_gaps(services, incident_id).await?;
    counts.source_gaps = source_gaps.len();

    let finding_ids = seed_findings(services, incident_id, &mappings, &source_gaps).await?;
    counts.findings = finding_ids.len();
    counts.root_causes = seed_root_causes(services, incident_id, &finding_ids).await?;
    counts.recommendations = seed_recommendations(services, incident_id, &finding_ids).await?;
    counts.plan_items = seed_plan(services, incident_id, &finding_ids).await?;

    // 4. A origem sintética fica na cadeia de auditoria.
    let metadata = serde_json::json!({
        "dataset_version": DEMO_DATASET_VERSION,
        "evidence_id": &evidence.id,
        "analysis_run": run_analysis,
        "counts": &counts,
    })
    .to_string();
    audit::append(
        &services.db,
        None,
        "DEMO_DATASET_SEEDED",
        "incident",
        incident_id,
        Some(incident_id),
        &metadata,
    )
    .await?;
    tracing::info!(
        incident_id,
        version = DEMO_DATASET_VERSION,
        "demo incident seeded"
    );

    Ok(DemoSeedReport {
        dataset_version: DEMO_DATASET_VERSION,
        incident_id: incident.id.clone(),
        incident_name: incident.name,
        evidence_id: evidence.id,
        analysis_job_ids,
        counts,
    })
}

// ---------------------------------------------------------------------
// Profiles NIST CSF 2.0
// ---------------------------------------------------------------------

fn entry(id: &str, label: &str, status: &str, why: &str) -> ProfileEntryInput {
    ProfileEntryInput {
        subcategory_id: id.into(),
        subcategory_label: label.into(),
        implementation_status: status.into(),
        justification: Some(format!("{SYNTHETIC}: {why}")),
        evidence_reference: None,
        confidence: Some("MEDIUM".into()),
    }
}

async fn seed_nist_profiles(services: &AppServices, incident_id: &str) -> AppResult<()> {
    let current = [
        entry(
            "ID.AM-01",
            "Inventário de hardware mantido",
            "IMPLEMENTED",
            "inventário de estações e servidores atualizado.",
        ),
        entry(
            "PR.AA-03",
            "Usuários, serviços e hardware são autenticados",
            "NOT_IMPLEMENTED",
            "e-mail e acesso remoto aceitam apenas senha.",
        ),
        entry(
            "PR.AA-05",
            "Permissões de acesso seguem menor privilégio",
            "PARTIALLY_IMPLEMENTED",
            "perfis definidos, mas RDP liberado entre estações e servidor.",
        ),
        entry(
            "PR.AT-01",
            "Pessoal recebe conscientização e treinamento",
            "PARTIALLY_IMPLEMENTED",
            "treinamento de admissão, sem reciclagem periódica.",
        ),
        entry(
            "PR.DS-01",
            "Dados em repouso são protegidos",
            "NOT_IMPLEMENTED",
            "diretório financeiro sem criptografia gerida.",
        ),
        entry(
            "PR.DS-02",
            "Dados em trânsito são protegidos",
            "PARTIALLY_IMPLEMENTED",
            "TLS externo presente; tráfego interno sem criptografia.",
        ),
        entry(
            "PR.PS-04",
            "Registros de log são gerados e disponibilizados",
            "PARTIALLY_IMPLEMENTED",
            "logs gerados, com retenção de 30 dias e sem centralização.",
        ),
        entry(
            "DE.CM-01",
            "Redes e serviços de rede são monitorados",
            "NOT_IMPLEMENTED",
            "sem monitoramento de tráfego interno.",
        ),
        entry(
            "DE.CM-03",
            "Atividade de pessoal e uso de tecnologia são monitorados",
            "NOT_IMPLEMENTED",
            "sem alerta para login externo ou regras de caixa.",
        ),
        entry(
            "DE.AE-02",
            "Eventos adversos são analisados",
            "NOT_ASSESSED",
            "capacidade de análise ainda não avaliada.",
        ),
        entry(
            "RS.MA-01",
            "Plano de resposta é executado quando um incidente é declarado",
            "PARTIALLY_IMPLEMENTED",
            "plano genérico, sem playbook de fraude por e-mail.",
        ),
    ];
    let target = [
        entry(
            "ID.AM-01",
            "Inventário de hardware mantido",
            "IMPLEMENTED",
            "manter.",
        ),
        entry(
            "PR.AA-03",
            "Usuários, serviços e hardware são autenticados",
            "IMPLEMENTED",
            "segundo fator resistente a phishing.",
        ),
        entry(
            "PR.AA-05",
            "Permissões de acesso seguem menor privilégio",
            "IMPLEMENTED",
            "RDP só por gateway administrativo.",
        ),
        entry(
            "PR.AT-01",
            "Pessoal recebe conscientização e treinamento",
            "IMPLEMENTED",
            "treinamento recorrente do financeiro.",
        ),
        entry(
            "PR.DS-01",
            "Dados em repouso são protegidos",
            "IMPLEMENTED",
            "criptografia de volume e do diretório financeiro.",
        ),
        entry(
            "PR.DS-02",
            "Dados em trânsito são protegidos",
            "IMPLEMENTED",
            "TLS e SMB com criptografia no tráfego interno.",
        ),
        entry(
            "PR.PS-04",
            "Registros de log são gerados e disponibilizados",
            "IMPLEMENTED",
            "retenção de 12 meses, coleta central.",
        ),
        entry(
            "DE.CM-01",
            "Redes e serviços de rede são monitorados",
            "PARTIALLY_IMPLEMENTED",
            "orçamento limita a meta ao tráfego do servidor de arquivos.",
        ),
        entry(
            "DE.CM-03",
            "Atividade de pessoal e uso de tecnologia são monitorados",
            "IMPLEMENTED",
            "alertas de login anômalo e de regras de caixa.",
        ),
        entry(
            "DE.AE-02",
            "Eventos adversos são analisados",
            "IMPLEMENTED",
            "triagem diária dos alertas.",
        ),
        entry(
            "RS.MA-01",
            "Plano de resposta é executado quando um incidente é declarado",
            "IMPLEMENTED",
            "playbook de BEC exercitado.",
        ),
    ];
    nist::create_profile(
        services,
        incident_id,
        "CURRENT",
        "demo-atual-1",
        "DEMO — autoavaliação sintética",
        Some("TIER_1_PARTIAL"),
        Some(
            "DEMO (sintético): práticas informais, sem processo repetível de segurança de e-mail.",
        ),
        &current,
    )
    .await?;
    nist::create_profile(
        services,
        incident_id,
        "TARGET",
        "demo-alvo-1",
        "DEMO — meta sintética",
        Some("TIER_3_REPEATABLE"),
        Some("DEMO (sintético): meta realista para o orçamento do cenário."),
        &target,
    )
    .await?;
    Ok(())
}

// ---------------------------------------------------------------------
// ATT&CK e caminho crítico
// ---------------------------------------------------------------------

/// Índices dos mapeamentos, na ordem de criação.
mod map {
    pub const PHISHING: usize = 0;
    pub const VALID_ACCOUNT: usize = 1;
    pub const FORWARDING: usize = 2;
    pub const HIDING: usize = 3;
    pub const POWERSHELL: usize = 4;
    pub const ACCOUNT_MANIPULATION: usize = 5;
    pub const RDP: usize = 6;
    pub const EXFILTRATION: usize = 7;
}

async fn seed_attack_mappings(services: &AppServices, incident_id: &str) -> AppResult<Vec<String>> {
    // (tática, técnica, sub-técnica, base, confiança)
    let rows: [(&str, &str, Option<&str>, &str, &str); 9] = [
        (
            "INITIAL_ACCESS",
            "T1566",
            Some("T1566.002"),
            "e-mail com link para um portal falso de login, recebido na caixa do financeiro",
            "MEDIUM",
        ),
        (
            "INITIAL_ACCESS",
            "T1078",
            Some("T1078.004"),
            "login bem-sucedido na caixa de e-mail a partir de IP externo, sem segundo fator",
            "HIGH",
        ),
        (
            "COLLECTION",
            "T1114",
            Some("T1114.003"),
            "regra de caixa encaminhando mensagens do financeiro para endereço externo",
            "HIGH",
        ),
        (
            "DEFENSE_EVASION",
            "T1564",
            Some("T1564.008"),
            "regra de caixa movendo respostas de fornecedores para uma pasta oculta",
            "MEDIUM",
        ),
        (
            "EXECUTION",
            "T1059",
            Some("T1059.001"),
            "PowerShell executado no servidor de arquivos fora do horário de uso",
            "MEDIUM",
        ),
        (
            "PRIVILEGE_ESCALATION",
            "T1098",
            None,
            "conta de serviço incluída num grupo administrativo durante o incidente",
            "LOW",
        ),
        (
            "LATERAL_MOVEMENT",
            "T1021",
            Some("T1021.001"),
            "sessão RDP da estação DEMO-WS07 para o servidor de arquivos",
            "MEDIUM",
        ),
        (
            "EXFILTRATION",
            "T1567",
            None,
            "tentativa de envio de planilhas do diretório financeiro a um serviço web",
            "LOW",
        ),
        (
            "IMPACT",
            "T1657",
            None,
            "e-mail fraudulento pedindo troca de dados bancários de um fornecedor",
            "MEDIUM",
        ),
    ];
    let mut ids = Vec::with_capacity(rows.len());
    for (tactic, technique, sub, basis, confidence) in rows {
        let id = findings::create_attack_mapping(
            services,
            incident_id,
            None,
            tactic,
            technique,
            sub,
            &format!("{SYNTHETIC}: {basis}. Não decorre do arquivo EVTX de amostra."),
            confidence,
            None,
        )
        .await?;
        ids.push(id);
    }
    Ok(ids)
}

async fn seed_critical_path(services: &AppServices, incident_id: &str) -> AppResult<usize> {
    let stages = [
        ("Phishing: e-mail com link para portal falso", "MEDIUM"),
        (
            "Credencial capturada e login anômalo na caixa do financeiro",
            "HIGH",
        ),
        (
            "Regras de caixa: encaminhamento externo e ocultação de respostas",
            "HIGH",
        ),
        (
            "PowerShell e movimento lateral até o servidor de arquivos",
            "MEDIUM",
        ),
        ("Tentativa de exfiltração de planilhas financeiras", "LOW"),
        ("Pedido fraudulento de troca de dados bancários", "MEDIUM"),
    ];
    let mut previous: Option<String> = None;
    for (position, (label, confidence)) in stages.iter().enumerate() {
        let id = findings::create_critical_path_stage(
            services,
            incident_id,
            None,
            position as i64 + 1,
            label,
            &format!(
                "{SYNTHETIC}: etapa {} da cadeia da seção 42 da especificação.",
                position + 1
            ),
            confidence,
        )
        .await?;
        if let Some(from) = &previous {
            findings::link_critical_path_stages(services, incident_id, from, &id).await?;
        }
        previous = Some(id);
    }
    Ok(stages.len())
}

// ---------------------------------------------------------------------
// Impacto, IOCs e lacunas de fonte
// ---------------------------------------------------------------------

async fn seed_impact(services: &AppServices, incident_id: &str) -> AppResult<()> {
    findings::create_impact_assessment(
        services,
        incident_id,
        "NONE",
        "HIGH",
        "MEDIUM",
        "Caixa de e-mail do financeiro; servidor de arquivos DEMO-FS01; estação DEMO-WS07",
        "Dados cadastrais de fornecedores e instruções de pagamento (fictícios)",
        "Pagamentos a fornecedores suspensos por dois dias úteis para conferência (cenário sintético)",
        Some("Tentativa de desvio de um pagamento a fornecedor; valor fictício, não informado na demonstração"),
        "MEDIUM",
    )
    .await?;
    Ok(())
}

async fn seed_iocs(
    services: &AppServices,
    incident_id: &str,
    evidence_id: &str,
) -> AppResult<usize> {
    let sha256 = synthetic_sha256();
    let rows: [(&str, &str, &str, &str); 6] = [
        (
            "IP",
            "203.0.113.45",
            "IP de origem do login anômalo (faixa TEST-NET-3, reservada para documentação)",
            "2026-03-10T14:32:00Z",
        ),
        (
            "DOMAIN",
            "portal-acesso-demo.example",
            "domínio do portal falso de login (TLD reservado .example)",
            "2026-03-10T09:12:00Z",
        ),
        (
            "URL",
            "https://portal-acesso-demo.example/entrar",
            "link do e-mail de phishing",
            "2026-03-10T09:12:00Z",
        ),
        (
            "EMAIL",
            "cobranca@fornecedor-demo.example",
            "remetente do e-mail fraudulento e destino da regra de encaminhamento",
            "2026-03-10T15:05:00Z",
        ),
        (
            "SHA256",
            sha256.as_str(),
            "hash de um texto fixo, sem arquivo correspondente; não é malware real",
            "2026-03-10T21:40:00Z",
        ),
        (
            "HOSTNAME",
            "DEMO-WS07",
            "estação de origem da sessão RDP",
            "2026-03-10T22:10:00Z",
        ),
    ];
    for (ioc_type, value, context, observed_at) in rows {
        findings::create_ioc(
            services,
            incident_id,
            evidence_id,
            ioc_type,
            value,
            Some(&format!("{SYNTHETIC}: {context}.")),
            "MEDIUM",
            Some(observed_at),
        )
        .await?;
    }
    Ok(rows.len())
}

async fn seed_source_gaps(services: &AppServices, incident_id: &str) -> AppResult<Vec<String>> {
    let rows: [(&str, &str, &str, &str, &str); 3] = [
        (
            "Trilha de mensagens do e-mail corporativo",
            "2026-03-01",
            "2026-03-12",
            "NOT_PROVIDED",
            "o provedor de e-mail ainda não exportou a trilha de mensagens do período",
        ),
        (
            "Logs do proxy web",
            "2026-02-01",
            "2026-03-08",
            "RETENTION",
            "a retenção de 30 dias já sobrescreveu o período anterior ao incidente",
        ),
        (
            "Logs de segurança do controlador de domínio",
            "2026-03-10",
            "2026-03-12",
            "NOT_COLLECTED",
            "a coleta não foi feita antes da reinicialização do servidor",
        ),
    ];
    let mut ids = Vec::with_capacity(rows.len());
    for (source, from, to, reason, note) in rows {
        let id = gaps::create_source_gap(
            services,
            incident_id,
            &SourceGapInput {
                expected_source: source.into(),
                period_from: Some(from.into()),
                period_to: Some(to.into()),
                reason: reason.into(),
                note: format!("{SYNTHETIC}: {note}."),
                supersedes: None,
            },
        )
        .await?;
        ids.push(id);
    }
    Ok(ids)
}

// ---------------------------------------------------------------------
// Achados, causas, recomendações e plano
// ---------------------------------------------------------------------

/// Índices dos achados, na ordem de criação.
mod fnd {
    pub const NO_MFA: usize = 0;
    pub const FORWARDING: usize = 1;
    pub const NO_POWERSHELL_ALERT: usize = 2;
    pub const FLAT_RDP: usize = 3;
    pub const SHORT_RETENTION: usize = 4;
    pub const NO_ENCRYPTION: usize = 5;
    pub const NO_TRAINING: usize = 6;
}

#[allow(clippy::too_many_arguments)]
fn finding(
    kind: &str,
    title: &str,
    description: &str,
    severity: &str,
    basis: &str,
    reference: Option<&str>,
    nist_ids: &[&str],
    attack: Vec<String>,
    supporting: Vec<String>,
    confidence: &str,
) -> FindingInput {
    FindingInput {
        evidence_id: None,
        kind: kind.into(),
        title: title.into(),
        description: description.into(),
        severity: severity.into(),
        basis: format!("{SYNTHETIC}: {basis}"),
        reference: reference.map(str::to_string),
        nist_subcategory_ids: own(nist_ids),
        attack_mapping_ids: attack,
        supporting_result_ids: supporting,
        confidence: confidence.into(),
        analyst_note: None,
        supersedes: None,
    }
}

async fn seed_findings(
    services: &AppServices,
    incident_id: &str,
    m: &[String],
    gap_ids: &[String],
) -> AppResult<Vec<String>> {
    let inputs = [
        finding(
            "MISSING_CONTROL",
            "Caixa do financeiro aceitou login sem segundo fator a partir de IP externo",
            "A autenticação da caixa de e-mail dependia só de senha; uma credencial capturada por phishing bastou para entrar.",
            "HIGH",
            "o login anômalo foi aceito com senha apenas (cenário da seção 42).",
            Some("CWE-308"),
            &["PR.AA-03"],
            pick(m, &[map::VALID_ACCOUNT, map::PHISHING]),
            vec![],
            "HIGH",
        ),
        finding(
            "MISCONFIGURATION",
            "Caixa permitia regra de encaminhamento automático para endereço externo",
            "Qualquer usuário podia criar regras que encaminham e-mail para fora da organização, sem alerta.",
            "HIGH",
            "a regra de encaminhamento foi criada pela sessão anômala e não gerou alerta.",
            None,
            &["PR.AA-05", "DE.CM-03"],
            pick(m, &[map::FORWARDING, map::HIDING]),
            vec![],
            "MEDIUM",
        ),
        finding(
            "MISSING_CONTROL",
            "Sem alerta para execução anômala de PowerShell no servidor de arquivos",
            "A execução fora do horário de uso passou despercebida até a revisão do incidente.",
            "MEDIUM",
            "não há regra de detecção para PowerShell no servidor de arquivos.",
            None,
            &["DE.AE-02", "DE.CM-01"],
            pick(m, &[map::POWERSHELL]),
            vec![],
            "MEDIUM",
        ),
        finding(
            "MISCONFIGURATION",
            "RDP liberado entre estações e o servidor de arquivos, sem segmentação",
            "Uma estação comprometida alcançou o servidor de arquivos por RDP e uma conta de serviço ganhou privilégios.",
            "HIGH",
            "a sessão RDP e a alteração de grupo ocorreram sem barreira de rede.",
            None,
            &["PR.AA-05"],
            pick(m, &[map::RDP, map::ACCOUNT_MANIPULATION]),
            vec![],
            "MEDIUM",
        ),
        finding(
            "MISSING_CONTROL",
            "Retenção de logs de e-mail e proxy insuficiente para reconstruir o período anterior",
            "Não foi possível reconstruir o que aconteceu antes do login anômalo.",
            "MEDIUM",
            "as lacunas de fonte registradas mostram o período sem log.",
            Some("CWE-778"),
            &["PR.PS-04"],
            vec![],
            gap_ids.to_vec(),
            "MEDIUM",
        ),
        finding(
            "MISSING_CONTROL",
            "Dados financeiros sem criptografia gerida em repouso e em trânsito interno",
            "A planilha financeira estava legível no volume e trafegava sem criptografia dentro da rede.",
            "HIGH",
            "o diretório financeiro não tem proteção de volume nem SMB criptografado.",
            Some("CWE-311"),
            &["PR.DS-01", "PR.DS-02"],
            pick(m, &[map::EXFILTRATION]),
            vec![],
            "MEDIUM",
        ),
        finding(
            "OTHER",
            "Sem treinamento periódico de reconhecimento de phishing para o financeiro",
            "O e-mail inicial não foi reconhecido nem reportado.",
            "MEDIUM",
            "o e-mail de phishing não foi reportado pelo destinatário.",
            None,
            &["PR.AT-01"],
            pick(m, &[map::PHISHING]),
            vec![],
            "LOW",
        ),
    ];
    let mut ids = Vec::with_capacity(inputs.len());
    for input in &inputs {
        ids.push(findings::create_finding(services, incident_id, input).await?);
    }
    Ok(ids)
}

async fn seed_root_causes(
    services: &AppServices,
    incident_id: &str,
    f: &[String],
) -> AppResult<usize> {
    let inputs = [
        RootCauseInput {
            level: "ROOT_CAUSE".into(),
            statement: "O acesso à caixa do financeiro dependia só de senha: sem segundo fator resistente a phishing, uma credencial capturada bastou para o login externo e para as regras de caixa.".into(),
            basis: format!("{SYNTHETIC}: os achados sobre autenticação e regras de caixa sustentam a cadeia."),
            gaps: vec![],
            finding_ids: pick(f, &[fnd::NO_MFA, fnd::FORWARDING]),
            confidence: "HIGH".into(),
            supersedes: None,
        },
        RootCauseInput {
            level: "PROBABLE_CAUSE".into(),
            statement: "A falta de treinamento e a retenção curta de logs provavelmente deixaram o phishing inicial sem detecção e sem reconstrução posterior.".into(),
            basis: format!("{SYNTHETIC}: indícios indiretos; a trilha de mensagens não foi entregue."),
            gaps: own(&["Trilha de mensagens do período não fornecida: não se sabe quantos usuários receberam o e-mail"]),
            finding_ids: pick(f, &[fnd::NO_TRAINING, fnd::SHORT_RETENTION]),
            confidence: "MEDIUM".into(),
            supersedes: None,
        },
        RootCauseInput {
            level: "CONTRIBUTING_FACTOR".into(),
            statement: "Rede sem segmentação e sem alerta de PowerShell facilitaram o movimento lateral e atrasaram a detecção.".into(),
            basis: format!("{SYNTHETIC}: a sessão RDP e o PowerShell ocorreram sem barreira nem alerta."),
            gaps: own(&["Logs do controlador de domínio não coletados"]),
            finding_ids: pick(f, &[fnd::NO_POWERSHELL_ALERT, fnd::FLAT_RDP]),
            confidence: "MEDIUM".into(),
            supersedes: None,
        },
    ];
    for input in &inputs {
        findings::create_root_cause(services, incident_id, input).await?;
    }
    Ok(inputs.len())
}

async fn seed_recommendations(
    services: &AppServices,
    incident_id: &str,
    f: &[String],
) -> AppResult<usize> {
    let rows: [(&str, &str, &str, &[usize]); 6] = [
        (
            "IMMEDIATE",
            "Redefinir credenciais e revogar sessões da conta do financeiro",
            "Troque a senha, revogue tokens e sessões ativas e confira os métodos de autenticação cadastrados.",
            &[fnd::NO_MFA],
        ),
        (
            "IMMEDIATE",
            "Remover regras de encaminhamento externo e bloquear novas",
            "Apague as regras criadas na sessão anômala e proíba encaminhamento automático para fora da organização.",
            &[fnd::FORWARDING],
        ),
        (
            "IMMEDIATE",
            "Isolar e examinar a estação DEMO-WS07 e o servidor DEMO-FS01",
            "Isole os dois equipamentos da rede e preserve a evidência antes de qualquer limpeza.",
            &[fnd::NO_POWERSHELL_ALERT, fnd::FLAT_RDP],
        ),
        (
            "STRATEGIC",
            "Adotar segundo fator resistente a phishing para e-mail e acesso remoto",
            "Chaves de segurança FIDO2 ou passkeys, começando pelo financeiro e pelos administradores.",
            &[fnd::NO_MFA, fnd::NO_TRAINING],
        ),
        (
            "STRATEGIC",
            "Estender a retenção de logs e centralizar a coleta",
            "Retenção mínima de 12 meses para e-mail, proxy e controlador de domínio.",
            &[fnd::SHORT_RETENTION],
        ),
        (
            "STRATEGIC",
            "Criptografar dados financeiros em repouso e em trânsito e formalizar a gestão de chaves",
            "Proteja volume e diretório financeiro, exija SMB e TLS criptografados e defina quem guarda e rotaciona as chaves.",
            &[fnd::NO_ENCRYPTION],
        ),
    ];
    for (category, title, description, supports) in rows {
        findings::create_recommendation(
            services,
            incident_id,
            category,
            title,
            &format!("{SYNTHETIC}: {description}"),
            Some("MEDIUM"),
            &pick(f, supports),
            None,
        )
        .await?;
    }
    Ok(rows.len())
}

#[allow(clippy::too_many_arguments)]
fn item(
    domain: &str,
    title: &str,
    rationale: &str,
    priority: &str,
    effort: &str,
    cost: &str,
    phase: &str,
    pre: &str,
    post: &str,
    verification: &str,
) -> PlanItemInput {
    PlanItemInput {
        domain: domain.into(),
        title: title.into(),
        rationale: format!("{SYNTHETIC}: {rationale}"),
        priority: priority.into(),
        effort: effort.into(),
        cost_class: cost.into(),
        phase: phase.into(),
        precondition: pre.into(),
        postcondition: post.into(),
        verification: verification.into(),
        ..PlanItemInput::default()
    }
}

fn crypto(
    data_class: &str,
    location: &str,
    mechanism: &str,
    rotation: &str,
    recovery: &str,
) -> CryptoInput {
    CryptoInput {
        data_class: data_class.into(),
        location: location.into(),
        mechanism: mechanism.into(),
        rotation: rotation.into(),
        recovery: recovery.into(),
    }
}

async fn seed_plan(services: &AppServices, incident_id: &str, f: &[String]) -> AppResult<usize> {
    // Cada item cita a posição dos itens dos quais depende; como `depends_on`
    // precisa de ids, os itens são criados em ordem e os ids guardados.
    let mut created: Vec<String> = Vec::new();
    let mut items: Vec<(PlanItemInput, Vec<usize>)> = Vec::new();

    let mut mfa = item(
        "AUTENTICACAO_ACESSO",
        "Exigir segundo fator resistente a phishing no e-mail e no acesso remoto",
        "uma senha capturada bastou para entrar na caixa do financeiro.",
        "P1",
        "M",
        "LOW",
        "SHORT_TERM",
        "Inventário das contas de e-mail e de acesso remoto; autenticadores FIDO2 adquiridos para o financeiro.",
        "Todas as contas do financeiro e dos administradores exigem segundo fator resistente a phishing.",
        "Tentar entrar com senha apenas numa conta de teste e confirmar o bloqueio; conferir o relatório de autenticadores cadastrados.",
    );
    mfa.framework_reference = Some("NIST SP 800-63B (AAL2 ou superior)".into());
    mfa.regulatory_reference = Some("LGPD art. 46 (medidas de segurança)".into());
    mfa.nist_subcategory_ids = own(&["PR.AA-03"]);
    mfa.finding_ids = pick(f, &[fnd::NO_MFA, fnd::NO_TRAINING]);
    items.push((mfa, vec![]));

    let mut forwarding = item(
        "AUTENTICACAO_ACESSO",
        "Desativar o encaminhamento automático externo e auditar as regras de caixa",
        "regras criadas pela sessão anômala desviavam e escondiam mensagens.",
        "P1",
        "S",
        "NONE",
        "IMMEDIATE",
        "Acesso administrativo ao serviço de e-mail.",
        "Encaminhamento externo bloqueado por política; regras existentes revisadas e removidas quando indevidas.",
        "Criar uma regra de teste com destino externo e confirmar que o serviço a recusa.",
    );
    forwarding.nist_subcategory_ids = own(&["PR.AA-05", "DE.CM-03"]);
    forwarding.finding_ids = pick(f, &[fnd::FORWARDING]);
    items.push((forwarding, vec![]));

    let mut rdp = item(
        "REDE_DISPOSITIVOS",
        "Segmentar o RDP: aceitar apenas conexões vindas de um gateway administrativo",
        "uma estação comprometida alcançou o servidor de arquivos por RDP.",
        "P1",
        "M",
        "LOW",
        "SHORT_TERM",
        "Mapa das conexões RDP legítimas; gateway administrativo definido.",
        "Servidores só aceitam RDP do gateway; estações não se conectam entre si.",
        "Tentar RDP de uma estação comum para o servidor e confirmar a recusa no firewall.",
    );
    rdp.framework_reference = Some("CIS Controls v8, controles 4 e 12".into());
    rdp.nist_subcategory_ids = own(&["PR.IR-01", "PR.AA-05"]);
    rdp.finding_ids = pick(f, &[fnd::FLAT_RDP]);
    items.push((rdp, vec![]));

    let mut hardening = item(
        "OS_HARDENING",
        "Aplicar baseline de hardening nos servidores e registrar blocos de script do PowerShell",
        "o PowerShell rodou sem restrição nem registro no servidor de arquivos.",
        "P2",
        "M",
        "LOW",
        "SHORT_TERM",
        "Servidor de arquivos em janela de manutenção aprovada.",
        "Baseline aplicado; registro de blocos de script ativo e enviado ao coletor.",
        "Executar um script de teste e localizar o registro do bloco no coletor; rodar a verificação do baseline.",
    );
    hardening.framework_reference = Some("CIS Benchmarks para Windows Server (nível 1)".into());
    hardening.nist_subcategory_ids = own(&["PR.PS-01"]);
    hardening.finding_ids = pick(f, &[fnd::NO_POWERSHELL_ALERT]);
    items.push((hardening, vec![]));

    let mut alerts = item(
        "MONITORAMENTO",
        "Alertar login externo anômalo, criação de regras de caixa e PowerShell codificado",
        "nenhum dos três sinais gerou alerta durante o incidente.",
        "P1",
        "M",
        "LOW",
        "SHORT_TERM",
        "Regras de caixa auditadas (item de encaminhamento concluído); logs chegando ao coletor.",
        "Três alertas ativos, com responsável e prazo de triagem definidos.",
        "Simular cada um dos três sinais em ambiente de teste e confirmar que o alerta chega ao responsável.",
    );
    alerts.nist_subcategory_ids = own(&["DE.AE-02", "DE.CM-03"]);
    alerts.finding_ids = pick(f, &[fnd::NO_MFA, fnd::FORWARDING, fnd::NO_POWERSHELL_ALERT]);
    items.push((alerts, vec![1]));

    let mut retention = item(
        "MONITORAMENTO",
        "Centralizar os logs e estender a retenção para 12 meses",
        "a retenção de 30 dias apagou o período anterior ao incidente.",
        "P2",
        "L",
        "MEDIUM",
        "STRATEGIC",
        "Alertas em operação, para dimensionar o volume de logs.",
        "E-mail, proxy e controlador de domínio enviam logs a um repositório central com 12 meses de retenção.",
        "Consultar um evento com 11 meses de idade em cada fonte.",
    );
    retention.framework_reference = Some("NIST SP 800-92 (gestão de logs)".into());
    retention.nist_subcategory_ids = own(&["PR.PS-04"]);
    retention.finding_ids = pick(f, &[fnd::SHORT_RETENTION]);
    items.push((retention, vec![4]));

    let mut at_rest = item(
        "CRIPTO_REPOUSO",
        "Criptografar o volume e o diretório financeiro do servidor de arquivos",
        "a planilha financeira estava legível no volume.",
        "P2",
        "L",
        "MEDIUM",
        "STRATEGIC",
        "Backup verificado do volume; chaves e cofre definidos.",
        "Volume e diretório financeiro criptografados; backups também.",
        "Ler o volume fora do sistema e confirmar que o conteúdo está ilegível; restaurar um backup de teste.",
    );
    at_rest.framework_reference =
        Some("NIST SP 800-57 Parte 1 (algoritmos e tamanhos de chave)".into());
    at_rest.regulatory_reference = Some("LGPD art. 46 (medidas de segurança)".into());
    at_rest.nist_subcategory_ids = own(&["PR.DS-01"]);
    at_rest.finding_ids = pick(f, &[fnd::NO_ENCRYPTION]);
    at_rest.crypto = Some(crypto(
        FINANCIAL_DATA,
        "Servidor de arquivos DEMO-FS01 (volume de dados) e backups",
        "Criptografia de volume com AES-256 (XTS) e protetor de chave em TPM com PIN; criptografia por arquivo no diretório financeiro",
        "Protetor de recuperação trocado a cada 12 meses e após saída de quem tinha acesso",
        "Chave de recuperação em cofre offline com dupla custódia; teste de recuperação a cada 6 meses",
    ));
    items.push((at_rest, vec![]));

    let mut in_transit = item(
        "CRIPTO_TRANSITO",
        "Exigir TLS e SMB criptografados no acesso a e-mail, ERP e arquivos",
        "o tráfego interno com a planilha financeira não era criptografado.",
        "P2",
        "M",
        "LOW",
        "SHORT_TERM",
        "Inventário de serviços e clientes; certificados disponíveis.",
        "TLS 1.2 ou superior nos serviços; SMB 3 com criptografia e assinatura; RDP só com NLA.",
        "Capturar tráfego de teste entre estação e servidor e confirmar que o conteúdo não é legível.",
    );
    in_transit.framework_reference = Some("NIST SP 800-52 Rev. 2 (TLS)".into());
    in_transit.nist_subcategory_ids = own(&["PR.DS-02"]);
    in_transit.finding_ids = pick(f, &[fnd::NO_ENCRYPTION]);
    in_transit.crypto = Some(crypto(
        FINANCIAL_DATA,
        "Estação para servidor de arquivos (SMB), cliente para servidor de e-mail e administração remota",
        "TLS 1.2 ou superior (preferir 1.3) com HSTS; SMB 3 criptografado e assinado; RDP com NLA atrás de gateway",
        "Certificados com validade de até 13 meses e renovação automatizada; revisão de cifras a cada 6 meses",
        "Certificado de contingência já emitido; procedimento de revogação e reemissão documentado",
    ));
    items.push((in_transit, vec![]));

    let mut keys = item(
        "GESTAO_CHAVES",
        "Definir a gestão das chaves: guarda, acesso, rotação e recuperação",
        "sem dono e sem processo, a criptografia vira outro ponto de falha.",
        "P2",
        "L",
        "MEDIUM",
        "STRATEGIC",
        "Criptografia de repouso e de trânsito definida (itens anteriores).",
        "Chaves mestras sob responsável nomeado, com uso registrado e recuperação testada.",
        "Simular a perda de uma chave e restaurar a partir da custódia dividida.",
    );
    keys.framework_reference = Some("NIST SP 800-57 Parte 2 (gestão de chaves)".into());
    keys.nist_subcategory_ids = own(&["PR.DS-01", "PR.DS-02"]);
    keys.finding_ids = pick(f, &[fnd::NO_ENCRYPTION]);
    keys.crypto = Some(crypto(
        FINANCIAL_DATA,
        "Cofre de segredos ou módulo de hardware com cópia mestra em custódia dividida",
        "Chaves de volume protegidas por chave mestra; acesso por função, com registro de uso e separação entre quem administra e quem usa",
        "Rotação anual da chave mestra e imediata em caso de suspeita de comprometimento",
        "Custódia dividida (M de N) da chave mestra; simulação anual de perda de chave",
    ));
    items.push((keys, vec![6, 7]));

    // Lacuna deliberada: só a dimensão "gestão de chaves" cobre esta classe de
    // dado; a matriz de proteção do Entregável 5 a marca como "sem item".
    let mut secrets = item(
        "GESTAO_CHAVES",
        "Guardar as credenciais de serviço num cofre de segredos",
        "uma conta de serviço mudou de grupo sem que ninguém notasse.",
        "P3",
        "M",
        "LOW",
        "STRATEGIC",
        "Inventário das contas de serviço e de onde guardam suas senhas.",
        "Credenciais de serviço no cofre, com rotação e registro de uso.",
        "Listar as contas de serviço e confirmar que nenhuma tem segredo fora do cofre.",
    );
    secrets.nist_subcategory_ids = own(&["PR.AA-01"]);
    secrets.finding_ids = pick(f, &[fnd::FLAT_RDP]);
    secrets.crypto = Some(crypto(
        "Credenciais e segredos de serviço",
        "Cofre de segredos gerenciado",
        "Segredos fora do código e das planilhas; acesso por função, com registro de uso",
        "Rotação a cada 90 dias e imediata após suspeita de vazamento",
        "Cópia do cofre em custódia dividida; teste de restauração semestral",
    ));
    items.push((secrets, vec![]));

    let mut playbook = item(
        "POLITICA_IR",
        "Criar o playbook de fraude por e-mail (BEC) com bloqueio financeiro e contato com o banco",
        "o plano genérico não diz quem bloqueia pagamentos nem quem aciona o banco.",
        "P2",
        "S",
        "NONE",
        "SHORT_TERM",
        "Contatos do financeiro, do banco e do provedor de e-mail levantados.",
        "Playbook aprovado, com papéis, prazos e decisão de comunicação.",
        "Ler o playbook com o financeiro e conferir que cada passo tem responsável e prazo.",
    );
    playbook.framework_reference = Some("NIST SP 800-61 Rev. 3".into());
    playbook.regulatory_reference =
        Some("LGPD art. 48 (comunicação de incidente de segurança)".into());
    playbook.nist_subcategory_ids = own(&["RS.MA-01"]);
    // Sem achado: é recomendação geral, e o documento a declara como tal.
    items.push((playbook, vec![]));

    let mut tabletop = item(
        "POLITICA_IR",
        "Exercício de mesa semestral do playbook de BEC",
        "um playbook que ninguém exercitou costuma falhar no primeiro uso.",
        "P3",
        "S",
        "NONE",
        "STRATEGIC",
        "Playbook aprovado.",
        "Exercício realizado e ajustes do playbook registrados.",
        "Ata do exercício com as falhas encontradas e os prazos de correção.",
    );
    tabletop.nist_subcategory_ids = own(&["RS.MA-01"]);
    tabletop.finding_ids = pick(f, &[fnd::NO_TRAINING]);
    items.push((tabletop, vec![10]));

    let mut training = item(
        "POLITICA_IR",
        "Treinamento recorrente de reconhecimento e reporte de phishing para o financeiro",
        "o e-mail inicial não foi reconhecido nem reportado.",
        "P3",
        "S",
        "LOW",
        "STRATEGIC",
        "Canal de reporte de e-mail suspeito definido.",
        "Financeiro treinado a cada 6 meses, com simulação e taxa de reporte medida.",
        "Resultado da simulação de phishing e taxa de reporte do último ciclo.",
    );
    training.nist_subcategory_ids = own(&["PR.AT-01"]);
    training.finding_ids = pick(f, &[fnd::NO_TRAINING]);
    items.push((training, vec![]));

    for (mut input, depends) in items {
        input.depends_on = depends
            .iter()
            .map(|&position| created[position].clone())
            .collect();
        created.push(plan::create_plan_item(services, incident_id, &input).await?);
    }
    Ok(created.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::deliverables::{self, RenderOptions};
    use crate::service::test_support::{Fixture, fixture};

    const DOCUMENTS: [&str; 5] = [
        "01-relatorio-forense.md",
        "02-vulnerabilidades-attack.md",
        "03-nist-recomendacoes.md",
        "04-hardening.md",
        "05-criptografia.md",
    ];

    /// Sem engine nos testes unitários: o seed roda sem a etapa de análise,
    /// que o e2e headless cobre com o engine e o `.evtx` reais.
    async fn seeded() -> (Fixture, DemoSeedReport, tempfile::TempDir) {
        let fx = fixture().await;
        let dir = tempfile::tempdir().expect("temporary directory");
        let evtx = dir.path().join("amostra.evtx");
        std::fs::write(&evtx, b"conteudo de teste").expect("sample file");
        let report = seed(&fx.services, &evtx, 1024 * 1024, false)
            .await
            .expect("the demo scenario must be accepted by every service rule");
        (fx, report, dir)
    }

    async fn render(
        fx: &Fixture,
        incident_id: &str,
        mask: bool,
    ) -> (tempfile::TempDir, std::path::PathBuf) {
        let out = tempfile::tempdir().expect("output directory");
        let generated = deliverables::generate(
            &fx.services,
            incident_id,
            out.path(),
            RenderOptions {
                mask_personal_data: mask,
            },
        )
        .await
        .expect("deliverables");
        let dir = generated.output_dir;
        (out, dir)
    }

    fn read(dir: &std::path::Path, name: &str) -> String {
        std::fs::read_to_string(dir.join(name)).expect("document")
    }

    #[tokio::test]
    async fn seed_registers_the_whole_scenario() {
        let (_fx, report, _dir) = seeded().await;
        assert_eq!(report.dataset_version, DEMO_DATASET_VERSION);
        assert!(report.incident_name.starts_with("[DEMO]"));
        assert!(report.analysis_job_ids.is_empty());
        let counts = &report.counts;
        assert_eq!(counts.attack_mappings, 9);
        assert_eq!(counts.critical_path_stages, 6);
        assert_eq!(counts.iocs, 6);
        assert_eq!(counts.findings, 7);
        assert_eq!(counts.root_causes, 3);
        assert_eq!(counts.recommendations, 6);
        assert_eq!(counts.plan_items, 13);
        assert_eq!(counts.source_gaps, 3);
        // 11 subcategorias no profile atual; só ID.AM-01 já está na meta.
        assert_eq!(counts.nist_gaps, 10);
    }

    #[tokio::test]
    async fn the_evidence_is_classified_demo_and_the_seed_is_audited() {
        let (fx, report, _dir) = seeded().await;
        let class: String = sqlx::query_scalar("SELECT status FROM evidence WHERE id=?")
            .bind(&report.evidence_id)
            .fetch_one(&fx.services.db)
            .await
            .expect("evidence row");
        assert_eq!(class, "DEMO");
        let audited: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM audit_events WHERE action='DEMO_DATASET_SEEDED' AND incident_id=?",
        )
        .bind(&report.incident_id)
        .fetch_one(&fx.services.db)
        .await
        .expect("audit count");
        assert_eq!(audited, 1);
    }

    #[tokio::test]
    async fn all_five_documents_carry_the_demo_label() {
        let (fx, report, _dir) = seeded().await;
        let (_out, dir) = render(&fx, &report.incident_id, true).await;
        for name in DOCUMENTS {
            let text = read(&dir, name);
            let title = text.lines().next().expect("title");
            assert!(
                title.contains("DEMO"),
                "{name}: title without the DEMO label: {title}"
            );
            assert!(
                text.contains("DEMO (sintético)"),
                "{name}: no synthetic marker"
            );
        }
    }

    #[tokio::test]
    async fn the_plan_covers_the_eight_domains_and_the_crypto_gap_is_marked() {
        let (fx, report, _dir) = seeded().await;
        let (_out, dir) = render(&fx, &report.incident_id, true).await;
        let hardening = read(&dir, "04-hardening.md");
        for heading in [
            "## 1. Hardening de SO e servidores",
            "## 2. Redes e dispositivos",
            "## 3. Autenticação e controle de acesso",
            "## 4. Monitoramento de segurança",
            "## 5. Políticas de resposta a incidentes",
        ] {
            assert!(hardening.contains(heading), "missing {heading}");
        }
        let crypto = read(&dir, "05-criptografia.md");
        for heading in [
            "## 1. Criptografia de dados em repouso",
            "## 2. Criptografia de dados em trânsito",
            "## 3. Gestão de chaves criptográficas",
        ] {
            assert!(crypto.contains(heading), "missing {heading}");
        }
        // O dado financeiro fecha as três dimensões; os segredos de serviço
        // têm só a gestão de chaves, e a matriz deve dizer isso.
        let matrix: Vec<&str> = crypto
            .lines()
            .filter(|line| line.starts_with("| Credenciais e segredos"))
            .collect();
        assert_eq!(matrix.len(), 1, "one matrix row for the service secrets");
        assert_eq!(matrix[0].matches("— (sem item)").count(), 2);
        let financial: Vec<&str> = crypto
            .lines()
            .filter(|line| line.starts_with("| Dados financeiros"))
            .collect();
        assert_eq!(financial.len(), 1);
        assert_eq!(financial[0].matches("— (sem item)").count(), 0);
    }

    #[tokio::test]
    async fn masking_hides_the_documentation_values_and_unmasked_shows_them() {
        let (fx, report, _dir) = seeded().await;
        let (_masked_out, masked) = render(&fx, &report.incident_id, true).await;
        let (_raw_out, raw) = render(&fx, &report.incident_id, false).await;
        let masked_text: String = DOCUMENTS.iter().map(|name| read(&masked, name)).collect();
        let raw_text: String = DOCUMENTS.iter().map(|name| read(&raw, name)).collect();
        for sensitive in [
            "203.0.113.45",
            "cobranca@fornecedor-demo.example",
            "Ana Exemplo",
        ] {
            assert!(
                !masked_text.contains(sensitive),
                "{sensitive} leaked in the masked documents"
            );
            assert!(
                raw_text.contains(sensitive),
                "{sensitive} missing from the unmasked documents"
            );
        }
        assert!(masked_text.contains("{IP-1}") && masked_text.contains("{EMAIL-1}"));
    }

    #[tokio::test]
    async fn rendering_twice_gives_identical_documents() {
        let (fx, report, _dir) = seeded().await;
        let (_first_out, first) = render(&fx, &report.incident_id, true).await;
        let (_second_out, second) = render(&fx, &report.incident_id, true).await;
        for name in DOCUMENTS {
            assert_eq!(
                read(&first, name),
                read(&second, name),
                "{name} is not deterministic"
            );
        }
    }

    /// Regressão: depois do descarte local, o Entregável 1 afirmava que
    /// "nenhuma evidência foi citada" — falso, pois os IOCs a citam. O
    /// arquivo some; hash e custódia ficam e precisam continuar no relatório.
    #[tokio::test]
    async fn a_destroyed_evidence_is_still_cited_with_its_hash_and_custody() {
        let (fx, report, _dir) = seeded().await;
        crate::service::destroy_evidence(&fx.services, &report.evidence_id)
            .await
            .expect("destroy");
        let (_out, dir) = render(&fx, &report.incident_id, true).await;
        let text = read(&dir, "01-relatorio-forense.md");
        assert!(
            !text.contains("nenhuma evidência citada"),
            "the report denies the evidence that the IOCs cite"
        );
        assert!(text.contains("Evidências citadas pelo incidente: 1."));
        assert!(text.contains("foi descartada localmente a pedido do operador"));
        assert!(
            text.contains("`DESTROYED`"),
            "custody must show the destruction"
        );
        let sha256: String = sqlx::query_scalar("SELECT sha256 FROM evidence WHERE id=?")
            .bind(&report.evidence_id)
            .fetch_one(&fx.services.db)
            .await
            .expect("hash");
        assert!(text.contains(&sha256), "the hash must stay in the report");
        assert!(text.contains("evidência descartada"), "raw timeline row");
        assert!(!text.contains("nenhuma análise de linha do tempo concluída"));
    }

    #[tokio::test]
    async fn an_unknown_data_class_is_refused_before_anything_is_copied() {
        let fx = fixture().await;
        let dir = tempfile::tempdir().expect("temporary directory");
        let file = dir.path().join("amostra.evtx");
        std::fs::write(&file, b"x").expect("sample file");
        let error = import_evidence_classified(
            &fx.services,
            &file,
            1024,
            &EvidenceProvenance::default(),
            "REAL",
        )
        .await
        .expect_err("unknown class");
        assert!(error.to_string().contains("data class"), "{error}");
    }
}
