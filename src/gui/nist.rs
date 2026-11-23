//! Aba "NIST CSF": criação de profiles CURRENT/TARGET (com entradas por
//! subcategoria) e execução/visualização da Gap Analysis (F1-03).

use super::backend::{Backend, Command};
use super::{
    IMPLEMENTATION_STATUS_OPTIONS, Loadable, TIER_OPTIONS, enum_combo, non_empty, str_field,
};
use crate::persistence::nist_functions::GapAnalysisState;
use crate::persistence::nist_profiles::ProfileType;
use crate::service::AnalyticalResultResponse;
use crate::service::nist::{
    NistFunctionView, ProfileEntryInput, ProfileHistoryView, ProfileSummary,
};

#[derive(Default, Clone)]
struct EntryForm {
    subcategory_id: String,
    subcategory_label: String,
    implementation_status: String,
    justification: String,
    evidence_reference: String,
    confidence: String,
}

struct ProfileForm {
    profile_type: String,
    version_label: String,
    source: String,
    tier: String,
    tier_justification: String,
    entries: Vec<EntryForm>,
}

impl Default for ProfileForm {
    fn default() -> Self {
        Self {
            profile_type: ProfileType::Current.as_str().to_string(),
            version_label: String::new(),
            source: String::new(),
            tier: String::new(),
            tier_justification: String::new(),
            entries: vec![EntryForm::default()],
        }
    }
}

fn gap_ui(ui: &mut egui::Ui, gap: &AnalyticalResultResponse) {
    let payload = &gap.payload;
    ui.group(|ui| {
        ui.set_width(ui.available_width());
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);

        let id = str_field(payload, "subcategory_id");
        let label = str_field(payload, "subcategory_label");
        if label.is_empty() {
            ui.strong(id);
        } else {
            ui.strong(format!("{id} — {label}"));
        }

        if str_field(payload, "status") == "CLOSED" {
            ui.label("Situação: CLOSED");
            ui.label(format!(
                "Motivo do fechamento: {}",
                str_field(payload, "closed_reason")
            ));
            ui.label(format!(
                "Status atual na resolução: {}",
                str_field(payload, "resolved_current_status")
            ));
        } else {
            ui.label(format!(
                "Status atual: {}",
                str_field(payload, "current_status")
            ));
            ui.label(format!(
                "Status alvo: {}",
                str_field(payload, "target_status")
            ));
            ui.label(format!("Prioridade: {}", str_field(payload, "priority")));
            ui.label(format!(
                "Nível de risco: {}",
                str_field(payload, "risk_level")
            ));
        }
        ui.label(format!(
            "Confiança: {}",
            gap.confidence.as_deref().unwrap_or("não informada")
        ));
    });
}

pub struct NistState {
    loaded_for: Option<String>,
    current: Loadable<ProfileHistoryView>,
    target: Loadable<ProfileHistoryView>,
    current_version: Option<usize>,
    target_version: Option<usize>,
    gaps: Loadable<Vec<AnalyticalResultResponse>>,
    functions: Loadable<NistFunctionView>,
    form: ProfileForm,
    creating: bool,
    create_error: Option<String>,
    running_gap: bool,
    gap_error: Option<String>,
}

impl Default for NistState {
    fn default() -> Self {
        Self {
            loaded_for: None,
            current: Loadable::Idle,
            target: Loadable::Idle,
            current_version: None,
            target_version: None,
            gaps: Loadable::Idle,
            functions: Loadable::Idle,
            form: ProfileForm::default(),
            creating: false,
            create_error: None,
            running_gap: false,
            gap_error: None,
        }
    }
}

impl NistState {
    fn ensure_loaded(&mut self, incident_id: &str, backend: &Backend) {
        if self.loaded_for.as_deref() == Some(incident_id) {
            return;
        }
        self.loaded_for = Some(incident_id.to_string());
        self.current = Loadable::Loading;
        self.target = Loadable::Loading;
        self.current_version = None;
        self.target_version = None;
        self.gaps = Loadable::Loading;
        self.functions = Loadable::Loading;
        backend.send(Command::LoadNistFunctionView(incident_id.to_string()));
        backend.send(Command::NistProfileHistory {
            incident_id: incident_id.to_string(),
            profile_type: ProfileType::Current,
        });
        backend.send(Command::NistProfileHistory {
            incident_id: incident_id.to_string(),
            profile_type: ProfileType::Target,
        });
        backend.send(Command::CurrentGapAnalysis(incident_id.to_string()));
    }

