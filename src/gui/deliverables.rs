//! Aba "Entregáveis": pré-visualiza e gera os cinco documentos dos
//! entregáveis (Markdown) e o `manifest.json` (F1-07, RF-013).
//!
//! A pré-visualização mostra exatamente o que será gravado (mesma
//! renderização, sem escrita). Gerar abre o seletor nativo de pasta e grava
//! em `<pasta>/entregaveis-<id do incidente>/`.
//!
//! **Dados pessoais são mascarados por padrão** (F1-32). Exportar sem a
//! máscara é uma escolha explícita: desmarcar a opção mostra um aviso e só
//! libera a geração depois de uma confirmação.

use super::Loadable;
use super::backend::{Backend, Command};
use super::file_dialog::{FilePicker, Picked, Request};
use crate::service::deliverables::{DeliverableSet, GeneratedDeliverables};
use std::path::PathBuf;

#[derive(Default)]
pub struct DeliverablesState {
    /// Incidente da pré-visualização pedida (evita aplicar resposta atrasada
    /// de outro incidente).
    previewing_for: Option<String>,
    preview: Loadable<DeliverableSet>,
    selected: usize,
    /// Geração em curso e incidente dono do pedido. Respostas de outro
    /// incidente não podem concluir nem substituir este estado.
    generating_for: Option<String>,
    generated: Option<Result<GeneratedDeliverables, String>>,
    generated_for: Option<String>,
    /// O padrão é mascarar: só a escolha explícita de exportar sem máscara
    /// liga este sinalizador.
    export_without_mask: bool,
    confirmed_without_mask: bool,
    /// Aviso neutro do seletor de pasta (cancelado/indisponível).
    picker_hint: Option<String>,
    /// Pasta digitada, alternativa ao seletor nativo (GUI-01).
    manual_out_dir: String,
    /// Incidente que abriu o seletor; uma escolha depois da troca de
    /// incidente é descartada em vez de gerar o caso errado.
    picker_for: Option<String>,
    picker: FilePicker,
}

impl DeliverablesState {
    pub fn on_previewed(&mut self, incident_id: &str, result: Result<DeliverableSet, String>) {
        if self.previewing_for.as_deref() != Some(incident_id) {
            return;
        }
        self.preview = match result {
            Ok(set) => Loadable::Ready(set),
            Err(message) => Loadable::Failed(message),
        };
    }

    pub fn on_generated(
        &mut self,
        incident_id: &str,
        result: Result<GeneratedDeliverables, String>,
    ) {
        if self.generating_for.as_deref() != Some(incident_id) {
            return;
        }
        self.generating_for = None;
        self.generated_for = Some(incident_id.to_string());
        self.generated = Some(result);
    }

    fn mask_personal_data(&self) -> bool {
        !self.export_without_mask
    }

    /// Gerar exige que não haja uma geração em curso e, se a máscara foi
    /// desligada, que o operador tenha confirmado.
    fn can_generate(&self) -> bool {
        self.generating_for.is_none() && (self.mask_personal_data() || self.confirmed_without_mask)
    }

    fn request_preview(&mut self, incident_id: &str, backend: &Backend) {
        self.previewing_for = Some(incident_id.to_string());
        self.preview = Loadable::Loading;
        self.selected = 0;
        backend.send(Command::PreviewDeliverables {
            incident_id: incident_id.to_string(),
            mask_personal_data: self.mask_personal_data(),
        });
    }

    fn request_generation(&mut self, incident_id: &str, out_dir: PathBuf, backend: &Backend) {
        if self.generating_for.is_some() {
            self.picker_hint = Some("Já existe uma geração em andamento; aguarde o fim.".into());
            return;
        }
        if !self.mask_personal_data() && !self.confirmed_without_mask {
            self.picker_hint = Some(
                "Confirme explicitamente a exportação sem máscara antes de gerar os arquivos."
                    .into(),
            );
            return;
        }
        // Se o caminho digitado foi usado enquanto o seletor estava aberto, a
        // resposta tardia dele não pode iniciar uma segunda geração.
        self.picker_for = None;
        self.generating_for = Some(incident_id.to_string());
        self.generated = None;
        self.generated_for = None;
        self.picker_hint = None;
        backend.send(Command::GenerateDeliverables {
            incident_id: incident_id.to_string(),
            out_dir,
            mask_personal_data: self.mask_personal_data(),
        });
    }

