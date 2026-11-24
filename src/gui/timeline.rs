//! Timeline paginada (ADR-0021).
//!
//! Os eventos vivem no SQLite, ordenados por UTC. A GUI só guarda algumas
//! páginas de [`PAGE_SIZE`] linhas em volta do que está visível, então a
//! memória não cresce com o tamanho do log. A tabela é virtualizada: só as
//! linhas visíveis são desenhadas e só as páginas que faltam são pedidas.

use super::backend::{Backend, Command};
use super::format_timestamp;
use crate::service::{JobStatusResponse, TimelineEvent, TimelinePage};
use egui_extras::{Column, TableBuilder};
use std::collections::{HashMap, HashSet};

/// Linhas por página buscada no banco.
pub const PAGE_SIZE: usize = 200;
/// Páginas mantidas em memória (~9.600 eventos). As mais distantes da que
/// acabou de chegar são descartadas.
const MAX_CACHED_PAGES: usize = 48;

/// Cache de páginas da timeline. Sem dependência de egui: é testável.
#[derive(Default)]
pub struct TimelinePager {
    pages: HashMap<usize, Vec<TimelineEvent>>,
    pending: HashSet<usize>,
}

impl TimelinePager {
    /// A linha `index`, se a página dela já chegou.
    pub fn row(&self, index: usize) -> Option<&TimelineEvent> {
        self.pages
            .get(&(index / PAGE_SIZE))
            .and_then(|page| page.get(index % PAGE_SIZE))
    }

    /// Se a página da linha `index` não está em memória nem já foi pedida,
    /// marca como pedida e devolve o offset a buscar. Uma página é pedida uma
    /// única vez, não a cada quadro.
    pub fn request(&mut self, index: usize) -> Option<usize> {
        let page = index / PAGE_SIZE;
        if self.pages.contains_key(&page) || !self.pending.insert(page) {
            return None;
        }
        Some(page * PAGE_SIZE)
    }

    /// Guarda a página que começa em `offset` e descarta as mais distantes se
    /// passar do limite de cache.
    pub fn insert(&mut self, offset: usize, events: Vec<TimelineEvent>) {
        let page = offset / PAGE_SIZE;
        self.pending.remove(&page);
        self.pages.insert(page, events);
        while self.pages.len() > MAX_CACHED_PAGES {
            let farthest = self
                .pages
                .keys()
                .copied()
                .max_by_key(|candidate| candidate.abs_diff(page));
            match farthest {
                Some(victim) if victim != page => {
                    self.pages.remove(&victim);
                }
                _ => break,
            }
        }
    }

    pub fn clear(&mut self) {
        self.pages.clear();
        self.pending.clear();
    }

    #[cfg(test)]
    fn cached_pages(&self) -> usize {
        self.pages.len()
    }
}

/// Nível do evento do Windows (0–5) em texto; outros valores passam como estão.
fn severity_label(level: &str) -> String {
    match level.trim() {
        "0" => "Sempre".to_string(),
        "1" => "Crítico".to_string(),
        "2" => "Erro".to_string(),
        "3" => "Aviso".to_string(),
        "4" => "Informação".to_string(),
        "5" => "Detalhado".to_string(),
        other => other.to_string(),
    }
}

enum ViewState {
    Loading,
    NoAnalysis,
    Ready,
    Failed(String),
}

/// Ajuda do filtro (dica da caixa de texto).
const FILTER_HELP: &str = "Texto livre (título, descrição, provedor, canal) e termos campo:valor: ref, event, user, host, ip, src, dst, proc, action, outcome, provider, channel, from, to. Ex.: ref:record:5,record:9 ou event:4625 user:admin from:2026-03-01. Vírgula = OU; aspas agrupam.";

/// Estado da timeline de uma evidência.
pub struct TimelineView {
    state: ViewState,
    job: Option<JobStatusResponse>,
    /// Job que esta visão espera (quando a análise acabou de terminar):
    /// respostas de outro job, que chegam atrasadas, são descartadas.
    expected_job: Option<String>,
    total: usize,
    applied_filter: String,
    filter_input: String,
    /// Erro de sintaxe do filtro digitado (ex.: data inválida). É mostrado ao
    /// lado da caixa; nada é enviado ao backend e a timeline continua visível.
    filter_error: Option<String>,
    filtering: bool,
    pager: TimelinePager,
    selected_event: Option<usize>,
}

