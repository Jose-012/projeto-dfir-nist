//! Aba "IOCs": indicadores de comprometimento observados, ligados a uma
//! evidência específica (F1-05).

use super::backend::{Backend, Command};
use super::{CONFIDENCE_OPTIONS, IOC_TYPE_OPTIONS, Loadable, enum_combo, non_empty};
use crate::service::AnalyticalResultResponse;

#[derive(Default)]
struct Form {
    evidence_id: String,
    ioc_type: String,
    value: String,
    event_context: String,
    confidence: String,
    observed_at: String,
}

pub struct IocsState {
    loaded_for: Option<String>,
    list: Loadable<Vec<AnalyticalResultResponse>>,
    form: Form,
    creating: bool,
    create_error: Option<String>,
}

impl Default for IocsState {
    fn default() -> Self {
        Self {
            loaded_for: None,
            list: Loadable::Idle,
            form: Form::default(),
            creating: false,
            create_error: None,
        }
    }
}

impl IocsState {
    fn ensure_loaded(&mut self, incident_id: &str, backend: &Backend) {
        if self.loaded_for.as_deref() == Some(incident_id) {
            return;
        }
        self.loaded_for = Some(incident_id.to_string());
        self.list = Loadable::Loading;
        backend.send(Command::ListIocs(incident_id.to_string()));
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
                backend.send(Command::ListIocs(incident_id.to_string()));
            }
            Err(message) => self.create_error = Some(message),
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, incident_id: &str, backend: &Backend) {
        self.ensure_loaded(incident_id, backend);

        ui.heading("IOCs observados");
        self.list.show(ui, |ui, items| {
            if items.is_empty() {
                ui.label("Nenhum IOC registrado ainda.");
                return;
            }
            for item in items {
                ui.group(|ui| {
                    ui.small(item.payload.to_string());
                });
            }
        });

        ui.separator();
        ui.collapsing("Novo IOC", |ui| {
            let form = &mut self.form;
            super::field_text(ui, "ID da evidência de origem", &mut form.evidence_id);
            enum_combo(
                ui,
                "ioc_type",
                "Tipo",
                &mut form.ioc_type,
                &IOC_TYPE_OPTIONS,
            );
            super::field_text(ui, "Valor observado", &mut form.value);
            super::field_text(ui, "Contexto do evento (opcional)", &mut form.event_context);
            enum_combo(
                ui,
                "ioc_confidence",
                "Confiança",
                &mut form.confidence,
                &CONFIDENCE_OPTIONS,
            );
            super::field_text(
                ui,
                "Observado em (RFC3339, opcional)",
                &mut form.observed_at,
            );
            if let Some(message) = &self.create_error {
                super::error_banner(ui, message);
            }
            if ui
                .add_enabled(!self.creating, egui::Button::new("Registrar IOC"))
                .clicked()
            {
                self.creating = true;
                backend.send(Command::CreateIoc {
                    incident_id: incident_id.to_string(),
                    evidence_id: form.evidence_id.clone(),
                    ioc_type: form.ioc_type.clone(),
                    value: form.value.clone(),
                    event_context: non_empty(&form.event_context),
                    confidence: form.confidence.clone(),
                    observed_at: non_empty(&form.observed_at),
                });
            }
            if self.creating {
                ui.spinner();
            }
        });
    }
}
