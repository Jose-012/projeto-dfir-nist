//! Painel "Evidência": lista, importação via diálogo nativo de arquivo,
//! destruição, disparo de análise (EVTX_TIMELINE / YARA_SCAN) e exibição dos
//! resultados (timeline de eventos EVTX, matches YARA). Porta funcional do
//! antigo `frontend/src/app.ts` + `timeline.ts`, sem o transporte HTTP.

use super::backend::{Backend, Command};
use super::detection::DetectionState;
use super::file_dialog::{FilePicker, Picked, Request};
use super::lineage::LineageState;
use super::timeline::{PAGE_SIZE, TimelineView};
use super::{Loadable, str_field};
use crate::persistence::audit::CustodyEntry;
use crate::service::lineage::{LineageScope, LineageView};
use crate::service::provenance::EvidenceProvenance;
use crate::service::{
    AnalyticalResultResponse, AnalyzeResponse, EvidenceResponse, IncidentResponse, TimelinePage,
};
use egui_extras::{Column, TableBuilder};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

const TIMELINE_HEADING: &str = "Timeline (EVTX)";

/// Proveniência que o operador declara **antes** de escolher o arquivo
/// (F1-31). O que fica em branco vai como não informado.
#[derive(Default)]
struct ProvenanceForm {
    source: String,
    provided_by: String,
    collection_method: String,
    collection_tool: String,
    collection_tool_version: String,
    collected_at: String,
    authorization_reference: String,
    contains_personal_data: bool,
}

impl ProvenanceForm {
    fn to_provenance(&self) -> EvidenceProvenance {
        let text = |value: &str| {
            let trimmed = value.trim();
            (!trimmed.is_empty()).then(|| trimmed.to_string())
        };
        EvidenceProvenance {
            source: text(&self.source),
            provided_by: text(&self.provided_by),
            collection_method: text(&self.collection_method),
            collection_tool: text(&self.collection_tool),
            collection_tool_version: text(&self.collection_tool_version),
            collected_at: text(&self.collected_at),
            authorization_reference: text(&self.authorization_reference),
            contains_personal_data: self.contains_personal_data,
        }
    }
}

#[derive(Default)]
pub struct EvidenceState {
    list: Loadable<Vec<EvidenceResponse>>,
    selected: Option<String>,
    pending_destroy: Option<String>,
    analyzing: HashSet<String>,
    cancelling: HashSet<String>,
    destroying: HashSet<String>,
    importing: bool,
    import_error: Option<String>,
    /// Aviso neutro (seletor cancelado/indisponível, arquivos múltiplos
    /// soltos) — não é erro, e não deve ser pintado como tal (GUI-01).
    import_hint: Option<String>,
    /// Caminho digitado pelo operador, alternativa ao seletor nativo.
    manual_path: String,
    /// Seletor nativo rodando fora da thread da UI (GUI-01).
    picker: FilePicker,
    provenance: ProvenanceForm,
    last_analysis_error: HashMap<String, String>,
    analysis_notice: HashMap<String, String>,
    timeline: HashMap<String, TimelineView>,
    /// Expansão transitória pedida pela navegação de um candidato.
    timeline_expand: Option<String>,
    analytical: HashMap<String, Loadable<Vec<AnalyticalResultResponse>>>,
    lineage: LineageState,
    /// Candidatos de detecção por regras (F1-28).
    pub detection: DetectionState,
    /// Cadeia de custódia por evidência (F1-24), carregada sob demanda.
    custody: HashMap<String, Loadable<Vec<CustodyEntry>>>,
    /// Vínculo pendente de confirmação: (evidence_id, incident_id) (F1-19).
    pending_link: Option<(String, String)>,
    /// Mensagem curta do último vínculo (sucesso ou erro).
    link_notice: Option<String>,
}

fn yara_number_field(payload: &serde_json::Value, key: &str) -> String {
    payload
        .get(key)
        .and_then(serde_json::Value::as_u64)
        .map(|value| value.to_string())
        .unwrap_or_else(|| "não informado".into())
}

fn yara_result_ui(ui: &mut egui::Ui, item: &AnalyticalResultResponse) {
    let payload = &item.payload;
    ui.group(|ui| {
        ui.set_width(ui.available_width());
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);

        ui.horizontal_wrapped(|ui| {
            ui.strong(&item.result_type);
            if let Some(rule) = &item.rule_or_model_id {
                ui.label(format!("regra: {rule}"));
            }
            if let Some(confidence) = &item.confidence {
                ui.label(format!("confiança: {confidence}"));
            }
        });

        let severity = str_field(payload, "severity");
        ui.label(format!(
            "Severidade: {}",
            if severity.is_empty() {
                "não informada"
            } else {
                severity
            }
        ));
        let description = str_field(payload, "description");
        ui.label(format!(
            "Descrição: {}",
            if description.is_empty() {
                "não informada"
            } else {
                description
            }
        ));
        let reference = str_field(payload, "reference");
        ui.label(format!(
            "Referência: {}",
            if reference.is_empty() {
                "não informada"
            } else {
                reference
            }
        ));
        ui.label(format!("Namespace: {}", str_field(payload, "namespace")));
        ui.label(format!(
            "Ruleset: {} · versão {}",
            str_field(payload, "ruleset_id"),
            str_field(payload, "ruleset_version")
        ));
        ui.label(format!(
            "SHA-256 do ruleset: {}",
            str_field(payload, "ruleset_sha256")
        ));
        ui.label(format!(
            "Licença do ruleset: {}",
            str_field(payload, "ruleset_license")
        ));
        ui.label(format!(
            "Origem do ruleset: {}",
            str_field(payload, "ruleset_source_origin")
        ));
        ui.label(format!(
            "Compatibilidade do engine: {}",
            str_field(payload, "engine_compatibility")
        ));
        ui.label(format!(
            "Referência da evidência: {}",
            str_field(payload, "evidence_id")
        ));

        if let Some(patterns) = payload
            .get("matched_patterns")
            .and_then(serde_json::Value::as_array)
        {
            ui.separator();
            ui.strong(format!("Ocorrências detalhadas ({})", patterns.len()));
            for pattern in patterns {
                ui.label(format!(
                    "Padrão {} · offset {} · comprimento {}",
                    str_field(pattern, "pattern_id"),
                    yara_number_field(pattern, "offset"),
                    yara_number_field(pattern, "length")
                ));
            }
        }
        if let Some(counts) = payload
            .get("pattern_match_counts")
            .and_then(serde_json::Value::as_array)
        {
            ui.label("Contagens por padrão:");
            for count in counts {
                ui.label(format!(
                    "{}: {} total(is), {} exibida(s)",
                    str_field(count, "pattern_id"),
                    yara_number_field(count, "total_matches"),
                    yara_number_field(count, "reported_matches")
                ));
            }
        }
        match payload
            .get("matches_truncated")
            .and_then(serde_json::Value::as_bool)
        {
            Some(true) => {
                ui.label("Ocorrências truncadas: sim (totais podem ser limites inferiores)");
            }
            Some(false) => {
                ui.label("Ocorrências truncadas: não");
            }
            None => {
                ui.label("Ocorrências truncadas: não informado");
            }
        }
    });
}

impl EvidenceState {
    pub fn items(&self) -> &[EvidenceResponse] {
        self.list.ready().map(Vec::as_slice).unwrap_or(&[])
    }

    /// Nome da evidência já selecionada, com o mesmo fallback de id usado
    /// pelos detalhes quando uma listagem nova ainda não contém o item.
    pub(super) fn selected_label(&self) -> Option<&str> {
        let selected = self.selected.as_deref()?;
        Some(
            self.items()
                .iter()
                .find(|item| item.id == selected)
                .map(|item| item.original_filename.as_str())
                .unwrap_or(selected),
        )
    }

    pub fn on_listed(&mut self, result: Result<Vec<EvidenceResponse>, String>) {
        self.list = match result {
            Ok(items) => Loadable::Ready(items),
            Err(message) => Loadable::Failed(message),
        };
    }

    pub fn on_imported(&mut self, result: Result<EvidenceResponse, String>) {
        self.importing = false;
        tracing::info!(success = result.is_ok(), "native UI import finished");
        match result {
            Ok(evidence) => {
                self.import_error = None;
                self.provenance = ProvenanceForm::default();
                if let Loadable::Ready(items) = &mut self.list {
                    items.insert(0, evidence);
                }
            }
            Err(message) => self.import_error = Some(message),
        }
    }

    pub fn on_destroyed(&mut self, evidence_id: &str, result: Result<(), String>) {
        self.destroying.remove(evidence_id);
        match result {
            Ok(()) => {
                if let Loadable::Ready(items) = &mut self.list {
                    items.retain(|item| item.id != evidence_id);
                }
                if self.selected.as_deref() == Some(evidence_id) {
                    self.selected = None;
                    self.lineage.clear();
                }
            }
            Err(message) => {
                self.last_analysis_error
                    .insert(evidence_id.to_string(), message);
            }
        }
    }

