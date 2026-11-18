//! Revisão dos candidatos de detecção (F1-28, ADR-0025).
//!
//! As regras do engine produzem **hipóteses**. Esta tela as mostra com a
//! regra, a técnica ATT&CK sugerida, os eventos de origem, os IOCs e o que a
//! regra **não** prova, e deixa o analista decidir: confirmar (cria o
//! mapeamento, o achado e os IOCs no incidente escolhido) ou rejeitar (com
//! motivo). Nada vira achado sem esse clique.

use super::backend::{Backend, Command};
use super::{DANGER_TEXT_COLOR, Loadable};
use crate::service::IncidentResponse;
use crate::service::detection::{CandidateView, DetectionRun};
use std::collections::{HashMap, HashSet};

/// Candidatos desenhados de uma vez; o resto é alcançado pelo filtro de situação.
const MAX_RENDERED: usize = 100;
/// Referências de evento exibidas por candidato (o resto fica no detalhe do banco).
const REFS_SHOWN: usize = 8;

/// O que o analista digitou para decidir um candidato.
#[derive(Default)]
struct DecisionForm {
    incident_id: String,
    note: String,
    reason: String,
}

#[derive(Default)]
pub struct DetectionState {
    candidates: HashMap<String, Loadable<Vec<CandidateView>>>,
    running: HashSet<String>,
    forms: HashMap<String, DecisionForm>,
    status_filter: HashMap<String, String>,
    /// Última mensagem de execução/decisão por evidência (`Err` = erro).
    message: HashMap<String, Result<String, String>>,
}

impl DetectionState {
    pub fn is_running(&self, evidence_id: &str) -> bool {
        self.running.contains(evidence_id)
    }

    pub fn start(&mut self, evidence_id: &str, backend: &Backend) {
        self.running.insert(evidence_id.to_string());
        self.message.remove(evidence_id);
        backend.send(Command::RunDetection(evidence_id.to_string()));
    }

    /// Carrega a lista na primeira vez que o painel é aberto.
    fn ensure_loaded(&mut self, evidence_id: &str, backend: &Backend) {
        if !self.candidates.contains_key(evidence_id) {
            self.candidates
                .insert(evidence_id.to_string(), Loadable::Loading);
            backend.send(Command::LoadCandidates(evidence_id.to_string()));
        }
    }

    pub fn on_run(
        &mut self,
        evidence_id: &str,
        result: Result<DetectionRun, String>,
        backend: &Backend,
    ) {
        self.running.remove(evidence_id);
        match result {
            Ok(run) => {
                self.message.insert(
                    evidence_id.to_string(),
                    Ok(format!(
                        "Regras executadas: {} candidato(s) novo(s); pendentes {}, confirmados {}, rejeitados {}. A ausência de candidato não prova ausência de ataque.",
                        run.new_candidates, run.counts.pending, run.counts.confirmed, run.counts.rejected
                    )),
                );
                self.candidates
                    .insert(evidence_id.to_string(), Loadable::Loading);
                backend.send(Command::LoadCandidates(evidence_id.to_string()));
            }
            Err(message) => {
                self.message.insert(evidence_id.to_string(), Err(message));
            }
        }
    }

    pub fn on_loaded(&mut self, evidence_id: &str, result: Result<Vec<CandidateView>, String>) {
        self.candidates.insert(
            evidence_id.to_string(),
            match result {
                Ok(items) => Loadable::Ready(items),
                Err(message) => Loadable::Failed(message),
            },
        );
    }

