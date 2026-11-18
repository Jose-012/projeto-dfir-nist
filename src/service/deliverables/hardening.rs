//! Entregável 4 — Hardening e fortalecimento da infraestrutura (spec §4.2).

use super::markdown::{Doc, code, short};
use super::{
    DeliverableData, FILE_E4, RenderedDocument, domain_label, field, into_document, plan_sort_key,
    write_plan_item,
};
use crate::service::AnalyticalResultResponse;

/// Ações do plano de fortalecimento que não são criptografia nem chaves
/// (essas ficam no Entregável 5).
const DOMAINS: [&str; 5] = [
    "OS_HARDENING",
    "REDE_DISPOSITIVOS",
    "AUTENTICACAO_ACESSO",
    "MONITORAMENTO",
    "POLITICA_IR",
];

pub(super) fn render(data: &DeliverableData) -> RenderedDocument {
    let mut doc = Doc::new();
    doc.title(&format!(
        "Hardening e fortalecimento da infraestrutura — {}",
        data.incident.name
    ));
    doc.quote(&format!(
        "Incidente {} · Entregável 4 de 5. Cada item segue a cadeia achado → recomendação → referência → pré e pós-condição → como verificar. Item sem achado de suporte aparece como recomendação geral, nunca como achado.",
        code(&short(&data.incident.id))
    ));
    for (index, domain) in DOMAINS.iter().enumerate() {
        doc.h2(&format!("{}. {}", index + 1, domain_label(domain)));
        let mut items: Vec<&AnalyticalResultResponse> = data
            .plan
            .items
            .iter()
            .filter(|item| field(item, "domain") == *domain)
            .collect();
        if items.is_empty() {
            doc.none("nenhum item de plano registrado para esta ação");
            continue;
        }
        items.sort_by_key(|item| plan_sort_key(item));
        for item in items {
            write_plan_item(&mut doc, data, item, false);
        }
    }
    doc.para("*Criptografia de dados e gestão de chaves estão no Entregável 5.*");
    into_document(doc, FILE_E4, "Hardening e fortalecimento da infraestrutura")
}
