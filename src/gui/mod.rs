//! Interface nativa (eframe/egui/egui_extras), substituindo o antigo par
//! Axum + `frontend/` TypeScript (ADR-0017). `launch()` é chamado por
//! `main.rs` quando nenhum subcomando de CLI é passado.

pub mod attack;
pub mod backend;
pub mod critical_path;
pub mod deliverables;
pub mod detection;
pub mod evidence;
pub mod export;
pub mod file_dialog;
pub mod findings;
pub mod gaps;
pub mod impact;
pub mod incidents;
pub mod iocs;
pub mod nist;
pub mod plan;
pub mod recommendations;
mod timeline;

#[cfg(test)]
mod id_regression_tests;

use crate::{config::AppConfig, error::AppError, infrastructure::storage::StoragePaths};
use backend::{Backend, Command, Event};
use std::collections::BTreeSet;

/// Vermelho legível para erros e severidade alta/crítica no tema escuro.
/// O tom anterior `(200, 60, 60)` tinha somente 3,42:1 no painel padrão.
pub(crate) const DANGER_TEXT_COLOR: egui::Color32 = egui::Color32::from_rgb(240, 110, 110);

pub fn launch(config: AppConfig, paths: StoragePaths) -> Result<(), AppError> {
    let backend = backend::spawn(config, paths)?;
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("DFIR — Análise Forense Local")
            .with_inner_size([1280.0, 820.0])
            .with_min_inner_size([960.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native(
        "DFIR — Análise Forense Local",
        options,
        Box::new(move |cc| {
            log_renderer(cc);
            Ok(Box::new(DfirApp::new(backend)))
        }),
    )
    .map_err(|error| AppError::Config(format!("failed to launch native UI: {error}")))
}

/// Registra qual adaptador de GPU o wgpu escolheu (GFX-01).
///
/// Sem esta linha, o log de inicialização mostra apenas os `ERROR` de
/// `wgpu_hal` de backends que falharam ao criar contexto (comum em GPUs
/// antigas: EGL recusa e o wgpu cai para Vulkan/llvmpipe), mas não diz qual
/// adaptador ficou em uso. A diferença importa: `llvmpipe` renderiza na CPU e
/// explica lentidão que, de outro modo, pareceria defeito do aplicativo.
///
/// O nome do adaptador é dado de hardware local, não conteúdo de evidência
/// (ADR-0022).
fn log_renderer(cc: &eframe::CreationContext<'_>) {
    let Some(state) = cc.wgpu_render_state.as_ref() else {
        tracing::warn!(
            "native UI is not using the wgpu renderer; the GPU adapter in use was not recorded"
        );
        return;
    };
    let info = state.adapter.get_info();
    tracing::info!(
        adapter = %info.name,
        backend = ?info.backend,
        device_type = ?info.device_type,
        driver = %info.driver,
        driver_info = %info.driver_info,
        candidates = state.available_adapters.len(),
        "native UI GPU adapter"
    );
}

/// Estado de uma consulta/ação assíncrona no backend. Toda tela de lista ou
/// formulário desta interface usa este mesmo pequeno estado de máquina —
/// nunca bloqueamos a thread da UI esperando o Tokio responder.
#[derive(Debug, Default)]
pub enum Loadable<T> {
    #[default]
    Idle,
    Loading,
    Ready(T),
    Failed(String),
}

impl<T> Loadable<T> {
    pub fn is_loading(&self) -> bool {
        matches!(self, Self::Loading)
    }

    pub fn ready(&self) -> Option<&T> {
        match self {
            Self::Ready(value) => Some(value),
            _ => None,
        }
    }

    /// Devolve `&mut T` pronto ou insere o valor inicial producedo por
    /// `init` (útil para disparar um comando no momento da insertion).
    pub fn get_or_insert_with(&mut self, init: impl FnOnce() -> Self) -> &mut Self {
        if matches!(self, Self::Idle) {
            *self = init();
        }
        self
    }

    /// Mostra um spinner/erro/nada conforme o estado, e entrega o valor
    /// pronto (se houver) para `render`.
    pub fn show(&self, ui: &mut egui::Ui, mut render: impl FnMut(&mut egui::Ui, &T)) {
        match self {
            Self::Idle => {}
            Self::Loading => {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Carregando…");
                });
            }
            Self::Failed(message) => {
                ui.colored_label(DANGER_TEXT_COLOR, format!("Erro: {message}"));
            }
            Self::Ready(value) => render(ui, value),
        }
    }
}

