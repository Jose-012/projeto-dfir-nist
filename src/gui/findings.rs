//! Aba "Achados": vulnerabilidades e condições exploradas, conclusões de
//! causa (causa-raiz, causa provável, fatores contribuintes) e a matriz de
//! risco severidade × confiança (F1-25, RF-051).
//!
//! A interface só coleta e mostra; as regras (achado exige evidência,
//! causa-raiz exige confiança alta, toda causa cita um achado) vivem em
//! `persistence/findings.rs` e chegam aqui como mensagem de erro.

use super::backend::{Backend, Command};
use super::{
    CAUSE_LEVEL_OPTIONS, CONFIDENCE_OPTIONS, FINDING_KIND_OPTIONS, FINDING_SEVERITY_OPTIONS,
    Loadable, enum_combo, error_banner, non_empty, split_ids, str_field, string_list, toggle,
};
use crate::persistence::findings::RiskCell;
use crate::service::AnalyticalResultResponse;
use crate::service::findings::{FindingInput, FindingsView, RootCauseInput};
use std::collections::BTreeSet;

#[derive(Default)]
struct FindingForm {
    kind: String,
    title: String,
    description: String,
    severity: String,
    confidence: String,
    basis: String,
    evidence_id: String,
    reference: String,
    nist_ids: String,
    attack_ids: BTreeSet<String>,
    analyst_note: String,
}

#[derive(Default)]
struct CauseForm {
    level: String,
    statement: String,
    basis: String,
    confidence: String,
    /// Uma lacuna por linha.
    gaps: String,
    finding_ids: BTreeSet<String>,
}

#[derive(Default)]
pub struct FindingsState {
    loaded_for: Option<String>,
    view: Loadable<FindingsView>,
    finding_form: FindingForm,
    cause_form: CauseForm,
    creating_finding: bool,
    creating_cause: bool,
    finding_error: Option<String>,
    cause_error: Option<String>,
}

impl FindingsState {
    fn ensure_loaded(&mut self, incident_id: &str, backend: &Backend) {
        if self.loaded_for.as_deref() == Some(incident_id) {
            return;
        }
        self.loaded_for = Some(incident_id.to_string());
        self.view = Loadable::Loading;
        backend.send(Command::LoadFindingsView(incident_id.to_string()));
    }

    pub fn on_loaded(&mut self, incident_id: &str, result: Result<FindingsView, String>) {
        if self.loaded_for.as_deref() != Some(incident_id) {
            return;
        }
        self.view = match result {
            Ok(view) => Loadable::Ready(view),
            Err(message) => Loadable::Failed(message),
        };
    }

    pub fn on_finding_created(
        &mut self,
        incident_id: &str,
        result: Result<String, String>,
        backend: &Backend,
    ) {
        self.creating_finding = false;
        match result {
            Ok(_) => {
                self.finding_error = None;
                self.finding_form = FindingForm::default();
                backend.send(Command::LoadFindingsView(incident_id.to_string()));
            }
            Err(message) => self.finding_error = Some(message),
        }
    }

