//! Aba "Plano": itens do plano de fortalecimento por domínio e a cobertura
//! das ações da demanda (F1-29, RF-052; absorve F1-06).
//!
//! A interface só coleta e mostra; as regras (prioridade, esforço, custo e
//! fase obrigatórios; detalhes de proteção só nos domínios de criptografia
//! e chaves; recomendação geral quando não há achado) vivem em
//! `persistence/plan_items.rs` e chegam aqui como mensagem de erro.

use super::backend::{Backend, Command};
use super::{
    Loadable, PLAN_COST_OPTIONS, PLAN_DOMAIN_OPTIONS, PLAN_EFFORT_OPTIONS, PLAN_PHASE_OPTIONS,
    PLAN_PRIORITY_OPTIONS, enum_combo, error_banner, non_empty, split_ids, str_field, string_list,
    toggle,
};
use crate::persistence::plan_items::{DomainCoverage, PlanDomain};
use crate::service::AnalyticalResultResponse;
use crate::service::plan::{CryptoInput, PlanItemInput, PlanView};
use std::collections::BTreeSet;

#[derive(Default)]
struct PlanForm {
    domain: String,
    title: String,
    rationale: String,
    priority: String,
    effort: String,
    cost_class: String,
    phase: String,
    precondition: String,
    postcondition: String,
    verification: String,
    framework_reference: String,
    regulatory_reference: String,
    nist_ids: String,
    finding_ids: BTreeSet<String>,
    depends_on: BTreeSet<String>,
    data_class: String,
    location: String,
    mechanism: String,
    rotation: String,
    recovery: String,
}

impl PlanForm {
    fn needs_crypto_detail(&self) -> bool {
        PlanDomain::from_str(&self.domain).is_some_and(PlanDomain::needs_crypto_detail)
    }
}

#[derive(Default)]
pub struct PlanState {
    loaded_for: Option<String>,
    view: Loadable<PlanView>,
    form: PlanForm,
    creating: bool,
    error: Option<String>,
}

impl PlanState {
    fn ensure_loaded(&mut self, incident_id: &str, backend: &Backend) {
        if self.loaded_for.as_deref() == Some(incident_id) {
            return;
        }
        self.loaded_for = Some(incident_id.to_string());
        self.view = Loadable::Loading;
        backend.send(Command::LoadPlanView(incident_id.to_string()));
    }

    pub fn on_loaded(&mut self, incident_id: &str, result: Result<PlanView, String>) {
        if self.loaded_for.as_deref() != Some(incident_id) {
            return;
        }
        self.view = match result {
            Ok(view) => Loadable::Ready(view),
            Err(message) => Loadable::Failed(message),
        };
    }

