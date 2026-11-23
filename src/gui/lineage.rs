//! Tela de linhagem analítica (somente leitura, inspecionável).
//!
//! Fatia 4 do plano ADR-0030: dado um resultado analítico selecionado, mostra
//! o resultado em si, o histórico completo da mesma `correlation_key` (do
//! mais antigo ao vigente) e as relações de saída declaradas por ele.
//!
//! Sem mutation, sem grafo global, sem novo protocolo. Consome
//! [`crate::service::lineage::LineageView`] por `Command::LineageForResult`.

use super::backend::{Backend, Command};
use super::{Loadable, error_banner};
use crate::service::lineage::{LineageScope, LineageView, OutgoingRelation};

/// Estado da tela de linhagem.
#[derive(Default)]
pub struct LineageState {
    loaded_for: Option<(String, LineageScope)>,
    view: Loadable<LineageView>,
}

impl LineageState {
    /// Marca o resultado como carregando e dispara a consulta no backend.
    pub fn ensure_loaded(&mut self, result_id: &str, scope: LineageScope, backend: &Backend) {
        let key = (result_id.to_string(), scope);
        let retry = matches!(&self.view, Loadable::Failed(_));
        if self.loaded_for.as_ref() != Some(&key) || retry {
            self.loaded_for = Some(key.clone());
            self.view = Loadable::Loading;
            backend.send(Command::LineageForResult {
                result_id: key.0,
                scope: key.1,
            });
        }
    }

    /// Remove o contexto ao trocar de evidência ou eliminar a selecionada.
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// Injeta uma view já resolvida nos testes de apresentação.
    #[cfg(test)]
    pub fn set_loaded(&mut self, view: LineageView) {
        self.view = Loadable::Ready(view);
    }

    /// Aplica o evento de retorno do backend, descarregando se o escopo não
    /// mais corresponder à requisição em voo.
    pub fn on_loaded(
        &mut self,
        result_id: &str,
        scope: &LineageScope,
        result: Result<LineageView, String>,
    ) {
        if self.loaded_for.as_ref() != Some(&(result_id.to_string(), scope.clone())) {
            return;
        }
        self.view = match result {
            Ok(view) => Loadable::Ready(view),
            Err(message) => Loadable::Failed(message),
        };
    }
}

impl LineageState {
    /// Desenha a tela de linhagem para o resultado atualmente carregado.
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        match &self.view {
            Loadable::Idle => {}
            Loadable::Loading => {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Carregando linhagem…");
                });
            }
            Loadable::Failed(message) => {
                error_banner(ui, message);
            }
            Loadable::Ready(view) => {
                render_view(ui, view);
            }
        }
    }
}

/// Renderiza a view completa da linhagem.
fn render_view(ui: &mut egui::Ui, view: &LineageView) {
    ui.heading("Linhagem do resultado analítico");
    ui.separator();

    // Resultado selecionado.
    render_selected(ui, &view.selected);

    ui.separator();

    // Histórico da correlation_key.
    render_history(ui, &view.history, view.current.as_ref());

    ui.separator();

    // Relações de saída.
    render_outgoing(ui, &view.outgoing);
}

/// Renderiza o resultado selecionado (alvo da consulta), com campos-chave
/// visíveis e sem despejo de JSON bruto.
fn render_selected(ui: &mut egui::Ui, item: &crate::service::AnalyticalResultResponse) {
    ui.group(|ui| {
        ui.set_width(ui.available_width());
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
        ui.heading("Resultado selecionado");
        ui.label(format!("ID: {}", item.id));
        ui.label(format!("Tipo: {}", item.result_type));
        ui.label(format!("Correlation key: {}", item.correlation_key));
        if let Some(evidence_id) = &item.evidence_id {
            ui.label(format!("Evidência: {evidence_id}"));
        }
        if let Some(incident_id) = &item.incident_id {
            ui.label(format!("Incidente: {incident_id}"));
        }
        if let Some(confidence) = &item.confidence {
            ui.label(format!("Confiança: {confidence}"));
        }
        if let Some(version) = &item.rule_or_model_version {
            ui.label(format!("Versão da regra/modelo: {version}"));
        }
        if let Some(job) = &item.analysis_job_id {
            ui.label(format!("Job de análise: {job}"));
        }
        ui.label(format!("Criado em: {}", item.created_at));

        ui.separator();
        ui.heading("Payload");
        render_payload_fields(ui, &item.payload);

        ui.separator();
        ui.heading("Provenance");
        render_payload_fields(ui, &item.provenance);
    });
}

