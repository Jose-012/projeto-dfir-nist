//! Entregável 5 — Criptografia e proteção de dados (spec §4.2).

use super::markdown::{Doc, code, esc, short};
use super::{
    DeliverableData, FILE_E5, RenderedDocument, domain_label, field, into_document, plan_sort_key,
    write_plan_item,
};
use crate::service::AnalyticalResultResponse;
use std::collections::BTreeMap;

const DOMAINS: [&str; 3] = ["CRIPTO_REPOUSO", "CRIPTO_TRANSITO", "GESTAO_CHAVES"];

fn crypto_field<'a>(item: &'a AnalyticalResultResponse, key: &str) -> &'a str {
    item.payload
        .get("crypto")
        .and_then(|crypto| crypto.get(key))
        .and_then(|value| value.as_str())
        .unwrap_or("")
}

fn items_of<'a>(data: &'a DeliverableData, domain: &str) -> Vec<&'a AnalyticalResultResponse> {
    let mut items: Vec<&AnalyticalResultResponse> = data
        .plan
        .items
        .iter()
        .filter(|item| field(item, "domain") == domain)
        .collect();
    items.sort_by_key(|item| plan_sort_key(item));
    items
}

pub(super) fn render(data: &DeliverableData) -> RenderedDocument {
    let mut doc = Doc::new();
    doc.title(&format!(
        "Criptografia e proteção de dados — {}",
        data.incident.name
    ));
    doc.quote(&format!(
        "Incidente {} · Entregável 5 de 5. Cada item traz classe do dado, local, mecanismo, rotação e recuperação de chave, como exige o plano de fortalecimento da demanda.",
        code(&short(&data.incident.id))
    ));

    for (index, domain) in DOMAINS.iter().enumerate() {
        doc.h2(&format!("{}. {}", index + 1, domain_label(domain)));
        let items = items_of(data, domain);
        if items.is_empty() {
            doc.none("nenhum item de plano registrado para esta ação");
            continue;
        }
        for item in items {
            write_plan_item(&mut doc, data, item, true);
        }
    }

    protection_matrix(&mut doc, data);
    implementation_plan(&mut doc, data);
    into_document(doc, FILE_E5, "Criptografia e proteção de dados")
}

/// Classe do dado × (repouso, trânsito, chaves). Célula sem item é uma
/// lacuna e aparece como tal.
fn protection_matrix(doc: &mut Doc, data: &DeliverableData) {
    doc.h2("4. Matriz de proteção");
    // Chave normalizada (minúsculas, sem espaços nas pontas); rótulo da
    // primeira ocorrência, em ordem estável.
    let mut classes: BTreeMap<String, String> = BTreeMap::new();
    for domain in DOMAINS {
        for item in items_of(data, domain) {
            let label = crypto_field(item, "data_class").trim();
            if !label.is_empty() {
                classes
                    .entry(label.to_lowercase())
                    .or_insert_with(|| label.to_string());
            }
        }
    }
    if classes.is_empty() {
        doc.none("nenhuma classe de dado registrada nos itens de proteção");
        return;
    }
    let rows: Vec<Vec<String>> = classes
        .iter()
        .map(|(key, label)| {
            let mut row = vec![esc(label)];
            for domain in DOMAINS {
                let cells: Vec<String> = items_of(data, domain)
                    .into_iter()
                    .filter(|item| crypto_field(item, "data_class").trim().to_lowercase() == *key)
                    .map(|item| {
                        format!(
                            "{} @ {}",
                            esc(crypto_field(item, "mechanism")),
                            esc(crypto_field(item, "location"))
                        )
                    })
                    .collect();
                row.push(if cells.is_empty() {
                    "— (sem item)".to_string()
                } else {
                    cells.join("; ")
                });
            }
            row
        })
        .collect();
    doc.table(
        &[
            "Classe do dado",
            "Em repouso",
            "Em trânsito",
            "Gestão de chaves",
        ],
        &rows,
    );
    doc.para("*\"— (sem item)\" é uma lacuna do plano: nenhum item cobre aquela classe de dado naquela dimensão.*");
}

fn implementation_plan(doc: &mut Doc, data: &DeliverableData) {
    doc.h2("5. Plano de implementação");
    let mut items: Vec<&AnalyticalResultResponse> = DOMAINS
        .iter()
        .flat_map(|domain| items_of(data, domain))
        .collect();
    if items.is_empty() {
        doc.none("nenhum item de plano de criptografia ou chaves");
        return;
    }
    items.sort_by_key(|item| plan_sort_key(item));
    let mut rows = Vec::new();
    for item in items {
        rows.push(vec![
            esc(field(item, "title")),
            domain_label(field(item, "domain")),
            code(field(item, "priority")),
            code(field(item, "effort")),
            code(field(item, "cost_class")),
            code(field(item, "phase")),
            doc.cite(&item.id),
        ]);
    }
    doc.table(
        &[
            "Item",
            "Ação",
            "Prioridade",
            "Esforço",
            "Custo",
            "Fase",
            "Origem",
        ],
        &rows,
    );
}