    pub fn on_cause_created(
        &mut self,
        incident_id: &str,
        result: Result<String, String>,
        backend: &Backend,
    ) {
        self.creating_cause = false;
        match result {
            Ok(_) => {
                self.cause_error = None;
                self.cause_form = CauseForm::default();
                backend.send(Command::LoadFindingsView(incident_id.to_string()));
            }
            Err(message) => self.cause_error = Some(message),
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, incident_id: &str, backend: &Backend) {
        self.ensure_loaded(incident_id, backend);

        self.view.show(ui, |ui, view| {
            ui.heading("Matriz de risco (severidade × confiança)");
            show_matrix(ui, &view.risk_matrix);
            ui.separator();
            ui.heading("Achados");
            show_findings(ui, &view.findings);
            ui.separator();
            ui.heading("Conclusões de causa");
            show_causes(ui, &view.root_causes);
        });
        ui.separator();

        let Self {
            view,
            finding_form,
            cause_form,
            creating_finding,
            creating_cause,
            finding_error,
            cause_error,
            ..
        } = self;
        let (attack_mappings, findings): (
            &[AnalyticalResultResponse],
            &[AnalyticalResultResponse],
        ) = match view.ready() {
            Some(view) => (&view.attack_mappings, &view.findings),
            None => (&[], &[]),
        };

        ui.collapsing("Novo achado", |ui| {
            finding_form_ui(
                ui,
                finding_form,
                attack_mappings,
                creating_finding,
                finding_error.as_deref(),
                incident_id,
                backend,
            );
        });
        ui.collapsing("Nova conclusão de causa", |ui| {
            cause_form_ui(
                ui,
                cause_form,
                findings,
                creating_cause,
                cause_error.as_deref(),
                incident_id,
                backend,
            );
        });
    }
}

fn show_matrix(ui: &mut egui::Ui, cells: &[RiskCell]) {
    if cells.is_empty() {
        ui.label("Sem achados registrados: a matriz só conta o que existe.");
        return;
    }
    egui::Grid::new("risk_matrix").striped(true).show(ui, |ui| {
        ui.strong("Severidade");
        ui.strong("Confiança");
        ui.strong("Achados");
        ui.end_row();
        for cell in cells {
            ui.label(&cell.severity);
            ui.label(&cell.confidence);
            ui.label(cell.count.to_string());
            ui.end_row();
        }
    });
}

fn show_findings(ui: &mut egui::Ui, findings: &[AnalyticalResultResponse]) {
    if findings.is_empty() {
        ui.label("Nenhum achado ainda.");
        return;
    }
    for finding in findings {
        let payload = &finding.payload;
        ui.group(|ui| {
            ui.strong(str_field(payload, "title"));
            ui.label(format!(
                "{} · confiança {} · {}",
                str_field(payload, "severity"),
                finding.confidence.as_deref().unwrap_or("—"),
                str_field(payload, "finding_kind"),
            ));
            let description = str_field(payload, "description");
            if !description.is_empty() {
                ui.label(description);
            }
            let basis = str_field(payload, "basis");
            if !basis.is_empty() {
                ui.label(format!("Base: {basis}"));
            }
            let nist = string_list(payload, "nist_subcategories");
            if !nist.is_empty() {
                ui.small(format!("NIST: {}", nist.join(", ")));
            }
            ui.small(format!("id: {}", finding.id));
        });
    }
}

fn show_causes(ui: &mut egui::Ui, causes: &[AnalyticalResultResponse]) {
    if causes.is_empty() {
        ui.label("Nenhuma conclusão de causa ainda.");
        return;
    }
    for cause in causes {
        let payload = &cause.payload;
        ui.group(|ui| {
            ui.strong(format!(
                "{} · confiança {}",
                str_field(payload, "level"),
                cause.confidence.as_deref().unwrap_or("—"),
            ));
            ui.label(str_field(payload, "statement"));
            let basis = str_field(payload, "basis");
            if !basis.is_empty() {
                ui.label(format!("Base: {basis}"));
            }
            let gaps = string_list(payload, "gaps");
            if gaps.is_empty() {
                ui.label("Lacunas: nenhuma declarada.");
            } else {
                ui.label("Lacunas:");
                for gap in gaps {
                    ui.label(format!("• {gap}"));
                }
            }
            ui.small(format!("id: {}", cause.id));
        });
    }
}

fn finding_form_ui(
    ui: &mut egui::Ui,
    form: &mut FindingForm,
    attack_mappings: &[AnalyticalResultResponse],
    creating: &mut bool,
    error: Option<&str>,
    incident_id: &str,
    backend: &Backend,
) {
    enum_combo(
        ui,
        "finding_kind",
        "Tipo",
        &mut form.kind,
        &FINDING_KIND_OPTIONS,
    );
    super::field_text(ui, "Título", &mut form.title);
    super::field_text_multiline(ui, "Descrição", &mut form.description);
    enum_combo(
        ui,
        "finding_severity",
        "Severidade",
        &mut form.severity,
        &FINDING_SEVERITY_OPTIONS,
    );
    enum_combo(
        ui,
        "finding_confidence",
        "Confiança",
        &mut form.confidence,
        &CONFIDENCE_OPTIONS,
    );
    super::field_text_multiline(
        ui,
        "Base (onde está a evidência: log, evento, host)",
        &mut form.basis,
    );
    super::field_text(ui, "ID da evidência (opcional)", &mut form.evidence_id);
    super::field_text(
        ui,
        "Referência externa (opcional, ex.: CVE-2024-1234)",
        &mut form.reference,
    );
    super::field_text(
        ui,
        "Subcategorias NIST (opcional, separadas por vírgula)",
        &mut form.nist_ids,
    );
    if !attack_mappings.is_empty() {
        ui.label("Mapeamentos ATT&CK ligados a este achado:");
        for mapping in attack_mappings {
            let mut checked = form.attack_ids.contains(&mapping.id);
            if ui.checkbox(&mut checked, attack_label(mapping)).changed() {
                toggle(&mut form.attack_ids, &mapping.id, checked);
            }
        }
    }
    super::field_text(ui, "Nota do analista (opcional)", &mut form.analyst_note);
    if let Some(message) = error {
        error_banner(ui, message);
    }
    if ui
        .add_enabled(!*creating, egui::Button::new("Registrar achado"))
        .clicked()
    {
        *creating = true;
        backend.send(Command::CreateFinding {
            incident_id: incident_id.to_string(),
            input: FindingInput {
                evidence_id: non_empty(&form.evidence_id),
                kind: form.kind.clone(),
                title: form.title.clone(),
                description: form.description.clone(),
                severity: form.severity.clone(),
                basis: form.basis.clone(),
                reference: non_empty(&form.reference),
                nist_subcategory_ids: split_ids(&form.nist_ids),
                attack_mapping_ids: form.attack_ids.iter().cloned().collect(),
                supporting_result_ids: Vec::new(),
                confidence: form.confidence.clone(),
                analyst_note: non_empty(&form.analyst_note),
                supersedes: None,
            },
        });
    }
    if *creating {
        ui.spinner();
    }
}

fn cause_form_ui(
    ui: &mut egui::Ui,
    form: &mut CauseForm,
    findings: &[AnalyticalResultResponse],
    creating: &mut bool,
    error: Option<&str>,
    incident_id: &str,
    backend: &Backend,
) {
    enum_combo(
        ui,
        "cause_level",
        "Nível",
        &mut form.level,
        &CAUSE_LEVEL_OPTIONS,
    );
    super::field_text_multiline(ui, "Conclusão (uma ou duas frases)", &mut form.statement);
    super::field_text_multiline(ui, "Base", &mut form.basis);
    enum_combo(
        ui,
        "cause_confidence",
        "Confiança",
        &mut form.confidence,
        &CONFIDENCE_OPTIONS,
    );
    super::field_text_multiline(
        ui,
        "Lacunas (uma por linha: o que não se sabe ou não foi verificado)",
        &mut form.gaps,
    );
    if findings.is_empty() {
        ui.label("Registre ao menos um achado: toda conclusão de causa cita um.");
    } else {
        ui.label("Achados que sustentam a conclusão:");
        for finding in findings {
            let mut checked = form.finding_ids.contains(&finding.id);
            if ui.checkbox(&mut checked, finding_label(finding)).changed() {
                toggle(&mut form.finding_ids, &finding.id, checked);
            }
        }
    }
    if let Some(message) = error {
        error_banner(ui, message);
    }
    if ui
        .add_enabled(!*creating, egui::Button::new("Registrar conclusão"))
        .clicked()
    {
        *creating = true;
        backend.send(Command::CreateRootCause {
            incident_id: incident_id.to_string(),
            input: RootCauseInput {
                level: form.level.clone(),
                statement: form.statement.clone(),
                basis: form.basis.clone(),
                gaps: form
                    .gaps
                    .lines()
                    .map(str::trim)
                    .filter(|line| !line.is_empty())
                    .map(str::to_string)
                    .collect(),
                finding_ids: form.finding_ids.iter().cloned().collect(),
                confidence: form.confidence.clone(),
                supersedes: None,
            },
        });
    }
    if *creating {
        ui.spinner();
    }
}

fn attack_label(mapping: &AnalyticalResultResponse) -> String {
    format!(
        "{} — {}",
        str_field(&mapping.payload, "technique_id"),
        str_field(&mapping.payload, "tactic"),
    )
}

fn finding_label(finding: &AnalyticalResultResponse) -> String {
    format!(
        "{} ({}, {})",
        str_field(&finding.payload, "title"),
        str_field(&finding.payload, "severity"),
        finding.confidence.as_deref().unwrap_or("—"),
    )
}

#[cfg(test)]
mod tests {
    use crate::gui::{non_empty, split_ids};