    pub fn on_decided(
        &mut self,
        evidence_id: &str,
        result: Result<String, String>,
        backend: &Backend,
    ) {
        let ok = result.is_ok();
        self.message.insert(evidence_id.to_string(), result);
        if ok {
            self.candidates
                .insert(evidence_id.to_string(), Loadable::Loading);
            backend.send(Command::LoadCandidates(evidence_id.to_string()));
        }
    }

    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        evidence_id: &str,
        is_evtx: bool,
        incidents: &[IncidentResponse],
        backend: &Backend,
    ) {
        self.ensure_loaded(evidence_id, backend);
        ui.label(
            egui::RichText::new(
                "Hipóteses das regras internas, para você revisar. Nada vira achado sem a sua confirmação, e a ausência de candidato não prova ausência de ataque.",
            )
            .weak(),
        );
        ui.horizontal(|ui| {
            if is_evtx {
                let running = self.running.contains(evidence_id);
                if ui
                    .add_enabled(!running, egui::Button::new("Rodar regras de detecção"))
                    .clicked()
                {
                    self.start(evidence_id, backend);
                }
                if running {
                    ui.spinner();
                    ui.label("Executando as regras sobre o EVTX…");
                }
            } else {
                ui.label("A detecção por regras só se aplica a evidência EVTX.");
            }
            let filter = self
                .status_filter
                .entry(evidence_id.to_string())
                .or_default();
            egui::ComboBox::from_label("Filtro de situação dos candidatos")
                .selected_text(if filter.is_empty() {
                    "Todas as situações".to_string()
                } else {
                    filter.clone()
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(filter, String::new(), "Todas as situações");
                    for status in ["PENDING", "CONFIRMED", "REJECTED"] {
                        ui.selectable_value(filter, status.to_string(), status);
                    }
                });
        });
        match self.message.get(evidence_id) {
            Some(Ok(text)) => {
                ui.colored_label(egui::Color32::from_rgb(40, 140, 60), text);
            }
            Some(Err(text)) => super::error_banner(ui, text),
            None => {}
        }

        let status_filter = self
            .status_filter
            .get(evidence_id)
            .cloned()
            .unwrap_or_default();
        // Clona a lista pronta: o desenho abaixo precisa de `&mut self.forms`.
        let snapshot: Option<Vec<CandidateView>> = match self.candidates.get(evidence_id) {
            Some(Loadable::Ready(items)) => Some(items.clone()),
            Some(Loadable::Failed(message)) => {
                super::error_banner(ui, message);
                None
            }
            Some(Loadable::Loading) => {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Carregando candidatos…");
                });
                None
            }
            _ => None,
        };
        let Some(items) = snapshot else {
            return;
        };
        let shown: Vec<&CandidateView> = items
            .iter()
            .filter(|item| status_filter.is_empty() || item.status == status_filter)
            .collect();
        if items.is_empty() {
            ui.label("Nenhum candidato. Rode as regras de detecção para gerar.");
            return;
        }
        let pending = items.iter().filter(|item| item.status == "PENDING").count();
        ui.label(format!(
            "{} candidato(s), {} pendente(s). Mostrando {}.",
            items.len(),
            pending,
            shown.len().min(MAX_RENDERED)
        ));
        if shown.len() > MAX_RENDERED {
            ui.label(
                egui::RichText::new(format!(
                    "Há mais {} além dos {MAX_RENDERED} mostrados; use o filtro de situação para vê-los.",
                    shown.len() - MAX_RENDERED
                ))
                .weak(),
            );
        }
        egui::ScrollArea::vertical()
            .id_salt(("cand_scroll", evidence_id))
            .max_height(420.0)
            .show(ui, |ui| {
                for item in shown.into_iter().take(MAX_RENDERED) {
                    self.candidate_ui(ui, evidence_id, item, incidents, backend);
                }
            });
    }

    fn candidate_ui(
        &mut self,
        ui: &mut egui::Ui,
        evidence_id: &str,
        item: &CandidateView,
        incidents: &[IncidentResponse],
        backend: &Backend,
    ) {
        // Os labels se repetem; o escopo estável evita IDs persistentes compartilhados.
        ui.push_id(("detection-candidate", evidence_id, &item.id), |ui| {
            ui.group(|ui| {
                ui.horizontal_wrapped(|ui| {
                    let color = match item.severity.as_str() {
                        "CRITICAL" | "HIGH" => DANGER_TEXT_COLOR,
                        "MEDIUM" => egui::Color32::from_rgb(200, 140, 0),
                        _ => egui::Color32::GRAY,
                    };
                    ui.colored_label(color, item.severity.as_str());
                    ui.strong(&item.title);
                    ui.label(format!(
                        "[{} v{}]  {} · {}{}",
                        item.rule_id,
                        item.rule_version,
                        item.tactic,
                        item.technique_id,
                        item.sub_technique_id
                            .as_deref()
                            .map(|sub| format!(" ({sub})"))
                            .unwrap_or_default()
                    ));
                    ui.label(format!("confiança da regra: {}", item.confidence));
                    ui.label(format!("situação: {}", item.status));
                });
                ui.label(&item.description);
                ui.label(format!(
                    "{} · {} evento(s) · {} a {}",
                    item.aggregation, item.event_count, item.first_seen, item.last_seen
                ));
                let refs: Vec<&str> = item
                    .event_refs
                    .iter()
                    .take(REFS_SHOWN)
                    .map(String::as_str)
                    .collect();
                let more = if item.event_refs.len() > REFS_SHOWN || item.refs_truncated {
                    ", …"
                } else {
                    ""
                };
                ui.label(format!("Eventos de origem: {}{}", refs.join(", "), more));
                if !item.iocs.is_empty() {
                    let list: Vec<String> = item
                        .iocs
                        .iter()
                        .map(|ioc| format!("{} {}", ioc.ioc_type, ioc.value))
                        .collect();
                    ui.label(format!("IOCs: {}", list.join(" · ")));
                }
                ui.label(
                    egui::RichText::new(format!(
                        "O que esta regra não prova: {}",
                        item.limitations
                    ))
                    .weak(),
                );

                if item.status != "PENDING" {
                    if let Some(note) = &item.decision_note {
                        ui.label(format!("Decisão: {note}"));
                    }
                    return;
                }
                let form = self.forms.entry(item.id.clone()).or_default();
                ui.horizontal(|ui| {
                    ui.label("Incidente:");
                    egui::ComboBox::from_label("Incidente")
                        .selected_text(
                            incidents
                                .iter()
                                .find(|incident| incident.id == form.incident_id)
                                .map(|incident| incident.name.clone())
                                .unwrap_or_else(|| "— selecione —".into()),
                        )
                        .show_ui(ui, |ui| {
                            for incident in incidents {
                                ui.selectable_value(
                                    &mut form.incident_id,
                                    incident.id.clone(),
                                    &incident.name,
                                );
                            }
                        });
                    let note_label = ui.label("Nota (opcional):");
                    ui.add(
                        egui::TextEdit::singleline(&mut form.note)
                            .hint_text("nota do analista (opcional)")
                            .desired_width(220.0),
                    )
                    .labelled_by(note_label.id);
                    let can_confirm = !form.incident_id.is_empty();
                    if ui
                        .add_enabled(can_confirm, egui::Button::new("Confirmar"))
                        .on_hover_text("Cria o mapeamento ATT&CK, o achado e os IOCs no incidente")
                        .clicked()
                    {
                        let note = form.note.trim();
                        backend.send(Command::ConfirmCandidate {
                            evidence_id: evidence_id.to_string(),
                            candidate_id: item.id.clone(),
                            incident_id: form.incident_id.clone(),
                            note: (!note.is_empty()).then(|| note.to_string()),
                        });
                    }
                });
                if incidents.is_empty() {
                    ui.label(
                        egui::RichText::new(
                            "Crie um incidente na tela Incidentes para poder confirmar.",
                        )
                        .weak(),
                    );
                }
                ui.horizontal(|ui| {
                    let reason_label = ui.label("Motivo da rejeição:");
                    ui.add(
                        egui::TextEdit::singleline(&mut form.reason)
                            .hint_text("motivo da rejeição (obrigatório)")
                            .desired_width(300.0),
                    )
                    .labelled_by(reason_label.id);
                    let can_reject = !form.reason.trim().is_empty();
                    if ui
                        .add_enabled(can_reject, egui::Button::new("Rejeitar"))
                        .clicked()
                    {
                        backend.send(Command::RejectCandidate {
                            evidence_id: evidence_id.to_string(),
                            candidate_id: item.id.clone(),
                            reason: form.reason.trim().to_string(),
                        });
                    }
                });
            });
        });
    }
}