    pub fn on_analyze_completed(
        &mut self,
        evidence_id: &str,
        result: Result<AnalyzeResponse, String>,
        backend: &Backend,
    ) {
        self.analyzing.remove(evidence_id);
        self.cancelling.remove(evidence_id);
        match result {
            Ok(response) if response.status == "CANCELLED" => {
                self.last_analysis_error.remove(evidence_id);
                self.analysis_notice.insert(
                    evidence_id.to_string(),
                    "Análise cancelada pelo operador (CANCELLED).".into(),
                );
            }
            Ok(response) => {
                self.last_analysis_error.remove(evidence_id);
                self.analysis_notice.remove(evidence_id);
                // AUD-32: só a análise EVTX produz eventos de timeline. Um
                // scan YARA não pode sobrescrever o painel de timeline da
                // evidência com um resultado sem eventos.
                if should_reload_timeline(&response.analysis_kind) {
                    self.timeline.insert(
                        evidence_id.to_string(),
                        TimelineView::for_job(&response.job_id),
                    );
                    backend.send(Command::TimelinePage {
                        evidence_id: evidence_id.to_string(),
                        job_id: Some(response.job_id),
                        filter: String::new(),
                        offset: 0,
                        limit: PAGE_SIZE as u64,
                        want_total: true,
                    });
                }
                backend.send(Command::EvidenceAnalyticalResults(evidence_id.to_string()));
            }
            Err(message) => {
                self.analysis_notice.remove(evidence_id);
                self.last_analysis_error
                    .insert(evidence_id.to_string(), message);
            }
        }
    }

    pub fn on_analysis_cancelled(
        &mut self,
        evidence_id: &str,
        result: Result<crate::service::JobStatusResponse, String>,
    ) {
        self.cancelling.remove(evidence_id);
        match result {
            Ok(job) => {
                self.last_analysis_error.remove(evidence_id);
                if job.status == "CANCELLED" {
                    self.analyzing.remove(evidence_id);
                }
                self.analysis_notice.insert(
                    evidence_id.to_string(),
                    format!("Cancelamento processado: estado {}.", job.status),
                );
            }
            Err(message) => {
                self.analysis_notice.remove(evidence_id);
                self.last_analysis_error
                    .insert(evidence_id.to_string(), message);
            }
        }
    }

    pub fn on_timeline_page(
        &mut self,
        evidence_id: &str,
        filter: &str,
        offset: u64,
        result: Result<Option<TimelinePage>, String>,
    ) {
        let event_count = result
            .as_ref()
            .ok()
            .and_then(Option::as_ref)
            .map(|page| page.events.len());
        let has_page = result.as_ref().ok().is_some_and(Option::is_some);
        tracing::info!(
            evidence_id,
            offset,
            event_count,
            has_page,
            "native UI timeline page loaded"
        );
        if let Some(view) = self.timeline.get_mut(evidence_id) {
            view.on_page(filter, offset, result);
        }
    }

    /// Abre refs de um candidato na mesma evidência. `job_id: None` resolve o
    /// último job EVTX concluído; o `job_id` da detecção nunca entra na consulta.
    fn open_candidate_timeline(&mut self, evidence_id: &str, filter: String, backend: &Backend) {
        self.timeline_expand = Some(evidence_id.to_string());
        self.timeline
            .insert(evidence_id.to_string(), TimelineView::for_filter(&filter));
        backend.send(Command::TimelinePage {
            evidence_id: evidence_id.to_string(),
            job_id: None,
            filter,
            offset: 0,
            limit: PAGE_SIZE as u64,
            want_total: true,
        });
    }

    pub fn on_analytical_results(
        &mut self,
        evidence_id: &str,
        result: Result<Vec<AnalyticalResultResponse>, String>,
    ) {
        self.analytical.insert(
            evidence_id.to_string(),
            match result {
                Ok(items) => Loadable::Ready(items),
                Err(message) => Loadable::Failed(message),
            },
        );
    }

    pub fn on_lineage_loaded(
        &mut self,
        result_id: &str,
        scope: &LineageScope,
        result: Result<LineageView, String>,
    ) {
        self.lineage.on_loaded(result_id, scope, result);
    }

    pub fn on_linked_to_incident(
        &mut self,
        evidence_id: &str,
        result: Result<EvidenceResponse, String>,
    ) {
        match result {
            Ok(updated) => {
                if let Loadable::Ready(items) = &mut self.list
                    && let Some(slot) = items.iter_mut().find(|item| item.id == evidence_id)
                {
                    *slot = updated;
                }
                self.link_notice = Some("Evidência vinculada ao incidente.".into());
            }
            Err(message) => {
                self.link_notice = Some(format!("Vínculo não feito: {message}"));
            }
        }
        self.pending_link = None;
    }

    pub fn on_custody_loaded(
        &mut self,
        evidence_id: &str,
        result: Result<Vec<CustodyEntry>, String>,
    ) {
        self.custody.insert(
            evidence_id.to_string(),
            match result {
                Ok(entries) => Loadable::Ready(entries),
                Err(message) => Loadable::Failed(message),
            },
        );
    }

    /// Envia a importação com a proveniência declarada no formulário.
    fn begin_import(&mut self, path: PathBuf, backend: &Backend) {
        tracing::info!("native UI import requested");
        self.importing = true;
        self.import_error = None;
        self.import_hint = None;
        backend.send(Command::ImportEvidence {
            path,
            max_upload_bytes: backend.max_upload_bytes(),
            provenance: self.provenance.to_provenance(),
        });
    }

    /// Único ponto de importação da interface: seletor nativo, caminho
    /// digitado ou arrastar-e-soltar chegam todos aqui. Rejeita diretório
    /// antes de qualquer trabalho (um caminho escolhido pelo operador
    /// continua sendo entrada a validar, AGENTS §6).
    fn request_import(&mut self, path: PathBuf, backend: &Backend) {
        if self.importing {
            self.import_hint = Some("Já existe uma importação em andamento; aguarde o fim.".into());
            return;
        }
        if path.as_os_str().is_empty() {
            self.import_hint = Some("Informe o caminho de um arquivo de evidência.".into());
            return;
        }
        if path.is_dir() {
            self.import_hint =
                Some("O caminho informado é uma pasta; selecione um arquivo de evidência.".into());
            return;
        }
        self.manual_path.clear();
        self.begin_import(path, backend);
    }

    fn open_picker(&mut self, context: egui::Context) {
        if let Err(message) = self.picker.open(
            Request::OpenFile {
                title: "Selecionar arquivo de evidência".into(),
            },
            context,
        ) {
            self.import_hint = Some(message);
        }
    }