pub const CONFIDENCE_OPTIONS: [&str; 4] = ["LOW", "MEDIUM", "HIGH", "CONFIRMED"];
pub const IMPLEMENTATION_STATUS_OPTIONS: [&str; 5] = [
    "IMPLEMENTED",
    "PARTIALLY_IMPLEMENTED",
    "NOT_IMPLEMENTED",
    "NOT_ASSESSED",
    "NOT_APPLICABLE",
];
pub const TIER_OPTIONS: [&str; 4] = [
    "TIER_1_PARTIAL",
    "TIER_2_RISK_INFORMED",
    "TIER_3_REPEATABLE",
    "TIER_4_ADAPTIVE",
];
pub const ATTACK_TACTIC_OPTIONS: [&str; 14] = [
    "RECONNAISSANCE",
    "RESOURCE_DEVELOPMENT",
    "INITIAL_ACCESS",
    "EXECUTION",
    "PERSISTENCE",
    "PRIVILEGE_ESCALATION",
    "DEFENSE_EVASION",
    "CREDENTIAL_ACCESS",
    "DISCOVERY",
    "LATERAL_MOVEMENT",
    "COLLECTION",
    "COMMAND_AND_CONTROL",
    "EXFILTRATION",
    "IMPACT",
];
pub const IMPACT_LEVEL_OPTIONS: [&str; 5] = ["NONE", "LOW", "MEDIUM", "HIGH", "CRITICAL"];
pub const IOC_TYPE_OPTIONS: [&str; 9] = [
    "IP",
    "DOMAIN",
    "URL",
    "EMAIL",
    "SHA256",
    "SHA1",
    "MD5",
    "FILE_PATH",
    "HOSTNAME",
];
pub const RECOMMENDATION_CATEGORY_OPTIONS: [&str; 2] = ["IMMEDIATE", "STRATEGIC"];
/// Severidade de um achado: `NONE` fica de fora, pois uma condição sem
/// severidade não é um achado (F1-25).
pub const FINDING_SEVERITY_OPTIONS: [&str; 4] = ["LOW", "MEDIUM", "HIGH", "CRITICAL"];
pub const FINDING_KIND_OPTIONS: [&str; 5] = [
    "VULNERABILITY",
    "MISCONFIGURATION",
    "MISSING_CONTROL",
    "WEAK_CREDENTIAL",
    "OTHER",
];
pub const CAUSE_LEVEL_OPTIONS: [&str; 3] = ["ROOT_CAUSE", "PROBABLE_CAUSE", "CONTRIBUTING_FACTOR"];
/// Domínios do item de plano (F1-29): um por ação da demanda.
pub const PLAN_DOMAIN_OPTIONS: [&str; 8] = [
    "OS_HARDENING",
    "REDE_DISPOSITIVOS",
    "AUTENTICACAO_ACESSO",
    "MONITORAMENTO",
    "CRIPTO_REPOUSO",
    "CRIPTO_TRANSITO",
    "GESTAO_CHAVES",
    "POLITICA_IR",
];
pub const PLAN_PRIORITY_OPTIONS: [&str; 3] = ["P1", "P2", "P3"];
/// Motivos de uma lacuna de fonte (F1-31).
pub const GAP_REASON_OPTIONS: [&str; 4] = ["RETENTION", "NOT_PROVIDED", "NOT_COLLECTED", "OTHER"];
pub const PLAN_EFFORT_OPTIONS: [&str; 3] = ["S", "M", "L"];
pub const PLAN_COST_OPTIONS: [&str; 4] = ["NONE", "LOW", "MEDIUM", "HIGH"];
pub const PLAN_PHASE_OPTIONS: [&str; 3] = ["IMMEDIATE", "SHORT_TERM", "STRATEGIC"];