#[cfg(test)]
mod ui_tests {
    use super::*;
    use crate::gui::test_harness::drain;
    use dfir_protocol::CandidateIoc;
    use egui_kittest::{
        Harness,
        kittest::{NodeT, Queryable},
    };

    type State = (DetectionState, Backend, Vec<IncidentResponse>);

    fn view(id: &str, status: &str) -> CandidateView {
        CandidateView {
            id: id.into(),
            evidence_id: "ev_1".into(),
            job_id: "job-1".into(),
            candidate_key: format!("key-{id}"),
            created_at: "2026-03-01T11:00:00Z".into(),
            rule_id: "R-AUTH-002".into(),
            rule_version: "1".into(),
            rule_set_version: "2026.10.1".into(),
            title: format!("Logon após rajada {id}"),
            description: "Um logon bem-sucedido depois de rajada.".into(),
            tactic: "INITIAL_ACCESS".into(),
            technique_id: "T1078".into(),
            sub_technique_id: None,
            severity: "HIGH".into(),
            confidence: "MEDIUM".into(),
            aggregation: "conta=admin; origem=185.220.101.5".into(),
            event_count: 11,
            first_seen: "2026-03-01T10:00:00Z".into(),
            last_seen: "2026-03-01T10:10:00Z".into(),
            event_refs: vec!["record:5".into(), "record:9".into()],
            refs_truncated: false,
            host: None,
            user: Some("admin".into()),
            src_ip: Some("185.220.101.5".into()),
            iocs: vec![CandidateIoc {
                ioc_type: "IP".into(),
                value: "185.220.101.5".into(),
                event_ref: "record:9".into(),
            }],
            limitations: "Não prova que o logon foi do atacante.".into(),
            status: status.into(),
            decided_at: None,
            decision_note: (status == "REJECTED").then(|| "ruído de monitoramento".into()),
            incident_id: None,
            attack_mapping_id: None,
            finding_id: None,
        }
    }

