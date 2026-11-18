//! Aba "Caminho crítico": estágios sequenciais do ataque e as arestas
//! `SEQUENCE_NEXT` entre eles (F1-04).

use super::backend::{Backend, Command};
use super::{CONFIDENCE_OPTIONS, Loadable, enum_combo, non_empty, str_field};
use crate::service::AnalyticalResultResponse;
use crate::service::findings::CriticalPathView;

#[derive(Default)]
struct StageForm {
    evidence_id: String,
    sequence_index: String,
    stage_label: String,
    basis: String,
    confidence: String,
}

#[derive(Default)]
struct LinkForm {
    from_stage_id: String,
    to_stage_id: String,
}

fn stage_ui(ui: &mut egui::Ui, stage: &AnalyticalResultResponse) {
    let payload = &stage.payload;
    ui.group(|ui| {
        ui.set_width(ui.available_width());
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);

        ui.horizontal_wrapped(|ui| {
            ui.strong(format!("#{}", stage.id));
            let sequence = payload
                .get("sequence_index")
                .and_then(serde_json::Value::as_i64)
                .map(|value| value.to_string())
                .unwrap_or_else(|| "não informada".into());
            ui.label(format!("Sequência: {sequence}"));
            ui.label(format!(
                "Confiança: {}",
                stage.confidence.as_deref().unwrap_or("não informada")
            ));
        });
        ui.strong(str_field(payload, "stage_label"));
        ui.label(format!("Base analítica: {}", str_field(payload, "basis")));
        ui.label(format!(
            "Evidência: {}",
            stage.evidence_id.as_deref().unwrap_or("não vinculada")
        ));
        ui.label(format!(
            "Situação da evidência: {}",
            str_field(payload, "evidence_status")
        ));
    });
}

pub struct CriticalPathState {
    loaded_for: Option<String>,
    graph: Loadable<CriticalPathView>,
    stage_form: StageForm,
    creating_stage: bool,
    stage_error: Option<String>,
    link_form: LinkForm,
    linking: bool,
    link_error: Option<String>,
}

impl Default for CriticalPathState {
    fn default() -> Self {
        Self {
            loaded_for: None,
            graph: Loadable::Idle,
            stage_form: StageForm::default(),
            creating_stage: false,
            stage_error: None,
            link_form: LinkForm::default(),
            linking: false,
            link_error: None,
        }
    }
}

impl CriticalPathState {
    fn ensure_loaded(&mut self, incident_id: &str, backend: &Backend) {
        if self.loaded_for.as_deref() == Some(incident_id) {
            return;
        }
        self.loaded_for = Some(incident_id.to_string());
        self.graph = Loadable::Loading;
        backend.send(Command::CriticalPath(incident_id.to_string()));
    }

    pub fn on_loaded(&mut self, incident_id: &str, result: Result<CriticalPathView, String>) {
        if self.loaded_for.as_deref() != Some(incident_id) {
            return;
        }
        self.graph = match result {
            Ok(view) => Loadable::Ready(view),
            Err(message) => Loadable::Failed(message),
        };
    }

    pub fn on_stage_created(
        &mut self,
        incident_id: &str,
        result: Result<String, String>,
        backend: &Backend,
    ) {
        self.creating_stage = false;
        match result {
            Ok(_) => {
                self.stage_error = None;
                self.stage_form = StageForm::default();
                backend.send(Command::CriticalPath(incident_id.to_string()));
            }
            Err(message) => self.stage_error = Some(message),
        }
    }