    /// Arquivos soltos na janela (GUI-01). Só o primeiro é importado; os
    /// demais são mencionados, nunca descartados em silêncio.
    pub fn on_files_dropped(&mut self, paths: &[PathBuf], backend: &Backend) {
        let Some(first) = paths.first().cloned() else {
            return;
        };
        self.request_import(first.clone(), backend);
        // `begin_import` limpa avisos antigos; o aviso de múltiplos arquivos
        // precisa ser gravado depois, senão o operador nunca o vê.
        if paths.len() <= 1 {
            return;
        }
        self.import_hint = Some(format!(
            "{} arquivos soltos; apenas o primeiro será importado. Para importar os demais, arraste um arquivo por vez.",
            paths.len()
        ));
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, backend: &Backend, incidents: &[IncidentResponse]) {
        egui::ScrollArea::vertical()
            .id_salt("evidence_screen_vertical")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                self.ui_content(ui, backend, incidents);
            });
    }

    fn ui_content(&mut self, ui: &mut egui::Ui, backend: &Backend, incidents: &[IncidentResponse]) {
        // Drena o seletor nativo: o resultado chega por canal, nunca por uma
        // chamada bloqueante dentro deste quadro (GUI-01).
        match self.picker.poll() {
            Some(Picked::Path(path)) => self.request_import(path, backend),
            Some(Picked::Cancelled) => {
                self.import_hint =
                    Some("Nenhum arquivo escolhido; nenhuma importação foi iniciada.".into());
            }
            Some(Picked::Unavailable(message)) => self.import_hint = Some(message),
            Some(Picked::Failed(message)) => self.import_error = Some(message),
            None => {}
        }
        ui.collapsing(
            "Proveniência da próxima importação (opcional; preencha antes de importar)",
            |ui| provenance_form_ui(ui, &mut self.provenance),
        );
        ui.horizontal(|ui| {
            ui.heading("Evidência");
            if ui.button("Atualizar").clicked() {
                self.list = Loadable::Loading;
                backend.send(Command::ListEvidence);
            }
            if ui
                .add_enabled(
                    !self.importing && !self.picker.is_busy(),
                    egui::Button::new("Importar evidência…"),
                )
                .clicked()
            {
                self.open_picker(ui.ctx().clone());
            }
            if self.picker.is_busy() {
                ui.spinner();
                ui.label("Seletor de arquivos aberto…");
            }
            if self.importing {
                ui.spinner();
                ui.label("Importando (hash BLAKE3+SHA-256 em streaming)…");
            }
        });
        ui.horizontal_wrapped(|ui| {
            let label_id = ui.label("Ou informe o caminho:").id;
            let entry = ui
                .add(
                    egui::TextEdit::singleline(&mut self.manual_path)
                        .hint_text("/caminho/para/arquivo.evtx")
                        .desired_width(420.0),
                )
                .labelled_by(label_id);
            if entry.changed() {
                tracing::info!(
                    non_empty = !self.manual_path.trim().is_empty(),
                    "native UI import path edited"
                );
            }
            let submitted =
                entry.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter));
            let can_import_typed = !self.importing && !self.manual_path.trim().is_empty();
            let clicked = ui
                .add_enabled(can_import_typed, egui::Button::new("Importar caminho"))
                .clicked();
            if can_import_typed && (clicked || submitted) {
                let path = PathBuf::from(self.manual_path.trim());
                self.request_import(path, backend);
            }
            ui.label(egui::RichText::new("(ou arraste o arquivo para a janela)").weak());
        });
        if let Some(hint) = &self.import_hint {
            super::hint_banner(ui, hint);
        }
        if let Some(message) = &self.import_error {
            super::error_banner(ui, message);
        }
        ui.separator();

        let list_snapshot = self.items().to_vec();
        match &self.list {
            Loadable::Idle | Loadable::Loading => {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Carregando evidências…");
                });
            }
            Loadable::Failed(message) => super::error_banner(ui, message),
            Loadable::Ready(_) if list_snapshot.is_empty() => {
                ui.label("Nenhuma evidência importada ainda.");
                ui.weak("Use “Importar evidência…”, informe o caminho ou arraste um arquivo para a janela.");
            }
            Loadable::Ready(_) => {
                self.evidence_table(ui, &list_snapshot, backend);
            }
        }

        if let Some(evidence_id) = self.selected.clone() {
            ui.separator();
            let filename = list_snapshot
                .iter()
                .find(|item| item.id == evidence_id)
                .map(|item| item.original_filename.clone())
                .unwrap_or_else(|| evidence_id.clone());
            ui.heading(format!("Detalhes — {filename}"));
            if let Some(item) = list_snapshot.iter().find(|item| item.id == evidence_id) {
                provenance_summary(ui, item);
            }
            if let Some(message) = self.analysis_notice.get(&evidence_id) {
                ui.label(message);
            }
            if let Some(message) = self.last_analysis_error.get(&evidence_id) {
                super::error_banner(ui, message);
            }
            let is_evtx = list_snapshot
                .iter()
                .find(|item| item.id == evidence_id)
                .is_some_and(|item| item.format == "evtx");
            self.details(ui, &evidence_id, is_evtx, incidents, backend);
        }
    }

    fn evidence_table(&mut self, ui: &mut egui::Ui, items: &[EvidenceResponse], backend: &Backend) {
        // Uma linha de 26 px não comportava timestamp quebrado nem quatro
        // ações. A altura acompanha o conteúdo, com teto e scroll próprios;
        // o eixo horizontal preserva o acesso às ações em janelas estreitas.
        let table_height = (22.0 + items.len() as f32 * 48.0 + 16.0).min(360.0);
        let scroll_visibility = if ui.available_width() < 1000.0 {
            egui::scroll_area::ScrollBarVisibility::AlwaysVisible
        } else {
            egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded
        };
        egui::ScrollArea::horizontal()
            .id_salt("evidence_table_horizontal")
            .scroll_bar_visibility(scroll_visibility)
            .auto_shrink([false, true])
            .show(ui, |ui| {
                ui.set_width(ui.clip_rect().width().max(1000.0));
        TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .min_scrolled_height(table_height)
            .max_scroll_height(table_height)
                    .column(Column::initial(180.0).at_least(140.0).clip(true))
                    .column(Column::auto().at_least(50.0).clip(true))
                    .column(Column::auto().at_least(70.0).clip(true))
                    .column(Column::auto().at_least(70.0).clip(true))
                    .column(Column::auto().at_least(130.0).clip(true))
                    .column(Column::auto().at_least(150.0).clip(true))
                    .column(Column::remainder().at_least(300.0).clip(true))
            .header(22.0, |mut header| {
                header.col(|ui| {
                    ui.strong("Arquivo");
                });
                header.col(|ui| {
                    ui.strong("Formato");
                });
                header.col(|ui| {
                    ui.strong("Tamanho");
                });
                header.col(|ui| {
                    ui.strong("Status");
                });
                header.col(|ui| {
                    ui.strong("Importado em");
                });
                header.col(|ui| {
                    ui.strong("Tratamento");
                });
                header.col(|ui| {
                    ui.strong("Ações");
                });
            })
            .body(|mut body| {
                for item in items {
                    let selected = self.selected.as_deref() == Some(item.id.as_str());
                    body.row(48.0, |mut row| {
                        row.set_selected(selected);
                        row.col(|ui| {
                                    if ui
                                        .selectable_label(selected, &item.original_filename)
                                        .on_hover_text(&item.original_filename)
                                        .clicked()
                            {
                                self.selected = Some(item.id.clone());
                                self.lineage.clear();
                                self.analytical
                                    .entry(item.id.clone())
                                    .or_insert(Loadable::Loading);
                                backend.send(Command::EvidenceAnalyticalResults(item.id.clone()));
                            }
                        });
                        row.col(|ui| {
                            ui.label(&item.format);
                        });
                        row.col(|ui| {
                            ui.label(format_bytes(item.size_bytes));
                        });
                        row.col(|ui| {
                            ui.label(&item.status);
                        });
                        row.col(|ui| {
                            ui.add(egui::Label::new(&item.imported_at).truncate())
                                .on_hover_text(&item.imported_at);
                        });
                        row.col(|ui| {
                            let treatment = treatment_label(item);
                            ui.add(egui::Label::new(treatment).truncate())
                                .on_hover_text(treatment);
                        });
                        row.col(|ui| {
                            let analyzing = self.analyzing.contains(&item.id);
                            let destroying = self.destroying.contains(&item.id);
                            ui.horizontal_wrapped(|ui| {
                                if item.format == "evtx"
                                    && ui
                                        .add_enabled(!analyzing, egui::Button::new("Analisar EVTX"))
                                        .clicked()
                                {
                                    self.analyzing.insert(item.id.clone());
                                    backend.send(Command::Analyze {
                                        evidence_id: item.id.clone(),
                                        kind: dfir_protocol::ANALYSIS_KIND_EVTX_TIMELINE
                                            .to_string(),
                                    });
                                }
                                if item.format == "evtx" {
                                    let detecting = self.detection.is_running(&item.id);
                                    if ui
                                        .add_enabled(
                                            !analyzing && !detecting,
                                            egui::Button::new("Detectar"),
                                        )
                                        .on_hover_text(
                                            "Roda as regras de detecção e cria candidatos para revisão",
                                        )
                                        .clicked()
                                    {
                                        self.detection.start(&item.id, backend);
                                        self.selected = Some(item.id.clone());
                                        self.lineage.clear();
                                    }
                                    if detecting {
                                        ui.spinner();
                                    }
                                }
                                if ui
                                    .add_enabled(!analyzing, egui::Button::new("Analisar YARA"))
                                    .clicked()
                                {
                                    self.analyzing.insert(item.id.clone());
                                    backend.send(Command::Analyze {
                                        evidence_id: item.id.clone(),
                                        kind: dfir_protocol::ANALYSIS_KIND_YARA_SCAN.to_string(),
                                    });
                                }
                                if analyzing {
                                    let cancelling = self.cancelling.contains(&item.id);
                                    if ui
                                        .add_enabled(
                                            !cancelling,
                                            egui::Button::new("Cancelar análise"),
                                        )
                                        .clicked()
                                    {
                                        self.cancelling.insert(item.id.clone());
                                        backend.send(Command::CancelAnalysis(item.id.clone()));
                                    }
                                    if cancelling {
                                        ui.label("Cancelando…");
                                    }
                                    ui.spinner();
                                }
                                if self.pending_destroy.as_deref() == Some(item.id.as_str()) {
                                    ui.label("Confirmar?");
                                    if ui.button("Sim").clicked() {
                                        self.pending_destroy = None;
                                        self.destroying.insert(item.id.clone());
                                        backend.send(Command::DestroyEvidence(item.id.clone()));
                                    }
                                    if ui.button("Cancelar").clicked() {
                                        self.pending_destroy = None;
                                    }
                                } else if ui
                                    .add_enabled(!destroying, egui::Button::new("Destruir"))
                                    .clicked()
                                {
                                    self.pending_destroy = Some(item.id.clone());
                                }
                                if destroying {
                                    ui.spinner();
                                }
                            });
                        });
                    });
                }
            });
            });
    }

    fn details(
        &mut self,
        ui: &mut egui::Ui,
        evidence_id: &str,
        is_evtx: bool,
        incidents: &[IncidentResponse],
        backend: &Backend,
    ) {
        self.link_and_custody_panel(ui, evidence_id, incidents, backend);

        ui.collapsing("Detecções por regras (candidatos a revisar)", |ui| {
            self.detection
                .ui(ui, evidence_id, is_evtx, incidents, backend);
            if let Some(filter) = self.detection.take_timeline_filter() {
                self.open_candidate_timeline(evidence_id, filter, backend);
            }
        });

        ui.collapsing("Resultados analíticos (YARA)", |ui| {
            let lineage = &mut self.lineage;
            let loadable = self
                .analytical
                .entry(evidence_id.to_string())
                .or_insert(Loadable::Idle);
            loadable.show(ui, |ui, items| {
                if items.is_empty() {
                    ui.label("Nenhum resultado analítico para esta evidência ainda.");
                    return;
                }
                for item in items {
                    yara_result_ui(ui, item);
                    if ui
                        .button("Inspecionar linhagem")
                        .on_hover_text(
                            "Mostra o resultado selecionado, o histórico e as relações de saída",
                        )
                        .clicked()
                    {
                        lineage.ensure_loaded(
                            &item.id,
                            LineageScope::Evidence(evidence_id.to_string()),
                            backend,
                        );
                    }
                }
            });
        });
        self.lineage.ui(ui);

        let expand_timeline = self.timeline_expand.as_deref() == Some(evidence_id);
        if expand_timeline {
            self.timeline_expand = None;
        }
        egui::CollapsingHeader::new(TIMELINE_HEADING)
            .open(expand_timeline.then_some(true))
            .show(ui, |ui| {
                // Ao abrir, carrega o último job EVTX concluído da evidência (a
                // timeline sobrevive a reiniciar o app: está no banco).
                let view = self
                    .timeline
                    .entry(evidence_id.to_string())
                    .or_insert_with(|| {
                        backend.send(Command::TimelinePage {
                            evidence_id: evidence_id.to_string(),
                            job_id: None,
                            filter: String::new(),
                            offset: 0,
                            limit: PAGE_SIZE as u64,
                            want_total: true,
                        });
                        TimelineView::loading()
                    });
                view.show(ui, evidence_id, backend);
            });
    }

    /// Painel de vínculo evidência↔incidente (F1-19) e cadeia de custódia
    /// (F1-24), ambos visíveis nos detalhes da evidência.
    fn link_and_custody_panel(
        &mut self,
        ui: &mut egui::Ui,
        evidence_id: &str,
        incidents: &[IncidentResponse],
        backend: &Backend,
    ) {
        ui.collapsing("Vínculo com incidente e custódia (F1-19/F1-24)", |ui| {
            let current_link = self.list.ready().and_then(|items| {
                items
                    .iter()
                    .find(|item| item.id == evidence_id)
                    .and_then(|item| item.incident_id.clone())
            });
            match current_link {
                Some(incident_id) => {
                    let name = incidents
                        .iter()
                        .find(|incident| incident.id == incident_id)
                        .map(|incident| incident.name.as_str())
                        .unwrap_or("(incidente não carregado)");
                    ui.label(format!("Vinculada ao incidente: {name}"));
                }
                None => {
                    ui.label("Esta evidência ainda não está vinculada a nenhum incidente.");
                }
            }

            if incidents.is_empty() {
                ui.weak("Nenhum incidente aberto; crie um na aba Incidentes para vincular.");
            } else {
                let link_id = ui.make_persistent_id("link-incident-combo");
                let selected = self.pending_link.as_ref().map(|(_, id)| id.clone());
                let mut chosen = selected.unwrap_or_else(|| incidents[0].id.clone());
                egui::ComboBox::from_id_salt(link_id)
                    .width(220.0)
                    .selected_text(
                        incidents
                            .iter()
                            .find(|incident| incident.id == chosen)
                            .map(|incident| incident.name.as_str())
                            .unwrap_or("selecione o incidente"),
                    )
                    .show_ui(ui, |ui| {
                        for incident in incidents {
                            ui.selectable_value(&mut chosen, incident.id.clone(), &incident.name);
                        }
                    })
                    .response
                    .on_hover_text("Incidente ao qual vincular a evidência");
                ui.horizontal(|ui| {
                    if ui.button("Vincular ao incidente").clicked() {
                        self.pending_link = Some((evidence_id.to_string(), chosen.clone()));
                        backend.send(Command::LinkEvidenceToIncident {
                            evidence_id: evidence_id.to_string(),
                            incident_id: chosen,
                        });
                    }
                });
            }
            if let Some(notice) = &self.link_notice {
                super::hint_banner(ui, notice);
            }

            ui.separator();
            ui.collapsing("Cadeia de custódia (somente leitura)", |ui| {
                let loadable = self
                    .custody
                    .entry(evidence_id.to_string())
                    .or_insert_with(|| {
                        backend.send(Command::LoadCustody(evidence_id.to_string()));
                        Loadable::Loading
                    });
                loadable.show(ui, |ui, entries| {
                    if entries.is_empty() {
                        ui.label("Nenhuma entrada de custódia registrada.");
                        return;
                    }
                    egui_extras::TableBuilder::new(ui)
                        .striped(true)
                        .column(Column::auto().at_least(50.0))
                        .column(Column::auto().at_least(110.0))
                        .column(Column::auto().at_least(110.0))
                        .column(Column::auto().at_least(90.0))
                        .column(Column::remainder().at_least(200.0))
                        .header(20.0, |mut header| {
                            header.col(|ui| {
                                ui.strong("Seq");
                            });
                            header.col(|ui| {
                                ui.strong("Ação");
                            });
                            header.col(|ui| {
                                ui.strong("Ator");
                            });
                            header.col(|ui| {
                                ui.strong("Quando");
                            });
                            header.col(|ui| {
                                ui.strong("Motivo");
                            });
                        })
                        .body(|mut body| {
                            for entry in entries {
                                body.row(18.0, |mut row| {
                                    row.col(|ui| {
                                        ui.label(entry.sequence.to_string());
                                    });
                                    row.col(|ui| {
                                        ui.label(&entry.action);
                                    });
                                    row.col(|ui| {
                                        ui.label(&entry.actor);
                                    });
                                    row.col(|ui| {
                                        ui.label(&entry.timestamp);
                                    });
                                    row.col(|ui| {
                                        ui.label(&entry.reason);
                                    });
                                });
                            }
                        });
                    ui.small(format!(
                        "{} entrada(s); hash da última: {}…",
                        entries.len(),
                        entries
                            .last()
                            .map(|entry| &entry.entry_hash)
                            .map_or("-", |hash| &hash[..12])
                    ));
                });
            });
        });
    }
}

