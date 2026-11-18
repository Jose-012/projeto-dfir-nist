//! Painel "Incidentes": lista/criação de incidentes + abas com os painéis
//! F1-03 (NIST) / F1-04 (ATT&CK, caminho crítico, impacto) / F1-05 (IOCs,
//! recomendações, exportação). Porta funcional do antigo
//! `frontend/src/incident-ui.ts`.

use super::Loadable;
use super::backend::{Backend, Command};
use super::{
    attack, critical_path, deliverables, export, findings, gaps, impact, iocs, nist, plan,
    recommendations,
};
use crate::persistence::audit::AuditEventView;
use crate::service::{EvidenceResponse, IncidentResponse};
use egui_extras::{Column, TableBuilder};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Tab {
    #[default]
    Nist,
    Attack,
    CriticalPath,
    Impact,
    Findings,
    Plan,
    Gaps,
    Iocs,
    Recommendations,
    Deliverables,
    Export,
    Evidence,
    Audit,
}

impl IncidentsState {
    /// Incidentes já carregados (vazio enquanto a lista não chegou).
    pub fn items(&self) -> &[IncidentResponse] {
        self.list.ready().map(Vec::as_slice).unwrap_or(&[])
    }
}

pub struct IncidentsState {
    list: Loadable<Vec<IncidentResponse>>,
    new_name: String,
    creating: bool,
    create_error: Option<String>,
    selected: Option<IncidentResponse>,
    tab: Tab,
    pub nist: nist::NistState,
    pub attack: attack::AttackState,
    pub critical_path: critical_path::CriticalPathState,
    pub impact: impact::ImpactState,
    pub findings: findings::FindingsState,
    pub plan: plan::PlanState,
    pub gaps: gaps::GapsState,
    pub deliverables: deliverables::DeliverablesState,
    pub iocs: iocs::IocsState,
    pub recommendations: recommendations::RecommendationsState,
    pub export: export::ExportState,
    /// Evidências vinculadas ao incidente selecionado (F1-19).
    evidence: Loadable<Vec<EvidenceResponse>>,
    /// Eventos de auditoria do incidente selecionado (F1-24).
    audit: Loadable<Vec<AuditEventView>>,
}

impl Default for IncidentsState {
    fn default() -> Self {
        Self {
            list: Loadable::Idle,
            new_name: String::new(),
            creating: false,
            create_error: None,
            selected: None,
            tab: Tab::default(),
            nist: Default::default(),
            attack: Default::default(),
            critical_path: Default::default(),
            impact: Default::default(),
            findings: Default::default(),
            plan: Default::default(),
            gaps: Default::default(),
            deliverables: Default::default(),
            iocs: Default::default(),
            recommendations: Default::default(),
            export: Default::default(),
            evidence: Loadable::Idle,
            audit: Loadable::Idle,
        }
    }
}

impl IncidentsState {
    pub fn on_listed(&mut self, result: Result<Vec<IncidentResponse>, String>) {
        self.list = match result {
            Ok(items) => Loadable::Ready(items),
            Err(message) => Loadable::Failed(message),
        };
    }

    pub fn on_incident_evidence_loaded(
        &mut self,
        _incident_id: &str,
        result: Result<Vec<EvidenceResponse>, String>,
    ) {
        self.evidence = match result {
            Ok(items) => Loadable::Ready(items),
            Err(message) => Loadable::Failed(message),
        };
    }

    pub fn on_audit_loaded(
        &mut self,
        _incident_id: &str,
        result: Result<Vec<AuditEventView>, String>,
    ) {
        self.audit = match result {
            Ok(items) => Loadable::Ready(items),
            Err(message) => Loadable::Failed(message),
        };
    }