    pub fn on_linked(&mut self, incident_id: &str, result: Result<(), String>, backend: &Backend) {
        self.linking = false;
        match result {
            Ok(()) => {
                self.link_error = None;
                self.link_form = LinkForm::default();
                backend.send(Command::CriticalPath(incident_id.to_string()));
            }
            Err(message) => self.link_error = Some(message),
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, incident_id: &str, backend: &Backend) {
        self.ensure_loaded(incident_id, backend);

        ui.heading("Estágios do caminho crítico");
        self.graph.show(ui, |ui, graph| {
            if graph.stages.is_empty() {
                ui.label("Nenhum estágio ainda.");
            } else {
                for stage in &graph.stages {
                    stage_ui(ui, stage);
                }
                ui.label(format!("{} ligações (SEQUENCE_NEXT):", graph.edges.len()));
                for edge in &graph.edges {
                    ui.label(format!("{} -> {}", edge.from, edge.to));
                }
            }
        });

        ui.separator();
        ui.collapsing("Novo estágio", |ui| {
            let form = &mut self.stage_form;
            super::field_text(ui, "ID da evidência (opcional)", &mut form.evidence_id);
            super::field_text(
                ui,
                "Índice de sequência (inteiro)",
                &mut form.sequence_index,
            );
            super::field_text(ui, "Rótulo do estágio", &mut form.stage_label);
            super::field_text_multiline(ui, "Base analítica", &mut form.basis);
            enum_combo(
                ui,
                "path_confidence",
                "Confiança",
                &mut form.confidence,
                &CONFIDENCE_OPTIONS,
            );
            if let Some(message) = &self.stage_error {
                super::error_banner(ui, message);
            }
            if ui
                .add_enabled(!self.creating_stage, egui::Button::new("Criar estágio"))
                .clicked()
            {
                match form.sequence_index.trim().parse::<i64>() {
                    Ok(sequence_index) => {
                        self.creating_stage = true;
                        backend.send(Command::CreateCriticalPathStage {
                            incident_id: incident_id.to_string(),
                            evidence_id: non_empty(&form.evidence_id),
                            sequence_index,
                            stage_label: form.stage_label.clone(),
                            basis: form.basis.clone(),
                            confidence: form.confidence.clone(),
                        });
                    }
                    Err(_) => {
                        self.stage_error =
                            Some("índice de sequência deve ser um número inteiro".into())
                    }
                }
            }
            if self.creating_stage {
                ui.spinner();
            }
        });

        ui.collapsing("Ligar dois estágios (SEQUENCE_NEXT)", |ui| {
            let form = &mut self.link_form;
            super::field_text(ui, "ID do estágio de origem", &mut form.from_stage_id);
            super::field_text(ui, "ID do estágio de destino", &mut form.to_stage_id);
            if let Some(message) = &self.link_error {
                super::error_banner(ui, message);
            }
            if ui
                .add_enabled(!self.linking, egui::Button::new("Ligar estágios"))
                .clicked()
            {
                self.linking = true;
                backend.send(Command::LinkCriticalPathStages {
                    incident_id: incident_id.to_string(),
                    from_stage_id: form.from_stage_id.clone(),
                    to_stage_id: form.to_stage_id.clone(),
                });
            }
            if self.linking {
                ui.spinner();
            }
        });
    }
}

#[cfg(test)]
mod ui_tests {
    use super::CriticalPathState;
    use crate::gui::Loadable;
    use crate::gui::backend::Backend;
    use crate::service::AnalyticalResultResponse;
    use crate::service::findings::{CriticalPathEdge, CriticalPathView};
    use egui_kittest::{Harness, kittest::Queryable};

    type State = (CriticalPathState, Backend);

    fn stage(payload: serde_json::Value) -> AnalyticalResultResponse {
        AnalyticalResultResponse {
            id: "stage-1".into(),
            result_type: "CRITICAL_PATH_LINK".into(),
            correlation_key: "critical-path-stage:inc-1:7".into(),
            evidence_id: Some("ev-42".into()),
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

    fn harness_for(state: CriticalPathState, backend: Backend) -> Harness<'static, State> {
        // Mantém o default estrito do kittest: inclusive as arestas não podem
        // depender de uma fonte de emoji/símbolos ausente na máquina.
        egui_kittest::HarnessBuilder::default()
            .with_size(egui::vec2(760.0, 1300.0))
            .build_ui_state(
                |ui, (state, backend): &mut State| state.ui(ui, "inc-1", backend),
                (state, backend),
            )
    }

    #[test]
    fn loaded_stage_shows_structured_fields_and_long_basis_instead_of_raw_json() {
        let (backend, _commands) = Backend::for_test();
        let basis = "Os eventos de logon, a alteração das regras de caixa e a execução do PowerShell foram correlacionados pela mesma conta administrativa e pela janela do incidente, com a evidência original preservada e revisada pelo analista antes do vínculo ao caminho crítico.";
        let payload = serde_json::json!({
            "kind": "CRITICAL_PATH_STAGE",
            "sequence_index": 7,
            "stage_label": "Credencial capturada e login anômalo na caixa do financeiro",
            "basis": basis,
            "evidence_status": "DOCUMENTED"
        });
        let raw_payload = payload.to_string();
        let state = CriticalPathState {
            loaded_for: Some("inc-1".into()),
            graph: Loadable::Ready(CriticalPathView {
                stages: vec![stage(payload)],
                edges: vec![CriticalPathEdge {
                    from: "stage-1".into(),
                    to: "stage-2".into(),
                }],
            }),
            ..CriticalPathState::default()
        };
        let mut harness = harness_for(state, backend);
        harness.run();

        for label in [
            "#stage-1".to_string(),
            "Sequência: 7".into(),
            "Confiança: HIGH".into(),
            "Credencial capturada e login anômalo na caixa do financeiro".into(),
            format!("Base analítica: {basis}"),
            "Evidência: ev-42".into(),
            "Situação da evidência: DOCUMENTED".into(),
            "stage-1 -> stage-2".into(),
        ] {
            harness.get_by_label(label.as_str());
        }
        assert!(
            harness.query_by_label(raw_payload.as_str()).is_none(),
            "o payload JSON minificado não deve ser exibido"
        );
        assert!(
            harness.query_by_label_contains("\"basis\":").is_none(),
            "nenhuma chave JSON bruta deve virar interface"
        );
    }
}