fn format_bytes(size: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut value = size as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{size} B")
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

/// Somente jobs `EVTX_TIMELINE` alimentam o painel de timeline (AUD-32).
fn should_reload_timeline(analysis_kind: &str) -> bool {
    analysis_kind == dfir_protocol::ANALYSIS_KIND_EVTX_TIMELINE
}

/// O que o aplicativo faz com a evidência: só EVTX é interpretado.
fn treatment_label(item: &EvidenceResponse) -> &'static str {
    if item.is_analyzed_automatically() {
        "analisada automaticamente"
    } else {
        "somente preservada (sem análise)"
    }
}

fn provenance_form_ui(ui: &mut egui::Ui, form: &mut ProvenanceForm) {
    ui.label(
        "O que ficar em branco é gravado como não informado: nada é preenchido por suposição.",
    );
    for (label, value) in [
        ("Origem (sistema ou área)", &mut form.source),
        ("Fornecida por (pessoa ou área)", &mut form.provided_by),
        ("Método de coleta", &mut form.collection_method),
        ("Ferramenta de coleta", &mut form.collection_tool),
        ("Versão da ferramenta", &mut form.collection_tool_version),
        ("Data da coleta (AAAA-MM-DD)", &mut form.collected_at),
        (
            "Autorização (ofício, ticket)",
            &mut form.authorization_reference,
        ),
    ] {
        super::field_text(ui, label, value);
    }
    ui.checkbox(
        &mut form.contains_personal_data,
        "Contém dado pessoal (a exportação oferecerá mascarar)",
    );
}

fn declared(value: Option<&str>) -> String {
    value
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map_or_else(|| "não informado".to_string(), str::to_string)
}

/// Proveniência e tratamento da evidência selecionada.
fn provenance_summary(ui: &mut egui::Ui, item: &EvidenceResponse) {
    egui::Grid::new("evidence_provenance")
        .num_columns(2)
        .show(ui, |ui| {
            for (label, value) in [
                ("Tratamento", treatment_label(item).to_string()),
                ("Origem", declared(item.declared_source())),
                ("Fornecida por", declared(item.provided_by.as_deref())),
                (
                    "Método de coleta",
                    declared(item.declared_collection_method()),
                ),
                ("Ferramenta", declared(item.collection_tool.as_deref())),
                (
                    "Versão da ferramenta",
                    declared(item.collection_tool_version.as_deref()),
                ),
                ("Data da coleta", declared(item.declared_collected_at())),
                (
                    "Autorização",
                    declared(item.authorization_reference.as_deref()),
                ),
                (
                    "Dado pessoal",
                    if item.contains_personal_data {
                        "sim".to_string()
                    } else {
                        "não marcado".to_string()
                    },
                ),
            ] {
                ui.label(format!("{label}:"));
                ui.label(value);
                ui.end_row();
            }
        });
}