    pub fn on_created(
        &mut self,
        incident_id: &str,
        result: Result<String, String>,
        backend: &Backend,
    ) {
        self.creating = false;
        match result {
            Ok(_) => {
                self.error = None;
                self.form = PlanForm::default();
                backend.send(Command::LoadPlanView(incident_id.to_string()));
            }
            Err(message) => self.error = Some(message),
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, incident_id: &str, backend: &Backend) {
        self.ensure_loaded(incident_id, backend);

        self.view.show(ui, |ui, view| {
            ui.heading("Cobertura das ações do plano");
            show_coverage(ui, &view.coverage);
            ui.separator();
            ui.heading("Itens do plano");
            show_items(ui, &view.items);
        });
        ui.separator();

        let Self {
            view,
            form,
            creating,
            error,
            ..
        } = self;
        let (items, findings): (&[AnalyticalResultResponse], &[AnalyticalResultResponse]) =
            match view.ready() {
                Some(view) => (&view.items, &view.findings),
                None => (&[], &[]),
            };
        ui.collapsing("Novo item de plano", |ui| {
            form_ui(
                ui,
                form,
                items,
                findings,
                creating,
                error.as_deref(),
                (incident_id, backend),
            );
        });
    }
}

fn show_coverage(ui: &mut egui::Ui, coverage: &[DomainCoverage]) {
    egui::Grid::new("plan_coverage")
        .striped(true)
        .show(ui, |ui| {
            ui.strong("Ação da demanda");
            ui.strong("Itens");
            ui.strong("Recomendações gerais");
            ui.end_row();
            for row in coverage {
                ui.label(&row.label);
                ui.label(row.items.to_string());
                ui.label(row.general_items.to_string());
                ui.end_row();
            }
        });
    ui.small("Uma ação sem itens aparece com 0: o plano ainda não a cobre.");
}

/// Linha "Rótulo: valor" só quando há valor.
fn labelled(ui: &mut egui::Ui, label: &str, value: &str) {
    if !value.trim().is_empty() {
        ui.label(format!("{label}: {value}"));
    }
}

fn show_items(ui: &mut egui::Ui, items: &[AnalyticalResultResponse]) {
    if items.is_empty() {
        ui.label("Nenhum item de plano ainda.");
        return;
    }
    let mut current_domain = "";
    for item in items {
        let payload = &item.payload;
        let domain = str_field(payload, "domain");
        if domain != current_domain {
            current_domain = domain;
            let heading: &str = match PlanDomain::from_str(domain) {
                Some(known) => known.label_pt(),
                None => domain,
            };
            ui.add_space(6.0);
            ui.strong(format!("▸ {heading}"));
        }
        ui.group(|ui| {
            ui.strong(str_field(payload, "title"));
            ui.label(format!(
                "{} · esforço {} · custo {} · {}",
                str_field(payload, "priority"),
                str_field(payload, "effort"),
                str_field(payload, "cost_class"),
                str_field(payload, "phase"),
            ));
            if str_field(payload, "support_kind") == "GENERAL_RECOMMENDATION" {
                ui.label("Recomendação geral (sem achado de suporte)");
            } else {
                ui.label(format!(
                    "Motivado por {} achado(s)",
                    string_list(payload, "finding_ids").len()
                ));
            }
            labelled(ui, "Motivo", str_field(payload, "rationale"));
            labelled(ui, "Pré-condição", str_field(payload, "precondition"));
            labelled(ui, "Pós-condição", str_field(payload, "postcondition"));
            labelled(ui, "Como verificar", str_field(payload, "verification"));
            labelled(ui, "Referência", str_field(payload, "framework_reference"));
            labelled(
                ui,
                "Regulatório / política interna",
                str_field(payload, "regulatory_reference"),
            );
            let nist = string_list(payload, "nist_subcategories");
            if !nist.is_empty() {
                ui.small(format!("NIST: {}", nist.join(", ")));
            }
            let dependencies = string_list(payload, "depends_on").len();
            if dependencies > 0 {
                ui.small(format!("Depende de {dependencies} item(ns)"));
            }
            if let Some(crypto) = payload.get("crypto").filter(|value| value.is_object()) {
                labelled(ui, "Dado protegido", str_field(crypto, "data_class"));
                labelled(ui, "Local", str_field(crypto, "location"));
                labelled(ui, "Mecanismo", str_field(crypto, "mechanism"));
                labelled(ui, "Rotação de chave", str_field(crypto, "rotation"));
                labelled(ui, "Recuperação de chave", str_field(crypto, "recovery"));
            }
            ui.small(format!("id: {}", item.id));
        });
    }
}

fn form_ui(
    ui: &mut egui::Ui,
    form: &mut PlanForm,
    items: &[AnalyticalResultResponse],
    findings: &[AnalyticalResultResponse],
    creating: &mut bool,
    error: Option<&str>,
    (incident_id, backend): (&str, &Backend),
) {
    enum_combo(
        ui,
        "plan_domain",
        "Domínio",
        &mut form.domain,
        &PLAN_DOMAIN_OPTIONS,
    );
    super::field_text(ui, "Título", &mut form.title);
    super::field_text_multiline(ui, "Motivo (por que este item existe)", &mut form.rationale);
    enum_combo(
        ui,
        "plan_priority",
        "Prioridade",
        &mut form.priority,
        &PLAN_PRIORITY_OPTIONS,
    );
    enum_combo(
        ui,
        "plan_effort",
        "Esforço",
        &mut form.effort,
        &PLAN_EFFORT_OPTIONS,
    );
    enum_combo(
        ui,
        "plan_cost",
        "Classe de custo",
        &mut form.cost_class,
        &PLAN_COST_OPTIONS,
    );
    enum_combo(
        ui,
        "plan_phase",
        "Fase",
        &mut form.phase,
        &PLAN_PHASE_OPTIONS,
    );
    super::field_text(ui, "Pré-condição", &mut form.precondition);
    super::field_text(ui, "Pós-condição", &mut form.postcondition);
    super::field_text_multiline(ui, "Como verificar que foi feito", &mut form.verification);
    super::field_text(
        ui,
        "Referência (CIS, NIST; opcional)",
        &mut form.framework_reference,
    );
    super::field_text(
        ui,
        "Regulatório ou política interna (opcional)",
        &mut form.regulatory_reference,
    );
    super::field_text(
        ui,
        "Subcategorias NIST (opcional, separadas por vírgula)",
        &mut form.nist_ids,
    );

    if form.needs_crypto_detail() {
        ui.separator();
        ui.label("Proteção de dados (obrigatório neste domínio)");
        for (label, value) in [
            ("Classe do dado", &mut form.data_class),
            ("Local (onde vive ou trafega)", &mut form.location),
            ("Mecanismo", &mut form.mechanism),
            ("Rotação de chave", &mut form.rotation),
            ("Recuperação de chave", &mut form.recovery),
        ] {
            super::field_text(ui, label, value);
        }
        ui.separator();
    }

    if findings.is_empty() {
        ui.label("Sem achados registrados: o item será uma recomendação geral.");
    } else {
        ui.label("Achados que motivam o item (sem nenhum, é recomendação geral):");
        for finding in findings {
            let mut checked = form.finding_ids.contains(&finding.id);
            if ui
                .checkbox(&mut checked, item_label(finding, "severity"))
                .changed()
            {
                toggle(&mut form.finding_ids, &finding.id, checked);
            }
        }
    }
    if !items.is_empty() {
        ui.label("Itens que precisam vir antes:");
        for item in items {
            let mut checked = form.depends_on.contains(&item.id);
            if ui
                .checkbox(&mut checked, item_label(item, "domain"))
                .changed()
            {
                toggle(&mut form.depends_on, &item.id, checked);
            }
        }
    }

    if let Some(message) = error {
        error_banner(ui, message);
    }
    if ui
        .add_enabled(!*creating, egui::Button::new("Registrar item"))
        .clicked()
    {
        *creating = true;
        backend.send(Command::CreatePlanItem {
            incident_id: incident_id.to_string(),
            input: build_input(form),
        });
    }
    if *creating {
        ui.spinner();
    }
}

/// Os detalhes de proteção só seguem nos domínios que os exigem: o que
/// sobrou digitado antes de trocar de domínio não vai junto.
fn build_input(form: &PlanForm) -> PlanItemInput {
    PlanItemInput {
        domain: form.domain.clone(),
        title: form.title.clone(),
        rationale: form.rationale.clone(),
        priority: form.priority.clone(),
        effort: form.effort.clone(),
        cost_class: form.cost_class.clone(),
        phase: form.phase.clone(),
        precondition: form.precondition.clone(),
        postcondition: form.postcondition.clone(),
        verification: form.verification.clone(),
        framework_reference: non_empty(&form.framework_reference),
        regulatory_reference: non_empty(&form.regulatory_reference),
        nist_subcategory_ids: split_ids(&form.nist_ids),
        finding_ids: form.finding_ids.iter().cloned().collect(),
        depends_on: form.depends_on.iter().cloned().collect(),
        crypto: form.needs_crypto_detail().then(|| CryptoInput {
            data_class: form.data_class.clone(),
            location: form.location.clone(),
            mechanism: form.mechanism.clone(),
            rotation: form.rotation.clone(),
            recovery: form.recovery.clone(),
        }),
        supersedes: None,
    }
}

fn item_label(result: &AnalyticalResultResponse, tag_key: &str) -> String {
    format!(
        "{} ({})",
        str_field(&result.payload, "title"),
        str_field(&result.payload, tag_key),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::{
        PLAN_COST_OPTIONS, PLAN_DOMAIN_OPTIONS, PLAN_EFFORT_OPTIONS, PLAN_PHASE_OPTIONS,
        PLAN_PRIORITY_OPTIONS,
    };
    use crate::persistence::plan_items::{CostClass, PlanEffort, PlanPhase, PlanPriority};

    /// As listas da interface não podem divergir do vocabulário do modelo.
    #[test]
    fn option_lists_match_the_model_vocabulary() {
        let domains: Vec<&str> = PlanDomain::ALL
            .into_iter()
            .map(PlanDomain::as_str)
            .collect();
        assert_eq!(domains, PLAN_DOMAIN_OPTIONS);
        let priorities: Vec<&str> = [PlanPriority::P1, PlanPriority::P2, PlanPriority::P3]
            .into_iter()
            .map(PlanPriority::as_str)
            .collect();
        assert_eq!(priorities, PLAN_PRIORITY_OPTIONS);
        let efforts: Vec<&str> = [PlanEffort::Small, PlanEffort::Medium, PlanEffort::Large]
            .into_iter()
            .map(PlanEffort::as_str)
            .collect();
        assert_eq!(efforts, PLAN_EFFORT_OPTIONS);
        let costs: Vec<&str> = [
            CostClass::NoExtraCost,
            CostClass::Low,
            CostClass::Medium,
            CostClass::High,
        ]
        .into_iter()
        .map(CostClass::as_str)
        .collect();
        assert_eq!(costs, PLAN_COST_OPTIONS);
        let phases: Vec<&str> = [
            PlanPhase::Immediate,
            PlanPhase::ShortTerm,
            PlanPhase::Strategic,
        ]
        .into_iter()
        .map(PlanPhase::as_str)
        .collect();
        assert_eq!(phases, PLAN_PHASE_OPTIONS);
    }

    #[test]
    fn protection_details_only_travel_with_the_domains_that_require_them() {
        let mut form = PlanForm {
            domain: "OS_HARDENING".into(),
            mechanism: "AES-256".into(),
            ..PlanForm::default()
        };
        assert!(
            build_input(&form).crypto.is_none(),
            "sobra digitada de outro domínio"
        );
        form.domain = "GESTAO_CHAVES".into();
        let crypto = build_input(&form).crypto.expect("crypto domain keeps them");
        assert_eq!(crypto.mechanism, "AES-256");
    }

    #[test]
    fn helpers_normalise_ids_and_blank_optionals() {
        assert_eq!(
            split_ids(" pr.aa-03; de.cm-01 "),
            vec!["PR.AA-03", "DE.CM-01"]
        );
        assert_eq!(non_empty("  "), None);
        assert_eq!(non_empty(" x "), Some("x".to_string()));
    }
}

/// Testes da aba com `egui_kittest` (ADR-0022).
#[cfg(test)]
mod ui_tests {
    use super::PlanState;
    use crate::gui::Loadable;
    use crate::gui::backend::{Backend, Command};
    use crate::gui::test_harness::drain;
    use crate::persistence::plan_items::PlanDomain;
    use crate::persistence::plan_items::domain_coverage;
    use crate::service::AnalyticalResultResponse;
    use crate::service::plan::PlanView;
    use egui_kittest::{Harness, kittest::Queryable};

    type State = (PlanState, Backend);

    fn result(id: &str, result_type: &str, payload: serde_json::Value) -> AnalyticalResultResponse {
        AnalyticalResultResponse {
            id: id.into(),
            result_type: result_type.into(),
            correlation_key: format!("key-{id}"),
            evidence_id: None,
            incident_id: Some("inc-1".into()),
            analysis_job_id: None,
            rule_or_model_id: None,
            rule_or_model_version: None,
            confidence: None,
            payload,
            provenance: serde_json::Value::Null,
            supersedes: None,
            created_at: "2026-10-14T00:00:00Z".into(),
        }
    }

    fn loaded_state() -> PlanState {
        let items = vec![
            result(
                "p-1",
                "PLAN_ITEM",
                serde_json::json!({
                    "domain": "AUTENTICACAO_ACESSO",
                    "title": "MFA no acesso remoto",
                    "priority": "P1", "effort": "S", "cost_class": "NONE", "phase": "IMMEDIATE",
                    "rationale": "acesso inicial por RDP",
                    "verification": "varredura externa sem 3389",
                    "regulatory_reference": "Política de acesso remoto",
                    "support_kind": "FINDING_BACKED",
                    "finding_ids": ["f-1"],
                    "crypto": null,
                }),
            ),
            result(
                "p-2",
                "PLAN_ITEM",
                serde_json::json!({
                    "domain": "CRIPTO_REPOUSO",
                    "title": "Cifrar o volume do banco",
                    "priority": "P2", "effort": "M", "cost_class": "LOW", "phase": "SHORT_TERM",
                    "verification": "volume reporta criptografia ativa",
                    "support_kind": "GENERAL_RECOMMENDATION",
                    "finding_ids": [],
                    "crypto": {
                        "data_class": "dados pessoais", "location": "DB01",
                        "mechanism": "AES-256", "rotation": "anual", "recovery": "cofre",
                    },
                }),
            ),
        ];
        let coverage = domain_coverage(&[
            (PlanDomain::AccessAuthentication, false),
            (PlanDomain::CryptoAtRest, true),
        ]);
        PlanState {
            loaded_for: Some("inc-1".into()),
            view: Loadable::Ready(PlanView {
                items,
                findings: vec![],
                coverage,
            }),
            ..PlanState::default()
        }
    }

    /// Janela alta: a aba é longa e um clique fora da área visível não chega
    /// ao botão.
    fn harness(state: PlanState, backend: Backend) -> Harness<'static, State> {
        crate::gui::test_harness::builder()
            .with_size(egui::vec2(900.0, 3000.0))
            .build_ui_state(
                |ui, (state, backend): &mut State| state.ui(ui, "inc-1", backend),
                (state, backend),
            )
    }

    #[test]
    fn loaded_view_shows_coverage_items_general_marker_and_protection_details() {
        let (backend, mut commands) = Backend::for_test();
        let mut harness = harness(loaded_state(), backend);
        harness.run();

        // Cobertura: domínio sem itens aparece (com 0), não é omitido.
        harness.get_by_label("Redes e dispositivos");
        harness.get_by_label("Políticas de resposta a incidentes");
        // Itens e cabeçalhos por domínio.
        harness.get_by_label("▸ Autenticação e controle de acesso");
        harness.get_by_label("MFA no acesso remoto");
        harness.get_by_label("P1 · esforço S · custo NONE · IMMEDIATE");
        harness.get_by_label("Motivado por 1 achado(s)");
        harness.get_by_label("Regulatório / política interna: Política de acesso remoto");
        // Item sem achado é marcado como recomendação geral.
        harness.get_by_label("Recomendação geral (sem achado de suporte)");
        // Detalhes de proteção do Entregável 5.
        harness.get_by_label("Mecanismo: AES-256");
        harness.get_by_label("Recuperação de chave: cofre");
        assert!(drain(&mut commands).is_empty());
    }

    #[test]
    fn opening_a_tab_for_a_new_incident_requests_the_view_once() {
        let (backend, mut commands) = Backend::for_test();
        let mut harness = harness(PlanState::default(), backend);
        // Carregando mostra um spinner, que repinta sem parar: `step`, não `run`.
        harness.step();
        harness.step();

        let sent = drain(&mut commands);
        assert_eq!(sent.len(), 1, "um único pedido: {sent:?}");
        assert!(matches!(&sent[0], Command::LoadPlanView(id) if id == "inc-1"));
    }

    #[test]
    fn registering_an_item_in_a_crypto_domain_sends_the_protection_details() {
        let (backend, mut commands) = Backend::for_test();
        let mut state = loaded_state();
        state.form.domain = "GESTAO_CHAVES".into();
        state.form.title = " Rotacionar chaves ".into();
        state.form.rationale = "chaves nunca rotacionadas".into();
        state.form.priority = "P2".into();
        state.form.effort = "M".into();
        state.form.cost_class = "LOW".into();
        state.form.phase = "SHORT_TERM".into();
        state.form.verification = "inventário de chaves com data".into();
        state.form.nist_ids = "pr.ds-01".into();
        state.form.data_class = "dados pessoais".into();
        state.form.location = "KMS".into();
        state.form.mechanism = "AES-256".into();
        state.form.rotation = "anual".into();
        state.form.recovery = "duas pessoas".into();
        let mut harness = harness(state, backend);
        harness.run();

        harness.get_by_label("Novo item de plano").click();
        harness.run();
        harness.get_by_label("Registrar item").click();
        harness.step();
        harness.step();

        let sent = drain(&mut commands);
        assert_eq!(sent.len(), 1, "um único comando: {sent:?}");
        match &sent[0] {
            Command::CreatePlanItem { incident_id, input } => {
                assert_eq!(incident_id, "inc-1");
                assert_eq!(input.domain, "GESTAO_CHAVES");
                assert_eq!(input.priority, "P2");
                assert_eq!(input.cost_class, "LOW");
                assert_eq!(input.nist_subcategory_ids, vec!["PR.DS-01"]);
                let crypto = input.crypto.as_ref().expect("protection details sent");
                assert_eq!(crypto.location, "KMS");
                assert_eq!(crypto.recovery, "duas pessoas");
            }
            other => panic!("comando inesperado: {other:?}"),
        }
    }

    #[test]
    fn a_failed_creation_shows_the_reason_and_keeps_the_form() {
        let (backend, _commands) = Backend::for_test();
        let mut state = loaded_state();
        state.form.title = "x".into();
        state.on_created(
            "inc-1",
            Err("a plan item requires a rationale".into()),
            &backend,
        );
        assert_eq!(state.form.title, "x", "o formulário não é apagado");
        let mut harness = harness(state, backend);
        harness.run();
        harness.get_by_label("Novo item de plano").click();
        harness.run();
        harness.get_by_label("Erro: a plan item requires a rationale");
    }
}