    /// A máscara de dados pessoais e a escolha explícita de exportar sem ela.
    fn mask_options_ui(&mut self, ui: &mut egui::Ui, incident_id: &str) {
        let mut mask = self.mask_personal_data();
        if ui
            .checkbox(
                &mut mask,
                "Mascarar dados pessoais (IPs, e-mails, contas DOMÍNIO\\usuário e nomes de quem forneceu ou custodiou a evidência)",
            )
            .changed()
        {
            self.export_without_mask = !mask;
            self.confirmed_without_mask = false;
            // Respostas do pedido anterior precisam ser invalidadas: em
            // particular, uma prévia sem máscara não pode chegar depois de a
            // máscara ter sido reativada e expor o conteúdo antigo.
            self.previewing_for = None;
            self.preview = Loadable::Idle;
            self.selected = 0;
        }
        if self.export_without_mask {
            let flagged = self
                .preview
                .ready()
                .filter(|_| self.previewing_for.as_deref() == Some(incident_id))
                .map_or(0, |set| set.personal_data_evidence);
            let marked = if flagged > 0 {
                format!(" {flagged} evidência(s) foi/foram marcada(s) como contendo dado pessoal.")
            } else {
                String::new()
            };
            ui.colored_label(
                egui::Color32::from_rgb(200, 60, 40),
                format!(
                    "Sem a máscara, os documentos trarão IPs, e-mails, contas e nomes reais.{marked}"
                ),
            );
            ui.checkbox(
                &mut self.confirmed_without_mask,
                "Entendo e quero exportar sem máscara",
            );
        } else {
            ui.small(
                "Mascara por padrão de texto; nomes em texto livre não são detectados. O mapeamento dos marcadores não é gravado.",
            );
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, incident_id: &str, backend: &Backend) {
        ui.heading("Entregáveis");
        ui.label(
            "Gera os cinco documentos da demanda em Markdown, a partir do que está registrado neste incidente, e um manifest.json com os hashes. Seções sem dados aparecem como \"sem registro\"; nada é inventado.",
        );
        ui.label("Escopo do protótipo: só logs EVTX são analisados automaticamente.");
        self.mask_options_ui(ui, incident_id);
        match self.picker.poll() {
            Some(Picked::Path(folder)) => match self.picker_for.take() {
                Some(requested_for) if requested_for == incident_id => {
                    self.request_generation(incident_id, folder, backend);
                }
                Some(_) => {
                    self.picker_hint = Some(
                        "A pasta foi descartada porque o incidente mudou; escolha novamente."
                            .into(),
                    );
                }
                None => {}
            },
            Some(Picked::Nothing) => {
                if self.picker_for.take().as_deref() == Some(incident_id) {
                    self.picker_hint = Some(
                        "Nenhuma pasta escolhida. Se o seletor não abriu, informe a pasta no campo abaixo."
                            .into(),
                    );
                }
            }
            None => {}
        }
        ui.horizontal(|ui| {
            let preview_loading =
                self.previewing_for.as_deref() == Some(incident_id) && self.preview.is_loading();
            if ui
                .add_enabled(!preview_loading, egui::Button::new("Pré-visualizar"))
                .clicked()
            {
                self.request_preview(incident_id, backend);
            }
            let clicked = ui
                .add_enabled(
                    self.can_generate() && !self.picker.is_busy(),
                    egui::Button::new("Gerar arquivos…"),
                )
                .clicked();
            if clicked {
                self.picker_for = Some(incident_id.to_string());
                if let Err(message) = self.picker.open(Request::Folder {
                    title: "Pasta onde gravar os entregáveis".into(),
                }) {
                    self.picker_for = None;
                    self.picker_hint = Some(message);
                }
            }
        });
        ui.horizontal(|ui| {
            let label_id = ui.label("Ou informe a pasta:").id;
            let entry = ui
                .add(
                    egui::TextEdit::singleline(&mut self.manual_out_dir)
                        .hint_text("/caminho/para/a/saida")
                        .desired_width(420.0),
                )
                .labelled_by(label_id);
            let submitted =
                entry.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter));
            let can_generate_typed = self.can_generate() && !self.manual_out_dir.trim().is_empty();
            let clicked = ui
                .add_enabled(can_generate_typed, egui::Button::new("Gerar nesta pasta"))
                .clicked();
            if can_generate_typed && (clicked || submitted) {
                let folder = PathBuf::from(self.manual_out_dir.trim());
                self.manual_out_dir.clear();
                self.request_generation(incident_id, folder, backend);
            }
            ui.label(egui::RichText::new("(a pasta precisa existir)").weak());
        });
        if let Some(hint) = &self.picker_hint {
            super::hint_banner(ui, hint);
        }
        if self.generating_for.as_deref() == Some(incident_id) {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("Gerando…");
            });
        }
        if self.generated_for.as_deref() == Some(incident_id) {
            match &self.generated {
                Some(Ok(done)) => {
                    ui.colored_label(
                        egui::Color32::from_rgb(60, 150, 60),
                        format!("Arquivos gerados em {}", done.output_dir.display()),
                    );
                    egui::Grid::new("generated_files")
                        .striped(true)
                        .show(ui, |ui| {
                            ui.strong("Arquivo");
                            ui.strong("Bytes");
                            ui.strong("SHA-256");
                            ui.end_row();
                            for file in &done.files {
                                ui.label(&file.file_name);
                                ui.label(file.bytes.to_string());
                                ui.monospace(&file.sha256);
                                ui.end_row();
                            }
                        });
                }
                Some(Err(message)) => super::error_banner(ui, message),
                None => {}
            }
        }
        ui.separator();

        if self.previewing_for.as_deref() == Some(incident_id) {
            let Self {
                preview, selected, ..
            } = self;
            preview.show(ui, |ui, set| {
            match &set.redaction {
                Some(summary) => ui.small(format!(
                    "Pré-visualização com máscara: {} IP(s), {} e-mail(s), {} conta(s) e {} pessoa(s) trocados por marcadores.",
                    summary.ips, summary.emails, summary.accounts, summary.persons
                )),
                None => ui.small("Pré-visualização sem máscara."),
            };
            ui.horizontal_wrapped(|ui| {
                for (index, document) in set.documents.iter().enumerate() {
                    ui.selectable_value(selected, index, document.title);
                }
            });
            if let Some(document) = set.documents.get(*selected) {
                ui.small(format!(
                    "{} · {} resultado(s) citado(s)",
                    document.file_name,
                    document.source_result_ids.len()
                ));
                egui::ScrollArea::vertical()
                    .max_height(600.0)
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        ui.add(egui::Label::new(
                            egui::RichText::new(&document.content).monospace(),
                        ));
                    });
            }
            });
        }
    }
}

