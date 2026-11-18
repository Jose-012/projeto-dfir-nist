//! Aba "ATT&CK": mapeamento de táticas/técnicas MITRE ATT&CK a evidências
//! (F1-04).

use super::backend::{Backend, Command};
use super::{ATTACK_TACTIC_OPTIONS, CONFIDENCE_OPTIONS, Loadable, enum_combo, non_empty};
use crate::service::AnalyticalResultResponse;

#[derive(Default)]
struct Form {
    evidence_id: String,
    tactic: String,
    technique_id: String,
    sub_technique_id: String,
    basis: String,
    confidence: String,
    analyst_note: String,
}

pub struct AttackState {
    loaded_for: Option<String>,
    list: Loadable<Vec<AnalyticalResultResponse>>,
    form: Form,
    creating: bool,
    create_error: Option<String>,
}

impl Default for AttackState {
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

impl AttackState {
    fn ensure_loaded(&mut self, incident_id: &str, backend: &Backend) {
        if self.loaded_for.as_deref() == Some(incident_id) {
            return;
        }
        self.loaded_for = Some(incident_id.to_string());
        self.list = Loadable::Loading;
        backend.send(Command::ListAttackMappings(incident_id.to_string()));
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
                backend.send(Command::ListAttackMappings(incident_id.to_string()));
            }
            Err(message) => self.create_error = Some(message),
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, incident_id: &str, backend: &Backend) {
        self.ensure_loaded(incident_id, backend);

        ui.heading("Mapeamentos ATT&CK registrados");
        self.list.show(ui, |ui, items| {
            if items.is_empty() {
                ui.label("Nenhum mapeamento ainda.");
                return;
            }
            for item in items {
                ui.group(|ui| {
                    ui.small(item.payload.to_string());
                });
            }
        });

        ui.separator();
        ui.collapsing("Novo mapeamento", |ui| {
            let form = &mut self.form;
            super::field_text(ui, "ID da evidência (opcional)", &mut form.evidence_id);
            enum_combo(
                ui,
                "attack_tactic",
                "Tática",
                &mut form.tactic,
                &ATTACK_TACTIC_OPTIONS,
            );
            super::field_text(ui, "Technique ID (ex.: T1078)", &mut form.technique_id);
            super::field_text(
                ui,
                "Sub-technique ID (opcional, ex.: T1078.001)",
                &mut form.sub_technique_id,
            );
            super::field_text_multiline(
                ui,
                "Base analítica (evidência do porquê)",
                &mut form.basis,
            );
            enum_combo(
                ui,
                "attack_confidence",
                "Confiança",
                &mut form.confidence,
                &CONFIDENCE_OPTIONS,
            );
            super::field_text(ui, "Nota do analista (opcional)", &mut form.analyst_note);
            if let Some(message) = &self.create_error {
                super::error_banner(ui, message);
            }
            if ui
                .add_enabled(!self.creating, egui::Button::new("Registrar mapeamento"))
                .clicked()
            {
                self.creating = true;
                backend.send(Command::CreateAttackMapping {
                    incident_id: incident_id.to_string(),
                    evidence_id: non_empty(&form.evidence_id),
                    tactic: form.tactic.clone(),
                    technique_id: form.technique_id.clone(),
                    sub_technique_id: non_empty(&form.sub_technique_id),
                    basis: form.basis.clone(),
                    confidence: form.confidence.clone(),
                    analyst_note: non_empty(&form.analyst_note),
                });
            }
            if self.creating {
                ui.spinner();
            }
        });
    }
}