#[cfg(test)]
mod timeline_reload_tests {
    use super::{EvidenceState, TIMELINE_HEADING, should_reload_timeline};
    use crate::gui::backend::{Backend, Command};
    use crate::gui::test_harness::drain;
    use crate::service::{AnalyzeResponse, TimelinePage};
    use egui_kittest::kittest::Queryable;

    #[test]
    fn only_evtx_jobs_reload_the_timeline() {
        assert!(should_reload_timeline(
            dfir_protocol::ANALYSIS_KIND_EVTX_TIMELINE
        ));
        assert!(!should_reload_timeline(
            dfir_protocol::ANALYSIS_KIND_YARA_SCAN
        ));
        assert!(!should_reload_timeline(""));
    }

    fn analysis(job_id: &str, kind: &str) -> AnalyzeResponse {
        AnalyzeResponse {
            request_id: format!("request-{job_id}"),
            job_id: job_id.into(),
            evidence_id: "ev-1".into(),
            status: "COMPLETED".into(),
            analysis_kind: kind.into(),
            output_reference: None,
            message_count: 1,
            item_count: 1,
            warnings: vec![],
        }
    }

    #[test]
    fn candidate_refs_open_same_evidence_at_page_zero_with_latest_job_and_total() {
        let (backend, mut commands) = Backend::for_test();
        let mut state = EvidenceState::default();
        state
            .timeline
            .insert("ev-1".into(), super::TimelineView::for_filter("event:4624"));

        state.open_candidate_timeline("ev-1", "ref:record:5,record:9".into(), &backend);

        let sent = drain(&mut commands);
        assert_eq!(sent.len(), 1, "somente a nova página inicial: {sent:?}");
        assert!(matches!(
            &sent[0],
            Command::TimelinePage {
                evidence_id,
                job_id: None,
                filter,
                offset: 0,
                limit,
                want_total: true,
            } if evidence_id == "ev-1"
                && filter == "ref:record:5,record:9"
                && *limit == super::PAGE_SIZE as u64
        ));
    }

    #[test]
    fn candidate_navigation_expands_the_collapsed_timeline_section() {
        const REFS: &str = "ref:record:5,record:9";
        let (backend, mut commands) = Backend::for_test();
        let mut state = EvidenceState::default();
        state.detection.queue_timeline_filter_for_test(REFS);
        let mut harness = crate::gui::test_harness::builder()
            .with_size(egui::vec2(1000.0, 1200.0))
            .build_ui_state(
                |ui, (state, backend): &mut (EvidenceState, Backend)| {
                    state.details(ui, "ev-1", true, &[], backend);
                },
                (state, backend),
            );
        harness.step();
        assert!(harness.query_by_label("Carregando a timeline…").is_none());

        harness
            .get_by_label("Detecções por regras (candidatos a revisar)")
            .click();
        harness.run_steps(2);

        let sent = drain(&mut commands);
        assert!(
            sent.iter().any(|command| matches!(
                command,
                Command::TimelinePage { filter, .. } if filter == REFS
            )),
            "a navegação precisa pedir a página filtrada: {sent:?}"
        );
        let (state, _) = harness.state();
        assert!(state.timeline.contains_key("ev-1"));
        assert!(state.timeline_expand.is_none());
        harness.get_by_label(TIMELINE_HEADING);
        harness.get_by_label("Carregando a timeline…");
        harness
            .state_mut()
            .0
            .timeline
            .get_mut("ev-1")
            .expect("visão da evidência")
            .on_page(
                REFS,
                0,
                Ok(Some(TimelinePage::for_test(
                    "job-1",
                    Some(0),
                    0,
                    vec![],
                    vec![],
                ))),
            );
        harness.step();
        assert_eq!(
            harness
                .get_by_role(egui::accesskit::Role::TextInput)
                .value()
                .as_deref(),
            Some(REFS)
        );
    }

    #[test]
    fn completed_jobs_release_the_action_and_request_only_their_own_results() {
        let (backend, mut commands) = Backend::for_test();
        let mut state = EvidenceState::default();
        state.analyzing.insert("ev-1".into());
        state.on_analyze_completed(
            "ev-1",
            Ok(analysis(
                "job-evtx",
                dfir_protocol::ANALYSIS_KIND_EVTX_TIMELINE,
            )),
            &backend,
        );
        assert!(!state.analyzing.contains("ev-1"));
        let sent = drain(&mut commands);
        assert_eq!(sent.len(), 2, "{sent:?}");
        assert!(matches!(
            &sent[0],
            Command::TimelinePage { evidence_id, job_id, offset: 0, want_total: true, .. }
                if evidence_id == "ev-1" && job_id.as_deref() == Some("job-evtx")
        ));
        assert!(matches!(&sent[1], Command::EvidenceAnalyticalResults(id) if id == "ev-1"));

        state.analyzing.insert("ev-1".into());
        state.on_analyze_completed(
            "ev-1",
            Ok(analysis("job-yara", dfir_protocol::ANALYSIS_KIND_YARA_SCAN)),
            &backend,
        );
        assert!(!state.analyzing.contains("ev-1"));
        let sent = drain(&mut commands);
        assert_eq!(sent.len(), 1, "YARA não recarrega a timeline: {sent:?}");
        assert!(matches!(&sent[0], Command::EvidenceAnalyticalResults(id) if id == "ev-1"));
    }

    #[test]
    fn cancelled_jobs_do_not_reload_timeline_or_analytical_results() {
        let (backend, mut commands) = Backend::for_test();
        let mut state = EvidenceState::default();
        state.analyzing.insert("ev-1".into());
        state.cancelling.insert("ev-1".into());
        let mut response = analysis("job-cancelled", dfir_protocol::ANALYSIS_KIND_EVTX_TIMELINE);
        response.status = "CANCELLED".into();
        state.on_analyze_completed("ev-1", Ok(response), &backend);
        assert!(!state.analyzing.contains("ev-1"));
        assert!(!state.cancelling.contains("ev-1"));
        assert!(drain(&mut commands).is_empty());
        assert_eq!(
            state.analysis_notice.get("ev-1").map(String::as_str),
            Some("Análise cancelada pelo operador (CANCELLED).")
        );
    }
}

#[cfg(test)]
mod analytical_ui_tests {
    use super::*;
    use crate::gui::backend::Backend;
    use crate::gui::test_harness::drain;
    use egui_kittest::kittest::Queryable;

    type State = (EvidenceState, Backend);

    #[test]
    fn yara_results_show_metadata_matches_and_long_text_instead_of_raw_json() {
        let (backend, mut commands) = Backend::for_test();
        let description = "Detecta a sequência de teste EICAR preservada em um artefato, permitindo validar o funcionamento do scanner e a rastreabilidade das ocorrências sem incorporar os bytes casados ao resultado analítico exibido ao operador.";
        let reference = "https://www.example.invalid/dfir/regras/eicar/documentacao-completa-para-revisao-do-analista";
        let payload = serde_json::json!({
            "kind": "YARA_MATCH",
            "rule_id": "eicar_test_string",
            "namespace": "dfir_embedded",
            "ruleset_id": "dfir-embedded-yara",
            "ruleset_version": "2026.11.1",
            "ruleset_sha256": "ab".repeat(32),
            "ruleset_license": "Apache-2.0",
            "ruleset_source_origin": "embedded://dfir-engine/rules/embedded.yar",
            "engine_compatibility": "yara-x 1.20.0",
            "evidence_id": "evidence/ev-42",
            "confidence": "HIGH",
            "severity": "MEDIUM",
            "description": description,
            "reference": reference,
            "matched_patterns": [
                {"pattern_id": "$eicar", "offset": 128, "length": 68},
                {"pattern_id": "$eicar", "offset": 4096, "length": 68}
            ],
            "pattern_match_counts": [
                {"pattern_id": "$eicar", "total_matches": 12, "reported_matches": 2}
            ],
            "matches_truncated": true
        });
        let raw_payload = payload.to_string();
        let item = AnalyticalResultResponse {
            id: "yara-1".into(),
            result_type: "YARA_MATCH".into(),
            correlation_key: "yara:eicar_test_string:ev-42".into(),
            evidence_id: Some("ev-42".into()),
            incident_id: None,
            analysis_job_id: Some("job-7".into()),
            rule_or_model_id: Some("eicar_test_string".into()),
            rule_or_model_version: Some("2026.11.1".into()),
            confidence: Some("HIGH".into()),
            payload,
            provenance: serde_json::Value::Null,
            supersedes: None,
            created_at: "2026-10-16T00:00:00Z".into(),
        };
        let mut state = EvidenceState::default();
        state.on_analytical_results("ev-42", Ok(vec![item]));
        let mut harness = crate::gui::test_harness::builder()
            .with_size(egui::vec2(1000.0, 1800.0))
            .build_ui_state(
                |ui, (state, backend): &mut State| state.details(ui, "ev-42", false, &[], backend),
                (state, backend),
            );
        harness.step();
        harness.get_by_label("Resultados analíticos (YARA)").click();
        harness.run();

        for label in [
            "YARA_MATCH".to_string(),
            "regra: eicar_test_string".into(),
            "confiança: HIGH".into(),
            "Severidade: MEDIUM".into(),
            format!("Descrição: {description}"),
            format!("Referência: {reference}"),
            "Namespace: dfir_embedded".into(),
            "Ruleset: dfir-embedded-yara · versão 2026.11.1".into(),
            format!("SHA-256 do ruleset: {}", "ab".repeat(32)),
            "Licença do ruleset: Apache-2.0".into(),
            "Origem do ruleset: embedded://dfir-engine/rules/embedded.yar".into(),
            "Compatibilidade do engine: yara-x 1.20.0".into(),
            "Referência da evidência: evidence/ev-42".into(),
            "Ocorrências detalhadas (2)".into(),
            "Padrão $eicar · offset 128 · comprimento 68".into(),
            "Padrão $eicar · offset 4096 · comprimento 68".into(),
            "$eicar: 12 total(is), 2 exibida(s)".into(),
            "Ocorrências truncadas: sim (totais podem ser limites inferiores)".into(),
        ] {
            harness.get_by_label(label.as_str());
        }
        assert!(
            harness.query_by_label(raw_payload.as_str()).is_none(),
            "o payload JSON minificado não deve ser exibido"
        );
        assert!(
            harness
                .query_by_label_contains("\"matched_patterns\":")
                .is_none(),
            "nenhuma chave JSON bruta deve virar interface"
        );
        harness.get_by_label("Inspecionar linhagem").click();
        harness.step();
        let sent = drain(&mut commands);
        assert_eq!(sent.len(), 1, "uma consulta de linhagem: {sent:?}");
        assert!(matches!(
            &sent[0],
            Command::LineageForResult { result_id, scope: LineageScope::Evidence(evidence_id) }
                if result_id == "yara-1" && evidence_id == "ev-42"
        ));
    }
}