    pub fn on_created(&mut self, result: Result<IncidentResponse, String>) {
        self.creating = false;
        match result {
            Ok(incident) => {
                self.create_error = None;
                self.new_name.clear();
                self.selected = Some(incident.clone());
                self.tab = Tab::default();
                if let Loadable::Ready(items) = &mut self.list {
                    items.insert(0, incident);
                } else {
                    self.list = Loadable::Ready(vec![incident]);
                }
            }
            Err(message) => self.create_error = Some(message),
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, backend: &Backend) {
        egui::Panel::left("incident_list_panel")
            .resizable(true)
            .default_size(260.0)
            .show(ui, |ui| {
                ui.heading("Incidentes");
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.new_name)
                        .on_hover_text("Nome do novo incidente");
                    if ui
                        .add_enabled(
                            !self.creating && !self.new_name.trim().is_empty(),
                            egui::Button::new("Novo"),
                        )
                        .clicked()
                    {
                        self.creating = true;
                        backend.send(Command::CreateIncident(self.new_name.trim().to_string()));
                    }
                });
                if self.creating {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("Criando…");
                    });
                }
                if let Some(message) = &self.create_error {
                    super::error_banner(ui, message);
                }
                ui.separator();

                let items = self.list.ready().cloned().unwrap_or_default();
                match &self.list {
                    Loadable::Idle | Loadable::Loading => {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label("Carregando…");
                        });
                    }
                    Loadable::Failed(message) => super::error_banner(ui, message),
                    Loadable::Ready(_) if items.is_empty() => {
                        ui.label("Nenhum incidente ainda.");
                    }
                    Loadable::Ready(_) => {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            for incident in &items {
                                let selected = self
                                    .selected
                                    .as_ref()
                                    .is_some_and(|current| current.id == incident.id);
                                if ui
                                    .selectable_label(
                                        selected,
                                        format!("{} · {}", incident.name, incident.state),
                                    )
                                    .clicked()
                                {
                                    self.selected = Some(incident.clone());
                                    self.evidence = Loadable::Idle;
                                    self.audit = Loadable::Idle;
                                }
                            }
                        });
                    }
                }
            });

        egui::CentralPanel::default().show(ui, |ui| {
            let Some(incident) = self.selected.clone() else {
                ui.label("Selecione ou crie um incidente à esquerda.");
                return;
            };
            ui.heading(&incident.name);
            ui.horizontal_wrapped(|ui| {
                for (tab, label) in [
                    (Tab::Nist, "NIST CSF"),
                    (Tab::Attack, "ATT&CK"),
                    (Tab::CriticalPath, "Caminho crítico"),
                    (Tab::Impact, "Impacto"),
                    (Tab::Findings, "Achados"),
                    (Tab::Plan, "Plano"),
                    (Tab::Gaps, "Lacunas"),
                    (Tab::Iocs, "IOCs"),
                    (Tab::Recommendations, "Recomendações"),
                    (Tab::Deliverables, "Entregáveis"),
                    (Tab::Export, "Exportar"),
                    (Tab::Evidence, "Evidências"),
                    (Tab::Audit, "Auditoria"),
                ] {
                    if ui.selectable_label(self.tab == tab, label).clicked() {
                        self.tab = tab;
                    }
                }
            });
            ui.separator();
            egui::ScrollArea::vertical().show(ui, |ui| match self.tab {
                Tab::Nist => self.nist.ui(ui, &incident.id, backend),
                Tab::Attack => self.attack.ui(ui, &incident.id, backend),
                Tab::CriticalPath => self.critical_path.ui(ui, &incident.id, backend),
                Tab::Impact => self.impact.ui(ui, &incident.id, backend),
                Tab::Findings => self.findings.ui(ui, &incident.id, backend),
                Tab::Plan => self.plan.ui(ui, &incident.id, backend),
                Tab::Gaps => self.gaps.ui(ui, &incident.id, backend),
                Tab::Iocs => self.iocs.ui(ui, &incident.id, backend),
                Tab::Recommendations => self.recommendations.ui(ui, &incident.id, backend),
                Tab::Deliverables => self.deliverables.ui(ui, &incident.id, backend),
                Tab::Export => self.export.ui(ui, &incident.id, backend),
                Tab::Evidence => {
                    let loadable = self.evidence.get_or_insert_with(|| {
                        backend.send(Command::LoadIncidentEvidence(incident.id.clone()));
                        Loadable::Loading
                    });
                    loadable.show(ui, |ui, items| {
                        if items.is_empty() {
                            ui.label("Nenhuma evidência vinculada a este incidente ainda.");
                            ui.weak("Vincule na aba Evidências, nos detalhes da evidência.");
                            return;
                        }
                        TableBuilder::new(ui)
                            .striped(true)
                            .column(Column::auto().at_least(140.0))
                            .column(Column::auto().at_least(90.0))
                            .column(Column::remainder().at_least(200.0))
                            .header(20.0, |mut header| {
                                header.col(|ui| {
                                    ui.strong("ID");
                                });
                                header.col(|ui| {
                                    ui.strong("Tratamento");
                                });
                                header.col(|ui| {
                                    ui.strong("Nome lógico");
                                });
                            })
                            .body(|mut body| {
                                for item in items {
                                    body.row(18.0, |mut row| {
                                        row.col(|ui| {
                                            ui.label(&item.id);
                                        });
                                        row.col(|ui| {
                                            if item.is_analyzed_automatically() {
                                                ui.label("analisada automaticamente");
                                            } else {
                                                ui.label("somente preservada");
                                            }
                                        });
                                        row.col(|ui| {
                                            ui.label(&item.original_filename);
                                        });
                                    });
                                }
                            });
                    });
                }
                Tab::Audit => {
                    let loadable = self.audit.get_or_insert_with(|| {
                        backend.send(Command::LoadAudit {
                            incident_id: incident.id.clone(),
                            limit: 200,
                        });
                        Loadable::Loading
                    });
                    loadable.show(ui, |ui, events| {
                        if events.is_empty() {
                            ui.label("Nenhum evento de auditoria para este incidente.");
                            return;
                        }
                        TableBuilder::new(ui)
                            .striped(true)
                            .column(Column::auto().at_least(50.0))
                            .column(Column::auto().at_least(130.0))
                            .column(Column::auto().at_least(140.0))
                            .column(Column::auto().at_least(90.0))
                            .column(Column::remainder().at_least(220.0))
                            .header(20.0, |mut header| {
                                header.col(|ui| {
                                    ui.strong("Seq");
                                });
                                header.col(|ui| {
                                    ui.strong("Quando");
                                });
                                header.col(|ui| {
                                    ui.strong("Ação");
                                });
                                header.col(|ui| {
                                    ui.strong("Objeto");
                                });
                                header.col(|ui| {
                                    ui.strong("Hash");
                                });
                            })
                            .body(|mut body| {
                                for event in events {
                                    body.row(18.0, |mut row| {
                                        row.col(|ui| {
                                            ui.label(event.sequence.to_string());
                                        });
                                        row.col(|ui| {
                                            ui.label(&event.timestamp);
                                        });
                                        row.col(|ui| {
                                            ui.label(&event.action);
                                        });
                                        row.col(|ui| {
                                            ui.label(format!(
                                                "{} {}",
                                                event.object_type, event.object_id
                                            ));
                                        });
                                        row.col(|ui| {
                                            ui.label(
                                                &event.event_hash[..12.min(event.event_hash.len())],
                                            );
                                        });
                                    });
                                }
                            });
                        ui.small(format!("{} evento(s) neste incidente.", events.len()));
                    });
                }
            });
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::backend::Backend;
    use crate::gui::test_harness::drain;
    use egui_kittest::kittest::Queryable;

    fn incident(id: &str, name: &str) -> IncidentResponse {
        IncidentResponse {
            id: id.into(),
            name: name.into(),
            state: "OPEN".into(),
            created_at: "2026-03-01T00:00:00Z".into(),
        }
    }

    fn evidence(id: &str, filename: &str) -> EvidenceResponse {
        EvidenceResponse {
            id: id.into(),
            original_filename: filename.into(),
            storage_reference: format!("evidence/{id}"),
            format: "evtx".into(),
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
            incident_id: Some("inc-1".into()),
        }
    }

    fn audit_event(sequence: i64, action: &str) -> AuditEventView {
        AuditEventView {
            sequence,
            id: format!("ae-{sequence}"),
            timestamp: "2026-11-15T00:00:00Z".into(),
            actor_id: Some("operador".into()),
            action: action.into(),
            object_type: "evidence".into(),
            object_id: "ev-1".into(),
            incident_id: Some("inc-1".into()),
            metadata: serde_json::json!({}),
            event_hash: "cd".repeat(32),
        }
    }

    fn state_with_incident() -> IncidentsState {
        IncidentsState {
            list: Loadable::Ready(vec![incident("inc-1", "Incidente demo")]),
            selected: Some(incident("inc-1", "Incidente demo")),
            ..IncidentsState::default()
        }
    }

    #[test]
    fn all_tabs_stay_inside_a_narrow_window() {
        let (backend, _commands) = Backend::for_test();
        let mut state = state_with_incident();
        let mut harness = crate::gui::test_harness::builder()
            .with_size(egui::vec2(960.0, 700.0))
            .build_ui_state(
                |ui, state: &mut (&mut IncidentsState, &Backend)| state.0.ui(ui, state.1),
                (&mut state, &backend),
            );
        harness.step();

        for label in [
            "NIST CSF",
            "ATT&CK",
            "Caminho crítico",
            "Impacto",
            "Achados",
            "Plano",
            "Lacunas",
            "IOCs",
            "Recomendações",
            "Entregáveis",
            "Exportar",
            "Evidências",
            "Auditoria",
        ] {
            let rect = harness.get_by_label(label).rect();
            assert!(
                rect.min.x >= 0.0 && rect.max.x <= 960.0,
                "aba {label:?} fora da viewport: {rect:?}"
            );
        }
    }

    #[test]
    fn the_evidence_tab_loads_once_and_lists_the_linked_evidence() {
        let (backend, mut commands) = Backend::for_test();
        let mut state = state_with_incident();
        state.tab = Tab::Evidence;

        {
            let mut harness = crate::gui::test_harness::builder()
                .with_size(egui::vec2(1400.0, 900.0))
                .build_ui_state(
                    |ui, state: &mut (&mut IncidentsState, &Backend)| state.0.ui(ui, state.1),
                    (&mut state, &backend),
                );
            harness.step();
            harness.get_by_label("Evidências").click();
            // Painel em Loading (spinner): step em vez de run.
            harness.step();
            harness.step();
        }

        let loads = drain(&mut commands)
            .iter()
            .filter(|command| matches!(command, Command::LoadIncidentEvidence(id) if id == "inc-1"))
            .count();
        assert_eq!(loads, 1, "evidências do incidente carregam uma vez");

        state.on_incident_evidence_loaded("inc-1", Ok(vec![evidence("ev-1", "Security.evtx")]));
        let mut harness = crate::gui::test_harness::builder()
            .with_size(egui::vec2(1400.0, 900.0))
            .build_ui_state(
                |ui, state: &mut (&mut IncidentsState, &Backend)| state.0.ui(ui, state.1),
                (&mut state, &backend),
            );
        harness.run();
        harness.get_by_label("Evidências").click();
        harness.run();
        harness.get_by_label("ev-1");
        harness.get_by_label("Security.evtx");
        harness.get_by_label("analisada automaticamente");
    }

    #[test]
    fn the_audit_tab_loads_once_and_lists_events_with_hash_prefix() {
        let (backend, mut commands) = Backend::for_test();
        let mut state = state_with_incident();
        state.tab = Tab::Audit;

        {
            let mut harness = crate::gui::test_harness::builder()
                .with_size(egui::vec2(1400.0, 900.0))
                .build_ui_state(
                    |ui, state: &mut (&mut IncidentsState, &Backend)| state.0.ui(ui, state.1),
                    (&mut state, &backend),
                );
            harness.step();
            harness.get_by_label("Auditoria").click();
            harness.step();
            harness.step();
        }

        let loads = drain(&mut commands)
            .iter()
            .filter(|command| {
                matches!(command, Command::LoadAudit { incident_id, .. } if incident_id == "inc-1")
            })
            .count();
        assert_eq!(loads, 1, "auditoria do incidente carrega uma vez");

        state.on_audit_loaded(
            "inc-1",
            Ok(vec![
                audit_event(1, "INCIDENT_CREATED"),
                audit_event(2, "EVIDENCE_LINKED_TO_INCIDENT"),
            ]),
        );
        let mut harness = crate::gui::test_harness::builder()
            .with_size(egui::vec2(1400.0, 900.0))
            .build_ui_state(
                |ui, state: &mut (&mut IncidentsState, &Backend)| state.0.ui(ui, state.1),
                (&mut state, &backend),
            );
        harness.run();
        harness.get_by_label("Auditoria").click();
        harness.run();
        harness.get_by_label("EVIDENCE_LINKED_TO_INCIDENT");
        harness.get_by_label("2 evento(s) neste incidente.");
        assert_eq!(
            harness.query_all_by_label("cdcdcdcdcdcd").count(),
            2,
            "cada evento mostra o prefixo do hash"
        );
    }
}
