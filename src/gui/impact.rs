//! Aba "Impacto": matriz de impacto (disponibilidade/confidencialidade/
//! integridade + ativos/dados afetados) de um incidente (F1-04). Há no
//! máximo uma avaliação vigente por incidente — criar uma nova supersede a
//! anterior (mesma regra append-only de F1-01).

use super::backend::{Backend, Command};
use super::{CONFIDENCE_OPTIONS, IMPACT_LEVEL_OPTIONS, Loadable, enum_combo, non_empty, str_field};
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

fn assessment_ui(ui: &mut egui::Ui, assessment: &AnalyticalResultResponse) {
    let payload = &assessment.payload;
    ui.group(|ui| {
        ui.set_width(ui.available_width());
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);

        ui.label(format!(
            "Disponibilidade: {}",
            str_field(payload, "availability_impact")
        ));
        ui.label(format!(
            "Confidencialidade: {}",
            str_field(payload, "confidentiality_impact")
        ));
        ui.label(format!(
            "Integridade: {}",
            str_field(payload, "integrity_impact")
        ));
        ui.label(format!(
            "Ativos afetados: {}",
            str_field(payload, "affected_assets")
        ));
        ui.label(format!(
            "Dados afetados: {}",
            str_field(payload, "affected_data")
        ));
        ui.label(format!(
            "Disrupção operacional: {}",
            str_field(payload, "operational_disruption")
        ));
        let financial_impact = str_field(payload, "financial_impact");
        ui.label(format!(
            "Impacto financeiro: {}",
            if financial_impact.is_empty() {
                "não informado"
            } else {
                financial_impact
            }
        ));
        ui.label(format!(
            "Confiança: {}",
            assessment.confidence.as_deref().unwrap_or("não informada")
        ));
    });
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
                assessment_ui(ui, assessment);
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

#[cfg(test)]
mod ui_tests {
    use super::ImpactState;
    use crate::gui::Loadable;
    use crate::gui::backend::Backend;
    use crate::service::AnalyticalResultResponse;
    use egui_kittest::{Harness, kittest::Queryable};

    type State = (ImpactState, Backend);

    fn assessment(payload: serde_json::Value) -> AnalyticalResultResponse {
        AnalyticalResultResponse {
            id: "impact-1".into(),
            result_type: "IMPACT_ASSESSMENT".into(),
            correlation_key: "impact-key-1".into(),
            evidence_id: None,
            incident_id: Some("inc-1".into()),
            analysis_job_id: None,
            rule_or_model_id: None,
            rule_or_model_version: None,
            confidence: Some("MEDIUM".into()),
            payload,
            provenance: serde_json::Value::Null,
            supersedes: None,
            created_at: "2026-10-16T00:00:00Z".into(),
        }
    }

    fn harness_for(state: ImpactState, backend: Backend) -> Harness<'static, State> {
        crate::gui::test_harness::builder()
            .with_size(egui::vec2(700.0, 1300.0))
            .build_ui_state(
                |ui, (state, backend): &mut State| state.ui(ui, "inc-1", backend),
                (state, backend),
            )
    }

    #[test]
    fn current_assessment_shows_structured_fields_instead_of_raw_json() {
        let (backend, _commands) = Backend::for_test();
        let payload = serde_json::json!({
            "kind": "IMPACT_ASSESSMENT",
            "availability_impact": "HIGH",
            "confidentiality_impact": "MEDIUM",
            "integrity_impact": "LOW",
            "affected_assets": "Servidor de arquivos FS02 e estações do setor financeiro",
            "affected_data": "Documentos internos e registros de clientes com dados pessoais",
            "operational_disruption": "Serviço de arquivos indisponível por quatro horas",
            "financial_impact": "Estimativa preliminar de R$ 40 mil a R$ 60 mil",
            "financial_impact_known": true
        });
        let raw_payload = payload.to_string();
        let state = ImpactState {
            loaded_for: Some("inc-1".into()),
            current: Loadable::Ready(Some(assessment(payload))),
            ..ImpactState::default()
        };
        let mut harness = harness_for(state, backend);
        harness.run();

        for label in [
            "Disponibilidade: HIGH",
            "Confidencialidade: MEDIUM",
            "Integridade: LOW",
            "Ativos afetados: Servidor de arquivos FS02 e estações do setor financeiro",
            "Dados afetados: Documentos internos e registros de clientes com dados pessoais",
            "Disrupção operacional: Serviço de arquivos indisponível por quatro horas",
            "Impacto financeiro: Estimativa preliminar de R$ 40 mil a R$ 60 mil",
            "Confiança: MEDIUM",
        ] {
            harness.get_by_label(label);
        }
        assert!(
            harness.query_by_label(raw_payload.as_str()).is_none(),
            "o payload JSON minificado não deve ser exibido"
        );
        assert!(
            harness
                .query_by_label_contains("\"affected_assets\":")
                .is_none(),
            "nenhuma chave JSON bruta deve virar interface"
        );
    }
}