    #[test]
    fn nist_ids_are_split_and_uppercased() {
        assert_eq!(
            split_ids(" pr.aa-03, DE.CM-01;gv.oc-01  "),
            vec!["PR.AA-03", "DE.CM-01", "GV.OC-01"]
        );
        assert!(split_ids("  , ; ").is_empty());
    }

    #[test]
    fn blank_optional_fields_become_none() {
        assert_eq!(non_empty("   "), None);
        assert_eq!(non_empty(" x "), Some("x".to_string()));
    }
}

/// Testes da aba com `egui_kittest` (ADR-0022): renderizam sem janela e
/// operam pela árvore de acessibilidade.
#[cfg(test)]
mod ui_tests {
    use super::FindingsState;
    use crate::gui::Loadable;
    use crate::gui::backend::{Backend, Command};
    use crate::gui::test_harness::drain;
    use crate::persistence::findings::RiskCell;
    use crate::service::AnalyticalResultResponse;
    use crate::service::findings::FindingsView;
    use egui_kittest::{Harness, kittest::Queryable};

    type State = (FindingsState, Backend);

    fn result(
        id: &str,
        result_type: &str,
        confidence: &str,
        payload: serde_json::Value,
    ) -> AnalyticalResultResponse {
        AnalyticalResultResponse {
            id: id.into(),
            result_type: result_type.into(),
            correlation_key: format!("key-{id}"),
            evidence_id: None,
            incident_id: Some("inc-1".into()),
            analysis_job_id: None,
            rule_or_model_id: None,
            rule_or_model_version: None,
            confidence: Some(confidence.into()),
            payload,
            provenance: serde_json::Value::Null,
            supersedes: None,
            created_at: "2026-10-14T00:00:00Z".into(),
        }
    }

