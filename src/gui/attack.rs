//! Aba "ATT&CK": mapeamento de táticas/técnicas MITRE ATT&CK a evidências
//! (F1-04).

use super::backend::{Backend, Command};
use super::{
    ATTACK_TACTIC_OPTIONS, CONFIDENCE_OPTIONS, Loadable, enum_combo, non_empty, str_field,
};
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

fn mapping_ui(ui: &mut egui::Ui, item: &AnalyticalResultResponse) {
    let payload = &item.payload;
    ui.group(|ui| {
        ui.set_width(ui.available_width());
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);

        ui.strong(format!("Técnica: {}", str_field(payload, "technique_id")));
        ui.label(format!("Tática: {}", str_field(payload, "tactic")));
        let sub_technique = str_field(payload, "sub_technique_id");
        ui.label(format!(
            "Subtécnica: {}",
            if sub_technique.is_empty() {
                "não informada"
            } else {
                sub_technique
            }
        ));

        let evidence = item.evidence_id.as_deref().unwrap_or("não vinculada");
        let evidence_status = str_field(payload, "evidence_status");
        if evidence_status.is_empty() {
            ui.label(format!("Evidência: {evidence}"));
        } else {
            ui.label(format!(
                "Evidência: {evidence} · situação: {evidence_status}"
            ));
        }

        let basis = str_field(payload, "basis");
        ui.label(format!(
            "Base analítica: {}",
            if basis.is_empty() {
                "não informada"
            } else {
                basis
            }
        ));
        ui.label(format!(
            "Confiança: {}",
            item.confidence.as_deref().unwrap_or("não informada")
        ));
        let analyst_note = str_field(payload, "analyst_note");
        if !analyst_note.is_empty() {
            ui.label(format!("Nota do analista: {analyst_note}"));
        }
    });
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
                mapping_ui(ui, item);
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

#[cfg(test)]
mod ui_tests {
    use super::AttackState;
    use crate::gui::Loadable;
    use crate::gui::backend::Backend;
    use crate::service::AnalyticalResultResponse;
    use egui_kittest::{Harness, kittest::Queryable};

    type State = (AttackState, Backend);

    fn mapping(payload: serde_json::Value) -> AnalyticalResultResponse {
        AnalyticalResultResponse {
            id: "attack-1".into(),
            result_type: "ATTACK_MAPPING".into(),
            correlation_key: "attack-key-1".into(),
            evidence_id: Some("ev-42".into()),
            incident_id: Some("inc-1".into()),
            analysis_job_id: None,
            rule_or_model_id: Some("T1078".into()),
            rule_or_model_version: None,
            confidence: Some("HIGH".into()),
            payload,
            provenance: serde_json::Value::Null,
            supersedes: None,
            created_at: "2026-10-16T00:00:00Z".into(),
        }
    }

    fn harness_for(state: AttackState, backend: Backend) -> Harness<'static, State> {
        crate::gui::test_harness::builder()
            .with_size(egui::vec2(700.0, 1200.0))
            .build_ui_state(
                |ui, (state, backend): &mut State| state.ui(ui, "inc-1", backend),
                (state, backend),
            )
    }

    #[test]
    fn loaded_mapping_shows_structured_fields_instead_of_raw_json() {
        let (backend, _commands) = Backend::for_test();
        let payload = serde_json::json!({
            "kind": "ATTACK_MAPPING",
            "tactic": "CREDENTIAL_ACCESS",
            "technique_id": "T1078",
            "sub_technique_id": "T1078.001",
            "basis": "Eventos 4624 correlacionados com a conta administrativa durante a janela do incidente",
            "analyst_note": "Revisar a origem do logon antes do encerramento",
            "evidence_status": "DOCUMENTED"
        });
        let raw_payload = payload.to_string();
        let state = AttackState {
            loaded_for: Some("inc-1".into()),
            list: Loadable::Ready(vec![mapping(payload)]),
            ..AttackState::default()
        };
        let mut harness = harness_for(state, backend);
        harness.run();

        for label in [
            "Tática: CREDENTIAL_ACCESS",
            "Técnica: T1078",
            "Subtécnica: T1078.001",
            "Evidência: ev-42 · situação: DOCUMENTED",
            "Base analítica: Eventos 4624 correlacionados com a conta administrativa durante a janela do incidente",
            "Confiança: HIGH",
            "Nota do analista: Revisar a origem do logon antes do encerramento",
        ] {
            harness.get_by_label(label);
        }
        assert!(
            harness.query_by_label(raw_payload.as_str()).is_none(),
            "o payload JSON minificado não deve ser exibido"
        );
        assert!(
            harness
                .query_by_label_contains("\"technique_id\":")
                .is_none(),
            "nenhuma chave JSON bruta deve virar interface"
        );
    }
}