/// Renderiza campos de um objeto JSON como pares chave/valor (sem mostrar o
/// JSON cru).
fn render_payload_fields(ui: &mut egui::Ui, value: &serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            if map.is_empty() {
                ui.label("(sem campos)");
                return;
            }
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            for key in keys {
                ui.horizontal(|ui| {
                    ui.label(key);
                    ui.label(map[key].to_string());
                });
            }
        }
        other => {
            ui.label(other.to_string());
        }
    }
}

/// Renderiza o histórico completo da correlation_key, do mais antigo ao
/// vigente, sinalizando o vigente.
fn render_history(
    ui: &mut egui::Ui,
    history: &[crate::service::AnalyticalResultResponse],
    current: Option<&crate::service::AnalyticalResultResponse>,
) {
    ui.group(|ui| {
        ui.set_width(ui.available_width());
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
        ui.heading("Histórico da correlation_key");
        ui.label(format!("{} versão(ões)", history.len()));
        if history.is_empty() {
            ui.label("Nenhuma versão encontrada.");
            return;
        }
        for item in history {
            let is_current = current.is_some_and(|c| c.id == item.id);
            let label = if is_current { "vigente" } else { "histórica" };
            ui.collapsing(
                format!("{} · {} · {}", item.id, item.result_type, label),
                |ui| {
                    ui.label(format!("Correlation key: {}", item.correlation_key));
                    if let Some(confidence) = &item.confidence {
                        ui.label(format!("Confiança: {confidence}"));
                    }
                    if let Some(version) = &item.rule_or_model_version {
                        ui.label(format!("Versão da regra/modelo: {version}"));
                    }
                    if let Some(supersedes) = &item.supersedes {
                        ui.label(format!("Substitui: {supersedes}"));
                    }
                    ui.label(format!("Criado em: {}", item.created_at));
                    if is_current {
                        ui.colored_label(
                            egui::Color32::from_rgb(80, 160, 80),
                            "Esta é a versão vigente.",
                        );
                    }
                },
            );
        }
    });
}

/// Renderiza as relações de saída declaradas pelo resultado selecionado.
fn render_outgoing(ui: &mut egui::Ui, outgoing: &[OutgoingRelation]) {
    ui.group(|ui| {
        ui.set_width(ui.available_width());
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
        ui.heading("Relações de saída");
        if outgoing.is_empty() {
            ui.label("Nenhuma relação de saída declarada.");
            return;
        }
        for relation in outgoing {
            ui.collapsing(
                format!("{} → {}", relation.kind.as_str(), relation.result.id),
                |ui| {
                    ui.label(format!("Tipo: {}", relation.kind.as_str()));
                    ui.label(format!("Alvo: {}", relation.result.id));
                    ui.label(format!("Tipo do alvo: {}", relation.result.result_type));
                    ui.label(format!(
                        "Correlation key do alvo: {}",
                        relation.result.correlation_key
                    ));
                    if let Some(confidence) = &relation.result.confidence {
                        ui.label(format!("Confiança: {confidence}"));
                    }
                    if let Some(version) = &relation.result.rule_or_model_version {
                        ui.label(format!("Versão da regra/modelo: {version}"));
                    }
                },
            );
        }
    });
}

#[cfg(test)]
mod ui_tests {
    use super::*;
    use crate::service::AnalyticalResultResponse;
    use crate::service::lineage::{LineageScope, LineageView, OutgoingRelation};
    use egui_kittest::{Harness, kittest::Queryable};

    type State = (LineageState, Backend);

    fn response(
        id: &str,
        correlation_key: &str,
        payload: serde_json::Value,
    ) -> AnalyticalResultResponse {
        AnalyticalResultResponse {
            id: id.into(),
            result_type: "YARA_MATCH".into(),
            correlation_key: correlation_key.into(),
            evidence_id: Some("ev-1".into()),
            incident_id: Some("inc-1".into()),
            analysis_job_id: None,
            rule_or_model_id: Some("rule-1".into()),
            rule_or_model_version: Some("1.0".into()),
            confidence: Some("HIGH".into()),
            payload,
            provenance: serde_json::json!({"producer": "dfir-app"}),
            supersedes: None,
            created_at: "2026-10-01T00:00:00Z".into(),
        }
    }