/// Testes da aba com `egui_kittest` (ADR-0022). O seletor nativo de pasta
/// não é acionado aqui: a geração é exercida por `request_generation`.
#[cfg(test)]
mod ui_tests {
    use super::DeliverablesState;
    use crate::gui::Loadable;
    use crate::gui::backend::{Backend, Command};
    use crate::gui::file_dialog::FilePicker;
    use crate::gui::test_harness::drain;
    use crate::service::deliverables::{
        DeliverableSet, GeneratedDeliverables, GeneratedFile, RenderedDocument,
    };
    use egui_kittest::{Harness, kittest::Queryable};
    use std::path::PathBuf;

    type State = (DeliverablesState, Backend, &'static str);

    fn sample_set() -> DeliverableSet {
        DeliverableSet {
            incident_id: "inc-1".into(),
            incident_name: "Incidente".into(),
            documents: vec![
                RenderedDocument {
                    file_name: "01-relatorio-forense.md",
                    title: "Relatório DFIR",
                    content: "# Relatório DFIR — Incidente\n\nconteudo-do-primeiro\n".into(),
                    source_result_ids: vec!["a".into(), "b".into()],
                },
                RenderedDocument {
                    file_name: "02-vulnerabilidades-attack.md",
                    title: "Vulnerabilidades e MITRE ATT&CK",
                    content: "# Vulnerabilidades\n\nconteudo-do-segundo\n".into(),
                    source_result_ids: vec![],
                },
            ],
            redaction: None,
            personal_data_evidence: 2,
        }
    }

    fn harness(state: DeliverablesState, backend: Backend) -> Harness<'static, State> {
        harness_for_incident(state, backend, "inc-1")
    }