    pub fn on_history(&mut self, incident_id: &str, result: Result<ProfileHistoryView, String>) {
        if self.loaded_for.as_deref() != Some(incident_id) {
            return;
        }
        let loadable = match result {
            Ok(history) => Loadable::Ready(history),
            Err(message) => Loadable::Failed(message),
        };
        match &loadable {
            Loadable::Ready(history) if history.profile_type == ProfileType::Current => {
                self.current_version = history.versions.len().checked_sub(1);
                self.current = loadable;
            }
            Loadable::Ready(history) => {
                self.target_version = history.versions.len().checked_sub(1);
                self.target = loadable;
            }
            _ => {
                // Erro genérico: não sabemos a qual dos dois pertence: aplica
                // a quem ainda estiver carregando.
                if self.current.is_loading() {
                    self.current = loadable;
                } else {
                    self.target = loadable;
                }
            }
        }
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
                self.create_error = None;
                self.form = ProfileForm::default();
                self.loaded_for = None;
                self.ensure_loaded(incident_id, backend);
            }
            Err(message) => self.create_error = Some(message),
        }
    }

    pub fn on_gap_run(
        &mut self,
        incident_id: &str,
        result: Result<Vec<String>, String>,
        backend: &Backend,
    ) {
        self.running_gap = false;
        match result {
            Ok(_) => {
                self.gap_error = None;
                backend.send(Command::CurrentGapAnalysis(incident_id.to_string()));
                backend.send(Command::LoadNistFunctionView(incident_id.to_string()));
            }
            Err(message) => self.gap_error = Some(message),
        }
    }

    pub fn on_gap_loaded(
        &mut self,
        incident_id: &str,
        result: Result<Vec<AnalyticalResultResponse>, String>,
    ) {
        if self.loaded_for.as_deref() != Some(incident_id) {
            return;
        }
        self.gaps = match result {
            Ok(items) => Loadable::Ready(items),
            Err(message) => Loadable::Failed(message),
        };
    }

    pub fn on_functions_loaded(
        &mut self,
        incident_id: &str,
        result: Result<NistFunctionView, String>,
    ) {
        if self.loaded_for.as_deref() != Some(incident_id) {
            return;
        }
        self.functions = match result {
            Ok(view) => Loadable::Ready(view),
            Err(message) => Loadable::Failed(message),
        };
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, incident_id: &str, backend: &Backend) {
        self.ensure_loaded(incident_id, backend);

        function_section(ui, &self.functions);
        ui.separator();

        ui.columns(2, |columns| {
            profile_column(
                &mut columns[0],
                "current",
                "Current Profile",
                &self.current,
                &mut self.current_version,
            );
            profile_column(
                &mut columns[1],
                "target",
                "Target Profile",
                &self.target,
                &mut self.target_version,
            );
        });

        ui.separator();
        ui.horizontal(|ui| {
            ui.heading("Gap Analysis");
            if ui
                .add_enabled(!self.running_gap, egui::Button::new("Recalcular gaps"))
                .clicked()
            {
                self.running_gap = true;
                backend.send(Command::RunGapAnalysis(incident_id.to_string()));
            }
            if self.running_gap {
                ui.spinner();
            }
        });
        if let Some(message) = &self.gap_error {
            super::error_banner(ui, message);
        }
        self.gaps.show(ui, |ui, items| {
            if items.is_empty() {
                ui.label(
                    "Nenhum gap calculado ainda (crie um profile CURRENT e um TARGET e recalcule).",
                );
                return;
            }
            for gap in items {
                gap_ui(ui, gap);
            }
        });

        ui.separator();
        ui.collapsing("Novo profile (CURRENT ou TARGET)", |ui| {
            self.profile_form(ui, incident_id, backend);
        });
    }

    fn profile_form(&mut self, ui: &mut egui::Ui, incident_id: &str, backend: &Backend) {
        let form = &mut self.form;
        enum_combo(
            ui,
            "nist_profile_type",
            "Tipo de profile",
            &mut form.profile_type,
            &[ProfileType::Current.as_str(), ProfileType::Target.as_str()],
        );
        super::field_text(ui, "Rótulo da versão", &mut form.version_label);
        super::field_text(ui, "Fonte", &mut form.source);
        enum_combo(
            ui,
            "nist_tier",
            "Tier (opcional)",
            &mut form.tier,
            &TIER_OPTIONS,
        );
        super::field_text(ui, "Justificativa do tier", &mut form.tier_justification);

        ui.label("Entradas (uma por subcategoria avaliada, ex.: GV.OC-01):");
        let mut remove_index = None;
        for (index, entry) in form.entries.iter_mut().enumerate() {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    let label_response = ui.label("Subcategoria (FUNCTION.CATEGORY-NN)");
                    ui.text_edit_singleline(&mut entry.subcategory_id)
                        .labelled_by(label_response.id);
                    if ui.small_button("Remover").clicked() {
                        remove_index = Some(index);
                    }
                });
                super::field_text(ui, "Descrição", &mut entry.subcategory_label);
                enum_combo(
                    ui,
                    &format!("nist_entry_status_{index}"),
                    "Status de implementação",
                    &mut entry.implementation_status,
                    &IMPLEMENTATION_STATUS_OPTIONS,
                );
                super::field_text(ui, "Justificativa (opcional)", &mut entry.justification);
                super::field_text(
                    ui,
                    "Referência de evidência (opcional)",
                    &mut entry.evidence_reference,
                );
                enum_combo(
                    ui,
                    &format!("nist_entry_confidence_{index}"),
                    "Confiança (opcional)",
                    &mut entry.confidence,
                    &super::CONFIDENCE_OPTIONS,
                );
            });
        }
        if let Some(index) = remove_index {
            form.entries.remove(index);
        }
        if ui.button("+ Adicionar entrada").clicked() {
            form.entries.push(EntryForm::default());
        }

        if let Some(message) = &self.create_error {
            super::error_banner(ui, message);
        }
        if ui
            .add_enabled(!self.creating, egui::Button::new("Salvar profile"))
            .clicked()
        {
            let entries: Vec<ProfileEntryInput> = form
                .entries
                .iter()
                .map(|entry| ProfileEntryInput {
                    subcategory_id: entry.subcategory_id.clone(),
                    subcategory_label: entry.subcategory_label.clone(),
                    implementation_status: entry.implementation_status.clone(),
                    justification: non_empty(&entry.justification),
                    evidence_reference: non_empty(&entry.evidence_reference),
                    confidence: non_empty(&entry.confidence),
                })
                .collect();
            self.creating = true;
            backend.send(Command::CreateNistProfile {
                incident_id: incident_id.to_string(),
                profile_type: form.profile_type.clone(),
                version_label: form.version_label.clone(),
                source: form.source.clone(),
                tier: non_empty(&form.tier),
                tier_justification: non_empty(&form.tier_justification),
                entries,
            });
        }
        if self.creating {
            ui.spinner();
        }
    }
}