/// Testes da proveniência e do tratamento na aba de evidências
/// (`egui_kittest`, ADR-0022). O seletor nativo de arquivo não é acionado:
/// a importação é exercida por `begin_import`.
#[cfg(test)]
mod provenance_ui_tests {
    use super::*;
    use crate::gui::backend::Backend;
    use crate::gui::test_harness::drain;
    use egui_kittest::kittest::Queryable;

    type State = (EvidenceState, Backend);

    fn evidence(id: &str, filename: &str, format: &str) -> EvidenceResponse {
        EvidenceResponse {
            id: id.into(),
            original_filename: filename.into(),
            storage_reference: format!("evidence/{id}"),
            format: format.into(),
            size_bytes: 2048,
            blake3: "b3".repeat(32),
            sha256: "5a".repeat(32),
            imported_at: "2026-10-01T10:00:00+00:00".into(),
            status: "FORENSIC".into(),
            source: "NOT_PROVIDED".into(),
            provided_by: None,
            collection_method: "NOT_PROVIDED".into(),
            collection_tool: None,
            collection_tool_version: None,
            collected_at: None,
            authorization_reference: None,
            contains_personal_data: false,
            incident_id: None,
        }
    }

    #[test]
    fn importing_sends_the_declared_provenance_and_success_clears_the_form() {
        let (backend, mut commands) = Backend::for_test();
        let mut state = EvidenceState::default();
        state.provenance.source = "  servidor FS02 ".into();
        state.provenance.provided_by = "TI do cliente".into();
        state.provenance.collected_at = "2026-09-30".into();
        state.provenance.contains_personal_data = true;
        state.begin_import(PathBuf::from("/tmp/Security.evtx"), &backend);
        assert!(state.importing);

        let sent = drain(&mut commands);
        assert_eq!(sent.len(), 1, "{sent:?}");
        match &sent[0] {
            Command::ImportEvidence {
                path, provenance, ..
            } => {
                assert_eq!(path, &PathBuf::from("/tmp/Security.evtx"));
                assert_eq!(provenance.source.as_deref(), Some("servidor FS02"));
                assert_eq!(provenance.provided_by.as_deref(), Some("TI do cliente"));
                assert_eq!(provenance.collected_at.as_deref(), Some("2026-09-30"));
                assert_eq!(provenance.collection_tool, None, "em branco vira ausente");
                assert!(provenance.contains_personal_data);
            }
            other => panic!("comando inesperado: {other:?}"),
        }

        state.on_imported(Ok(evidence("ev-1", "Security.evtx", "evtx")));
        assert!(!state.importing);
        assert_eq!(
            state.provenance.source, "",
            "o formulário é limpo após importar"
        );
        assert!(!state.provenance.contains_personal_data);

        // Falha na importação mantém o que foi digitado.
        state.provenance.source = "x".into();
        state.begin_import(PathBuf::from("/tmp/b.evtx"), &backend);
        state.on_imported(Err("too large".into()));
        assert_eq!(state.provenance.source, "x");
    }

    #[test]
    fn a_blank_form_sends_everything_as_not_provided() {
        let (backend, mut commands) = Backend::for_test();
        let mut state = EvidenceState::default();
        state.begin_import(PathBuf::from("/tmp/a.evtx"), &backend);
        match &drain(&mut commands)[0] {
            Command::ImportEvidence { provenance, .. } => {
                assert!(provenance.source.is_none() && provenance.provided_by.is_none());
                assert!(provenance.collected_at.is_none());
                assert!(!provenance.contains_personal_data);
            }
            other => panic!("comando inesperado: {other:?}"),
        }
    }

    #[test]
    fn the_typed_path_rejects_empty_or_directory_without_sending_anything() {
        let (backend, mut commands) = Backend::for_test();
        let mut state = EvidenceState::default();

        state.request_import(PathBuf::new(), &backend);
        assert!(
            state
                .import_hint
                .as_deref()
                .unwrap_or("")
                .contains("caminho")
        );
        assert!(!state.importing);

        let directory = tempfile::tempdir().expect("temp dir");
        state.request_import(directory.path().to_path_buf(), &backend);
        assert!(state.import_hint.as_deref().unwrap_or("").contains("pasta"));
        assert!(drain(&mut commands).is_empty(), "nada foi importado");
    }

    #[test]
    fn directory_rejection_does_not_leak_path_in_the_hint() {
        let (backend, _commands) = Backend::for_test();
        let mut state = EvidenceState::default();

        // Diretório sintético e existente: request_import rejeita antes de
        // enviar qualquer comando, sem expor seus componentes no aviso.
        let temp = tempfile::tempdir().expect("temp dir");
        let deep = temp.path().join("caso-real").join("Senha do cliente");
        std::fs::create_dir_all(&deep).expect("synthetic directory");
        state.request_import(deep, &backend);
        let hint = state.import_hint.as_deref().unwrap_or("");
        assert!(hint.contains("pasta"));
        assert!(
            !hint.contains("caso-real") && !hint.contains("Senha do cliente"),
            "o caminho não deve vazar no aviso: {hint}"
        );
    }

    #[test]
    fn multiple_dropped_files_mention_count_without_naming_files() {
        let (backend, mut commands) = Backend::for_test();
        let mut state = EvidenceState::default();
        let dir = tempfile::tempdir().expect("temp dir");
        let first = dir.path().join("Security.evtx");
        let second = dir.path().join("extra.evtx");
        std::fs::write(&first, b"not real evtx").expect("first");
        std::fs::write(&second, b"not real evtx").expect("second");

        state.on_files_dropped(&[first.clone(), second], &backend);
        let hint = state.import_hint.as_deref().unwrap_or("");
        assert!(hint.contains("2 arquivos"));
        assert!(
            !hint.contains("Security.evtx"),
            "nome do arquivo não deve vazar: {hint}"
        );
        assert!(
            !hint.contains("extra.evtx"),
            "nome do arquivo não deve vazar: {hint}"
        );
        match &drain(&mut commands)[0] {
            Command::ImportEvidence { path, .. } => assert_eq!(path, &first),
            other => panic!("comando inesperado: {other:?}"),
        }
    }

    #[test]
    fn the_typed_path_button_imports_a_trimmed_file_path_and_clears_the_field() {
        let (backend, mut commands) = Backend::for_test();
        let directory = tempfile::tempdir().expect("temp dir");
        let file = directory.path().join("Security.evtx");
        std::fs::write(&file, b"not real evtx").expect("fixture");
        let state = EvidenceState {
            list: Loadable::Ready(vec![]),
            manual_path: format!("  {}  ", file.display()),
            ..EvidenceState::default()
        };
        let mut harness = crate::gui::test_harness::builder()
            .with_size(egui::vec2(1200.0, 800.0))
            .build_ui_state(
                |ui, (state, backend): &mut State| state.ui(ui, backend, &[]),
                (state, backend),
            );
        harness.step();
        harness.get_by_label("Importar caminho").click();
        harness.step();

        let sent = drain(&mut commands);
        assert_eq!(sent.len(), 1, "{sent:?}");
        assert!(matches!(
            &sent[0],
            Command::ImportEvidence { path, .. } if path == &file
        ));
        assert!(harness.state().0.importing);
        assert!(harness.state().0.manual_path.is_empty());
        harness.step();
        harness.get_by_label_contains("Importando (hash BLAKE3+SHA-256");
    }

