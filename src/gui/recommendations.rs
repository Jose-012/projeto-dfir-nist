//! Aba "Recomendações": ações IMMEDIATE/STRATEGIC recomendadas para um
//! incidente, com resultados analíticos de suporte opcionais (F1-05).

use super::backend::{Backend, Command};
use super::{
    CONFIDENCE_OPTIONS, Loadable, RECOMMENDATION_CATEGORY_OPTIONS, enum_combo, non_empty,
    str_field, string_list,
};
use crate::service::AnalyticalResultResponse;

#[derive(Default)]
struct Form {
    category: String,
    title: String,
    description: String,
    confidence: String,
    supporting_result_ids: String,
}

fn recommendation_ui(ui: &mut egui::Ui, item: &AnalyticalResultResponse) {
    let payload = &item.payload;
    ui.group(|ui| {
        ui.set_width(ui.available_width());
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);

        ui.label(format!("Categoria: {}", str_field(payload, "category")));
        ui.strong(str_field(payload, "title"));
        ui.label(format!("Descrição: {}", str_field(payload, "description")));
        ui.label(format!(
            "Confiança: {}",
            item.confidence.as_deref().unwrap_or("não informada")
        ));

        let supporting_results = string_list(&item.provenance, "supporting_result_ids");
        if !supporting_results.is_empty() {
            ui.label("Resultados de suporte:");
            for result_id in supporting_results {
                ui.label(format!("• {result_id}"));
            }
        }
    });
}

pub struct RecommendationsState {
    loaded_for: Option<String>,
    list: Loadable<Vec<AnalyticalResultResponse>>,
    filter: String,
    form: Form,
    creating: bool,
    create_error: Option<String>,
}

impl Default for RecommendationsState {
    fn default() -> Self {
        Self {
            loaded_for: None,
            list: Loadable::Idle,
            filter: String::new(),
            form: Form::default(),
            creating: false,
            create_error: None,
        }
    }
}

impl RecommendationsState {
    fn ensure_loaded(&mut self, incident_id: &str, backend: &Backend) {
        if self.loaded_for.as_deref() == Some(incident_id) {
            return;
        }
        self.loaded_for = Some(incident_id.to_string());
        self.list = Loadable::Loading;
        backend.send(Command::ListRecommendations {
            incident_id: incident_id.to_string(),
            category: None,
        });
    }

    pub fn on_loaded(
        &mut self,
        incident_id: &str,
        result: Result<Vec<AnalyticalResultResponse>, String>,
    ) {
        if self.loaded_for.as_deref() != Some(incident_id) {
            return;
        }
        self.list = match result {
            Ok(items) => Loadable::Ready(items),
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
                self.create_error = None;
                self.form = Form::default();
                backend.send(Command::ListRecommendations {
                    incident_id: incident_id.to_string(),
                    category: None,
                });
            }
            Err(message) => self.create_error = Some(message),
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, incident_id: &str, backend: &Backend) {
        self.ensure_loaded(incident_id, backend);

        ui.horizontal(|ui| {
            ui.heading("Recomendações registradas");
            enum_combo(
                ui,
                "rec_filter",
                "Filtrar por categoria",
                &mut self.filter,
                &RECOMMENDATION_CATEGORY_OPTIONS,
            );
            if ui
                .small_button("×")
                .on_hover_text("Limpar filtro")
                .clicked()
            {
                self.filter.clear();
            }
        });
        self.list.show(ui, |ui, items| {
            let filtered: Vec<&AnalyticalResultResponse> = items
                .iter()
                .filter(|item| {
                    self.filter.is_empty()
                        || item
                            .payload
                            .get("category")
                            .and_then(serde_json::Value::as_str)
                            == Some(self.filter.as_str())
                })
                .collect();
            if filtered.is_empty() {
                ui.label("Nenhuma recomendação registrada ainda.");
                return;
            }
            for item in filtered {
                recommendation_ui(ui, item);
            }
        });

        ui.separator();
        ui.collapsing("Nova recomendação", |ui| {
            let form = &mut self.form;
            enum_combo(
                ui,
                "rec_category",
                "Categoria",
                &mut form.category,
                &RECOMMENDATION_CATEGORY_OPTIONS,
            );
            super::field_text(ui, "Título", &mut form.title);
            super::field_text_multiline(ui, "Descrição", &mut form.description);
            enum_combo(
                ui,
                "rec_confidence",
                "Confiança (opcional)",
                &mut form.confidence,
                &CONFIDENCE_OPTIONS,
            );
            super::field_text(
                ui,
                "IDs de resultados de suporte (separados por vírgula, opcional)",
                &mut form.supporting_result_ids,
            );
            if let Some(message) = &self.create_error {
                super::error_banner(ui, message);
            }
            if ui
                .add_enabled(!self.creating, egui::Button::new("Registrar recomendação"))
                .clicked()
            {
                let supporting_result_ids = form
                    .supporting_result_ids
                    .split(',')
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(str::to_string)
                    .collect();
                self.creating = true;
                backend.send(Command::CreateRecommendation {
                    incident_id: incident_id.to_string(),
                    category: form.category.clone(),
                    title: form.title.clone(),
                    description: form.description.clone(),
                    confidence: non_empty(&form.confidence),
                    supporting_result_ids,
                    supersedes: None,
                });
            }
            if self.creating {
                ui.spinner();
            }
        });
    }
}