fn profile_column(
    ui: &mut egui::Ui,
    id: &str,
    title: &str,
    loadable: &Loadable<ProfileHistoryView>,
    selected_version: &mut Option<usize>,
) {
    ui.heading(title);
    loadable.show(ui, |ui, history| {
        if history.versions.is_empty() {
            *selected_version = None;
            ui.label("Nenhuma versão ainda.");
            return;
        }
        let latest_index = history.versions.len() - 1;
        let selected_index = match *selected_version {
            Some(index) if index <= latest_index => index,
            _ => latest_index,
        };
        *selected_version = Some(selected_index);
        let selected = &history.versions[selected_index];
        egui::ComboBox::new(
            format!("nist_{id}_profile_version"),
            format!("Versão do {title}"),
        )
        .selected_text(format!(
            "{} · {}",
            selected.version_label, selected.created_at
        ))
        .show_ui(ui, |ui| {
            for (index, version) in history.versions.iter().enumerate() {
                ui.selectable_value(
                    selected_version,
                    Some(index),
                    format!("{} · {}", version.version_label, version.created_at),
                );
            }
        });
        ui.label(format!(
            "Versão exibida: {} · fonte: {}",
            selected.version_label, selected.source
        ));
        if selected_index != latest_index {
            super::hint_banner(
                ui,
                "Versão não vigente: visualização somente leitura. Novas análises continuam usando a versão vigente.",
            );
            let latest = &history.versions[latest_index];
            ui.label(format!(
                "Versão vigente: {} · fonte: {}",
                latest.version_label, latest.source
            ));
        }
        if let Some(tier) = &selected.tier {
            ui.label(format!("Tier: {tier}"));
        }
        if let Some(justification) = &selected.tier_justification {
            ui.label(format!("Justificativa do tier: {justification}"));
        }
        ui.label(format!(
            "{} entradas · {} versões no histórico · criada em {}",
            selected.entries.len(),
            history.versions.len(),
            selected.created_at
        ));
        egui::ScrollArea::vertical()
            .id_salt(format!("nist_{id}_entries"))
            .max_height(240.0)
            .show(ui, |ui| {
                for entry in &selected.entries {
                    ui.label(format!(
                        "{} — {} ({})",
                        entry.subcategory_id, entry.subcategory_label, entry.implementation_status
                    ));
                }
            });
    });
}

