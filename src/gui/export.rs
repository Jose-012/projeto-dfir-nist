//! Aba "Exportar": grava IOCs + recomendações vigentes do incidente em um
//! arquivo JSON local, escolhido via diálogo nativo de "Salvar como" — sem
//! servidor, sem download de navegador (F1-05/RF-022).

use super::Loadable;
use super::backend::{Backend, Command};
use super::file_dialog::{FilePicker, Picked, Request};
use std::path::PathBuf;

#[derive(Default)]
pub struct ExportState {
    exporting: bool,
    last_result: Loadable<PathBuf>,
    /// Aviso neutro do seletor de arquivo (cancelado/indisponível).
    hint: Option<String>,
    /// Caminho digitado, alternativa ao seletor nativo (GUI-01).
    manual_path: String,
    picker: FilePicker,
}

impl ExportState {
    pub fn on_exported(&mut self, _incident_id: &str, result: Result<PathBuf, String>) {
        self.exporting = false;
        self.last_result = match result {
            Ok(path) => Loadable::Ready(path),
            Err(message) => Loadable::Failed(message),
        };
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, incident_id: &str, backend: &Backend) {
        ui.heading("Exportar achados");
        ui.label("Gera um arquivo JSON com os IOCs e recomendações vigentes deste incidente — a mesma origem exibida nas abas acima.");
        match self.picker.poll() {
            Some(Picked::Path(save_path)) => self.begin_export(incident_id, save_path, backend),
            Some(Picked::Nothing) => {
                self.hint = Some(
                    "Nenhum destino escolhido. Se o seletor não abriu, informe o caminho no campo abaixo."
                        .into(),
                );
            }
            None => {}
        }
        if ui
            .add_enabled(
                !self.exporting && !self.picker.is_busy(),
                egui::Button::new("Exportar para arquivo…"),
            )
            .clicked()
            && let Err(message) = self.picker.open(Request::SaveFile {
                title: "Salvar achados do incidente".into(),
                file_name: format!("incidente-{incident_id}-achados.json"),
                filter_name: "JSON".into(),
                extensions: vec!["json".into()],
            })
        {
            self.hint = Some(message);
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
            let can_export_typed = !self.exporting && !self.manual_path.trim().is_empty();
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
        if self.exporting {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("Exportando…");
            });
        }
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

    fn begin_export(&mut self, incident_id: &str, save_path: PathBuf, backend: &Backend) {
        self.hint = None;
        self.manual_path.clear();
        self.exporting = true;
        backend.send(Command::ExportFindings {
            incident_id: incident_id.to_string(),
            save_path,
        });
    }
}