impl TimelineView {
    /// Visão que vai carregar o último job EVTX concluído da evidência.
    pub fn loading() -> Self {
        Self {
            state: ViewState::Loading,
            job: None,
            expected_job: None,
            total: 0,
            applied_filter: String::new(),
            filter_input: String::new(),
            filter_error: None,
            filtering: false,
            pager: TimelinePager::default(),
            selected_event: None,
        }
    }

    /// Visão de um job recém-concluído.
    pub fn for_job(job_id: &str) -> Self {
        let mut view = Self::loading();
        view.expected_job = Some(job_id.to_string());
        view
    }

    /// Nova consulta sobre o último job EVTX concluído, sem páginas do filtro anterior.
    pub fn for_filter(filter: &str) -> Self {
        let mut view = Self::loading();
        view.applied_filter = filter.to_string();
        view.filter_input = filter.to_string();
        view.filtering = true;
        view
    }

    /// Aplica uma página recebida do backend. Respostas de um filtro ou de um
    /// job que já não é o atual são descartadas.
    pub fn on_page(
        &mut self,
        filter: &str,
        offset: u64,
        result: Result<Option<TimelinePage>, String>,
    ) {
        if filter != self.applied_filter {
            return;
        }
        match result {
            Err(message) => self.state = ViewState::Failed(message),
            Ok(None) => {
                self.selected_event = None;
                self.state = ViewState::NoAnalysis;
            }
            Ok(Some(page)) => {
                let job_id = &page.job.job_id;
                if self
                    .expected_job
                    .as_ref()
                    .is_some_and(|expected| expected != job_id)
                    || self
                        .job
                        .as_ref()
                        .is_some_and(|current| &current.job_id != job_id)
                {
                    return;
                }
                if self.job.is_none() {
                    self.expected_job = Some(job_id.clone());
                }
                if let Some(total) = page.total {
                    self.total = total as usize;
                    self.filtering = false;
                }
                self.pager.insert(offset as usize, page.events);
                self.job = Some(page.job);
                self.state = ViewState::Ready;
            }
        }
    }