/// "Rótulo: valor" do profile vigente; o Tier é do profile inteiro, e a
/// justificativa vem do analista.
fn profile_tier_line(ui: &mut egui::Ui, name: &str, profile: Option<&ProfileSummary>) {
    match profile {
        None => {
            ui.label(format!("{name}: ainda não informado"));
        }
        Some(profile) => {
            let tier = profile.tier.as_deref().unwrap_or("não informado");
            ui.label(format!(
                "{name}: versão {} · {} subcategorias · Tier {tier}",
                profile.version_label, profile.entries
            ));
            if let Some(justification) = profile
                .tier_justification
                .as_deref()
                .filter(|text| !text.trim().is_empty())
            {
                ui.small(format!("Justificativa do Tier: {justification}"));
            }
        }
    }
}

/// Visão por Function (F1-30): só contagens. Sem pontuação, porque o
/// NIST CSF 2.0 não define score oficial.
fn function_section(ui: &mut egui::Ui, loadable: &Loadable<NistFunctionView>) {
    ui.heading("Visão por Function");
    loadable.show(ui, |ui, view| {
        profile_tier_line(ui, "Current", view.current_profile.as_ref());
        profile_tier_line(ui, "Target", view.target_profile.as_ref());
        match view.gap_state {
            GapAnalysisState::MissingProfile => {
                ui.label("Informe os profiles Current e Target para comparar.");
            }
            GapAnalysisState::NoGapsRecorded => {
                ui.label(
                    "Nenhum gap vigente: a análise ainda não foi calculada ou o Current já atinge o Target.",
                );
            }
            GapAnalysisState::Stale => {
                ui.colored_label(
                    egui::Color32::from_rgb(200, 120, 0),
                    "Os gaps foram calculados com uma versão anterior dos profiles: recalcule para atualizar as contagens de gap.",
                );
            }
            GapAnalysisState::UpToDate => {}
        }
        let scroll_visibility = if ui.available_width() < 900.0 {
            egui::scroll_area::ScrollBarVisibility::AlwaysVisible
        } else {
            egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded
        };
        egui::ScrollArea::horizontal()
            .id_salt("nist_function_view_horizontal")
            .scroll_bar_visibility(scroll_visibility)
            .show(ui, |ui| {
                ui.set_width(ui.clip_rect().width().max(900.0));
                egui::Grid::new("nist_function_view")
                    .striped(true)
                    .show(ui, |ui| {
                        for header in [
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
                        ] {
                            ui.strong(header);
                        }
                        ui.end_row();
                        for function in &view.functions {
                            ui.label(format!("{} — {}", function.function, function.label));
                            for count in [
                                function.current.implemented,
                                function.current.partially_implemented,
                                function.current.not_implemented,
                                function.current.not_assessed,
                                function.current.not_applicable,
                                function.target_subcategories,
                                function.gaps.high,
                                function.gaps.medium,
                                function.gaps.low,
                            ] {
                                ui.label(count.to_string());
                            }
                            ui.end_row();
                        }
                    });
            });
        ui.small(
            "São contagens, não pontuação: o NIST CSF 2.0 não define score oficial. O Tier descreve o contexto de governança do profile inteiro, não cada Function.",
        );
    });
}

/// Testes da seção com `egui_kittest` (ADR-0022).
#[cfg(test)]
mod ui_tests {
    use super::{NistState, function_section, gap_ui};
    use crate::gui::Loadable;
    use crate::gui::backend::{Backend, Command};
    use crate::gui::test_harness::drain;
    use crate::persistence::nist_functions::{GapAnalysisState, summarize_by_function};
    use crate::persistence::nist_profiles::{ProfileEntryRecord, ProfileType};
    use crate::service::AnalyticalResultResponse;
    use crate::service::nist::{
        NistFunctionView, ProfileHistoryView, ProfileSummary, ProfileVersionView,
    };
    use egui_kittest::{Harness, kittest::Queryable};

    type State = (NistState, Backend);

    fn view(gap_state: GapAnalysisState) -> NistFunctionView {
        NistFunctionView {
            current_profile: Some(ProfileSummary {
                version_label: "c1".into(),
                tier: Some("TIER_1_PARTIAL".into()),
                tier_justification: Some("governança informal".into()),
                entries: 3,
            }),
            target_profile: None,
            functions: summarize_by_function(&[], &[], &[]),
            gap_state,
        }
    }

