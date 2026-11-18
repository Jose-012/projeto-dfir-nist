//! Entregável 2 — Vulnerabilidades e MITRE ATT&CK (spec §4.2).

use super::markdown::{Doc, code, esc, short};
use super::report::tactic_rank;
use super::{
    DeliverableData, FILE_E2, RenderedDocument, confidence_rank, evidence_name, field,
    into_document, severity_rank, strings,
};
use crate::service::AnalyticalResultResponse;

const SEVERITIES: [&str; 4] = ["CRITICAL", "HIGH", "MEDIUM", "LOW"];
const CONFIDENCES: [&str; 4] = ["CONFIRMED", "HIGH", "MEDIUM", "LOW"];

pub(super) fn render(data: &DeliverableData) -> RenderedDocument {
    let mut doc = Doc::new();
    doc.title(&format!(
        "Vulnerabilidades e MITRE ATT&CK — {}",
        data.incident.name
    ));
    doc.quote(&format!(
        "Incidente {} · Entregável 2 de 5. Achados e mapeamentos registrados pelo analista; cada linha cita o resultado de origem.",
        code(&short(&data.incident.id))
    ));
    findings_section(&mut doc, data);
    matrix_section(&mut doc, data);
    attack_section(&mut doc, data);
    into_document(doc, FILE_E2, "Vulnerabilidades e MITRE ATT&CK")
}

fn findings_section(doc: &mut Doc, data: &DeliverableData) {
    doc.h2("1. Vulnerabilidades e condições exploradas");
    let mut findings: Vec<&AnalyticalResultResponse> = data.findings.findings.iter().collect();
    if findings.is_empty() {
        doc.none("nenhum achado registrado");
        return;
    }
    findings.sort_by(|a, b| {
        severity_rank(field(b, "severity"))
            .cmp(&severity_rank(field(a, "severity")))
            .then_with(|| {
                confidence_rank(b.confidence.as_deref().unwrap_or(""))
                    .cmp(&confidence_rank(a.confidence.as_deref().unwrap_or("")))
            })
            .then_with(|| field(a, "title").cmp(field(b, "title")))
            .then_with(|| a.id.cmp(&b.id))
    });
    let mut rows = Vec::new();
    for finding in findings {
        let description = field(finding, "description");
        let title = if description.trim().is_empty() {
            esc(field(finding, "title"))
        } else {
            format!("{} — {}", esc(field(finding, "title")), esc(description))
        };
        let attack: Vec<String> = strings(finding, "attack_mapping_ids")
            .iter()
            .map(|id| {
                data.attack_mappings
                    .iter()
                    .find(|mapping| mapping.id == *id)
                    .map_or_else(
                        || code(&short(id)),
                        |mapping| code(field(mapping, "technique_id")),
                    )
            })
            .collect();
        let nist: Vec<String> = strings(finding, "nist_subcategories")
            .iter()
            .map(|id| code(id))
            .collect();
        let reference = field(finding, "reference");
        rows.push(vec![
            title,
            code(field(finding, "finding_kind")),
            code(field(finding, "severity")),
            code(finding.confidence.as_deref().unwrap_or("")),
            evidence_name(data, finding.evidence_id.as_deref()),
            esc(field(finding, "basis")),
            if reference.is_empty() {
                String::new()
            } else {
                esc(reference)
            },
            nist.join(", "),
            attack.join(", "),
            doc.cite(&finding.id),
        ]);
    }
    doc.table(
        &[
            "Achado",
            "Tipo",
            "Severidade",
            "Confiança",
            "Evidência",
            "Base",
            "Referência",
            "NIST",
            "ATT&CK",
            "Origem",
        ],
        &rows,
    );
}

fn matrix_section(doc: &mut Doc, data: &DeliverableData) {
    doc.h2("2. Matriz de risco (severidade × confiança)");
    if data.findings.findings.is_empty() {
        doc.none("sem achados, a matriz não tem o que contar");
        return;
    }
    let count_of = |severity: &str, confidence: &str| -> usize {
        data.findings
            .risk_matrix
            .iter()
            .find(|cell| cell.severity == severity && cell.confidence == confidence)
            .map_or(0, |cell| cell.count)
    };
    let rows: Vec<Vec<String>> = SEVERITIES
        .iter()
        .map(|severity| {
            let mut row = vec![code(severity)];
            row.extend(
                CONFIDENCES
                    .iter()
                    .map(|confidence| count_of(severity, confidence).to_string()),
            );
            row
        })
        .collect();
    doc.table(
        &[
            "Severidade × Confiança",
            "CONFIRMED",
            "HIGH",
            "MEDIUM",
            "LOW",
        ],
        &rows,
    );
    doc.para("*Cada célula é a contagem de achados vigentes com aquela severidade e confiança. Não há pontuação nem média: a matriz só conta o que foi registrado.*");
}

fn attack_section(doc: &mut Doc, data: &DeliverableData) {
    doc.h2("3. Mapeamento MITRE ATT&CK");
    let mut mappings: Vec<&AnalyticalResultResponse> = data.attack_mappings.iter().collect();
    if mappings.is_empty() {
        doc.none("nenhum mapeamento ATT&CK registrado");
        return;
    }
    mappings.sort_by(|a, b| {
        tactic_rank(field(a, "tactic"))
            .cmp(&tactic_rank(field(b, "tactic")))
            .then_with(|| field(a, "technique_id").cmp(field(b, "technique_id")))
            .then_with(|| a.id.cmp(&b.id))
    });
    let mut rows = Vec::new();
    for mapping in mappings {
        let basis = field(mapping, "basis");
        rows.push(vec![
            code(field(mapping, "tactic")),
            code(field(mapping, "technique_id")),
            code(field(mapping, "sub_technique_id")),
            if basis.trim().is_empty() {
                "sem base documentada".to_string()
            } else {
                esc(basis)
            },
            code(mapping.confidence.as_deref().unwrap_or("")),
            evidence_name(data, mapping.evidence_id.as_deref()),
            doc.cite(&mapping.id),
        ]);
    }
    doc.table(
        &[
            "Tática",
            "Técnica",
            "Sub-técnica",
            "Base",
            "Confiança",
            "Evidência",
            "Origem",
        ],
        &rows,
    );
    doc.para("*Uma técnica só é registrada quando o analista a justifica; as sem base aparecem como tal.*");
}