    fn request(&self, evidence_id: &str, offset: usize, want_total: bool) -> Command {
        Command::TimelinePage {
            evidence_id: evidence_id.to_string(),
            job_id: self.job.as_ref().map(|job| job.job_id.clone()),
            filter: self.applied_filter.clone(),
            offset: offset as u64,
            limit: PAGE_SIZE as u64,
            want_total,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, evidence_id: &str, backend: &Backend) {
        match &self.state {
            ViewState::Loading => {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Carregando a timeline…");
                });
            }
            ViewState::NoAnalysis => {
                ui.label("Execute \"Analisar EVTX\" para gerar a timeline.");
            }
            ViewState::Failed(message) => super::error_banner(ui, message),
            ViewState::Ready => self.show_ready(ui, evidence_id, backend),
        }
    }

    fn show_ready(&mut self, ui: &mut egui::Ui, evidence_id: &str, backend: &Backend) {
        if let Some(job) = &self.job {
            ui.horizontal_wrapped(|ui| {
                ui.strong(format!("{} na timeline", event_count_label(self.total)));
                let short_id: String = job.job_id.chars().take(8).collect();
                ui.weak(format!("Análise {short_id}…"))
                    .on_hover_text(format!("Job completo: {}", job.job_id));
                if self.filtering {
                    ui.spinner();
                }
            });
            for warning in &job.warnings {
                ui.colored_label(
                    egui::Color32::from_rgb(200, 140, 0),
                    format!(
                        "Aviso {} (×{}): {}",
                        warning.code, warning.count, warning.sample
                    ),
                );
            }
        }
        ui.horizontal(|ui| {
            let edit = ui
                .text_edit_singleline(&mut self.filter_input)
                .on_hover_text(FILTER_HELP);
            let submitted =
                edit.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter));
            if ui.button("Filtrar").clicked() || submitted {
                let wanted = self.filter_input.trim().to_string();
                // A sintaxe é conferida aqui, antes de enviar: um filtro
                // inválido não pode derrubar a timeline já carregada.
                match crate::service::timeline_filter::parse(&wanted) {
                    Err(message) => self.filter_error = Some(message),
                    Ok(_) => {
                        self.filter_error = None;
                        if wanted != self.applied_filter {
                            self.applied_filter = wanted;
                            self.pager.clear();
                            self.selected_event = None;
                            self.total = 0;
                            self.filtering = true;
                            backend.send(self.request(evidence_id, 0, true));
                        }
                    }
                }
            }
        });
        if let Some(message) = &self.filter_error {
            super::error_banner(ui, message);
        }

        let selected = self
            .selected_event
            .and_then(|index| self.pager.row(index))
            .cloned();
        let details_beside_table = selected.is_some() && ui.available_width() >= 1_000.0;
        if details_beside_table {
            if let Some(event) = &selected {
                egui::Panel::right("timeline-event-details")
                    .default_size(360.0)
                    .min_size(300.0)
                    .max_size(460.0)
                    .show(ui, |ui| self.event_details(ui, event));
            }
        }
        self.show_table(ui, evidence_id, backend);
        if !details_beside_table {
            if let Some(event) = &selected {
                self.event_details(ui, event);
            }
        }
        if self.selected_event.is_none() {
            ui.weak("Selecione um evento para ver seus campos normalizados.");
        }
    }

    fn show_table(&mut self, ui: &mut egui::Ui, evidence_id: &str, backend: &Backend) {
        let total = self.total;
        let mut requests: Vec<usize> = Vec::new();
        let mut clicked = None;
        TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .sense(egui::Sense::click())
            .max_scroll_height(360.0)
            .column(Column::auto().at_least(190.0))
            .column(Column::auto().at_least(70.0))
            .column(Column::auto().at_least(220.0))
            .column(Column::remainder().at_least(260.0))
            .header(22.0, |mut header| {
                header.col(|ui| {
                    ui.strong("Timestamp");
                });
                header.col(|ui| {
                    ui.strong("Severidade");
                });
                header.col(|ui| {
                    ui.strong("Título");
                });
                header.col(|ui| {
                    ui.strong("Descrição");
                });
            })
            .body(|body| {
                body.rows(20.0, total, |mut row| {
                    let index = row.index();
                    match self.pager.row(index) {
                        Some(event) => {
                            row.set_selected(self.selected_event == Some(index));
                            // Uma linha por evento (`truncate`): linhas de altura
                            // uniforme mantêm a rolagem virtualizada correta; o
                            // texto completo aparece ao passar o mouse.
                            row.col(|ui| {
                                let timestamp = format_timestamp(&event.timestamp);
                                ui.add(egui::Label::new(&timestamp).truncate())
                                    .on_hover_text(&timestamp);
                            });
                            row.col(|ui| {
                                ui.add(
                                    egui::Label::new(severity_label(&event.severity)).truncate(),
                                );
                            });
                            row.col(|ui| {
                                ui.add(egui::Label::new(&event.title).truncate())
                                    .on_hover_text(&event.title);
                            });
                            row.col(|ui| {
                                ui.add(egui::Label::new(&event.description).truncate())
                                    .on_hover_text(&event.description);
                            });
                            let response = row.response();
                            response.widget_info(|| {
                                egui::WidgetInfo::labeled(
                                    egui::Role::Button,
                                    response.enabled(),
                                    format!("Inspecionar evento: {}", event.title),
                                )
                            });
                            if response.clicked() {
                                clicked = Some(index);
                            }
                        }
                        None => {
                            if let Some(offset) = self.pager.request(index) {
                                requests.push(offset);
                            }
                            row.col(|ui| {
                                ui.label("carregando…");
                            });
                            row.col(|_| {});
                            row.col(|_| {});
                            row.col(|_| {});
                        }
                    }
                });
            });
        for offset in requests {
            backend.send(self.request(evidence_id, offset, false));
        }
        if let Some(index) = clicked {
            self.selected_event = Some(index);
        }
    }

    fn event_details(&mut self, ui: &mut egui::Ui, event: &TimelineEvent) {
        ui.horizontal(|ui| {
            ui.heading("Detalhes do evento");
            if ui.button("Fechar detalhes").clicked() {
                self.selected_event = None;
            }
        });
        ui.label(egui::RichText::new(&event.title).strong());
        ui.label(format_timestamp(&event.timestamp));
        if event.description.trim().is_empty() {
            ui.weak("Sem descrição normalizada.");
        } else {
            ui.label(&event.description);
        }
        ui.separator();
        detail_row(ui, "Categoria", &event.category);
        detail_row(ui, "Severidade", &severity_label(&event.severity));
        for (label, value) in [
            ("Event ID", event.event_id.as_deref()),
            ("Registro", event.record_id.as_deref()),
            ("Referência", event.raw_reference.as_deref()),
            ("Provedor", event.provider.as_deref()),
            ("Canal", event.channel.as_deref()),
            ("Host", event.host.as_deref()),
            ("Conta", event.user.as_deref()),
            ("IP de origem", event.src_ip.as_deref()),
            ("IP de destino", event.dst_ip.as_deref()),
            ("Processo", event.process.as_deref()),
            ("Ação", event.action.as_deref()),
            ("Resultado", event.outcome.as_deref()),
            ("Tipo de fonte", event.source_type.as_deref()),
        ] {
            if let Some(value) = value.filter(|value| !value.trim().is_empty()) {
                detail_row(ui, label, value);
            }
        }
    }
}