/// Um `egui::ComboBox` de seleção única sobre uma lista fixa de valores
/// válidos (os mesmos aceitos pela camada de persistência) — evita erros de
/// digitação que só seriam pegos depois, na validação do backend.
pub fn enum_combo(
    ui: &mut egui::Ui,
    id: &str,
    label: &str,
    current: &mut String,
    options: &[&str],
) {
    // `new` separa o salt persistente do label, que continua nomeando o
    // ComboBox para acessibilidade.
    egui::ComboBox::new(id, label)
        .selected_text(if current.is_empty() {
            "— selecione —".to_string()
        } else {
            current.clone()
        })
        .show_ui(ui, |ui| {
            for option in options {
                ui.selectable_value(current, (*option).to_string(), *option);
            }
        });
}

pub fn error_banner(ui: &mut egui::Ui, message: &str) {
    ui.colored_label(DANGER_TEXT_COLOR, format!("Erro: {message}"));
}

/// Campo de texto com `label` à esquerda, vinculando o widget ao label para
/// acessibilidade (`Response::labelled_by`). Sem o vínculo, a checagem de
/// acessibilidade do `egui_kittest` 0.37 acusa "widget lack accessible name".
pub fn field_text(ui: &mut egui::Ui, label: &str, text: &mut String) {
    ui.horizontal(|ui| {
        let label_response = ui.label(label);
        ui.text_edit_singleline(text).labelled_by(label_response.id);
    });
}

/// Variante multilinha de [`field_text`].
pub fn field_text_multiline(ui: &mut egui::Ui, label: &str, text: &mut String) {
    let label_response = ui.label(label);
    ui.text_edit_multiline(text).labelled_by(label_response.id);
}

/// Aviso neutro: informação ou ação que não completou por escolha do
/// operador (ex.: seletor de arquivo cancelado), nunca uma falha do
/// processamento. Não usar `error_banner` nesses casos.
pub fn hint_banner(ui: &mut egui::Ui, message: &str) {
    ui.colored_label(egui::Color32::from_rgb(170, 130, 30), message);
}

/// Texto de formulário já trimado: vazio vira `None` para que a requisição
/// envie ausência em vez de string em branco.
pub(crate) fn non_empty(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Campo textual de um payload JSON; ausência vira string vazia na exibição.
pub(crate) fn str_field<'a>(payload: &'a serde_json::Value, key: &str) -> &'a str {
    payload.get(key).and_then(|v| v.as_str()).unwrap_or("")
}