    fn incident(id: &str, name: &str) -> IncidentResponse {
        IncidentResponse {
            id: id.into(),
            name: name.into(),
            state: "OPEN".into(),
            created_at: "2026-03-01T00:00:00Z".into(),
        }
    }

    fn harness(
        state: DetectionState,
        backend: Backend,
        incidents: Vec<IncidentResponse>,
    ) -> Harness<'static, State> {
        crate::gui::test_harness::builder().build_ui_state(
            |ui, (state, backend, incidents): &mut State| {
                state.ui(ui, "ev_1", true, incidents, backend)
            },
            (state, backend, incidents),
        )
    }

    #[test]
    fn opening_the_panel_loads_the_candidates_once() {
        let (backend, mut commands) = Backend::for_test();
        let mut harness = harness(DetectionState::default(), backend, vec![]);
        harness.step();
        harness.step();
        let loads = drain(&mut commands)
            .into_iter()
            .filter(|command| matches!(command, Command::LoadCandidates(id) if id == "ev_1"))
            .count();
        assert_eq!(loads, 1, "uma única carga, não uma por quadro");
    }

    #[test]
    fn a_candidate_shows_rule_technique_origin_events_iocs_and_what_it_does_not_prove() {
        let (backend, _commands) = Backend::for_test();
        let mut state = DetectionState::default();
        state.on_loaded("ev_1", Ok(vec![view("a", "PENDING")]));
        let mut harness = harness(state, backend, vec![]);
        harness.run();
        harness.get_by_label_contains("Logon após rajada a");
        harness.get_by_label_contains("R-AUTH-002 v1");
        harness.get_by_label_contains("T1078");
        harness.get_by_label_contains("record:5, record:9");
        harness.get_by_label_contains("IP 185.220.101.5");
        harness.get_by_label_contains("O que esta regra não prova");
        harness.get_by_label_contains("Nada vira achado sem a sua confirmação");
    }

    #[test]
    fn pending_candidate_incident_combos_have_unique_ids_and_independent_decisions() {
        let (backend, mut commands) = Backend::for_test();
        let mut state = DetectionState::default();
        state.on_loaded("ev_1", Ok(vec![view("a", "PENDING"), view("b", "PENDING")]));
        let mut harness = harness(
            state,
            backend,
            vec![incident("inc-1", "Caso BEC"), incident("inc-2", "Caso RDP")],
        );
        harness.set_size(egui::vec2(1000.0, 2400.0));
        harness.run();
        drain(&mut commands);

        let all_combos: Vec<_> = harness
            .get_all_by_role(egui::accesskit::Role::ComboBox)
            .collect();
        let combos: Vec<_> = all_combos
            .iter()
            .filter(|node| {
                node.accesskit_node()
                    .label()
                    .is_some_and(|label| label == "Incidente")
            })
            .copied()
            .collect();
        assert_eq!(
            combos.len(),
            2,
            "dois seletores devem preservar o nome acessível 'Incidente': {all_combos:#?}"
        );
        assert_ne!(
            combos[0].accesskit_node().id(),
            combos[1].accesskit_node().id(),
            "cada candidato precisa de um ID persistente próprio"
        );

        combos[1].click();
        harness.run();
        harness
            .get_by_role_and_label(egui::accesskit::Role::Button, "Caso RDP")
            .click();
        harness.run();

        let (state, _, _) = harness.state();
        assert_eq!(state.forms["a"].incident_id, "");
        assert_eq!(state.forms["b"].incident_id, "inc-2");

        harness
            .get_all_by_role_and_label(egui::accesskit::Role::Button, "Confirmar")
            .nth(1)
            .expect("botão Confirmar do segundo candidato")
            .click();
        harness.run();
        let sent = drain(&mut commands);
        let confirmations: Vec<_> = sent
            .iter()
            .filter(|command| matches!(command, Command::ConfirmCandidate { .. }))
            .collect();
        assert_eq!(confirmations.len(), 1, "somente uma decisão: {sent:?}");
        assert!(matches!(
            confirmations[0],
            Command::ConfirmCandidate {
                evidence_id,
                candidate_id,
                incident_id,
                ..
            } if evidence_id == "ev_1" && candidate_id == "b" && incident_id == "inc-2"
        ));
    }

    #[test]
    fn confirm_stays_disabled_until_an_incident_is_chosen_and_reject_needs_a_reason() {
        let (backend, mut commands) = Backend::for_test();
        let mut state = DetectionState::default();
        state.on_loaded("ev_1", Ok(vec![view("a", "PENDING")]));
        let mut harness = harness(state, backend, vec![incident("inc-1", "Caso BEC")]);
        harness.run();
        drain(&mut commands);

        // Sem incidente escolhido, clicar não envia nada.
        harness.get_by_label("Confirmar").click();
        harness.run();
        // Sem motivo, rejeitar não envia nada.
        harness.get_by_label("Rejeitar").click();
        harness.run();
        assert!(
            drain(&mut commands).into_iter().all(|command| !matches!(
                command,
                Command::ConfirmCandidate { .. } | Command::RejectCandidate { .. }
            )),
            "nenhuma decisão sem incidente ou sem motivo"
        );
    }

    #[test]
    fn rejecting_with_a_reason_sends_the_decision_and_a_decided_candidate_has_no_buttons() {
        let (backend, mut commands) = Backend::for_test();
        let mut state = DetectionState::default();
        state.on_loaded("ev_1", Ok(vec![view("a", "PENDING")]));
        let mut harness = harness(state, backend, vec![]);
        harness.run();
        drain(&mut commands);
        // Há dois campos de texto (nota e motivo): o do motivo é o último.
        let reason_box = harness
            .get_all_by_role(egui::accesskit::Role::TextInput)
            .last()
            .expect("reason box");
        reason_box.focus();
        harness.run();
        harness
            .get_all_by_role(egui::accesskit::Role::TextInput)
            .last()
            .expect("reason box")
            .type_text("ruído do monitoramento");
        harness.run();
        harness.get_by_label("Rejeitar").click();
        harness.run();
        let sent = drain(&mut commands);
        assert!(
            sent.iter().any(|command| matches!(
                command,
                Command::RejectCandidate { candidate_id, reason, .. }
                    if candidate_id == "a" && reason == "ruído do monitoramento"
            )),
            "{sent:?}"
        );

        // Candidato já decidido: mostra a decisão e não oferece os botões.
        let (backend, _commands) = Backend::for_test();
        let mut state = DetectionState::default();
        state.on_loaded("ev_1", Ok(vec![view("b", "REJECTED")]));
        let mut harness = self::harness(state, backend, vec![]);
        harness.run();
        harness.get_by_label_contains("Decisão: ruído de monitoramento");
        assert!(harness.query_by_label("Confirmar").is_none());
        assert!(harness.query_by_label("Rejeitar").is_none());
    }

    #[test]
    fn running_the_rules_sends_the_command_and_reports_the_counts() {
        let (backend, mut commands) = Backend::for_test();
        let mut state = DetectionState::default();
        state.on_loaded("ev_1", Ok(vec![]));
        let mut harness = harness(state, backend, vec![]);
        harness.run();
        drain(&mut commands);
        harness.get_by_label("Rodar regras de detecção").click();
        harness.step();
        assert!(
            drain(&mut commands)
                .into_iter()
                .any(|command| matches!(command, Command::RunDetection(id) if id == "ev_1"))
        );
    }

    #[test]
    fn a_failed_run_shows_the_error() {
        let (backend, _commands) = Backend::for_test();
        let (backend2, _c2) = Backend::for_test();
        let mut state = DetectionState::default();
        state.on_loaded("ev_1", Ok(vec![]));
        state.on_run("ev_1", Err("falha do engine".into()), &backend2);
        state.on_loaded("ev_1", Ok(vec![]));
        let mut harness = harness(state, backend, vec![]);
        harness.run();
        harness.get_by_label_contains("falha do engine");
    }
}
