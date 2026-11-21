//! Aba "Exportar": grava IOCs + recomendações vigentes do incidente em um
//! arquivo JSON local, escolhido via diálogo nativo de "Salvar como" — sem
//! servidor, sem download de navegador (F1-05/RF-022).

use super::Loadable;
use super::backend::{Backend, Command};
use super::file_dialog::{FilePicker, Picked, Request};
use std::path::PathBuf;

#[derive(Default)]
pub struct ExportState {
    /// Exportação em curso e incidente dono do pedido.
    exporting_for: Option<String>,
    last_result: Loadable<PathBuf>,
    result_for: Option<String>,
    /// Aviso neutro do seletor de arquivo (cancelado/indisponível).
    hint: Option<String>,
    /// Caminho digitado, alternativa ao seletor nativo (GUI-01).
    manual_path: String,
    /// Incidente que abriu o seletor; uma resposta depois da troca de
    /// incidente não pode exportar o caso errado.
    picker_for: Option<String>,
    picker: FilePicker,
}

impl ExportState {
    pub fn on_exported(&mut self, incident_id: &str, result: Result<PathBuf, String>) {
        if self.exporting_for.as_deref() != Some(incident_id) {
            return;
        }
        self.exporting_for = None;
        self.result_for = Some(incident_id.to_string());
        self.last_result = match result {
            Ok(path) => Loadable::Ready(path),
            Err(message) => Loadable::Failed(message),
        };
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, incident_id: &str, backend: &Backend) {
        ui.heading("Exportar achados");
        ui.label("Gera um arquivo JSON com os IOCs e recomendações vigentes deste incidente — a mesma origem exibida nas abas acima.");
        ui.small(
            "O JSON preserva os valores dos IOCs e não aplica a máscara dos entregáveis; grave-o em volume protegido.",
        );
        match self.picker.poll() {
            Some(Picked::Path(save_path)) => match self.picker_for.take() {
                Some(requested_for) if requested_for == incident_id => {
                    self.begin_export(incident_id, save_path, backend);
                }
                Some(_) => {
                    self.hint = Some(
                        "O destino foi descartado porque o incidente mudou; escolha novamente."
                            .into(),
                    );
                }
                None => {}
            },
            Some(Picked::Cancelled) if self.picker_for.take().as_deref() == Some(incident_id) => {
                self.hint =
                    Some("Nenhum destino escolhido; nenhuma exportação foi iniciada.".into());
            }
            Some(Picked::Unavailable(message)) => {
                if self.picker_for.take().as_deref() == Some(incident_id) {
                    self.hint = Some(message);
                }
            }
            Some(Picked::Failed(message)) => {
                if self.picker_for.take().as_deref() == Some(incident_id) {
                    self.hint = Some(message);
                }
            }
            Some(Picked::Cancelled) | None => {}
        }
        let clicked = ui
            .add_enabled(
                self.exporting_for.is_none() && !self.picker.is_busy(),
                egui::Button::new("Exportar para arquivo…"),
            )
            .clicked();
        if clicked {
            self.picker_for = Some(incident_id.to_string());
            if let Err(message) = self.picker.open(
                Request::SaveFile {
                    title: "Salvar achados do incidente".into(),
                    file_name: format!("incidente-{incident_id}-achados.json"),
                    filter_name: "JSON".into(),
                    extensions: vec!["json".into()],
                },
                ui.ctx().clone(),
            ) {
                self.picker_for = None;
                self.hint = Some(message);
            }
        }
        ui.horizontal(|ui| {
            let label_id = ui.label("Ou informe o caminho de destino:").id;
            let entry = ui
                .add(
                    egui::TextEdit::singleline(&mut self.manual_path)
                        .hint_text("/caminho/para/achados.json")
                        .desired_width(420.0),
                )
                .labelled_by(label_id);
            let submitted =
                entry.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter));
            let can_export_typed =
                self.exporting_for.is_none() && !self.manual_path.trim().is_empty();
            let clicked = ui
                .add_enabled(
                    can_export_typed,
                    egui::Button::new("Exportar para este caminho"),
                )
                .clicked();
            if can_export_typed && (clicked || submitted) {
                let path = PathBuf::from(self.manual_path.trim());
                self.begin_export(incident_id, path, backend);
            }
        });
        if let Some(hint) = &self.hint {
            super::hint_banner(ui, hint);
        }
        if self.exporting_for.as_deref() == Some(incident_id) {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("Exportando…");
            });
        }
        if self.result_for.as_deref() == Some(incident_id) {
            match &self.last_result {
                Loadable::Ready(path) => {
                    ui.colored_label(
                        egui::Color32::from_rgb(60, 150, 60),
                        format!("Exportado para {}", path.display()),
                    );
                }
                Loadable::Failed(message) => super::error_banner(ui, message),
                _ => {}
            }
        }
    }

    fn begin_export(&mut self, incident_id: &str, save_path: PathBuf, backend: &Backend) {
        if self.exporting_for.is_some() {
            self.hint = Some("Já existe uma exportação em andamento; aguarde o fim.".into());
            return;
        }
        // O caminho manual é uma alternativa ao seletor. Se ambos terminarem,
        // só o primeiro inicia um job; a resposta tardia do outro é ignorada.
        self.picker_for = None;
        self.exporting_for = Some(incident_id.to_string());
        self.last_result = Loadable::Idle;
        self.result_for = None;
        self.hint = None;
        self.manual_path.clear();
        backend.send(Command::ExportFindings {
            incident_id: incident_id.to_string(),
            save_path,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::ExportState;
    use crate::gui::backend::{Backend, Command};
    use crate::gui::file_dialog::{FilePicker, Picked};
    use crate::gui::test_harness::drain;
    use egui_kittest::{Harness, kittest::Queryable};
    use std::path::PathBuf;

    type State = (ExportState, Backend, &'static str);

    fn harness(state: ExportState, backend: Backend) -> Harness<'static, State> {
        harness_for_incident(state, backend, "inc-1")
    }

    fn harness_for_incident(
        state: ExportState,
        backend: Backend,
        incident_id: &'static str,
    ) -> Harness<'static, State> {
        crate::gui::test_harness::builder().build_ui_state(
            |ui, (state, backend, incident_id): &mut State| {
                state.ui(ui, incident_id, backend);
            },
            (state, backend, incident_id),
        )
    }

    #[test]
    fn the_typed_destination_is_trimmed_and_sent_to_the_backend() {
        let (backend, mut commands) = Backend::for_test();
        let state = ExportState {
            manual_path: "  /tmp/achados.json  ".into(),
            ..ExportState::default()
        };
        let mut harness = harness(state, backend);
        harness.step();
        harness.get_by_label("Exportar para este caminho").click();
        harness.step();

        let sent = drain(&mut commands);
        assert_eq!(sent.len(), 1, "{sent:?}");
        assert!(matches!(
            &sent[0],
            Command::ExportFindings { incident_id, save_path }
                if incident_id == "inc-1" && save_path == &PathBuf::from("/tmp/achados.json")
        ));
        assert_eq!(harness.state().0.exporting_for.as_deref(), Some("inc-1"));
        assert!(harness.state().0.manual_path.is_empty());
        harness.step();
        harness.get_by_label("Exportando…");

        harness
            .state_mut()
            .0
            .on_exported("inc-1", Ok(PathBuf::from("/tmp/achados.json")));
        harness.run();
        assert!(harness.state().0.exporting_for.is_none());
        harness.get_by_label("Exportado para /tmp/achados.json");
    }

    #[test]
    fn a_failed_export_shows_the_reason_and_allows_a_retry() {
        let (backend, mut commands) = Backend::for_test();
        let mut state = ExportState::default();
        state.begin_export("inc-1", PathBuf::from("/tmp/primeiro.json"), &backend);
        assert_eq!(drain(&mut commands).len(), 1);
        state.on_exported("inc-1", Err("destino protegido".into()));
        assert!(state.exporting_for.is_none());

        let mut harness = harness(state, backend);
        harness.run();
        harness.get_by_label("Erro: destino protegido");

        let (state, backend, _) = harness.state_mut();
        state.begin_export("inc-1", PathBuf::from("/tmp/segundo.json"), backend);
        harness.step();
        harness.get_by_label("Exportando…");
        assert!(
            harness.query_by_label("Erro: destino protegido").is_none(),
            "o erro da tentativa anterior não pode permanecer durante a nova"
        );
        let sent = drain(&mut commands);
        assert!(matches!(
            sent.as_slice(),
            [Command::ExportFindings { save_path, .. }]
                if save_path == &PathBuf::from("/tmp/segundo.json")
        ));
    }

    #[test]
    fn cancelling_the_save_selector_shows_the_alternative_without_a_job() {
        let (picker, sender) = FilePicker::for_test();
        let state = ExportState {
            picker,
            picker_for: Some("inc-1".into()),
            ..ExportState::default()
        };
        let (backend, mut commands) = Backend::for_test();
        let mut harness = harness(state, backend);
        harness.step();
        sender
            .send(Picked::Cancelled)
            .expect("cancelamento sintético");
        harness.step();

        harness.get_by_label_contains("Nenhum destino escolhido");
        harness.get_by_label("Ou informe o caminho de destino:");
        assert!(drain(&mut commands).is_empty());
    }

    #[test]
    fn late_export_results_do_not_complete_or_leak_into_another_incident() {
        let (backend, mut commands) = Backend::for_test();
        let mut state = ExportState::default();
        state.begin_export("inc-1", PathBuf::from("/tmp/inc-1.json"), &backend);
        assert_eq!(drain(&mut commands).len(), 1);

        state.on_exported("inc-2", Err("resposta-atrasada".into()));
        assert_eq!(state.exporting_for.as_deref(), Some("inc-1"));
        assert!(state.result_for.is_none());

        state.on_exported("inc-1", Err("falha-do-inc-1".into()));
        assert!(state.exporting_for.is_none());
        let mut harness = harness_for_incident(state, backend, "inc-2");
        harness.run();
        assert!(
            harness
                .query_by_label_contains("resposta-atrasada")
                .is_none()
        );
        assert!(harness.query_by_label_contains("falha-do-inc-1").is_none());
    }

    #[test]
    fn a_destination_chosen_after_switching_incidents_is_discarded() {
        let (picker, sender) = FilePicker::for_test();
        let state = ExportState {
            picker,
            picker_for: Some("inc-1".into()),
            ..ExportState::default()
        };
        let (backend, mut commands) = Backend::for_test();
        let mut harness = harness_for_incident(state, backend, "inc-2");
        sender
            .send(Picked::Path(PathBuf::from("/tmp/destino-de-inc-1.json")))
            .expect("escolha sintética");
        harness.step();

        harness.get_by_label_contains("o incidente mudou");
        assert!(drain(&mut commands).is_empty());
    }

    #[test]
    fn the_screen_warns_that_findings_json_is_not_masked() {
        let (backend, mut commands) = Backend::for_test();
        let mut harness = harness(ExportState::default(), backend);
        harness.run();
        harness.get_by_label_contains("não aplica a máscara dos entregáveis");
        harness.get_by_label_contains("volume protegido");
        assert!(drain(&mut commands).is_empty());
    }

    #[test]
    fn an_active_export_cannot_be_started_twice() {
        let (backend, mut commands) = Backend::for_test();
        let mut state = ExportState::default();
        state.begin_export("inc-1", PathBuf::from("/synthetic/primeiro.json"), &backend);
        assert_eq!(drain(&mut commands).len(), 1);

        state.begin_export("inc-1", PathBuf::from("/synthetic/segundo.json"), &backend);
        assert!(drain(&mut commands).is_empty());
        assert_eq!(state.exporting_for.as_deref(), Some("inc-1"));
        assert!(state.hint.as_deref().unwrap_or("").contains("em andamento"));
    }

    #[test]
    fn a_manual_export_invalidates_a_late_save_selection() {
        let (picker, sender) = FilePicker::for_test();
        let mut state = ExportState {
            picker,
            picker_for: Some("inc-1".into()),
            ..ExportState::default()
        };
        let (backend, mut commands) = Backend::for_test();
        state.begin_export("inc-1", PathBuf::from("/synthetic/digitado.json"), &backend);
        assert_eq!(drain(&mut commands).len(), 1);
        state.on_exported("inc-1", Err("exportação manual terminou".into()));

        let mut harness = harness(state, backend);
        sender
            .send(Picked::Path(PathBuf::from(
                "/synthetic/selecionado-depois.json",
            )))
            .expect("resposta tardia sintética");
        harness.step();
        assert!(
            drain(&mut commands).is_empty(),
            "o seletor antigo iniciou uma segunda exportação"
        );
    }
}