/// Lista textual de um payload JSON; ausência ou tipo inesperado vira lista
/// vazia (a tela mostra "nada", nunca inventa conteúdo).
pub(crate) fn string_list(payload: &serde_json::Value, key: &str) -> Vec<String> {
    payload
        .get(key)
        .and_then(|v| v.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

/// Liga ou desliga um id em um conjunto de seleção múltipla.
pub(crate) fn toggle(set: &mut BTreeSet<String>, id: &str, on: bool) {
    if on {
        set.insert(id.to_string());
    } else {
        set.remove(id);
    }
}

/// Separa por vírgula, ponto e vírgula ou espaço e normaliza para
/// maiúsculas (os identificadores NIST são `PR.AA-03`, não `pr.aa-03`).
pub(crate) fn split_ids(raw: &str) -> Vec<String> {
    raw.split(|c: char| c == ',' || c == ';' || c.is_whitespace())
        .map(str::trim)
        .filter(|id| !id.is_empty())
        .map(str::to_uppercase)
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Screen {
    Evidence,
    Incidents,
}

pub struct DfirApp {
    backend: Backend,
    screen: Screen,
    evidence: evidence::EvidenceState,
    incidents: incidents::IncidentsState,
}

impl DfirApp {
    fn new(backend: Backend) -> Self {
        backend.send(Command::ListEvidence);
        backend.send(Command::ListIncidents);
        Self {
            backend,
            screen: Screen::Evidence,
            evidence: evidence::EvidenceState::default(),
            incidents: incidents::IncidentsState::default(),
        }
    }

    fn navigation(&mut self, ui: &mut egui::Ui) {
        egui::Panel::left("dfir_nav")
            .resizable(false)
            .min_size(180.0)
            .show(ui, |ui| {
                ui.add_space(8.0);
                ui.heading("DFIR");
                ui.label(egui::RichText::new("Local · offline · operador único").weak());
                ui.separator();
                if ui
                    .selectable_label(self.screen == Screen::Evidence, "Evidência")
                    .clicked()
                {
                    self.screen = Screen::Evidence;
                }
                if ui
                    .selectable_label(self.screen == Screen::Incidents, "Incidentes")
                    .clicked()
                {
                    self.screen = Screen::Incidents;
                }
            });
    }

    fn apply_event(&mut self, event: Event) {
        match event {
            Event::EvidenceListed(result) => self.evidence.on_listed(result),
            Event::EvidenceImported(result) => self.evidence.on_imported(result),
            Event::EvidenceDestroyed {
                evidence_id,
                result,
            } => {
                self.evidence.on_destroyed(&evidence_id, result);
            }
            Event::AnalyzeCompleted {
                evidence_id,
                result,
            } => {
                self.evidence
                    .on_analyze_completed(&evidence_id, result, &self.backend);
            }
            Event::TimelinePageLoaded {
                evidence_id,
                filter,
                offset,
                result,
            } => {
                self.evidence
                    .on_timeline_page(&evidence_id, &filter, offset, result);
            }
            Event::EvidenceAnalyticalResultsLoaded {
                evidence_id,
                result,
            } => {
                self.evidence.on_analytical_results(&evidence_id, result);
            }
            Event::EvidenceLinkedToIncident {
                evidence_id,
                result,
            } => {
                self.evidence.on_linked_to_incident(&evidence_id, result);
            }
            Event::CustodyLoaded {
                evidence_id,
                result,
            } => {
                self.evidence.on_custody_loaded(&evidence_id, result);
            }
            Event::AuditLoaded {
                incident_id,
                result,
            } => {
                self.incidents.on_audit_loaded(&incident_id, result);
            }
            Event::IncidentEvidenceLoaded {
                incident_id,
                result,
            } => {
                self.incidents
                    .on_incident_evidence_loaded(&incident_id, result);
            }
            Event::DetectionRun {
                evidence_id,
                result,
            } => {
                self.evidence
                    .detection
                    .on_run(&evidence_id, result, &self.backend);
            }
            Event::CandidatesLoaded {
                evidence_id,
                result,
            } => {
                self.evidence.detection.on_loaded(&evidence_id, result);
            }
            Event::CandidateDecided {
                evidence_id,
                result,
            } => {
                // Confirmar cria achado, mapeamento e IOCs: recarrega o que o
                // incidente mostra. Rejeitar só muda o candidato.
                self.evidence
                    .detection
                    .on_decided(&evidence_id, result, &self.backend);
            }

            Event::IncidentCreated(result) => self.incidents.on_created(result),
            Event::IncidentsListed(result) => self.incidents.on_listed(result),

            Event::NistProfileHistoryLoaded {
                incident_id,
                result,
            } => {
                self.incidents.nist.on_history(&incident_id, result);
            }
            Event::NistProfileCreated {
                incident_id,
                result,
            } => {
                self.incidents
                    .nist
                    .on_created(&incident_id, result, &self.backend);
            }
            Event::GapAnalysisRun {
                incident_id,
                result,
            } => {
                self.incidents
                    .nist
                    .on_gap_run(&incident_id, result, &self.backend);
            }
            Event::GapAnalysisLoaded {
                incident_id,
                result,
            } => {
                self.incidents.nist.on_gap_loaded(&incident_id, result);
            }
            Event::NistFunctionViewLoaded {
                incident_id,
                result,
            } => {
                self.incidents
                    .nist
                    .on_functions_loaded(&incident_id, result);
            }

            Event::FindingCreated {
                incident_id,
                result,
            } => {
                self.incidents
                    .findings
                    .on_finding_created(&incident_id, result, &self.backend);
            }
            Event::RootCauseCreated {
                incident_id,
                result,
            } => {
                self.incidents
                    .findings
                    .on_cause_created(&incident_id, result, &self.backend);
            }
            Event::FindingsViewLoaded {
                incident_id,
                result,
            } => {
                self.incidents.findings.on_loaded(&incident_id, result);
            }

            Event::SourceGapCreated {
                incident_id,
                result,
            } => {
                self.incidents
                    .gaps
                    .on_created(&incident_id, result, &self.backend);
            }
            Event::SourceGapsLoaded {
                incident_id,
                result,
            } => {
                self.incidents.gaps.on_loaded(&incident_id, result);
            }
            Event::DeliverablesPreviewed {
                incident_id,
                result,
            } => {
                self.incidents
                    .deliverables
                    .on_previewed(&incident_id, result);
            }
            Event::DeliverablesGenerated {
                incident_id,
                result,
            } => {
                self.incidents
                    .deliverables
                    .on_generated(&incident_id, result);
            }
            Event::PlanItemCreated {
                incident_id,
                result,
            } => {
                self.incidents
                    .plan
                    .on_created(&incident_id, result, &self.backend);
            }
            Event::PlanViewLoaded {
                incident_id,
                result,
            } => {
                self.incidents.plan.on_loaded(&incident_id, result);
            }

            Event::AttackMappingCreated {
                incident_id,
                result,
            } => {
                self.incidents
                    .attack
                    .on_created(&incident_id, result, &self.backend);
            }
            Event::AttackMappingsLoaded {
                incident_id,
                result,
            } => {
                self.incidents.attack.on_loaded(&incident_id, result);
            }

            Event::CriticalPathStageCreated {
                incident_id,
                result,
            } => {
                self.incidents
                    .critical_path
                    .on_stage_created(&incident_id, result, &self.backend);
            }
            Event::CriticalPathStagesLinked {
                incident_id,
                result,
            } => {
                self.incidents
                    .critical_path
                    .on_linked(&incident_id, result, &self.backend);
            }
            Event::CriticalPathLoaded {
                incident_id,
                result,
            } => {
                self.incidents.critical_path.on_loaded(&incident_id, result);
            }

            Event::ImpactAssessmentCreated {
                incident_id,
                result,
            } => {
                self.incidents
                    .impact
                    .on_created(&incident_id, result, &self.backend);
            }
            Event::ImpactAssessmentLoaded {
                incident_id,
                result,
            } => {
                self.incidents.impact.on_loaded(&incident_id, result);
            }

            Event::IocCreated {
                incident_id,
                result,
            } => {
                self.incidents
                    .iocs
                    .on_created(&incident_id, result, &self.backend);
            }
            Event::IocsLoaded {
                incident_id,
                result,
            } => {
                self.incidents.iocs.on_loaded(&incident_id, result);
            }

            Event::RecommendationCreated {
                incident_id,
                result,
            } => {
                self.incidents
                    .recommendations
                    .on_created(&incident_id, result, &self.backend);
            }
            Event::RecommendationsLoaded {
                incident_id,
                result,
            } => {
                self.incidents
                    .recommendations
                    .on_loaded(&incident_id, result);
            }

            Event::FindingsExported {
                incident_id,
                result,
            } => {
                self.incidents.export.on_exported(&incident_id, result);
            }
        }
    }
}

impl eframe::App for DfirApp {
    // eframe 0.34 trocou o método obrigatório de `update(&Context, &mut
    // Frame)` para `ui(&mut Ui, &mut Frame)` (`update` virou um método
    // provido/depreciado, com `ui` como o único obrigatório) — ver
    // CHANGELOG do eframe, "Replace App::update with fn logic and fn ui".
    // `ui` já chega com uma área raiz própria; os painéis aninhados usam
    // `Panel::…show(ui, …)` / `CentralPanel::show(ui, …)` (egui 0.36: `SidePanel`
    // foi unificado em `Panel` e `show_inside` virou `show`), mesmo padrão já
    // usado por `gui/incidents.rs` para os painéis internos daquela tela.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        for event in self.backend.drain_events() {
            self.apply_event(event);
        }

        // GUI-01: arrastar-e-soltar é o caminho de importação que não depende
        // de `xdg-desktop-portal`/`zenity`. O evento chega ao contexto, então
        // é coletado aqui e encaminhado ao painel de evidência.
        let dropped: Vec<std::path::PathBuf> = ui.ctx().input(|input| {
            input
                .raw
                .dropped_files
                .iter()
                .map(|file| file.path().to_path_buf())
                .collect()
        });
        if !dropped.is_empty() {
            self.screen = Screen::Evidence;
            self.evidence.on_files_dropped(&dropped, &self.backend);
        }

        self.navigation(ui);

        egui::CentralPanel::default().show(ui, |ui| {
            let screen = self.screen;
            match screen {
                Screen::Evidence => self.evidence.ui(ui, &self.backend, self.incidents.items()),
                Screen::Incidents => self.incidents.ui(ui, &self.backend),
            }
        });

        // Ações em voo (análise, importação) terminam na thread de backend;
        // sem isto a tela só seria redesenhada em resposta a input do SO.
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(150));
    }
}