fn event_count_label(total: usize) -> String {
    if total == 1 {
        "1 evento".into()
    } else {
        format!("{total} eventos")
    }
}

fn detail_row(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.horizontal_wrapped(|ui| {
        ui.strong(format!("{label}:"));
        ui.label(value);
    });
}

#[cfg(test)]
mod tests {
    use super::{MAX_CACHED_PAGES, PAGE_SIZE, TimelinePager, severity_label};
    use crate::service::TimelineEvent;

    fn page(prefix: &str, rows: usize) -> Vec<TimelineEvent> {
        (0..rows)
            .map(|index| TimelineEvent::for_test(&format!("{prefix}-{index}"), None))
            .collect()
    }

    #[test]
    fn windows_levels_are_shown_as_words() {
        assert_eq!(severity_label("0"), "Sempre");
        assert_eq!(severity_label("2"), "Erro");
        assert_eq!(severity_label("4"), "Informação");
        assert_eq!(severity_label("INFO"), "INFO");
    }

    #[test]
    fn a_page_is_requested_once_until_it_arrives() {
        let mut pager = TimelinePager::default();
        assert_eq!(pager.request(0), Some(0));
        assert_eq!(pager.request(5), None, "mesma página, já pedida");
        assert_eq!(pager.request(PAGE_SIZE + 1), Some(PAGE_SIZE));
        assert!(pager.row(0).is_none());

        pager.insert(0, page("a", PAGE_SIZE));

        assert_eq!(pager.row(3).map(|e| e.title.as_str()), Some("a-3"));
        assert_eq!(pager.request(3), None, "página já em memória");
    }

    #[test]
    fn a_short_last_page_has_no_rows_past_its_end() {
        let mut pager = TimelinePager::default();
        pager.insert(PAGE_SIZE, page("b", 7));
        assert!(pager.row(PAGE_SIZE + 6).is_some());
        assert!(pager.row(PAGE_SIZE + 7).is_none());
    }

    #[test]
    fn far_pages_are_evicted_and_the_newest_is_kept() {
        let mut pager = TimelinePager::default();
        for index in 0..(MAX_CACHED_PAGES + 10) {
            pager.insert(index * PAGE_SIZE, page("p", 1));
        }
        assert_eq!(pager.cached_pages(), MAX_CACHED_PAGES);
        let newest = (MAX_CACHED_PAGES + 9) * PAGE_SIZE;
        assert!(pager.row(newest).is_some(), "a página recém-chegada fica");
        assert!(pager.row(0).is_none(), "a mais distante saiu");
        // Uma página descartada pode ser pedida de novo.
        assert_eq!(pager.request(0), Some(0));
    }

    #[test]
    fn clear_forgets_pages_and_pending_requests() {
        let mut pager = TimelinePager::default();
        pager.request(0);
        pager.insert(PAGE_SIZE, page("c", 1));
        pager.clear();
        assert!(pager.row(PAGE_SIZE).is_none());
        assert_eq!(pager.request(0), Some(0));
    }
}