    fn loaded_state() -> FindingsState {
        FindingsState {
            loaded_for: Some("inc-1".into()),
            view: Loadable::Ready(FindingsView {
                findings: vec![result(
                    "f-1",
                    "FINDING",
                    "HIGH",
                    serde_json::json!({
                        "title": "RDP exposto sem MFA",
                        "severity": "CRITICAL",
                        "finding_kind": "MISSING_CONTROL",
                        "description": "",
                        "basis": "EVTX 4625 em massa",
                        "nist_subcategories": ["PR.AA-03"],
                    }),
                )],
                root_causes: vec![result(
                    "c-1",
                    "ROOT_CAUSE",
                    "MEDIUM",
                    serde_json::json!({
                        "level": "PROBABLE_CAUSE",
                        "statement": "Acesso inicial por RDP",
                        "basis": "",
                        "gaps": ["sem logs do firewall"],
                        "gap_status": "DECLARED",
                    }),
                )],
                attack_mappings: vec![],
                risk_matrix: vec![RiskCell {
                    severity: "CRITICAL".into(),
                    confidence: "HIGH".into(),
                    count: 1,
                }],
            }),
            ..FindingsState::default()
        }
    }

    /// Janela alta: a aba é longa (lista + formulários) e um clique fora da
    /// área visível não chega ao botão.
    fn harness(state: FindingsState, backend: Backend) -> Harness<'static, State> {
        crate::gui::test_harness::builder()
            .with_size(egui::vec2(900.0, 2400.0))
            .build_ui_state(
                |ui, (state, backend): &mut State| state.ui(ui, "inc-1", backend),
                (state, backend),
            )
    }

