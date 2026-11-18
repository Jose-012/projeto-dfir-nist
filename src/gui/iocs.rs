//! Aba "IOCs": indicadores de comprometimento observados, ligados a uma
//! evidência específica (F1-05).

use super::backend::{Backend, Command};
use super::{CONFIDENCE_OPTIONS, IOC_TYPE_OPTIONS, Loadable, enum_combo, non_empty, str_field};
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

fn sighting_count(payload: &serde_json::Value) -> Option<u64> {
    payload
        .get("sighting_count")
        .and_then(serde_json::Value::as_u64)
        .or_else(|| {
            // Payloads anteriores ao contador agregado ainda podem ser lidos.
            payload
                .get("sightings")
                .and_then(serde_json::Value::as_array)
                .map(|sightings| sightings.len() as u64)
        })
}

fn sighting_field(ui: &mut egui::Ui, sighting: &serde_json::Value, key: &str, label: &str) {
    let value = str_field(sighting, key);
    if !value.is_empty() {
        ui.label(format!("{label}: {value}"));
    }
}

fn ioc_ui(ui: &mut egui::Ui, item: &AnalyticalResultResponse) {
    let payload = &item.payload;
    ui.group(|ui| {
        ui.set_width(ui.available_width());
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);

        ui.label(format!("Tipo: {}", str_field(payload, "ioc_type")));
        ui.strong(format!(
            "Valor normalizado: {}",
            str_field(payload, "normalized_value")
        ));
        ui.label(format!(
            "Primeira observação: {}",
            str_field(payload, "first_seen")
        ));
        ui.label(format!(
            "Última observação: {}",
            str_field(payload, "last_seen")
        ));
        match sighting_count(payload) {
            Some(count) => {
                ui.label(format!("Contagem de observações: {count}"));
            }
            None => {
                ui.label("Contagem de observações: não informada");
            }
        }
        if let Some(dropped) = payload
            .get("sightings_dropped")
            .and_then(serde_json::Value::as_u64)
            .filter(|dropped| *dropped > 0)
        {
            ui.label(format!(
                "Observações mais antigas fora do detalhe: {dropped}"
            ));
        }

        if let Some(sightings) = payload
            .get("sightings")
            .and_then(serde_json::Value::as_array)
        {
            if !sightings.is_empty() {
                ui.separator();
                ui.label("Observações detalhadas:");
            }
            for (index, sighting) in sightings.iter().enumerate() {
                ui.group(|ui| {
                    ui.set_width(ui.available_width());
                    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
                    ui.strong(format!("Observação {}", index + 1));
                    sighting_field(ui, sighting, "observed_at", "Data");
                    sighting_field(ui, sighting, "evidence_id", "Evidência");
                    sighting_field(ui, sighting, "event_context", "Contexto");
                    sighting_field(ui, sighting, "confidence", "Confiança");
                });
            }
        }
    });
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
                ioc_ui(ui, item);
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

#[cfg(test)]
mod ui_tests {
    use super::IocsState;
    use crate::gui::Loadable;
    use crate::gui::backend::Backend;
    use crate::service::AnalyticalResultResponse;
    use egui_kittest::{Harness, kittest::Queryable};

    type State = (IocsState, Backend);

    fn ioc(payload: serde_json::Value) -> AnalyticalResultResponse {
        AnalyticalResultResponse {
            id: "ioc-1".into(),
            result_type: "IOC".into(),
            correlation_key: "ioc-key-1".into(),
            evidence_id: Some("ev-1".into()),
            incident_id: Some("inc-1".into()),
            analysis_job_id: None,
            rule_or_model_id: None,
            rule_or_model_version: None,
            confidence: Some("HIGH".into()),
            payload,
            provenance: serde_json::Value::Null,
            supersedes: None,
            created_at: "2026-10-16T00:00:00Z".into(),
        }
    }

    fn harness_for(state: IocsState, backend: Backend) -> Harness<'static, State> {
        crate::gui::test_harness::builder()
            .with_size(egui::vec2(760.0, 1500.0))
            .build_ui_state(
                |ui, (state, backend): &mut State| state.ui(ui, "inc-1", backend),
                (state, backend),
            )
    }

    #[test]
    fn loaded_ioc_shows_summary_and_sightings_instead_of_raw_json() {
        let (backend, _commands) = Backend::for_test();
        let payload = serde_json::json!({
            "kind": "IOC",
            "ioc_type": "IP",
            "normalized_value": "203.0.113.7",
            "first_seen": "2026-10-01T10:00:00Z",
            "last_seen": "2026-10-03T12:30:00Z",
            "sighting_count": 3,
            "sightings_dropped": 1,
            "sightings": [
                {
                    "evidence_id": "ev-1",
                    "event_context": "Conexão de saída pelo servidor FS02",
                    "confidence": "HIGH",
                    "observed_at": "2026-10-01T10:00:00Z"
                },
                {
                    "evidence_id": "ev-2",
                    "event_context": "Nova conexão registrada pelo firewall de borda",
                    "confidence": "MEDIUM",
                    "observed_at": "2026-10-03T12:30:00Z"
                }
            ]
        });
        let raw_payload = payload.to_string();
        let state = IocsState {
            loaded_for: Some("inc-1".into()),
            list: Loadable::Ready(vec![ioc(payload)]),
            ..IocsState::default()
        };
        let mut harness = harness_for(state, backend);
        harness.run();

        for label in [
            "Tipo: IP",
            "Valor normalizado: 203.0.113.7",
            "Primeira observação: 2026-10-01T10:00:00Z",
            "Última observação: 2026-10-03T12:30:00Z",
            "Contagem de observações: 3",
            "Observações mais antigas fora do detalhe: 1",
            "Observações detalhadas:",
            "Observação 1",
            "Data: 2026-10-01T10:00:00Z",
            "Evidência: ev-1",
            "Contexto: Conexão de saída pelo servidor FS02",
            "Confiança: HIGH",
            "Observação 2",
            "Data: 2026-10-03T12:30:00Z",
            "Evidência: ev-2",
            "Contexto: Nova conexão registrada pelo firewall de borda",
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
                .query_by_label_contains("\"normalized_value\":")
                .is_none(),
            "nenhuma chave JSON bruta deve virar interface"
        );
    }
}