/// Harness de teste compartilhado pelos módulos de UI.
///
/// egui 0.37 passou a usar [`egui::MissingGlyphPolicy::Panic`] como padrão no
/// `egui_kittest` (mudança de comportamento na 0.37; na 0.36 o harness não
/// panicava). A fonte padrão embutida só cobre latim/cirílico e alguns emojis,
/// então glifos como `▸` (usado em `plan.rs`) quebravam os testes. Aqui
/// escolhemos [`egui::MissingGlyphPolicy::Tofu`], que é o comportamento de
/// produção quando nenhuma fonte do sistema cobre o caractere. Isto não enfraquece
/// os testes: eles continuam com asserts sobre texto e roles, e um glifo ausente
/// vira tofu em vez de panic.
#[cfg(test)]
pub(crate) mod test_harness {
    use egui_kittest::HarnessBuilder;

    /// Builder padrão dos testes de UI: tofu em vez de panic para glifos
    /// fora do latim/cirílico (egui 0.37 mudou o default do kittest para Panic).
    pub(crate) fn builder<State>() -> HarnessBuilder<State> {
        HarnessBuilder::default().allow_missing_glyphs()
    }

    /// Drena os comandos enviados à thread de backend durante um teste de UI.
    pub(crate) fn drain(
        commands: &mut tokio::sync::mpsc::UnboundedReceiver<super::Command>,
    ) -> Vec<super::Command> {
        let mut out = Vec::new();
        while let Ok(command) = commands.try_recv() {
            out.push(command);
        }
        out
    }
}