    fn view_with_history() -> LineageView {
        let root = response(
            "root-1",
            "corr-1",
            serde_json::json!({"rule": "rule-1", "severity": "high"}),
        );
        let current = response(
            "current-1",
            "corr-1",
            serde_json::json!({"rule": "rule-1", "severity": "critical"}),
        );
        LineageView {
            selected: current.clone(),
            history: vec![root, current.clone()],
            current: Some(current),
            outgoing: vec![OutgoingRelation {
                kind: crate::persistence::analytical_results::RelationType::DerivedFrom,
                result: response("root-1", "corr-1", serde_json::json!({})),
            }],
        }
    }

    fn harness_for(state: LineageState, backend: Backend) -> Harness<'static, State> {
        crate::gui::test_harness::builder()
            .with_size(egui::vec2(900.0, 700.0))
            .build_ui_state(
                |ui, (state, _backend): &mut State| state.ui(ui),
                (state, backend),
            )
    }

    #[test]
    fn lineage_shows_selected_history_and_outgoing_without_raw_json() {
        let (backend, _commands) = Backend::for_test();
        let mut state = LineageState::default();
        state.set_loaded(view_with_history());
        let mut harness = harness_for(state, backend);
        harness.step();

        // Resultado selecionado.
        harness.get_by_label("Resultado selecionado");
        harness.get_by_label("Tipo: YARA_MATCH");
        harness.get_by_label("Confiança: HIGH");
        harness.get_by_label("Versão da regra/modelo: 1.0");
        // Campos do payload estruturados, não JSON cru.
        harness.get_by_label("rule");
        harness.get_by_label("\"critical\"");
        assert!(
            harness.query_by_label("{\"rule\":\"rule-1\"").is_none(),
            "não deve mostrar JSON cru do payload"
        );

        // Abre o grupo e a versão vigente com cliques reais.
        harness.get_by_label("Histórico da correlation_key").click();
        harness.step();
        harness.get_by_label("2 versão(ões)");
        harness
            .get_by_label("current-1 · YARA_MATCH · vigente")
            .click();
        harness.step();
        harness.get_by_label("Esta é a versão vigente.");

        // Abre o grupo e a relação antes de afirmar o conteúdo interno.
        harness.get_by_label("Relações de saída").click();
        harness.step();
        harness.get_by_label("DERIVED_FROM → root-1").click();
        harness.step();
        harness.get_by_label("Alvo: root-1");
    }

    #[test]
    fn lineage_error_shows_diagnostic_and_not_null() {
        let (backend, _commands) = Backend::for_test();
        let mut state = LineageState::default();
        // ensure_loaded define loaded_for; sem ele, on_loaded retorna cedo
        // porque loaded_for está vazio.
        state.ensure_loaded("result-1", LineageScope::Evidence("ev-1".into()), &backend);
        state.on_loaded(
            "result-1",
            &LineageScope::Evidence("ev-1".into()),
            Err("analytical result not found: result-1".into()),
        );
        let mut harness = harness_for(state, backend);
        // step() processa o spinner/estado de loading antes de afirmar o erro.
        harness.step();

        harness.get_by_label("Erro: analytical result not found: result-1");
        // Não deve mostrar spinner nem vazio silencioso.
        assert!(
            harness.query_by_label("Carregando linhagem…").is_none(),
            "não deve ficar em loading após erro"
        );
    }

    #[test]
    fn failed_lineage_can_retry_the_same_result() {
        let (backend, mut commands) = Backend::for_test();
        let mut state = LineageState::default();
        let scope = LineageScope::Evidence("ev-1".into());
        state.ensure_loaded("result-1", scope.clone(), &backend);
        assert_eq!(crate::gui::test_harness::drain(&mut commands).len(), 1);
        state.on_loaded("result-1", &scope, Err("falha temporária".into()));
        state.ensure_loaded("result-1", scope, &backend);
        let sent = crate::gui::test_harness::drain(&mut commands);
        assert_eq!(sent.len(), 1, "a falha permite nova tentativa: {sent:?}");
        assert!(matches!(
            &sent[0],
            Command::LineageForResult { result_id, .. } if result_id == "result-1"
        ));
    }
}
