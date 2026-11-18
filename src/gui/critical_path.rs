//! Aba "Caminho crítico": estágios sequenciais do ataque e as arestas
//! `SEQUENCE_NEXT` entre eles (F1-04).

use super::backend::{Backend, Command};
use super::{CONFIDENCE_OPTIONS, Loadable, enum_combo, non_empty};
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
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.strong(format!("#{}", stage.id));
                            ui.small(stage.payload.to_string());
                        });
                    });
                }
                ui.label(format!("{} ligações (SEQUENCE_NEXT):", graph.edges.len()));
                for edge in &graph.edges {
                    ui.label(format!("{} → {}", edge.from, edge.to));
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
