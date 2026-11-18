//! Aba "Recomendações": ações IMMEDIATE/STRATEGIC recomendadas para um
//! incidente, com resultados analíticos de suporte opcionais (F1-05).

use super::backend::{Backend, Command};
use super::{CONFIDENCE_OPTIONS, Loadable, RECOMMENDATION_CATEGORY_OPTIONS, enum_combo, non_empty};
use crate::service::AnalyticalResultResponse;

#[derive(Default)]
struct Form {
    category: String,
    title: String,
    description: String,
    confidence: String,
    supporting_result_ids: String,
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
                ui.group(|ui| {
                    ui.small(item.payload.to_string());
                });
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