    #[test]
    fn loaded_view_shows_finding_cause_gaps_and_matrix() {
        let (backend, mut commands) = Backend::for_test();
        let mut harness = harness(loaded_state(), backend);
        harness.run();

        harness.get_by_label("RDP exposto sem MFA");
        harness.get_by_label("CRITICAL · confiança HIGH · MISSING_CONTROL");
        harness.get_by_label("PROBABLE_CAUSE · confiança MEDIUM");
        harness.get_by_label("• sem logs do firewall");
        harness.get_by_label("Severidade");
        assert!(
            drain(&mut commands).is_empty(),
            "com os dados em memória nada é pedido de novo"
        );
    }

    #[test]
    fn opening_a_tab_for_a_new_incident_requests_the_view_once() {
        let (backend, mut commands) = Backend::for_test();
        let mut harness = harness(FindingsState::default(), backend);
        // Carregando mostra um spinner, que repinta sem parar: `step`, não `run`.
        harness.step();
        harness.step();

        let sent = drain(&mut commands);
        assert_eq!(sent.len(), 1, "um único pedido: {sent:?}");
        assert!(matches!(&sent[0], Command::LoadFindingsView(id) if id == "inc-1"));
    }

    #[test]
    fn registering_a_finding_sends_the_typed_values() {
        let (backend, mut commands) = Backend::for_test();
        let mut state = loaded_state();
        state.finding_form.kind = "MISCONFIGURATION".into();
        state.finding_form.title = "  SMB exposto  ".into();
        state.finding_form.severity = "HIGH".into();
        state.finding_form.confidence = "MEDIUM".into();
        state.finding_form.basis = "porta 445 aberta".into();
        state.finding_form.nist_ids = "pr.ir-01, de.cm-01".into();
        let mut harness = harness(state, backend);
        harness.run();

        harness.get_by_label("Novo achado").click();
        harness.run();
        harness.get_by_label("Registrar achado").click();
        harness.step();
        harness.step();

        let sent = drain(&mut commands);
        assert_eq!(sent.len(), 1, "um único comando: {sent:?}");
        match &sent[0] {
            Command::CreateFinding { incident_id, input } => {
                assert_eq!(incident_id, "inc-1");
                assert_eq!(input.kind, "MISCONFIGURATION");
                assert_eq!(input.severity, "HIGH");
                assert_eq!(input.basis, "porta 445 aberta");
                assert_eq!(input.nist_subcategory_ids, vec!["PR.IR-01", "DE.CM-01"]);
                assert!(input.evidence_id.is_none());
            }
            other => panic!("comando inesperado: {other:?}"),
        }
    }

    #[test]
    fn a_failed_creation_shows_the_reason_and_keeps_the_form() {
        let (backend, _commands) = Backend::for_test();
        let mut state = loaded_state();
        state.finding_form.title = "x".into();
        state.on_finding_created("inc-1", Err("a finding requires evidence".into()), &backend);
        assert_eq!(state.finding_form.title, "x", "o formulário não é apagado");
        let mut harness = harness(state, backend);
        harness.run();
        harness.get_by_label("Novo achado").click();
        harness.run();
        harness.get_by_label("Erro: a finding requires evidence");
    }
}
