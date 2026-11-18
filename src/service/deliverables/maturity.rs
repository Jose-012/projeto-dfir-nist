//! Entregável 3 — Maturidade NIST CSF 2.0 e Recomendações (spec §4.2).

use super::markdown::{Doc, code, esc, short};
use super::{
    DeliverableData, FILE_E3, RenderedDocument, domain_label, field, into_document, plan_sort_key,
    strings,
};
use crate::persistence::nist_functions::GapAnalysisState;
use crate::service::AnalyticalResultResponse;
use crate::service::nist::ProfileSummary;

pub(super) fn render(data: &DeliverableData) -> RenderedDocument {
    let mut doc = Doc::new();
    doc.title(&format!(
        "Maturidade NIST CSF 2.0 e recomendações — {}",
        data.incident.name
    ));
    doc.quote(&format!(
        "Incidente {} · Entregável 3 de 5. O NIST CSF 2.0 não define pontuação oficial: este documento traz contagens e o Tier descritivo informado pelo analista, sem nota.",
        code(&short(&data.incident.id))
    ));
    profiles(&mut doc, data);
    by_function(&mut doc, data);
    gaps(&mut doc, data);
    recommendations(&mut doc, data);
    consolidated_plan(&mut doc, data);
    into_document(doc, FILE_E3, "Maturidade NIST CSF 2.0 e recomendações")
}

fn profile_line(name: &str, profile: Option<&ProfileSummary>) -> String {
    match profile {
        None => format!("{name}: ainda não informado."),
        Some(profile) => {
            let tier = profile
                .tier
                .as_deref()
                .map_or("não informado".to_string(), code);
            let mut line = format!(
                "{name}: versão {} · {} subcategorias · Tier {tier}",
                code(&profile.version_label),
                profile.entries
            );
            if let Some(justification) = profile
                .tier_justification
                .as_deref()
                .filter(|text| !text.trim().is_empty())
            {
                line.push_str(&format!(" · justificativa: {}", esc(justification)));
            }
            line
        }
    }
}

fn profiles(doc: &mut Doc, data: &DeliverableData) {
    doc.h2("1. Profiles");
    doc.bullets(&[
        profile_line("Current", data.nist.current_profile.as_ref()),
        profile_line("Target", data.nist.target_profile.as_ref()),
    ]);
    doc.para("*O Tier descreve o contexto de governança do profile inteiro; não existe um Tier por Function.*");
}

fn by_function(doc: &mut Doc, data: &DeliverableData) {
    doc.h2("2. Maturidade por Function (contagens, sem pontuação)");
    let rows: Vec<Vec<String>> = data
        .nist
        .functions
        .iter()
        .map(|function| {
            vec![
                format!("{} — {}", code(&function.function), esc(&function.label)),
                function.current.implemented.to_string(),
                function.current.partially_implemented.to_string(),
                function.current.not_implemented.to_string(),
                function.current.not_assessed.to_string(),
                function.current.not_applicable.to_string(),
                function.target_subcategories.to_string(),
                function.gaps.high.to_string(),
                function.gaps.medium.to_string(),
                function.gaps.low.to_string(),
            ]
        })
        .collect();
    doc.table(
        &[
            "Function",
            "Implementada",
            "Parcial",
            "Não implementada",
            "Não avaliada",
            "N/A",
            "Alvo",
            "Gap alto",
            "Gap médio",
            "Gap baixo",
        ],
        &rows,
    );
    let note = match data.nist.gap_state {
        GapAnalysisState::MissingProfile => {
            "Falta o profile Current ou o Target: não há o que comparar."
        }
        GapAnalysisState::NoGapsRecorded => {
            "Nenhum gap vigente: a análise ainda não foi calculada ou o Current já atinge o Target."
        }
        GapAnalysisState::Stale => {
            "**Atenção:** os gaps foram calculados com uma versão anterior dos profiles; as contagens de gap podem não refletir os profiles atuais."
        }
        GapAnalysisState::UpToDate => "Os gaps correspondem aos profiles vigentes.",
    };
    doc.para(&format!("*{note}*"));
}