/// Testes da tela com `egui_kittest` (ADR-0022): renderizam a tela sem janela e
/// operam pela árvore de acessibilidade, como um leitor de tela faria. Cobrem
/// o que testes de lógica não alcançam: digitar no campo, clicar no botão e o
/// que de fato é desenhado.
#[cfg(test)]
mod ui_tests {
    use super::{PAGE_SIZE, TimelineView, event_count_label};
    use crate::gui::backend::{Backend, Command};
    use crate::gui::test_harness::drain;
    use crate::service::{JobWarning, TimelineEvent, TimelinePage};
    use egui_kittest::{Harness, kittest::Queryable};

    type State = (TimelineView, Backend);

    fn events(count: usize) -> Vec<TimelineEvent> {
        (0..count)
            .map(|index| TimelineEvent::for_test(&format!("evento-{index}"), None))
            .collect()
    }

    fn harness(view: TimelineView, backend: Backend) -> Harness<'static, State> {
        crate::gui::test_harness::builder().build_ui_state(
            |ui, (view, backend): &mut State| view.show(ui, "ev_1", backend),
            (view, backend),
        )
    }

    #[test]
    fn timeline_formats_event_timestamp_without_raw_rfc3339() {
        let (backend, _commands) = Backend::for_test();
        let mut view = TimelineView::for_job("job-1");
        view.on_page(
            "",
            0,
            Ok(Some(TimelinePage::for_test(
                "job-1",
                Some(1),
                0,
                events(1),
                vec![],
            ))),
        );
        let mut harness = harness(view, backend);
        harness.run();

        harness.get_by_label("2026-01-01 00:00:00 UTC");
        assert!(
            harness
                .query_by_label("2026-01-01T00:00:00+00:00")
                .is_none()
        );
    }

    #[test]
    fn timeline_header_uses_plural_and_abbreviates_the_job_id() {
        assert_eq!(event_count_label(1), "1 evento");
        assert_eq!(event_count_label(2), "2 eventos");
        let (backend, _commands) = Backend::for_test();
        let full_job_id = "c489c12e-1111-4222-8333-444444444444";
        let mut view = TimelineView::for_job(full_job_id);
        view.on_page(
            "",
            0,
            Ok(Some(TimelinePage::for_test(
                full_job_id,
                Some(1),
                0,
                events(1),
                vec![],
            ))),
        );
        let mut harness = harness(view, backend);
        harness.run();

        harness.get_by_label("1 evento na timeline");
        harness.get_by_label_contains("Análise c489c12e…");
        assert!(
            harness.query_by_label_contains(full_job_id).is_none(),
            "o UUID completo fica apenas no tooltip sob demanda"
        );
    }

    #[test]
    fn clicking_an_event_inspects_normalized_fields_and_closes_cleanly() {
        let (backend, _commands) = Backend::for_test();
        let mut event = TimelineEvent::for_test(
            "Logon bem-sucedido",
            Some("Microsoft-Windows-Security-Auditing"),
        );
        event.description = "Uma conta concluiu autenticação neste host.".into();
        event.event_id = Some("4624".into());
        event.record_id = Some("42".into());
        event.raw_reference = Some("record:42".into());
        event.channel = Some("Security".into());
        event.host = Some("HOST-01".into());
        event.user = Some("ana".into());
        event.src_ip = Some("192.0.2.10".into());
        event.process = Some("C:\\Windows\\System32\\svchost.exe".into());
        event.action = Some("logon".into());
        event.outcome = Some("success".into());
        event.source_type = Some("EVTX".into());
        let mut view = TimelineView::for_job("job-1");
        view.on_page(
            "",
            0,
            Ok(Some(TimelinePage::for_test(
                "job-1",
                Some(1),
                0,
                vec![event],
                vec![],
            ))),
        );
        let mut harness = crate::gui::test_harness::builder()
            .with_size(egui::vec2(1_280.0, 760.0))
            .build_ui_state(
                |ui, (view, backend): &mut State| view.show(ui, "ev_1", backend),
                (view, backend),
            );
        harness.run();
        harness.get_by_label("Selecione um evento para ver seus campos normalizados.");
        harness
            .get_by_label("Inspecionar evento: Logon bem-sucedido")
            .click();
        harness.run_steps(2);

        let details = harness.get_by_label("Detalhes do evento").rect();
        assert!(
            details.min.x > 800.0,
            "em uma tela larga, detalhes usam o espaço à direita: {details:?}"
        );
        for value in [
            "Uma conta concluiu autenticação neste host.",
            "4624",
            "42",
            "Microsoft-Windows-Security-Auditing",
            "Security",
            "HOST-01",
            "ana",
            "192.0.2.10",
            "logon",
            "success",
            "EVTX",
        ] {
            assert!(
                harness.get_all_by_label(value).count() >= 1,
                "campo normalizado ausente da árvore acessível: {value}"
            );
        }
        assert_eq!(
            harness
                .get_all_by_label("Uma conta concluiu autenticação neste host.")
                .count(),
            2,
            "a descrição permanece na linha e aparece integralmente nos detalhes"
        );
        harness.get_by_label("Categoria:");
        harness.get_by_label("Event ID:");
        assert!(harness.query_by_label("IP de destino:").is_none());

        harness.get_by_label("Fechar detalhes").click();
        harness.run_steps(2);
        assert!(harness.query_by_label("Detalhes do evento").is_none());
        harness.get_by_label("Selecione um evento para ver seus campos normalizados.");
    }

    #[test]
    fn empty_event_details_do_not_invent_optional_fields() {
        let (backend, _commands) = Backend::for_test();
        let mut view = TimelineView::for_job("job-1");
        view.on_page(
            "",
            0,
            Ok(Some(TimelinePage::for_test(
                "job-1",
                Some(1),
                0,
                events(1),
                vec![],
            ))),
        );
        let mut harness = harness(view, backend);
        harness.run();
        harness.get_by_label("Inspecionar evento: evento-0").click();
        harness.run_steps(2);

        harness.get_by_label("Sem descrição normalizada.");
        for absent in ["Provedor:", "Conta:", "Registro:", "Processo:"] {
            assert!(harness.query_by_label(absent).is_none());
        }
        let details = harness.get_by_label("Detalhes do evento").rect();
        let row = harness.get_by_label("Inspecionar evento: evento-0").rect();
        assert!(
            details.min.y > row.min.y,
            "em tela estreita, os detalhes ficam abaixo da tabela"
        );
    }

    #[test]
    fn changing_the_filter_clears_the_selected_event() {
        let (backend, mut commands) = Backend::for_test();
        let mut view = TimelineView::for_job("job-1");
        view.on_page(
            "",
            0,
            Ok(Some(TimelinePage::for_test(
                "job-1",
                Some(1),
                0,
                events(1),
                vec![],
            ))),
        );
        let mut harness = harness(view, backend);
        harness.run();
        harness.get_by_label("Inspecionar evento: evento-0").click();
        harness.run_steps(2);
        harness.get_by_label("Detalhes do evento");
        drain(&mut commands);

        harness
            .get_by_role(egui::accesskit::Role::TextInput)
            .click();
        harness.run();
        harness
            .get_by_role(egui::accesskit::Role::TextInput)
            .type_text("4624");
        harness.run();
        harness.get_by_label("Filtrar").click();
        harness.run_steps(2);

        assert!(harness.query_by_label("Detalhes do evento").is_none());
        assert_eq!(drain(&mut commands).len(), 1);
    }

    #[test]
    fn typing_a_filter_and_pressing_the_button_requests_page_zero_with_that_filter() {
        let (backend, mut commands) = Backend::for_test();
        let mut view = TimelineView::for_job("job-1");
        view.on_page(
            "",
            0,
            Ok(Some(TimelinePage::for_test(
                "job-1",
                Some(3),
                0,
                events(3),
                vec![],
            ))),
        );
        let mut harness = harness(view, backend);
        harness.run();
        assert!(
            drain(&mut commands).is_empty(),
            "nada é pedido com as linhas em memória"
        );

        // Foca o campo antes de digitar, como o analista faz ao clicar nele.
        harness
            .get_by_role(egui::accesskit::Role::TextInput)
            .click();
        harness.run();
        harness
            .get_by_role(egui::accesskit::Role::TextInput)
            .type_text("  4624 ");
        // O texto digitado só chega ao estado da tela no quadro seguinte.
        harness.run();
        assert_eq!(
            harness
                .get_by_role(egui::accesskit::Role::TextInput)
                .value()
                .as_deref(),
            Some("  4624 "),
            "o texto digitado precisa aparecer no campo"
        );
        harness.get_by_label("Filtrar").click();
        // Aplicar o filtro mostra o spinner até a resposta chegar: `step`.
        harness.step();
        harness.step();

        let sent = drain(&mut commands);
        assert_eq!(sent.len(), 1, "um único pedido: {sent:?}");
        match &sent[0] {
            Command::TimelinePage {
                filter,
                offset,
                want_total,
                job_id,
                ..
            } => {
                assert_eq!(filter, "4624", "o filtro é enviado sem espaços nas pontas");
                assert_eq!(*offset, 0);
                assert!(
                    *want_total,
                    "o total precisa ser recalculado para o novo filtro"
                );
                assert_eq!(job_id.as_deref(), Some("job-1"));
            }
            other => panic!("comando inesperado: {other:?}"),
        }
    }

    #[test]
    fn pressing_filter_with_unchanged_text_does_not_hit_the_database_again() {
        let (backend, mut commands) = Backend::for_test();
        let mut view = TimelineView::for_job("job-1");
        view.on_page(
            "",
            0,
            Ok(Some(TimelinePage::for_test(
                "job-1",
                Some(3),
                0,
                events(3),
                vec![],
            ))),
        );
        let mut harness = harness(view, backend);
        harness.run();
        harness.get_by_label("Filtrar").click();
        harness.run();
        assert!(drain(&mut commands).is_empty());
    }

    #[test]
    fn parser_warnings_are_shown_to_the_analyst() {
        let (backend, _commands) = Backend::for_test();
        let mut view = TimelineView::for_job("job-1");
        let warning = JobWarning {
            code: "EVTX_TRUNCATED".into(),
            count: 1,
            sample: "arquivo truncado".into(),
        };
        view.on_page(
            "",
            0,
            Ok(Some(TimelinePage::for_test(
                "job-1",
                Some(3),
                0,
                events(3),
                vec![warning],
            ))),
        );
        let mut harness = harness(view, backend);
        harness.run();
        harness.get_by_label_contains("EVTX_TRUNCATED");
        harness.get_by_label_contains("3 eventos");
    }

    #[test]
    fn a_missing_page_is_requested_exactly_once_across_frames() {
        let (backend, mut commands) = Backend::for_test();
        let mut view = TimelineView::for_job("job-1");
        // A página 1 chegou; a 0 (a visível) não: a tabela precisa pedi-la.
        view.on_page(
            "",
            PAGE_SIZE as u64,
            Ok(Some(TimelinePage::for_test(
                "job-1",
                Some(1_000),
                PAGE_SIZE as u64,
                events(PAGE_SIZE),
                vec![],
            ))),
        );
        let mut harness = harness(view, backend);
        harness.run();
        harness.run();
        harness.run();

        let requested: Vec<u64> = drain(&mut commands)
            .into_iter()
            .filter_map(|command| match command {
                Command::TimelinePage {
                    offset, want_total, ..
                } => {
                    assert!(!want_total, "páginas de rolagem não pagam o COUNT");
                    Some(offset)
                }
                _ => None,
            })
            .collect();
        assert_eq!(requested, vec![0], "a página 0, uma única vez");
        assert!(
            harness.get_all_by_label("carregando…").count() >= 1,
            "as linhas sem página mostram o marcador"
        );
    }

    #[test]
    fn wheel_reaches_the_last_virtual_row_and_requests_its_page() {
        let (backend, mut commands) = Backend::for_test();
        let last_offset = PAGE_SIZE * 2;
        let total = last_offset + 50;
        let mut view = TimelineView::for_job("job-1");
        view.on_page(
            "",
            0,
            Ok(Some(TimelinePage::for_test(
                "job-1",
                Some(total as u64),
                0,
                events(PAGE_SIZE),
                vec![],
            ))),
        );
        let viewport = egui::vec2(900.0, 500.0);
        let mut harness = crate::gui::test_harness::builder()
            .with_size(viewport)
            .build_ui_state(
                |ui, (view, backend): &mut State| view.show(ui, "ev_1", backend),
                (view, backend),
            );
        harness.run();
        assert!(harness.query_by_label("evento-449").is_none());

        harness.get_by_label("evento-0").hover();
        harness.input_mut().events.push(egui::Event::MouseWheel {
            unit: egui::MouseWheelUnit::Point,
            delta: egui::vec2(0.0, -10_000.0),
            phase: egui::TouchPhase::Move,
            source: egui::MouseWheelSource::Unknown,
            modifiers: egui::Modifiers::default(),
        });
        harness.run_steps(8);

        let requested: Vec<u64> = drain(&mut commands)
            .into_iter()
            .filter_map(|command| match command {
                Command::TimelinePage { offset, .. } => Some(offset),
                _ => None,
            })
            .collect();
        assert!(
            requested.contains(&(last_offset as u64)),
            "a rolagem até o fim deve pedir a última página: {requested:?}"
        );

        let last_events = (last_offset..total)
            .map(|index| TimelineEvent::for_test(&format!("evento-{index}"), None))
            .collect();
        harness.state_mut().0.on_page(
            "",
            last_offset as u64,
            Ok(Some(TimelinePage::for_test(
                "job-1",
                None,
                last_offset as u64,
                last_events,
                vec![],
            ))),
        );
        harness.run_steps(8);

        let last = harness.get_by_label("evento-449").rect();
        assert!(
            last.min.y >= 0.0 && last.max.y <= viewport.y,
            "a última linha não ficou visível após a rolagem: {last:?}"
        );
    }

    #[test]
    fn an_invalid_filter_shows_an_error_beside_the_box_and_keeps_the_timeline() {
        let (backend, mut commands) = Backend::for_test();
        let mut view = TimelineView::for_job("job-1");
        view.on_page(
            "",
            0,
            Ok(Some(TimelinePage::for_test(
                "job-1",
                Some(3),
                0,
                events(3),
                vec![],
            ))),
        );
        let mut harness = harness(view, backend);
        harness.run();
        harness
            .get_by_role(egui::accesskit::Role::TextInput)
            .focus();
        harness.run();
        harness
            .get_by_role(egui::accesskit::Role::TextInput)
            .type_text("from:ontem");
        harness.run();
        harness.get_by_label("Filtrar").click();
        harness.run();

        harness.get_by_label_contains("invalid date");
        assert!(
            drain(&mut commands).is_empty(),
            "nada é enviado com a sintaxe inválida"
        );
        // A caixa de filtro e as linhas continuam na tela: o analista corrige e tenta de novo.
        harness.get_by_role(egui::accesskit::Role::TextInput);
        harness.get_by_label("evento-0");
    }

    #[test]
    fn candidate_filter_view_discards_previous_filter_responses() {
        const REFS: &str = "ref:record:5,record:9";
        let (backend, _commands) = Backend::for_test();
        let mut view = TimelineView::for_filter(REFS);
        view.on_page(
            "",
            0,
            Ok(Some(TimelinePage::for_test(
                "job-velho",
                Some(9),
                0,
                events(9),
                vec![],
            ))),
        );
        let mut harness = harness(view, backend);
        harness.step();
        harness.get_by_label("Carregando a timeline…");

        harness.state_mut().0.on_page(
            REFS,
            0,
            Ok(Some(TimelinePage::for_test(
                "job-novo",
                Some(2),
                0,
                events(2),
                vec![],
            ))),
        );
        harness.step();
        harness.get_by_label("evento-0");
        assert_eq!(
            harness
                .get_by_role(egui::accesskit::Role::TextInput)
                .value()
                .as_deref(),
            Some(REFS)
        );
    }

    #[test]
    fn the_empty_state_tells_the_analyst_what_to_do() {
        let (backend, _commands) = Backend::for_test();
        let mut view = TimelineView::loading();
        view.on_page("", 0, Ok(None));
        let mut harness = harness(view, backend);
        harness.run();
        harness.get_by_label_contains("Analisar EVTX");
    }

    #[test]
    fn responses_for_an_old_filter_or_another_job_are_discarded() {
        let (backend, _commands) = Backend::for_test();
        let mut view = TimelineView::for_job("job-novo");
        // Resposta do job anterior (chegou atrasada): descartada.
        view.on_page(
            "",
            0,
            Ok(Some(TimelinePage::for_test(
                "job-velho",
                Some(9),
                0,
                events(9),
                vec![],
            ))),
        );
        // Resposta de outro filtro: descartada.
        view.on_page(
            "x",
            0,
            Ok(Some(TimelinePage::for_test(
                "job-novo",
                Some(5),
                0,
                events(5),
                vec![],
            ))),
        );
        let mut harness = harness(view, backend);
        // O spinner repinta sem parar: `step` em vez de `run`.
        harness.step();
        harness.get_by_label("Carregando a timeline…");
    }
}