    fn loaded_state(gap_state: GapAnalysisState) -> NistState {
        NistState {
            loaded_for: Some("inc-1".into()),
            current: Loadable::Ready(ProfileHistoryView {
                profile_type: ProfileType::Current,
                versions: vec![],
            }),
            target: Loadable::Ready(ProfileHistoryView {
                profile_type: ProfileType::Target,
                versions: vec![],
            }),
            gaps: Loadable::Ready(vec![]),
            functions: Loadable::Ready(view(gap_state)),
            ..NistState::default()
        }
    }

    fn harness(state: NistState, backend: Backend) -> Harness<'static, State> {
        crate::gui::test_harness::builder()
            .with_size(egui::vec2(1400.0, 1600.0))
            .build_ui_state(
                |ui, (state, backend): &mut State| state.ui(ui, "inc-1", backend),
                (state, backend),
            )
    }

    fn history(profile_type: ProfileType) -> ProfileHistoryView {
        ProfileHistoryView {
            profile_type,
            versions: vec![ProfileVersionView {
                version_label: "v1".into(),
                source: "teste".into(),
                tier: None,
                tier_justification: None,
                created_at: "2026-11-18T00:00:00Z".into(),
                entries: vec![ProfileEntryRecord {
                    subcategory_id: "GV.OC-01".into(),
                    subcategory_label: "Contexto organizacional".into(),
                    implementation_status: "IMPLEMENTED".into(),
                    confidence: None,
                }],
            }],
        }
    }

    fn version(
        metadata: (&str, &str, &str, &str, &str),
        entry: (&str, &str, &str),
    ) -> ProfileVersionView {
        let (version_label, source, tier, tier_justification, created_at) = metadata;
        let (entry_id, entry_label, implementation_status) = entry;
        ProfileVersionView {
            version_label: version_label.into(),
            source: source.into(),
            tier: Some(tier.into()),
            tier_justification: Some(tier_justification.into()),
            created_at: created_at.into(),
            entries: vec![ProfileEntryRecord {
                subcategory_id: entry_id.into(),
                subcategory_label: entry_label.into(),
                implementation_status: implementation_status.into(),
                confidence: None,
            }],
        }
    }

    fn two_version_history(profile_type: ProfileType) -> ProfileHistoryView {
        ProfileHistoryView {
            profile_type,
            versions: vec![
                version(
                    (
                        "antiga",
                        "fonte-antiga",
                        "TIER_1_PARTIAL",
                        "contexto-antigo",
                        "2026-11-17T00:00:00Z",
                    ),
                    ("GV.OC-00", "Contexto antigo", "NOT_IMPLEMENTED"),
                ),
                version(
                    (
                        "atual",
                        "fonte-atual",
                        "TIER_2_RISK_INFORMED",
                        "contexto-atual",
                        "2026-11-18T00:00:00Z",
                    ),
                    ("GV.OC-01", "Contexto atual", "IMPLEMENTED"),
                ),
            ],
        }
    }

    fn select_version(harness: &mut Harness<'static, State>, title: &str, option: &str) {
        let combo_label = format!("Versão do {title}");
        harness
            .get_by_role_and_label(egui::accesskit::Role::ComboBox, combo_label.as_str())
            .click();
        harness.run();
        harness.get_by_label(option).click();
        harness.run();
    }

    fn gap(payload: serde_json::Value) -> AnalyticalResultResponse {
        AnalyticalResultResponse {
            id: "gap-1".into(),
            result_type: "NIST_GAP".into(),
            correlation_key: "nist-gap:inc-1:PR.AA-01".into(),
            evidence_id: None,
            incident_id: Some("inc-1".into()),
            analysis_job_id: None,
            rule_or_model_id: Some("nist-csf-2.0-gap".into()),
            rule_or_model_version: Some("1".into()),
            confidence: Some("HIGH".into()),
            payload,
            provenance: serde_json::Value::Null,
            supersedes: None,
            created_at: "2026-10-16T00:00:00Z".into(),
        }
    }

    #[test]
    fn gap_analysis_shows_structured_fields_and_long_label_instead_of_raw_json() {
        let (backend, _commands) = Backend::for_test();
        let subcategory_label = "Identidades, credenciais e acessos de usuários, serviços e dispositivos são gerenciados durante todo o ciclo de vida, com revisões periódicas documentadas e privilégios compatíveis com a função de cada titular";
        let payload = serde_json::json!({
            "kind": "NIST_GAP",
            "subcategory_id": "PR.AA-01",
            "subcategory_label": subcategory_label,
            "current_status": "PARTIALLY_IMPLEMENTED",
            "target_status": "IMPLEMENTED",
            "priority": "MEDIUM",
            "risk_level": "MEDIUM"
        });
        let raw_payload = payload.to_string();
        let mut state = loaded_state(GapAnalysisState::UpToDate);
        state.gaps = Loadable::Ready(vec![gap(payload)]);
        let mut harness = harness(state, backend);
        harness.run();

        for label in [
            format!("PR.AA-01 — {subcategory_label}"),
            "Status atual: PARTIALLY_IMPLEMENTED".into(),
            "Status alvo: IMPLEMENTED".into(),
            "Prioridade: MEDIUM".into(),
            "Nível de risco: MEDIUM".into(),
            "Confiança: HIGH".into(),
        ] {
            harness.get_by_label(label.as_str());
        }
        assert!(
            harness.query_by_label(raw_payload.as_str()).is_none(),
            "o payload JSON minificado não deve ser exibido"
        );
        assert!(
            harness
                .query_by_label_contains("\"current_status\":")
                .is_none(),
            "nenhuma chave JSON bruta deve virar interface"
        );
    }

    #[test]
    fn closed_gap_keeps_its_resolution_fields_structured() {
        let payload = serde_json::json!({
            "kind": "NIST_GAP",
            "status": "CLOSED",
            "closed_reason": "target_met",
            "subcategory_id": "PR.AA-01",
            "subcategory_label": "Identidades e credenciais gerenciadas",
            "resolved_current_status": "IMPLEMENTED"
        });
        let raw_payload = payload.to_string();
        let item = gap(payload);
        let mut harness = crate::gui::test_harness::builder().build_ui(move |ui| gap_ui(ui, &item));
        harness.run();

        for label in [
            "PR.AA-01 — Identidades e credenciais gerenciadas",
            "Situação: CLOSED",
            "Motivo do fechamento: target_met",
            "Status atual na resolução: IMPLEMENTED",
            "Confiança: HIGH",
        ] {
            harness.get_by_label(label);
        }
        assert!(harness.query_by_label(raw_payload.as_str()).is_none());
        assert!(
            harness
                .query_by_label_contains("\"closed_reason\":")
                .is_none()
        );
    }

    #[test]
    fn current_and_target_profiles_do_not_reuse_the_same_scroll_area_id() {
        let (backend, _commands) = Backend::for_test();
        let mut state = loaded_state(GapAnalysisState::UpToDate);
        state.current = Loadable::Ready(history(ProfileType::Current));
        state.target = Loadable::Ready(history(ProfileType::Target));
        let mut harness = harness(state, backend);
        harness.run();

        let paints_id_clash_warning = harness.output().shapes.iter().any(|clipped| match &clipped
            .shape
        {
            egui::Shape::Text(text) => text.galley.text().contains("use of ScrollArea ID"),
            _ => false,
        });
        assert!(
            !paints_id_clash_warning,
            "o egui não deve pintar avisos de ID duplicado sobre Current e Target"
        );
    }

    #[test]
    fn two_versions_are_selectable_and_old_content_is_read_only() {
        let (backend, mut commands) = Backend::for_test();
        let mut state = loaded_state(GapAnalysisState::Stale);
        state.current = Loadable::Ready(two_version_history(ProfileType::Current));
        let mut harness = harness(state, backend);
        harness.run();

        assert!(harness.query_by_label_contains("fonte-atual").is_some());
        assert!(harness.query_by_label_contains("fonte-antiga").is_none());
        assert!(harness.query_by_label_contains("Contexto antigo").is_none());
        harness
            .get_by_role_and_label(egui::accesskit::Role::ComboBox, "Versão do Current Profile")
            .click();
        harness.run();
        harness
            .get_by_label("antiga · 2026-11-17T00:00:00Z")
            .click();
        harness.run();

        harness.get_by_label("Versão exibida: antiga · fonte: fonte-antiga");
        harness.get_by_label("Tier: TIER_1_PARTIAL");
        harness.get_by_label("Justificativa do tier: contexto-antigo");
        harness
            .get_by_label("1 entradas · 2 versões no histórico · criada em 2026-11-17T00:00:00Z");
        harness.get_by_label("GV.OC-00 — Contexto antigo (NOT_IMPLEMENTED)");
        harness.get_by_label(
            "Versão não vigente: visualização somente leitura. Novas análises continuam usando a versão vigente.",
        );
        harness.get_by_label("Versão vigente: atual · fonte: fonte-atual");
        harness.get_by_label(
            "Os gaps foram calculados com uma versão anterior dos profiles: recalcule para atualizar as contagens de gap.",
        );
        assert!(harness.query_by_label_contains("Contexto atual").is_none());
        assert!(
            drain(&mut commands).is_empty(),
            "selecionar histórico não pode emitir criação, mutação ou recálculo"
        );
    }

    #[test]
    fn current_and_target_version_selections_are_independent_and_read_only() {
        let (backend, mut commands) = Backend::for_test();
        let mut state = loaded_state(GapAnalysisState::UpToDate);
        state.current = Loadable::Ready(two_version_history(ProfileType::Current));
        state.target = Loadable::Ready(two_version_history(ProfileType::Target));
        let mut harness = harness(state, backend);
        harness.run();

        select_version(
            &mut harness,
            "Current Profile",
            "antiga · 2026-11-17T00:00:00Z",
        );
        assert_eq!(
            harness
                .get_by_role_and_label(
                    egui::accesskit::Role::ComboBox,
                    "Versão do Current Profile",
                )
                .value()
                .as_deref(),
            Some("antiga · 2026-11-17T00:00:00Z")
        );
        assert_eq!(
            harness
                .get_by_role_and_label(egui::accesskit::Role::ComboBox, "Versão do Target Profile",)
                .value()
                .as_deref(),
            Some("atual · 2026-11-18T00:00:00Z")
        );

        select_version(
            &mut harness,
            "Target Profile",
            "antiga · 2026-11-17T00:00:00Z",
        );
        assert_eq!(
            harness
                .get_all_by_role_and_label(
                    egui::accesskit::Role::Label,
                    "Versão exibida: antiga · fonte: fonte-antiga",
                )
                .count(),
            2
        );

        select_version(
            &mut harness,
            "Current Profile",
            "atual · 2026-11-18T00:00:00Z",
        );
        harness.get_by_label("Versão exibida: atual · fonte: fonte-atual");
        harness.get_by_label("Versão exibida: antiga · fonte: fonte-antiga");
        assert!(
            drain(&mut commands).is_empty(),
            "nenhuma seleção de Current ou Target pode emitir comandos"
        );
    }

    #[test]
    fn history_reload_restores_the_latest_version_as_default() {
        let (backend, _commands) = Backend::for_test();
        let mut state = loaded_state(GapAnalysisState::UpToDate);
        state.current = Loadable::Ready(two_version_history(ProfileType::Current));
        let mut harness = harness(state, backend);
        harness.run();
        select_version(
            &mut harness,
            "Current Profile",
            "antiga · 2026-11-17T00:00:00Z",
        );
        harness.get_by_label("Versão exibida: antiga · fonte: fonte-antiga");

        harness
            .state_mut()
            .0
            .on_history("inc-1", Ok(two_version_history(ProfileType::Current)));
        harness.run();

        assert_eq!(
            harness
                .get_by_role_and_label(
                    egui::accesskit::Role::ComboBox,
                    "Versão do Current Profile",
                )
                .value()
                .as_deref(),
            Some("atual · 2026-11-18T00:00:00Z")
        );
        harness.get_by_label("Versão exibida: atual · fonte: fonte-atual");
        assert!(
            harness
                .query_by_label_contains("Versão não vigente:")
                .is_none()
        );
    }

    #[test]
    fn empty_and_single_version_histories_remain_stable() {
        let (backend, mut commands) = Backend::for_test();
        let mut harness = harness(loaded_state(GapAnalysisState::UpToDate), backend);
        harness.run();
        assert_eq!(
            harness
                .get_all_by_role_and_label(egui::accesskit::Role::Label, "Nenhuma versão ainda.",)
                .count(),
            2,
            "Current e Target vazios devem continuar estáveis"
        );
        for label in ["Versão do Current Profile", "Versão do Target Profile"] {
            assert!(
                harness.query_by_label(label).is_none(),
                "histórico vazio não deve criar seletor: {label}"
            );
        }

        {
            let (state, _) = harness.state_mut();
            state.current = Loadable::Ready(history(ProfileType::Current));
            state.target = Loadable::Ready(history(ProfileType::Target));
        }
        harness.run();
        for title in ["Current Profile", "Target Profile"] {
            let label = format!("Versão do {title}");
            assert_eq!(
                harness
                    .get_by_role_and_label(egui::accesskit::Role::ComboBox, label.as_str())
                    .value()
                    .as_deref(),
                Some("v1 · 2026-11-18T00:00:00Z")
            );
        }
        select_version(&mut harness, "Current Profile", "v1 · 2026-11-18T00:00:00Z");
        assert!(
            harness
                .query_by_label_contains("Versão não vigente:")
                .is_none()
        );
        assert!(drain(&mut commands).is_empty());
    }

    #[test]
    fn function_grid_horizontal_scroll_reveals_the_last_column() {
        let loadable = Loadable::Ready(view(GapAnalysisState::UpToDate));
        let width = 600.0;
        let mut harness = crate::gui::test_harness::builder()
            .with_size(egui::vec2(width, 700.0))
            .build_ui(move |ui| function_section(ui, &loadable));
        harness.step();

        let initial = harness.get_by_label("Gap baixo").rect();
        assert!(
            initial.max.x > width,
            "a última coluna deve começar fora da viewport para provar o scroll: {initial:?}"
        );
        harness.get_by_label("Function").hover();
        harness.input_mut().events.push(egui::Event::MouseWheel {
            unit: egui::MouseWheelUnit::Point,
            delta: egui::vec2(-300.0, 0.0),
            phase: egui::TouchPhase::Move,
            source: egui::MouseWheelSource::Unknown,
            modifiers: egui::Modifiers::default(),
        });
        harness.run();

        let revealed = harness.get_by_label("Gap baixo").rect();
        assert!(
            revealed.min.x < initial.min.x,
            "o conteúdo não se moveu horizontalmente: antes={initial:?}, depois={revealed:?}"
        );
        assert!(
            revealed.min.x >= 0.0 && revealed.max.x <= width,
            "a última coluna não ficou visível após o scroll: {revealed:?}"
        );
    }

    #[test]
    fn opening_the_tab_requests_profiles_gaps_and_the_function_view_once() {
        let (backend, mut commands) = Backend::for_test();
        let mut harness = harness(NistState::default(), backend);
        // Carregando mostra um spinner, que repinta sem parar: `step`, não `run`.
        harness.step();
        harness.step();

        let sent = drain(&mut commands);
        assert_eq!(sent.len(), 4, "dois históricos, gaps e a visão: {sent:?}");
        assert_eq!(
            sent.iter()
                .filter(|c| matches!(c, Command::LoadNistFunctionView(id) if id == "inc-1"))
                .count(),
            1
        );
    }

    #[test]
    fn function_view_shows_the_six_functions_tiers_and_the_no_score_note() {
        let (backend, mut commands) = Backend::for_test();
        let mut harness = harness(loaded_state(GapAnalysisState::UpToDate), backend);
        harness.run();

        for label in [
            "GV — Governar",
            "ID — Identificar",
            "PR — Proteger",
            "DE — Detectar",
            "RS — Responder",
            "RC — Recuperar",
        ] {
            harness.get_by_label(label);
        }
        harness.get_by_label("Current: versão c1 · 3 subcategorias · Tier TIER_1_PARTIAL");
        harness.get_by_label("Justificativa do Tier: governança informal");
        harness.get_by_label("Target: ainda não informado");
        harness.get_by_label(
            "São contagens, não pontuação: o NIST CSF 2.0 não define score oficial. O Tier descreve o contexto de governança do profile inteiro, não cada Function.",
        );
        assert!(harness
            .query_by_label(
                "Os gaps foram calculados com uma versão anterior dos profiles: recalcule para atualizar as contagens de gap."
            )
            .is_none());
        assert!(drain(&mut commands).is_empty());
    }

    #[test]
    fn stale_gaps_are_flagged_instead_of_shown_as_current() {
        let (backend, _commands) = Backend::for_test();
        let mut harness = harness(loaded_state(GapAnalysisState::Stale), backend);
        harness.run();
        harness.get_by_label(
            "Os gaps foram calculados com uma versão anterior dos profiles: recalcule para atualizar as contagens de gap.",
        );
    }

    #[test]
    fn a_finished_gap_run_refreshes_both_the_gap_list_and_the_function_view() {
        let (backend, mut commands) = Backend::for_test();
        let mut state = loaded_state(GapAnalysisState::Stale);
        state.on_gap_run("inc-1", Ok(vec![]), &backend);
        let sent = drain(&mut commands);
        assert!(
            sent.iter()
                .any(|c| matches!(c, Command::CurrentGapAnalysis(_)))
        );
        assert!(
            sent.iter()
                .any(|c| matches!(c, Command::LoadNistFunctionView(_)))
        );
    }
}