#[cfg(test)]
mod ui_tests {
    use super::RecommendationsState;
    use crate::gui::Loadable;
    use crate::gui::backend::Backend;
    use crate::service::AnalyticalResultResponse;
    use egui_kittest::{Harness, kittest::Queryable};

    type State = (RecommendationsState, Backend);

    fn recommendation(
        payload: serde_json::Value,
        provenance: serde_json::Value,
    ) -> AnalyticalResultResponse {
        AnalyticalResultResponse {
            id: "recommendation-1".into(),
            result_type: "RECOMMENDATION".into(),
            correlation_key: "recommendation-key-1".into(),
            evidence_id: None,
            incident_id: Some("inc-1".into()),
            analysis_job_id: None,
            rule_or_model_id: None,
            rule_or_model_version: None,
            confidence: Some("HIGH".into()),
            payload,
            provenance,
            supersedes: None,
            created_at: "2026-10-16T00:00:00Z".into(),
        }
    }

    fn harness_for(state: RecommendationsState, backend: Backend) -> Harness<'static, State> {
        crate::gui::test_harness::builder()
            .with_size(egui::vec2(760.0, 1300.0))
            .build_ui_state(
                |ui, (state, backend): &mut State| state.ui(ui, "inc-1", backend),
                (state, backend),
            )
    }

    #[test]
    fn loaded_recommendation_shows_structured_fields_and_support_instead_of_raw_json() {
        let (backend, _commands) = Backend::for_test();
        let payload = serde_json::json!({
            "kind": "RECOMMENDATION",
            "category": "IMMEDIATE",
            "title": "Isolar o servidor de arquivos FS02",
            "description": "Desconectar o ativo da rede e preservar os logs antes de iniciar a recuperação"
        });
        let raw_payload = payload.to_string();
        let provenance = serde_json::json!({
            "producer": "dfir-app",
            "method": "analyst-authored recommendation",
            "supporting_result_ids": ["finding-17", "attack-23"]
        });
        let state = RecommendationsState {
            loaded_for: Some("inc-1".into()),
            list: Loadable::Ready(vec![recommendation(payload, provenance)]),
            ..RecommendationsState::default()
        };
        let mut harness = harness_for(state, backend);
        harness.run();

        for label in [
            "Categoria: IMMEDIATE",
            "Isolar o servidor de arquivos FS02",
            "Descrição: Desconectar o ativo da rede e preservar os logs antes de iniciar a recuperação",
            "Confiança: HIGH",
            "Resultados de suporte:",
            "• finding-17",
            "• attack-23",
        ] {
            harness.get_by_label(label);
        }
        assert!(
            harness.query_by_label(raw_payload.as_str()).is_none(),
            "o payload JSON minificado não deve ser exibido"
        );
        assert!(
            harness
                .query_by_label_contains("\"description\":")
                .is_none(),
            "nenhuma chave JSON bruta deve virar interface"
        );
    }
}