#[cfg(test)]
mod presentation_tests {
    use super::DANGER_TEXT_COLOR;

    fn luminance(color: egui::Color32) -> f32 {
        fn channel(value: u8) -> f32 {
            let value = f32::from(value) / 255.0;
            if value <= 0.04045 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            }
        }
        0.2126 * channel(color.r()) + 0.7152 * channel(color.g()) + 0.0722 * channel(color.b())
    }

    #[test]
    fn danger_text_meets_normal_text_contrast_on_the_dark_panel() {
        let foreground = luminance(DANGER_TEXT_COLOR);
        let background = luminance(egui::Visuals::dark().window_fill);
        let contrast = (foreground.max(background) + 0.05) / (foreground.min(background) + 0.05);
        assert!(
            contrast >= 4.5,
            "contraste insuficiente para texto normal: {contrast:.2}:1"
        );
    }
}

#[cfg(test)]
mod navigation_tests {
    use super::{Backend, DfirApp};
    use egui_kittest::kittest::Queryable;

    #[test]
    fn navigation_does_not_depend_on_missing_emoji_fonts() {
        let (backend, _commands) = Backend::for_test();
        let app = DfirApp::new(backend);
        // Mantém o default estrito do kittest: um glifo ausente no caminho
        // real da navegação faz o teste falhar.
        let mut harness = egui_kittest::HarnessBuilder::default()
            .with_size(egui::vec2(240.0, 320.0))
            .build_ui_state(|ui, app: &mut DfirApp| app.navigation(ui), app);
        harness.run();
        harness.get_by_label("Evidência");
        harness.get_by_label("Incidentes");
    }
}