    #[test]
    fn import_completion_updates_the_list_and_failures_keep_existing_rows() {
        let existing = evidence("ev-1", "Security.evtx", "evtx");
        let imported = evidence("ev-2", "System.evtx", "evtx");
        let mut state = EvidenceState {
            list: Loadable::Ready(vec![existing.clone()]),
            importing: true,
            ..EvidenceState::default()
        };
        state.on_imported(Ok(imported));
        assert!(!state.importing);
        assert_eq!(state.items()[0].id, "ev-2");
        assert_eq!(state.items()[1].id, "ev-1");
        assert!(state.import_error.is_none());

        state.importing = true;
        state.on_imported(Err("arquivo excede o limite".into()));
        assert!(!state.importing);
        assert_eq!(
            state.items().len(),
            2,
            "falha não apaga evidências já listadas"
        );
        assert_eq!(
            state.import_error.as_deref(),
            Some("arquivo excede o limite")
        );
    }

    #[test]
    fn destroying_requires_confirmation_and_cancellation_sends_nothing() {
        let (backend, mut commands) = Backend::for_test();
        let state = EvidenceState {
            list: Loadable::Ready(vec![evidence("ev-1", "Security.evtx", "evtx")]),
            selected: Some("ev-1".into()),
            ..EvidenceState::default()
        };
        let mut harness = crate::gui::test_harness::builder()
            .with_size(egui::vec2(1600.0, 900.0))
            .build_ui_state(
                |ui, (state, backend): &mut State| state.ui(ui, backend, &[]),
                (state, backend),
            );
        harness.run();
        harness.get_by_label("Destruir").click();
        harness.run();
        harness.get_by_label("Confirmar?");
        harness.get_by_label("Cancelar").click();
        harness.run();
        assert!(harness.state().0.pending_destroy.is_none());
        assert!(drain(&mut commands).is_empty(), "cancelar não destrói");

        harness.get_by_label("Destruir").click();
        harness.run();
        harness.get_by_label("Sim").click();
        harness.step();
        assert!(matches!(
            drain(&mut commands).as_slice(),
            [Command::DestroyEvidence(id)] if id == "ev-1"
        ));
        assert!(harness.state().0.destroying.contains("ev-1"));

        harness.state_mut().0.on_destroyed("ev-1", Ok(()));
        harness.run();
        assert!(harness.state().0.items().is_empty());
        assert!(harness.state().0.selected.is_none());
        harness.get_by_label("Nenhuma evidência importada ainda.");
    }

    #[test]
    fn a_failed_destruction_preserves_the_evidence_and_shows_the_reason() {
        let (backend, _commands) = Backend::for_test();
        let mut state = EvidenceState {
            list: Loadable::Ready(vec![evidence("ev-1", "Security.evtx", "evtx")]),
            selected: Some("ev-1".into()),
            destroying: HashSet::from(["ev-1".into()]),
            ..EvidenceState::default()
        };
        state.on_destroyed("ev-1", Err("custódia não pôde ser registrada".into()));
        assert!(!state.destroying.contains("ev-1"));
        assert_eq!(state.items().len(), 1);
        assert_eq!(state.selected.as_deref(), Some("ev-1"));

        let mut harness = crate::gui::test_harness::builder()
            .with_size(egui::vec2(1600.0, 900.0))
            .build_ui_state(
                |ui, (state, backend): &mut State| state.ui(ui, backend, &[]),
                (state, backend),
            );
        harness.run();
        harness.get_by_label("Erro: custódia não pôde ser registrada");
        harness.get_by_label("Security.evtx");
    }

    #[test]
    fn dropping_files_imports_the_first_and_mentions_the_rest() {
        let (backend, mut commands) = Backend::for_test();
        let mut state = EvidenceState::default();
        let dir = tempfile::tempdir().expect("temp dir");
        let first = dir.path().join("Security.evtx");
        let second = dir.path().join("extra.evtx");
        std::fs::write(&first, b"not real evtx").expect("first");
        std::fs::write(&second, b"not real evtx").expect("second");

        state.on_files_dropped(&[first.clone(), second], &backend);
        assert!(state.importing);
        assert!(
            state
                .import_hint
                .as_deref()
                .unwrap_or("")
                .contains("2 arquivos"),
            "o arquivo extra é mencionado, nunca descartado em silêncio"
        );
        match &drain(&mut commands)[0] {
            Command::ImportEvidence { path, .. } => assert_eq!(path, &first),
            other => panic!("comando inesperado: {other:?}"),
        }
    }

    #[test]
    fn a_second_import_while_one_is_running_is_refused_with_a_hint() {
        let (backend, mut commands) = Backend::for_test();
        let mut state = EvidenceState::default();
        state.begin_import(PathBuf::from("/tmp/first.evtx"), &backend);
        drain(&mut commands);

        state.request_import(PathBuf::from("/tmp/segundo.evtx"), &backend);
        assert!(
            state
                .import_hint
                .as_deref()
                .unwrap_or("")
                .contains("andamento")
        );
        assert!(drain(&mut commands).is_empty(), "nada é enfileirado");
    }

    #[test]
    fn the_table_says_which_evidence_is_analysed_and_which_is_only_preserved() {
        let (backend, _commands) = Backend::for_test();
        let state = EvidenceState {
            list: Loadable::Ready(vec![
                evidence("ev-1", "Security.evtx", "evtx"),
                evidence("ev-2", "captura.pcap", "pcap"),
            ]),
            ..EvidenceState::default()
        };
        let mut harness = crate::gui::test_harness::builder()
            .with_size(egui::vec2(1400.0, 900.0))
            .build_ui_state(
                |ui, (state, backend): &mut State| state.ui(ui, backend, &[]),
                (state, backend),
            );
        harness.run();
        harness.get_by_label("Tratamento");
        harness.get_by_label("analisada automaticamente");
        harness.get_by_label("somente preservada (sem análise)");
    }

