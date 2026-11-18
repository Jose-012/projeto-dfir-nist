//! Aba "Lacunas": o que **faltou** na investigação — fonte esperada, período
//! sem dados e motivo (F1-31, RB-001). Entra no Entregável 1.
//!
//! A interface só coleta e mostra; as regras vivem em
//! `persistence/source_gaps.rs` e chegam aqui como mensagem de erro.

use super::backend::{Backend, Command};
use super::{GAP_REASON_OPTIONS, Loadable, enum_combo, error_banner, non_empty, str_field};
use crate::persistence::source_gaps::GapReason;
use crate::service::AnalyticalResultResponse;
use crate::service::gaps::SourceGapInput;

#[derive(Default)]
struct GapForm {
    expected_source: String,
    period_from: String,
    period_to: String,
    reason: String,
    note: String,
}

#[derive(Default)]
pub struct GapsState {
    loaded_for: Option<String>,
    list: Loadable<Vec<AnalyticalResultResponse>>,
    form: GapForm,
    creating: bool,
    error: Option<String>,
}

impl GapsState {
    fn ensure_loaded(&mut self, incident_id: &str, backend: &Backend) {
        if self.loaded_for.as_deref() == Some(incident_id) {
            return;
        }
        self.loaded_for = Some(incident_id.to_string());
        self.list = Loadable::Loading;
        backend.send(Command::ListSourceGaps(incident_id.to_string()));
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
                self.error = None;
                self.form = GapForm::default();
                backend.send(Command::ListSourceGaps(incident_id.to_string()));
            }
            Err(message) => self.error = Some(message),
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, incident_id: &str, backend: &Backend) {
        self.ensure_loaded(incident_id, backend);

        ui.heading("Lacunas de fonte");
        ui.label(
            "Registre o que se esperava ter e não se teve: logs de uma fonte, de um período, por retenção curta, por não terem sido fornecidos. O relatório diz o que não pôde afirmar.",
        );
        self.list.show(ui, |ui, items| {
            if items.is_empty() {
                ui.label("Nenhuma lacuna registrada.");
                return;
            }
            for gap in items {
                let payload = &gap.payload;
                ui.group(|ui| {
                    ui.strong(str_field(payload, "expected_source"));
                    let from = str_field(payload, "period_from");
                    let to = str_field(payload, "period_to");
                    let period = match (from.is_empty(), to.is_empty()) {
                        (true, true) => "período não informado".to_string(),
                        (false, true) => format!("a partir de {from}"),
                        (true, false) => format!("até {to}"),
                        (false, false) => format!("{from} a {to}"),
                    };
                    let reason: &str = match GapReason::from_str(str_field(payload, "reason")) {
                        Some(known) => known.label_pt(),
                        None => str_field(payload, "reason"),
                    };
                    ui.label(format!("{period} · motivo: {reason}"));
                    if !str_field(payload, "note").is_empty() {
                        ui.label(format!("Nota: {}", str_field(payload, "note")));
                    }
                    ui.small(format!("id: {}", gap.id));
                });
            }
        });
        ui.separator();

        let Self {
            form,
            creating,
            error,
            ..
        } = self;
        ui.collapsing("Nova lacuna", |ui| {
            super::field_text(
                ui,
                "Fonte esperada (ex.: logs do firewall de borda)",
                &mut form.expected_source,
            );
            super::field_text(
                ui,
                "Período de (AAAA-MM-DD, opcional)",
                &mut form.period_from,
            );
            super::field_text(
                ui,
                "Período até (AAAA-MM-DD, opcional)",
                &mut form.period_to,
            );
            enum_combo(
                ui,
                "gap_reason",
                "Motivo",
                &mut form.reason,
                &GAP_REASON_OPTIONS,
            );
            super::field_text(
                ui,
                "Nota (obrigatória se o motivo for OTHER)",
                &mut form.note,
            );
            if let Some(message) = error.as_deref() {
                error_banner(ui, message);
            }
            if ui
                .add_enabled(!*creating, egui::Button::new("Registrar lacuna"))
                .clicked()
            {
                *creating = true;
                backend.send(Command::CreateSourceGap {
                    incident_id: incident_id.to_string(),
                    input: SourceGapInput {
                        expected_source: form.expected_source.clone(),
                        period_from: non_empty(&form.period_from),
                        period_to: non_empty(&form.period_to),
                        reason: form.reason.clone(),
                        note: form.note.clone(),
                        supersedes: None,
                    },
                });
            }
            if *creating {
                ui.spinner();
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::source_gaps::GapReason;

    /// A lista da interface não pode divergir do vocabulário do modelo.
    #[test]
    fn option_list_matches_the_model_vocabulary() {
        let model: Vec<&str> = [
            GapReason::Retention,
            GapReason::NotProvided,
            GapReason::NotCollected,
            GapReason::Other,
        ]
        .into_iter()
        .map(GapReason::as_str)
        .collect();
        assert_eq!(model, GAP_REASON_OPTIONS);
    }
}

/// Testes da aba com `egui_kittest` (ADR-0022).
#[cfg(test)]
mod ui_tests {
    use super::GapsState;
    use crate::gui::Loadable;
    use crate::gui::backend::{Backend, Command};
    use crate::gui::test_harness::drain;
    use crate::service::AnalyticalResultResponse;
    use egui_kittest::{Harness, kittest::Queryable};

    type State = (GapsState, Backend);

    fn gap(id: &str, payload: serde_json::Value) -> AnalyticalResultResponse {
        AnalyticalResultResponse {
            id: id.into(),
            result_type: "SOURCE_GAP".into(),
            correlation_key: format!("key-{id}"),
            evidence_id: None,
            incident_id: Some("inc-1".into()),
            analysis_job_id: None,
            rule_or_model_id: None,
            rule_or_model_version: None,
            confidence: None,
            payload,
            provenance: serde_json::Value::Null,
            supersedes: None,
            created_at: "2026-10-16T00:00:00Z".into(),
        }
    }

    fn harness_for(state: GapsState, backend: Backend) -> Harness<'static, State> {
        crate::gui::test_harness::builder()
            .with_size(egui::vec2(1000.0, 1600.0))
            .build_ui_state(
                |ui, (state, backend): &mut State| state.ui(ui, "inc-1", backend),
                (state, backend),
            )
    }

    #[test]
    fn opening_the_tab_requests_the_gaps_once() {
        let (backend, mut commands) = Backend::for_test();
        let mut harness = harness_for(GapsState::default(), backend);
        harness.step();
        harness.step();
        let sent = drain(&mut commands);
        assert_eq!(sent.len(), 1, "{sent:?}");
        assert!(matches!(&sent[0], Command::ListSourceGaps(id) if id == "inc-1"));
    }

    #[test]
    fn loaded_gaps_show_the_source_period_reason_and_note() {
        let (backend, _commands) = Backend::for_test();
        let state = GapsState {
            loaded_for: Some("inc-1".into()),
            list: Loadable::Ready(vec![
                gap(
                    "g-1",
                    serde_json::json!({"expected_source": "logs do firewall de borda",
                        "period_from": "2026-09-01", "period_to": "2026-09-15",
                        "reason": "RETENTION", "note": "retenção de 7 dias"}),
                ),
                gap(
                    "g-2",
                    serde_json::json!({"expected_source": "logs de e-mail",
                        "period_from": null, "period_to": null,
                        "reason": "NOT_PROVIDED", "note": ""}),
                ),
            ]),
            ..GapsState::default()
        };
        let mut harness = harness_for(state, backend);
        harness.run();
        harness.get_by_label("logs do firewall de borda");
        harness.get_by_label("2026-09-01 a 2026-09-15 · motivo: retenção insuficiente de logs");
        harness.get_by_label("Nota: retenção de 7 dias");
        harness.get_by_label("logs de e-mail");
        harness.get_by_label("período não informado · motivo: não fornecido");
    }

    #[test]
    fn registering_a_gap_sends_the_typed_values_and_a_failure_keeps_the_form() {
        let (backend, mut commands) = Backend::for_test();
        let mut state = GapsState {
            loaded_for: Some("inc-1".into()),
            list: Loadable::Ready(vec![]),
            ..GapsState::default()
        };
        state.form.expected_source = "  logs do firewall  ".into();
        state.form.period_from = "2026-09-01".into();
        state.form.reason = "RETENTION".into();
        let mut harness = harness_for(state, backend);
        harness.run();
        harness.get_by_label("Nova lacuna").click();
        harness.run();
        harness.get_by_label("Registrar lacuna").click();
        harness.step();
        harness.step();

        let sent = drain(&mut commands);
        assert_eq!(sent.len(), 1, "{sent:?}");
        match &sent[0] {
            Command::CreateSourceGap { incident_id, input } => {
                assert_eq!(incident_id, "inc-1");
                assert_eq!(input.reason, "RETENTION");
                assert_eq!(input.period_from.as_deref(), Some("2026-09-01"));
                assert_eq!(input.period_to, None);
            }
            other => panic!("comando inesperado: {other:?}"),
        }

        let (backend, _commands) = Backend::for_test();
        let mut failed = GapsState {
            loaded_for: Some("inc-1".into()),
            list: Loadable::Ready(vec![]),
            ..GapsState::default()
        };
        failed.form.expected_source = "x".into();
        failed.on_created(
            "inc-1",
            Err("the period starts after it ends".into()),
            &backend,
        );
        assert_eq!(failed.form.expected_source, "x");
        let mut harness = harness_for(failed, backend);
        harness.run();
        harness.get_by_label("Nova lacuna").click();
        harness.run();
        harness.get_by_label("Erro: the period starts after it ends");
    }
}