fn gaps(doc: &mut Doc, data: &DeliverableData) {
    doc.h2("3. Gaps vigentes");
    if data.gaps.is_empty() {
        doc.none("nenhum gap vigente");
        return;
    }
    let rank = |token: &str| match token {
        "HIGH" => 0,
        "MEDIUM" => 1,
        "LOW" => 2,
        _ => 3,
    };
    let mut gaps: Vec<&AnalyticalResultResponse> = data.gaps.iter().collect();
    gaps.sort_by(|a, b| {
        rank(field(a, "priority"))
            .cmp(&rank(field(b, "priority")))
            .then_with(|| field(a, "subcategory_id").cmp(field(b, "subcategory_id")))
            .then_with(|| a.id.cmp(&b.id))
    });
    let mut rows = Vec::new();
    for gap in gaps {
        rows.push(vec![
            code(field(gap, "subcategory_id")),
            esc(field(gap, "subcategory_label")),
            code(field(gap, "current_status")),
            code(field(gap, "target_status")),
            code(field(gap, "priority")),
            doc.cite(&gap.id),
        ]);
    }
    doc.table(
        &[
            "Subcategoria",
            "Rótulo",
            "Atual",
            "Alvo",
            "Prioridade",
            "Origem",
        ],
        &rows,
    );
}

fn recommendations(doc: &mut Doc, data: &DeliverableData) {
    doc.h2("4. Recomendações");
    for (category, heading) in [
        ("IMMEDIATE", "4.1 Imediatas"),
        ("STRATEGIC", "4.2 Estratégicas"),
    ] {
        doc.h3(heading);
        let mut items: Vec<&AnalyticalResultResponse> = data
            .recommendations
            .iter()
            .filter(|item| field(item, "category") == category)
            .collect();
        if items.is_empty() {
            doc.none("nenhuma recomendação nesta categoria");
            continue;
        }
        items.sort_by(|a, b| {
            field(a, "title")
                .cmp(field(b, "title"))
                .then_with(|| a.id.cmp(&b.id))
        });
        let mut lines = Vec::new();
        for item in items {
            let mut line = format!("**{}**", esc(field(item, "title")));
            let description = field(item, "description");
            if !description.trim().is_empty() {
                line.push_str(&format!(": {}", esc(description)));
            }
            let support = strings(item, "supporting_result_ids");
            if !support.is_empty() {
                let cited: Vec<String> = support.iter().map(|id| doc.cite(id)).collect();
                line.push_str(&format!(" · apoio {}", cited.join(", ")));
            }
            line.push_str(&format!(" · origem {}", doc.cite(&item.id)));
            lines.push(line);
        }
        doc.bullets(&lines);
    }
}

fn consolidated_plan(doc: &mut Doc, data: &DeliverableData) {
    doc.h2("5. Plano consolidado por fase");
    if data.plan.items.is_empty() {
        doc.none("nenhum item de plano registrado");
        return;
    }
    for (phase, heading) in [
        ("IMMEDIATE", "5.1 Imediata"),
        ("SHORT_TERM", "5.2 Curto prazo"),
        ("STRATEGIC", "5.3 Estratégica"),
    ] {
        doc.h3(heading);
        let mut items: Vec<&AnalyticalResultResponse> = data
            .plan
            .items
            .iter()
            .filter(|item| field(item, "phase") == phase)
            .collect();
        if items.is_empty() {
            doc.none("nenhum item nesta fase");
            continue;
        }
        items.sort_by_key(|item| plan_sort_key(item));
        let mut rows = Vec::new();
        for item in items {
            let support = if strings(item, "finding_ids").is_empty() {
                "recomendação geral"
            } else {
                "motivado por achado"
            };
            rows.push(vec![
                domain_label(field(item, "domain")),
                esc(field(item, "title")),
                code(field(item, "priority")),
                code(field(item, "effort")),
                code(field(item, "cost_class")),
                support.to_string(),
                doc.cite(&item.id),
            ]);
        }
        doc.table(
            &[
                "Ação",
                "Item",
                "Prioridade",
                "Esforço",
                "Custo",
                "Suporte",
                "Origem",
            ],
            &rows,
        );
    }
}
