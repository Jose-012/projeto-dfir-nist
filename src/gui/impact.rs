//! Aba "Impacto": matriz de impacto (disponibilidade/confidencialidade/
//! integridade + ativos/dados afetados) de um incidente (F1-04). Há no
//! máximo uma avaliação vigente por incidente — criar uma nova supersede a
//! anterior (mesma regra append-only de F1-01).

use super::backend::{Backend, Command};
use super::{CONFIDENCE_OPTIONS, IMPACT_LEVEL_OPTIONS, Loadable, enum_combo, non_empty};
use crate::service::AnalyticalResultResponse;

#[derive(Default)]
struct Form {
    availability_impact: String,
    confidentiality_impact: String,
    integrity_impact: String,
    affected_assets: String,
    affected_data: String,
    operational_disruption: String,
    financial_impact: String,
    confidence: String,
}

pub struct ImpactState {
    loaded_for: Option<String>,
    current: Loadable<Option<AnalyticalResultResponse>>,
    form: Form,
    creating: bool,
    create_error: Option<String>,
}

impl Default for ImpactState {
    fn default() -> Self {
        Self {
            loaded_for: None,
            current: Loadable::Idle,
            form: Form::default(),
            creating: false,
            create_error: None,
        }
    }
}

impl ImpactState {
    fn ensure_loaded(&mut self, incident_id: &str, backend: &Backend) {
        if self.loaded_for.as_deref() == Some(incident_id) {
            return;
        }
        self.loaded_for = Some(incident_id.to_string());
        self.current = Loadable::Loading;
        backend.send(Command::CurrentImpactAssessment(incident_id.to_string()));
    }

    pub fn on_loaded(
        &mut self,
        incident_id: &str,
        result: Result<Option<AnalyticalResultResponse>, String>,
    ) {
        if self.loaded_for.as_deref() != Some(incident_id) {
            return;
        }
        self.current = match result {
            Ok(value) => Loadable::Ready(value),
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
                backend.send(Command::CurrentImpactAssessment(incident_id.to_string()));
            }
            Err(message) => self.create_error = Some(message),
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, incident_id: &str, backend: &Backend) {
        self.ensure_loaded(incident_id, backend);

        ui.heading("Avaliação de impacto vigente");
        self.current.show(ui, |ui, current| match current {
            Some(assessment) => {
                ui.small(assessment.payload.to_string());
            }
            None => {
                ui.label("Nenhuma avaliação de impacto registrada ainda.");
            }
        });

        ui.separator();
        ui.collapsing("Nova avaliação de impacto", |ui| {
            let form = &mut self.form;
            enum_combo(
                ui,
                "impact_availability",
                "Impacto na disponibilidade",
                &mut form.availability_impact,
                &IMPACT_LEVEL_OPTIONS,
            );
            enum_combo(
                ui,
                "impact_confidentiality",
                "Impacto na confidencialidade",
                &mut form.confidentiality_impact,
                &IMPACT_LEVEL_OPTIONS,
            );
            enum_combo(
                ui,
                "impact_integrity",
                "Impacto na integridade",
                &mut form.integrity_impact,
                &IMPACT_LEVEL_OPTIONS,
            );
            super::field_text_multiline(ui, "Ativos afetados", &mut form.affected_assets);
            super::field_text_multiline(ui, "Dados afetados", &mut form.affected_data);
            super::field_text_multiline(
                ui,
                "Disrupção operacional",
                &mut form.operational_disruption,
            );
            super::field_text(
                ui,
                "Impacto financeiro (opcional)",
                &mut form.financial_impact,
            );
            enum_combo(
                ui,
                "impact_confidence",
                "Confiança",
                &mut form.confidence,
                &CONFIDENCE_OPTIONS,
            );
            if let Some(message) = &self.create_error {
                super::error_banner(ui, message);
            }
            if ui
                .add_enabled(!self.creating, egui::Button::new("Salvar avaliação"))
                .clicked()
            {
                self.creating = true;
                backend.send(Command::CreateImpactAssessment {
                    incident_id: incident_id.to_string(),
                    availability_impact: form.availability_impact.clone(),
                    confidentiality_impact: form.confidentiality_impact.clone(),
                    integrity_impact: form.integrity_impact.clone(),
                    affected_assets: form.affected_assets.clone(),
                    affected_data: form.affected_data.clone(),
                    operational_disruption: form.operational_disruption.clone(),
                    financial_impact: non_empty(&form.financial_impact),
                    confidence: form.confidence.clone(),
                });
            }
            if self.creating {
                ui.spinner();
            }
        });
    }
}