    fn harness_for_incident(
        state: DeliverablesState,
        backend: Backend,
        incident_id: &'static str,
    ) -> Harness<'static, State> {
        crate::gui::test_harness::builder()
            .with_size(egui::vec2(1000.0, 1400.0))
            .build_ui_state(
                |ui, (state, backend, incident_id): &mut State| {
                    state.ui(ui, incident_id, backend);
                },
                (state, backend, incident_id),
            )
    }

    #[test]
    fn opening_the_tab_requests_nothing_and_states_the_scope() {
        let (backend, mut commands) = Backend::for_test();
        let mut harness = harness(DeliverablesState::default(), backend);
        harness.run();
        harness.get_by_label("Pré-visualizar");
        harness.get_by_label("Escopo do protótipo: só logs EVTX são analisados automaticamente.");
        assert!(drain(&mut commands).is_empty(), "nada é pedido sem clique");
    }

    #[test]
    fn clicking_preview_requests_it_once_for_the_incident() {
        let (backend, mut commands) = Backend::for_test();
        let mut harness = harness(DeliverablesState::default(), backend);
        harness.run();
        harness.get_by_label("Pré-visualizar").click();
        harness.step();
        harness.step();
        let sent = drain(&mut commands);
        assert_eq!(sent.len(), 1, "{sent:?}");
        assert!(matches!(
            &sent[0],
            Command::PreviewDeliverables { incident_id, mask_personal_data }
                if incident_id == "inc-1" && *mask_personal_data
        ));
    }

    #[test]
    fn a_loaded_preview_lists_the_documents_and_shows_the_selected_one() {
        let (backend, _commands) = Backend::for_test();
        let mut state = DeliverablesState {
            previewing_for: Some("inc-1".into()),
            ..DeliverablesState::default()
        };
        state.on_previewed("inc-1", Ok(sample_set()));
        let mut harness = harness(state, backend);
        harness.run();

        harness.get_by_label("Relatório DFIR");
        harness.get_by_label("Vulnerabilidades e MITRE ATT&CK");
        harness.get_by_label("01-relatorio-forense.md · 2 resultado(s) citado(s)");
        harness.get_by_label_contains("conteudo-do-primeiro");
        harness
            .get_by_label("Vulnerabilidades e MITRE ATT&CK")
            .click();
        harness.run();
        harness.get_by_label_contains("conteudo-do-segundo");
    }

    #[test]
    fn a_late_preview_for_another_incident_is_ignored() {
        let mut state = DeliverablesState {
            previewing_for: Some("inc-1".into()),
            ..DeliverablesState::default()
        };
        state.on_previewed("outro", Ok(sample_set()));
        assert!(matches!(state.preview, Loadable::Idle));
    }

    #[test]
    fn generation_sends_the_chosen_folder_and_reports_files_with_hashes() {
        let (backend, mut commands) = Backend::for_test();
        let mut state = DeliverablesState::default();
        state.request_generation("inc-1", PathBuf::from("/tmp/saida"), &backend);
        assert_eq!(state.generating_for.as_deref(), Some("inc-1"));
        let sent = drain(&mut commands);
        assert_eq!(sent.len(), 1);
        match &sent[0] {
            Command::GenerateDeliverables {
                incident_id,
                out_dir,
                mask_personal_data,
            } => {
                assert_eq!(incident_id, "inc-1");
                assert_eq!(out_dir, &PathBuf::from("/tmp/saida"));
                assert!(*mask_personal_data, "o padrão é mascarar");
            }
            other => panic!("comando inesperado: {other:?}"),
        }

        state.on_generated(
            "inc-1",
            Ok(GeneratedDeliverables {
                output_dir: PathBuf::from("/tmp/saida/entregaveis-inc-1"),
                generated_at: "2026-10-14T12:00:00Z".into(),
                files: vec![GeneratedFile {
                    file_name: "01-relatorio-forense.md".into(),
                    bytes: 1234,
                    sha256: "ab".repeat(32),
                }],
            }),
        );
        assert!(state.generating_for.is_none());
        let mut harness = harness(state, backend);
        harness.run();
        harness.get_by_label("Arquivos gerados em /tmp/saida/entregaveis-inc-1");
        harness.get_by_label("01-relatorio-forense.md");
        harness.get_by_label(&"ab".repeat(32));
    }

    #[test]
    fn a_failed_generation_shows_the_reason() {
        let (backend, mut commands) = Backend::for_test();
        let mut state = DeliverablesState::default();
        state.request_generation("inc-1", PathBuf::from("/tmp/saida"), &backend);
        drain(&mut commands);
        state.on_generated(
            "inc-1",
            Err("the output folder must be outside the application's internal storage".into()),
        );
        let mut harness = harness(state, backend);
        harness.run();
        harness.get_by_label(
            "Erro: the output folder must be outside the application's internal storage",
        );
    }

    #[test]
    fn masking_is_the_default_and_exporting_without_it_needs_an_explicit_confirmation() {
        let (backend, mut commands) = Backend::for_test();
        let mut state = DeliverablesState::default();
        assert!(state.mask_personal_data());
        assert!(state.can_generate());

        // Desligar a máscara não libera a geração por si só.
        state.export_without_mask = true;
        assert!(!state.mask_personal_data());
        assert!(!state.can_generate(), "falta a confirmação");
        state.request_generation("inc-1", PathBuf::from("/tmp/saida"), &backend);
        assert!(state.generating_for.is_none());
        assert!(
            drain(&mut commands).is_empty(),
            "nada foi enviado sem confirmar"
        );

        state.confirmed_without_mask = true;
        assert!(state.can_generate());
        state.request_generation("inc-1", PathBuf::from("/tmp/saida"), &backend);
        let sent = drain(&mut commands);
        match &sent[0] {
            Command::GenerateDeliverables {
                mask_personal_data, ..
            } => assert!(!*mask_personal_data, "a escolha vai ao serviço"),
            other => panic!("comando inesperado: {other:?}"),
        }

        // Pré-visualização também carrega a escolha.
        state.request_preview("inc-1", &backend);
        assert!(matches!(
            &drain(&mut commands)[0],
            Command::PreviewDeliverables {
                mask_personal_data: false,
                ..
            }
        ));
    }

    #[test]
    fn turning_the_mask_off_shows_the_warning_the_confirmation_and_the_flagged_evidence() {
        let (backend, _commands) = Backend::for_test();
        let mut state = DeliverablesState {
            previewing_for: Some("inc-1".into()),
            ..DeliverablesState::default()
        };
        state.on_previewed("inc-1", Ok(sample_set()));
        let mut harness = harness(state, backend);
        harness.run();
        harness.get_by_label_contains("Mascarar dados pessoais");
        assert!(
            harness
                .query_by_label_contains("Entendo e quero exportar sem máscara")
                .is_none(),
            "com a máscara ligada não há confirmação a dar"
        );

        // O operador desmarca a opção.
        harness
            .get_by_label_contains("Mascarar dados pessoais")
            .click();
        harness.run();
        harness.get_by_label_contains(
            "Sem a máscara, os documentos trarão IPs, e-mails, contas e nomes reais.",
        );
        harness.get_by_label_contains("Entendo e quero exportar sem máscara");
    }

    #[test]
    fn preview_loading_rejects_another_click_and_a_failure_is_visible() {
        let (backend, mut commands) = Backend::for_test();
        let mut harness = harness(DeliverablesState::default(), backend);
        harness.run();
        harness.get_by_label("Pré-visualizar").click();
        harness.step();
        harness.get_by_label("Carregando…");
        assert_eq!(drain(&mut commands).len(), 1);

        harness.get_by_label("Pré-visualizar").click();
        harness.step();
        assert!(
            drain(&mut commands).is_empty(),
            "a prévia em andamento não pode ser enfileirada de novo"
        );

        harness
            .state_mut()
            .0
            .on_previewed("inc-1", Err("falha sintética da prévia".into()));
        harness.step();
        harness.get_by_label("Erro: falha sintética da prévia");
    }

    #[test]
    fn a_preview_is_never_rendered_for_another_incident() {
        let (backend, _commands) = Backend::for_test();
        let mut state = DeliverablesState {
            previewing_for: Some("inc-1".into()),
            ..DeliverablesState::default()
        };
        state.on_previewed("inc-1", Ok(sample_set()));
        let mut harness = harness_for_incident(state, backend, "inc-2");
        harness.run();

        assert!(
            harness
                .query_by_label_contains("conteudo-do-primeiro")
                .is_none(),
            "documento do incidente anterior ficou visível"
        );
        assert!(harness.query_by_label("Relatório DFIR").is_none());
    }

    #[test]
    fn generation_ignores_a_late_response_and_hides_results_from_another_incident() {
        let (backend, mut commands) = Backend::for_test();
        let mut state = DeliverablesState::default();
        state.request_generation("inc-1", PathBuf::from("/synthetic/saida"), &backend);
        assert_eq!(drain(&mut commands).len(), 1);

        state.on_generated("inc-2", Err("resposta-atrasada".into()));
        assert_eq!(state.generating_for.as_deref(), Some("inc-1"));
        assert!(state.generated.is_none());

        state.on_generated("inc-1", Err("falha-do-inc-1".into()));
        assert!(state.generating_for.is_none());
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
    fn enabling_the_mask_invalidates_an_unmasked_preview_already_in_flight() {
        let (backend, mut commands) = Backend::for_test();
        let state = DeliverablesState {
            export_without_mask: true,
            confirmed_without_mask: true,
            ..DeliverablesState::default()
        };
        let mut harness = harness(state, backend);
        harness.run();
        harness.get_by_label("Pré-visualizar").click();
        harness.step();
        assert_eq!(drain(&mut commands).len(), 1);

        harness
            .get_by_label_contains("Mascarar dados pessoais")
            .click();
        harness.step();
        assert_eq!(harness.state().0.previewing_for, None);

        harness
            .state_mut()
            .0
            .on_previewed("inc-1", Ok(sample_set()));
        harness.step();
        assert!(
            harness
                .query_by_label_contains("conteudo-do-primeiro")
                .is_none(),
            "a resposta sem máscara foi aceita depois de reativar a máscara"
        );
    }

    #[test]
    fn cancelling_the_folder_selector_shows_the_alternative_without_a_job() {
        let (picker, sender) = FilePicker::for_test();
        let state = DeliverablesState {
            picker,
            picker_for: Some("inc-1".into()),
            ..DeliverablesState::default()
        };
        let (backend, mut commands) = Backend::for_test();
        let mut harness = harness(state, backend);
        harness.step();
        sender.send(None).expect("cancelamento sintético");
        harness.step();

        harness.get_by_label_contains("Nenhuma pasta escolhida");
        harness.get_by_label("Ou informe a pasta:");
        assert!(drain(&mut commands).is_empty());
    }

    #[test]
    fn a_folder_chosen_after_switching_incidents_does_not_generate_the_new_case() {
        let (picker, sender) = FilePicker::for_test();
        let state = DeliverablesState {
            picker,
            picker_for: Some("inc-1".into()),
            ..DeliverablesState::default()
        };
        let (backend, mut commands) = Backend::for_test();
        let mut harness = harness_for_incident(state, backend, "inc-2");
        sender
            .send(Some(PathBuf::from("/synthetic/pasta-de-inc-1")))
            .expect("escolha sintética");
        harness.step();

        harness.get_by_label_contains("o incidente mudou");
        assert!(drain(&mut commands).is_empty());
    }

    #[test]
    fn a_manual_generation_invalidates_a_late_folder_selection() {
        let (picker, sender) = FilePicker::for_test();
        let mut state = DeliverablesState {
            picker,
            picker_for: Some("inc-1".into()),
            ..DeliverablesState::default()
        };
        let (backend, mut commands) = Backend::for_test();
        state.request_generation("inc-1", PathBuf::from("/synthetic/digitada"), &backend);
        assert_eq!(drain(&mut commands).len(), 1);
        state.on_generated("inc-1", Err("geração manual terminou".into()));

        let mut harness = harness(state, backend);
        sender
            .send(Some(PathBuf::from("/synthetic/selecionada-depois")))
            .expect("resposta tardia sintética");
        harness.step();
        assert!(
            drain(&mut commands).is_empty(),
            "o seletor antigo iniciou uma segunda geração"
        );
    }
}