    #[test]
    fn evidence_actions_fit_the_row_and_remain_reachable_at_minimum_width() {
        let (backend, _commands) = Backend::for_test();
        let filename = "Security-with-a-much-longer-name-that-cannot-fit.evtx";
        let state = EvidenceState {
            list: Loadable::Ready(vec![evidence("ev-1", filename, "evtx")]),
            ..EvidenceState::default()
        };
        let width = 780.0;
        let mut harness = crate::gui::test_harness::builder()
            .with_size(egui::vec2(width, 700.0))
            .build_ui_state(
                |ui, (state, backend): &mut State| state.ui(ui, backend, &[]),
                (state, backend),
            );
        harness.step();

        let filename_rect = harness.get_by_label(filename).rect();
        let row_top = filename_rect.min.y;
        let row_bottom = row_top + 48.0;
        let mut action_rects = Vec::new();
        for label in ["Analisar EVTX", "Detectar", "Analisar YARA", "Destruir"] {
            let rect = harness.get_by_label(label).rect();
            assert!(
                rect.min.y >= row_top && rect.max.y <= row_bottom,
                "ação {label:?} fora da faixa vertical de 48 px: ação={rect:?}, topo={row_top}, fundo={row_bottom}"
            );
            for (other_label, other_rect) in &action_rects {
                assert!(
                    !rect.intersects(*other_rect),
                    "ações {label:?} e {other_label:?} sobrepostas: {rect:?} e {other_rect:?}"
                );
            }
            action_rects.push((label, rect));
        }

        let scroll_width = 600.0;
        harness.set_size(egui::vec2(scroll_width, 700.0));
        harness.step();
        let initial_destroy = harness.get_by_label("Destruir").rect();
        assert!(
            initial_destroy.max.x > scroll_width,
            "a fixture deve começar fora da viewport para provar o scroll: {initial_destroy:?}"
        );
        harness.get_by_label("Arquivo").hover();
        for _ in 0..2 {
            harness.input_mut().events.push(egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(-100.0, 0.0),
                phase: egui::TouchPhase::Move,
                source: egui::MouseWheelSource::Unknown,
                modifiers: egui::Modifiers::default(),
            });
        }
        harness.run();
        let destroy = harness.get_by_label("Destruir").rect();
        assert!(
            destroy.min.x >= 0.0 && destroy.max.x <= scroll_width,
            "Destruir não ficou acessível pelo scroll horizontal: {destroy:?}"
        );
        harness.get_by_label("Destruir").click();
        harness.run();
        assert_eq!(harness.state().0.pending_destroy.as_deref(), Some("ev-1"));
    }

    #[test]
    fn vertical_wheel_reveals_yara_content_below_the_viewport() {
        let (backend, _commands) = Backend::for_test();
        let patterns: Vec<_> = (0..40)
            .map(|index| {
                serde_json::json!({
                    "pattern_id": format!("$padrao_{index}"),
                    "offset": index * 128,
                    "length": 68
                })
            })
            .collect();
        let item = AnalyticalResultResponse {
            id: "yara-1".into(),
            result_type: "YARA_MATCH".into(),
            correlation_key: "yara:regra:ev-1".into(),
            evidence_id: Some("ev-1".into()),
            incident_id: None,
            analysis_job_id: Some("job-1".into()),
            rule_or_model_id: Some("regra_de_regressao".into()),
            rule_or_model_version: None,
            confidence: None,
            payload: serde_json::json!({
                "matched_patterns": patterns,
                "matches_truncated": false
            }),
            provenance: serde_json::Value::Null,
            supersedes: None,
            created_at: "2026-11-18T00:00:00Z".into(),
        };
        let mut state = EvidenceState {
            list: Loadable::Ready(vec![evidence("ev-1", "senai-demo.evtx", "evtx")]),
            selected: Some("ev-1".into()),
            ..EvidenceState::default()
        };
        state.on_analytical_results("ev-1", Ok(vec![item]));
        let viewport = egui::vec2(960.0, 600.0);
        let mut harness = crate::gui::test_harness::builder()
            .with_size(viewport)
            .build_ui_state(
                |ui, (state, backend): &mut State| state.ui(ui, backend, &[]),
                (state, backend),
            );
        harness.step();
        harness.get_by_label("Resultados analíticos (YARA)").click();
        harness.run();

        let marker = "Ocorrências truncadas: não";
        let initial = harness.get_by_label(marker).rect();
        assert!(
            initial.min.y > viewport.y,
            "o marcador deve começar abaixo da viewport: {initial:?}"
        );

        harness.get_by_label("Evidência").hover();
        for _ in 0..5 {
            harness.input_mut().events.push(egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(0.0, -300.0),
                phase: egui::TouchPhase::Move,
                source: egui::MouseWheelSource::Unknown,
                modifiers: egui::Modifiers::default(),
            });
        }
        harness.run();

        let revealed = harness.get_by_label(marker).rect();
        assert!(
            revealed.min.y < initial.min.y,
            "a roda vertical não moveu o conteúdo: antes={initial:?}, depois={revealed:?}"
        );
        assert!(
            revealed.min.y >= 0.0 && revealed.max.y <= viewport.y,
            "o marcador não ficou visível após a rolagem vertical: {revealed:?}"
        );
        assert!(
            revealed.min.x >= 0.0 && revealed.max.x <= viewport.x,
            "a rolagem vertical introduziu deslocamento horizontal espúrio: {revealed:?}"
        );
    }

    #[test]
    fn evidence_controls_do_not_depend_on_missing_emoji_fonts() {
        let (backend, _commands) = Backend::for_test();
        let state = EvidenceState {
            list: Loadable::Ready(vec![]),
            ..EvidenceState::default()
        };
        // Diferente do builder compartilhado, este mantém o default do
        // kittest: qualquer glifo ausente faz o teste falhar.
        let mut harness = egui_kittest::HarnessBuilder::default()
            .with_size(egui::vec2(1000.0, 700.0))
            .build_ui_state(
                |ui, (state, backend): &mut State| state.ui(ui, backend, &[]),
                (state, backend),
            );
        harness.run();
        harness.get_by_label("Atualizar");
        harness.get_by_label("Importar evidência…");
    }

    #[test]
    fn the_summary_shows_declared_provenance_and_marks_the_rest_as_not_informed() {
        let mut declared_item = evidence("ev-1", "Security.evtx", "evtx");
        declared_item.source = "servidor de arquivos FS02".into();
        declared_item.provided_by = Some("TI do cliente".into());
        declared_item.collection_method = "exportação com wevtutil".into();
        declared_item.collected_at = Some("2026-09-30".into());
        declared_item.contains_personal_data = true;
        let mut harness = crate::gui::test_harness::builder()
            .build_ui(move |ui| provenance_summary(ui, &declared_item));
        harness.run();
        harness.get_by_label("servidor de arquivos FS02");
        harness.get_by_label("TI do cliente");
        harness.get_by_label("2026-09-30");
        harness.get_by_label("sim");
        // Ferramenta, versão e autorização não foram informadas.
        assert_eq!(harness.query_all_by_label("não informado").count(), 3);

        // Linha antiga: constantes de importação e data de importação como
        // coleta não contam como informadas.
        let mut legacy = evidence("ev-0", "Old.evtx", "evtx");
        legacy.source = "LOCAL_OPERATOR".into();
        legacy.collection_method = "LOCAL_FILE_PICKER".into();
        legacy.collected_at = Some(legacy.imported_at.clone());
        let mut harness =
            crate::gui::test_harness::builder().build_ui(move |ui| provenance_summary(ui, &legacy));
        harness.run();
        assert_eq!(harness.query_all_by_label("não informado").count(), 7);
        assert!(harness.query_by_label("LOCAL_OPERATOR").is_none());
    }

    fn incident(id: &str, name: &str) -> IncidentResponse {
        IncidentResponse {
            id: id.into(),
            name: name.into(),
            state: "OPEN".into(),
            created_at: "2026-03-01T00:00:00Z".into(),
        }
    }

    #[test]
    fn linking_sends_the_command_and_the_result_updates_the_row() {
        let (backend, mut commands) = Backend::for_test();
        let incidents = [incident("inc-1", "Incidente demo")];

        let mut link_state = EvidenceState {
            list: Loadable::Ready(vec![evidence("ev-1", "Security.evtx", "evtx")]),
            selected: Some("ev-1".into()),
            ..EvidenceState::default()
        };
        {
            let mut harness = crate::gui::test_harness::builder()
                .with_size(egui::vec2(1400.0, 900.0))
                .build_ui_state(
                    |ui, state: &mut (&mut EvidenceState, &Backend)| {
                        state.0.ui(ui, state.1, &incidents)
                    },
                    (&mut link_state, &backend),
                );
            harness.run();
            harness
                .get_by_label("Vínculo com incidente e custódia (F1-19/F1-24)")
                .click();
            harness.run();
            harness.get_by_label("Esta evidência ainda não está vinculada a nenhum incidente.");
            harness.get_by_label("Vincular ao incidente").click();
            harness.run();
        }

        let sent = drain(&mut commands);
        assert!(
            sent.iter().any(|command| matches!(
                command,
                Command::LinkEvidenceToIncident { evidence_id, incident_id }
                    if evidence_id == "ev-1" && incident_id == "inc-1"
            )),
            "o vínculo dispara LinkEvidenceToIncident: {sent:?}"
        );

        let mut linked = evidence("ev-1", "Security.evtx", "evtx");
        linked.incident_id = Some("inc-1".into());
        link_state.on_linked_to_incident("ev-1", Ok(linked));
        assert_eq!(
            link_state.items()[0].incident_id.as_deref(),
            Some("inc-1"),
            "a linha é atualizada com o vínculo"
        );
        assert!(
            link_state
                .link_notice
                .as_deref()
                .unwrap_or("")
                .contains("vinculada")
        );

        link_state.on_linked_to_incident("ev-1", Err("incidente inexistente".into()));
        assert!(
            link_state
                .link_notice
                .as_deref()
                .unwrap_or("")
                .contains("não feito")
        );
    }

    #[test]
    fn opening_the_custody_panel_loads_it_once() {
        let (backend, mut commands) = Backend::for_test();
        let mut state = EvidenceState {
            list: Loadable::Ready(vec![evidence("ev-1", "Security.evtx", "evtx")]),
            selected: Some("ev-1".into()),
            ..EvidenceState::default()
        };

        {
            let mut harness = crate::gui::test_harness::builder()
                .with_size(egui::vec2(1400.0, 900.0))
                .build_ui_state(
                    |ui, state: &mut (&mut EvidenceState, &Backend)| state.0.ui(ui, state.1, &[]),
                    (&mut state, &backend),
                );
            harness.run();
            harness
                .get_by_label("Vínculo com incidente e custódia (F1-19/F1-24)")
                .click();
            harness.run();
            harness
                .get_by_label("Cadeia de custódia (somente leitura)")
                .click();
            // O painel fica em Loading (spinner) até o evento chegar: usa
            // step em vez de run para não estourar o limite de passos.
            harness.step();
            harness.step();
        }

        let sent = drain(&mut commands);
        let loads = sent
            .iter()
            .filter(|command| matches!(command, Command::LoadCustody(id) if id == "ev-1"))
            .count();
        assert_eq!(loads, 1, "custódia carrega uma única vez: {sent:?}");

        state.on_custody_loaded(
            "ev-1",
            Ok(vec![crate::persistence::audit::CustodyEntry {
                sequence: 1,
                entry_id: "ce-1".into(),
                evidence_id: "ev-1".into(),
                action: "IMPORTED".into(),
                actor: "operador".into(),
                timestamp: "2026-11-15T00:00:00Z".into(),
                reason: "importação inicial".into(),
                metadata: serde_json::json!({}),
                previous_entry_hash: None,
                entry_hash: "ab".repeat(32),
            }]),
        );
        let mut harness = crate::gui::test_harness::builder()
            .with_size(egui::vec2(1400.0, 900.0))
            .build_ui_state(
                |ui, (state, backend): &mut State| state.ui(ui, backend, &[]),
                (state, Backend::for_test().0),
            );
        harness.run();
        harness
            .get_by_label("Vínculo com incidente e custódia (F1-19/F1-24)")
            .click();
        harness.run();
        harness
            .get_by_label("Cadeia de custódia (somente leitura)")
            .click();
        harness.run();
        harness.get_by_label("IMPORTED");
        harness.get_by_label("1 entrada(s); hash da última: abababababab…");
    }
}