#[cfg(test)]
mod state_tests {
    use super::{Loadable, enum_combo};
    use egui_kittest::kittest::Queryable;

    #[test]
    fn loadable_accessors_and_lazy_initialization_cover_every_state() {
        let mut idle = Loadable::<u32>::Idle;
        assert!(!idle.is_loading());
        assert_eq!(idle.ready(), None);

        let mut initializations = 0;
        let inserted = idle.get_or_insert_with(|| {
            initializations += 1;
            Loadable::Loading
        });
        assert!(inserted.is_loading());
        assert_eq!(initializations, 1);

        idle.get_or_insert_with(|| {
            initializations += 1;
            Loadable::Ready(42)
        });
        assert_eq!(initializations, 1, "somente Idle dispara a carga inicial");

        let ready = Loadable::Ready(42);
        assert!(!ready.is_loading());
        assert_eq!(ready.ready(), Some(&42));
        assert_eq!(Loadable::<u32>::Failed("falha".into()).ready(), None);
    }

    #[test]
    fn loadable_show_renders_loading_failure_and_ready_content() {
        let mut loading = crate::gui::test_harness::builder().build_ui(|ui| {
            Loadable::<u32>::Loading.show(ui, |_, _| unreachable!());
        });
        // O spinner pede repaint contínuo; um passo basta para inspecioná-lo.
        loading.step();
        loading.get_by_label("Carregando…");

        let mut failed = crate::gui::test_harness::builder().build_ui(|ui| {
            Loadable::<u32>::Failed("banco indisponível".into()).show(ui, |_, _| unreachable!());
        });
        failed.run();
        failed.get_by_label("Erro: banco indisponível");

        let mut ready = crate::gui::test_harness::builder().build_ui(|ui| {
            Loadable::Ready(42).show(ui, |ui, value| {
                ui.label(format!("valor pronto: {value}"));
            });
        });
        ready.run();
        ready.get_by_label("valor pronto: 42");

        let mut idle = crate::gui::test_harness::builder().build_ui(|ui| {
            Loadable::<u32>::Idle.show(ui, |_, _| unreachable!());
            ui.label("conteúdo externo");
        });
        idle.run();
        idle.get_by_label("conteúdo externo");
        assert!(idle.query_by_label("Carregando…").is_none());
    }

    #[test]
    fn enum_combo_exposes_the_placeholder_and_applies_the_selected_option() {
        let mut harness = crate::gui::test_harness::builder::<String>().build_ui_state(
            |ui, current| {
                enum_combo(ui, "confidence", "Confiança", current, &["LOW", "HIGH"]);
            },
            String::new(),
        );
        harness.run();
        assert_eq!(
            harness
                .get_by_role(egui::accesskit::Role::ComboBox)
                .value()
                .as_deref(),
            Some("— selecione —")
        );
        harness.get_by_label("Confiança").click();
        harness.run();
        harness.get_by_label("HIGH").click();
        harness.run();

        assert_eq!(harness.state(), "HIGH");
        assert_eq!(
            harness
                .get_by_role(egui::accesskit::Role::ComboBox)
                .value()
                .as_deref(),
            Some("HIGH")
        );
    }
}
