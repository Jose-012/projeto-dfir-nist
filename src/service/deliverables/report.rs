//! Entregável 1 — Relatório DFIR (spec §4.2).

use super::markdown::{Doc, code, esc, short};
use super::{
    DeliverableData, DetectionSummary, FILE_E1, RenderedDocument, TimelineState, confidence_rank,
    evidence_name, field, field_text, finding_label, into_document, severity_rank, strings,
};
use crate::persistence::source_gaps::GapReason;
use crate::service::{AnalyticalResultResponse, EvidenceResponse};

const TACTIC_ORDER: [&str; 14] = [
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

pub(super) fn tactic_rank(token: &str) -> usize {
    TACTIC_ORDER
        .iter()
        .position(|tactic| *tactic == token)
        .unwrap_or(TACTIC_ORDER.len())
}

fn cause_label(level: &str) -> &'static str {
    match level {
        "ROOT_CAUSE" => "Causa-raiz",
        "PROBABLE_CAUSE" => "Causa provável",
        "CONTRIBUTING_FACTOR" => "Fator contribuinte",
        _ => "Conclusão de causa",
    }
}

fn cause_rank(level: &str) -> u8 {
    match level {
        "ROOT_CAUSE" => 0,
        "PROBABLE_CAUSE" => 1,
        "CONTRIBUTING_FACTOR" => 2,
        _ => 3,
    }
}

fn stage_index(stage: &AnalyticalResultResponse) -> i64 {
    stage
        .payload
        .get("sequence_index")
        .and_then(|value| value.as_i64())
        .unwrap_or(i64::MAX)
}

fn sorted_causes(data: &DeliverableData) -> Vec<&AnalyticalResultResponse> {
    let mut causes: Vec<&AnalyticalResultResponse> = data.findings.root_causes.iter().collect();
    causes.sort_by(|a, b| {
        cause_rank(field(a, "level"))
            .cmp(&cause_rank(field(b, "level")))
            .then_with(|| {
                confidence_rank(b.confidence.as_deref().unwrap_or(""))
                    .cmp(&confidence_rank(a.confidence.as_deref().unwrap_or("")))
            })
            .then_with(|| a.id.cmp(&b.id))
    });
    causes
}

fn sorted_stages(data: &DeliverableData) -> Vec<&AnalyticalResultResponse> {
    let mut stages: Vec<&AnalyticalResultResponse> = data.critical_path.stages.iter().collect();
    stages.sort_by(|a, b| {
        stage_index(a)
            .cmp(&stage_index(b))
            .then_with(|| a.id.cmp(&b.id))
    });
    stages
}

pub(super) fn render(data: &DeliverableData) -> RenderedDocument {
    let mut doc = Doc::new();
    doc.title(&format!("Relatório DFIR — {}", data.incident.name));
    doc.quote(&format!(
        "Incidente {} · Entregável 1 de 5. Este documento só reorganiza o que foi registrado no incidente; cada item cita o resultado de origem entre crases, e a lista completa de origens está no `manifest.json`.",
        code(&short(&data.incident.id))
    ));
    executive_summary(&mut doc, data);
    scope_and_limitations(&mut doc, data);
    evidence_and_custody(&mut doc, data);
    timeline(&mut doc, data);
    tactic_sections(&mut doc, data);
    critical_path(&mut doc, data);
    impact(&mut doc, data);
    causes(&mut doc, data);
    conclusions(&mut doc, data);
    iocs(&mut doc, data);
    into_document(doc, FILE_E1, "Relatório DFIR")
}

fn executive_summary(doc: &mut Doc, data: &DeliverableData) {
    doc.h2("1. Resumo executivo");
    let mut lines = vec![format!(
        "Evidências citadas pelo incidente: {}.",
        data.evidence.len()
    )];

    let findings = &data.findings.findings;
    match findings
        .iter()
        .max_by_key(|finding| severity_rank(field(finding, "severity")))
    {
        None => lines
            .push("Achados (vulnerabilidades ou condições exploradas): nenhum registrado.".into()),
        Some(top) => lines.push(format!(
            "Achados registrados: {} (maior severidade: {}).",
            findings.len(),
            code(field(top, "severity"))
        )),
    }

    match sorted_causes(data).first() {
        None => lines.push(
            "Causa: nenhuma conclusão registrada; este relatório não afirma causa-raiz.".into(),
        ),
        Some(cause) => lines.push(format!(
            "Causa registrada: {} (confiança {}): {}",
            cause_label(field(cause, "level")),
            code(cause.confidence.as_deref().unwrap_or("")),
            esc(field(cause, "statement"))
        )),
    }

    match &data.impact {
        None => lines.push("Impacto: não registrado.".into()),
        Some(impact) => lines.push(format!(
            "Impacto registrado: confidencialidade {}, integridade {}, disponibilidade {}.",
            code(field(impact, "confidentiality_impact")),
            code(field(impact, "integrity_impact")),
            code(field(impact, "availability_impact"))
        )),
    }

    lines.push(format!(
        "Estágios do caminho crítico: {} · técnicas ATT&CK: {} · IOCs: {}.",
        data.critical_path.stages.len(),
        data.attack_mappings.len(),
        data.iocs.len()
    ));
    let without_basis = unsupported_count(data);
    lines.push(format!(
        "Itens registrados sem base documentada (mapeamentos ATT&CK e estágios): {without_basis}."
    ));
    lines.push(format!(
        "Lacunas de fonte declaradas pelo analista: {}.",
        data.source_gaps.len()
    ));
    doc.bullets(&lines);
    doc.para("*Este resumo é montado só com o que está registrado; não acrescenta inferência.*");
}

fn unsupported_count(data: &DeliverableData) -> usize {
    data.attack_mappings
        .iter()
        .chain(data.critical_path.stages.iter())
        .filter(|item| field(item, "evidence_status") == "GAP")
        .count()
}

fn scope_and_limitations(doc: &mut Doc, data: &DeliverableData) {
    doc.h2("2. Escopo e limitações");
    let mut lines = vec![
        "Análise automática: somente logs EVTX do Windows. Nenhuma outra fonte (servidores de aplicação, SIEM, firewall, IDS, tráfego de rede) foi analisada automaticamente; o que consta sobre elas foi registrado manualmente pelo analista.".to_string(),
    ];
    for section in &data.evidence {
        let name = esc(&section.evidence.original_filename);
        if section.is_destroyed() {
            lines.push(format!(
                "{name}: descartada localmente; a linha do tempo e os artefatos da análise foram removidos e não podem mais ser reconferidos."
            ));
        } else {
            match &section.timeline {
                TimelineState::NoCompletedJob => {
                    lines.push(format!(
                        "{name}: nenhuma análise de linha do tempo concluída."
                    ));
                }
                TimelineState::Unavailable => lines.push(format!(
                    "{name}: linha do tempo indisponível (o índice não pôde ser lido)."
                )),
                TimelineState::Summary(summary) if summary.warning_count > 0 => {
                    lines.push(format!(
                        "{name}: análise concluída com {} aviso(s); não é uma conclusão limpa.",
                        summary.warning_count
                    ))
                }
                TimelineState::Summary(_) => {}
            }
        }
        match section.custody_chain_intact {
            Some(true) => {}
            Some(false) => lines.push(format!(
                "{name}: a verificação da cadeia de custódia **falhou**."
            )),
            None => lines.push(format!(
                "{name}: a cadeia de custódia não pôde ser verificada."
            )),
        }
        if !section.evidence.is_analyzed_automatically() {
            lines.push(format!(
                "{name}: o formato {} é apenas preservado (hash e custódia); nenhum conteúdo foi interpretado.",
                code(&section.evidence.format)
            ));
        }
        let missing = missing_provenance(&section.evidence);
        if !missing.is_empty() {
            lines.push(format!(
                "{name}: proveniência não informada ({}).",
                missing.join(", ")
            ));
        }
        if section.evidence.contains_personal_data {
            lines.push(format!(
                "{name}: marcada pelo operador como contendo dado pessoal."
            ));
        }
    }
    let attack_gaps = data
        .attack_mappings
        .iter()
        .filter(|item| field(item, "evidence_status") == "GAP")
        .count();
    if attack_gaps > 0 {
        lines.push(format!(
            "{attack_gaps} mapeamento(s) ATT&CK sem base documentada."
        ));
    }
    let stage_gaps = data
        .critical_path
        .stages
        .iter()
        .filter(|item| field(item, "evidence_status") == "GAP")
        .count();
    if stage_gaps > 0 {
        lines.push(format!(
            "{stage_gaps} estágio(s) do caminho crítico sem base documentada."
        ));
    }
    for cause in sorted_causes(data) {
        let origin = doc.cite(&cause.id);
        for gap in strings(cause, "gaps") {
            lines.push(format!(
                "Lacuna na conclusão ({}): {} · origem {origin}",
                cause_label(field(cause, "level")).to_lowercase(),
                esc(&gap)
            ));
        }
    }
    if !data.other_evidence.is_empty() {
        let names: Vec<String> = data
            .other_evidence
            .iter()
            .map(|evidence| esc(&evidence.original_filename))
            .collect();
        lines.push(format!(
            "Evidências importadas e não citadas por este incidente: {}.",
            names.join("; ")
        ));
    }
    match &data.redaction {
        Some(summary) => lines.push(format!(
            "Dados pessoais mascarados: {} IP(s), {} e-mail(s), {} conta(s) e {} pessoa(s) trocados por marcadores como {}; o mapeamento não é gravado. Nomes em texto livre e outros identificadores não são detectados.",
            summary.ips,
            summary.emails,
            summary.accounts,
            summary.persons,
            code("{IP-1}")
        )),
        None => lines.push(
            "Este documento **não** mascara dados pessoais: pode conter IPs, e-mails, contas e nomes reais."
                .to_string(),
        ),
    }
    doc.bullets(&lines);
    source_gaps(doc, data);
}

/// O que da proveniência o operador não declarou (RB-002).
fn missing_provenance(evidence: &EvidenceResponse) -> Vec<&'static str> {
    let blank = |value: Option<&str>| value.is_none_or(|text| text.trim().is_empty());
    let mut missing = Vec::new();
    if evidence.declared_source().is_none() {
        missing.push("origem");
    }
    if blank(evidence.provided_by.as_deref()) {
        missing.push("quem forneceu");
    }
    if evidence.declared_collection_method().is_none() {
        missing.push("método de coleta");
    }
    if evidence.declared_collected_at().is_none() {
        missing.push("data da coleta");
    }
    if blank(evidence.authorization_reference.as_deref()) {
        missing.push("autorização");
    }
    missing
}

fn period_text(from: &str, to: &str) -> String {
    match (from.is_empty(), to.is_empty()) {
        (true, true) => "período não informado".to_string(),
        (false, true) => format!("a partir de {}", code(from)),
        (true, false) => format!("até {}", code(to)),
        (false, false) => format!("{} a {}", code(from), code(to)),
    }
}

/// Lacunas de fonte que o analista declarou (RB-001).
fn source_gaps(doc: &mut Doc, data: &DeliverableData) {
    doc.h3("2.1 Lacunas de fonte declaradas");
    if data.source_gaps.is_empty() {
        doc.none(
            "nenhuma lacuna de fonte declarada pelo analista; isso não prova que não houve lacunas",
        );
        return;
    }
    let mut gaps: Vec<&AnalyticalResultResponse> = data.source_gaps.iter().collect();
    gaps.sort_by(|a, b| {
        field(a, "expected_source")
            .cmp(field(b, "expected_source"))
            .then_with(|| field(a, "period_from").cmp(field(b, "period_from")))
            .then_with(|| a.id.cmp(&b.id))
    });
    let mut rows = Vec::new();
    for gap in gaps {
        let reason = GapReason::from_str(field(gap, "reason")).map_or_else(
            || esc(field(gap, "reason")),
            |known| known.label_pt().to_string(),
        );
        rows.push(vec![
            esc(field(gap, "expected_source")),
            period_text(field(gap, "period_from"), field(gap, "period_to")),
            reason,
            esc(field(gap, "note")),
            doc.cite(&gap.id),
        ]);
    }
    doc.table(
        &["Fonte esperada", "Período", "Motivo", "Nota", "Origem"],
        &rows,
    );
}

fn evidence_and_custody(doc: &mut Doc, data: &DeliverableData) {
    doc.h2("3. Evidências e cadeia de custódia");
    if data.evidence.is_empty() {
        doc.none("nenhuma evidência citada pelos resultados deste incidente");
        return;
    }
    let rows: Vec<Vec<String>> = data
        .evidence
        .iter()
        .map(|section| {
            let evidence = &section.evidence;
            vec![
                esc(&evidence.original_filename),
                code(&evidence.format),
                evidence.size_bytes.to_string(),
                code(&evidence.blake3),
                code(&evidence.sha256),
                code(&evidence.imported_at),
                code(&evidence.status),
                match section.custody_chain_intact {
                    Some(true) => "sim".to_string(),
                    Some(false) => "**não**".to_string(),
                    None => "não verificada".to_string(),
                },
            ]
        })
        .collect();
    doc.table(
        &[
            "Arquivo",
            "Formato",
            "Bytes",
            "BLAKE3",
            "SHA-256",
            "Importada em",
            "Status",
            "Cadeia íntegra",
        ],
        &rows,
    );
    for section in data
        .evidence
        .iter()
        .filter(|section| section.is_destroyed())
    {
        doc.para(&format!(
            "*{} foi descartada localmente a pedido do operador: o arquivo e os artefatos da análise não existem mais; o hash e a custódia abaixo permanecem registrados.*",
            esc(&section.evidence.original_filename)
        ));
    }
    provenance_table(doc, data);
    for section in &data.evidence {
        doc.h3(&format!(
            "Custódia — {}",
            section.evidence.original_filename
        ));
        if section.custody.is_empty() {
            doc.none("nenhum evento de custódia");
            continue;
        }
        let rows: Vec<Vec<String>> = section
            .custody
            .iter()
            .map(|entry| {
                vec![
                    entry.sequence.to_string(),
                    code(&entry.action),
                    esc(&entry.actor),
                    code(&entry.timestamp),
                    esc(&entry.reason),
                ]
            })
            .collect();
        doc.table(&["Nº", "Ação", "Responsável", "Quando", "Motivo"], &rows);
    }
}

fn declared(value: Option<&str>) -> String {
    value
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map_or_else(|| "não informado".to_string(), esc)
}

/// Proveniência declarada e tratamento de cada evidência (RB-002).
fn provenance_table(doc: &mut Doc, data: &DeliverableData) {
    doc.h3("Proveniência e tratamento");
    let rows: Vec<Vec<String>> = data
        .evidence
        .iter()
        .map(|section| {
            let evidence = &section.evidence;
            let text = |value: Option<&str>| {
                value
                    .map(str::trim)
                    .filter(|candidate| !candidate.is_empty())
                    .map(str::to_string)
            };
            // Ferramenta com a versão, quando houver.
            let tool = match (
                text(evidence.collection_tool.as_deref()),
                text(evidence.collection_tool_version.as_deref()),
            ) {
                (Some(tool), Some(version)) => Some(format!("{tool} {version}")),
                (Some(tool), None) => Some(tool),
                (None, Some(version)) => Some(format!("versão {version}")),
                (None, None) => None,
            };
            let collection = match (evidence.declared_collection_method(), tool) {
                (None, None) => "não informado".to_string(),
                (Some(method), None) => esc(method),
                (None, Some(tool)) => esc(&tool),
                (Some(method), Some(tool)) => format!("{} ({})", esc(method), esc(&tool)),
            };
            vec![
                esc(&evidence.original_filename),
                if evidence.is_analyzed_automatically() {
                    "analisada automaticamente (EVTX)".to_string()
                } else {
                    "somente preservada".to_string()
                },
                declared(evidence.declared_source()),
                declared(evidence.provided_by.as_deref()),
                collection,
                declared(evidence.declared_collected_at()),
                declared(evidence.authorization_reference.as_deref()),
                if evidence.contains_personal_data {
                    "sim".to_string()
                } else {
                    "não marcado".to_string()
                },
            ]
        })
        .collect();
    doc.table(
        &[
            "Arquivo",
            "Tratamento",
            "Origem",
            "Fornecida por",
            "Coleta",
            "Data da coleta",
            "Autorização",
            "Dado pessoal",
        ],
        &rows,
    );
}

fn timeline(doc: &mut Doc, data: &DeliverableData) {
    doc.h2("4. Linha do tempo");
    doc.h3("4.1 Marcos do incidente (caminho crítico)");
    let stages = sorted_stages(data);
    if stages.is_empty() {
        doc.none("nenhum estágio do caminho crítico registrado");
    } else {
        let mut rows = Vec::new();
        for stage in stages {
            rows.push(vec![
                field_text(stage, "sequence_index"),
                esc(field(stage, "stage_label")),
                esc(field(stage, "basis")),
                code(field(stage, "evidence_status")),
                doc.cite(&stage.id),
            ]);
        }
        doc.table(
            &["Nº", "Estágio", "Base", "Base documentada", "Origem"],
            &rows,
        );
    }

    doc.h3("4.2 Linha do tempo bruta por evidência");
    if data.evidence.is_empty() {
        doc.none("nenhuma evidência citada");
        return;
    }
    let rows: Vec<Vec<String>> = data
        .evidence
        .iter()
        .map(|section| {
            let name = esc(&section.evidence.original_filename);
            if section.is_destroyed() {
                return vec![
                    name,
                    "evidência descartada".into(),
                    "—".into(),
                    "—".into(),
                    "—".into(),
                ];
            }
            match &section.timeline {
                TimelineState::Summary(summary) => vec![
                    name,
                    summary.total_events.to_string(),
                    summary.first_event.as_deref().map_or("—".into(), code),
                    summary.last_event.as_deref().map_or("—".into(), code),
                    summary.warning_count.to_string(),
                ],
                TimelineState::NoCompletedJob => {
                    vec![
                        name,
                        "sem análise concluída".into(),
                        "—".into(),
                        "—".into(),
                        "—".into(),
                    ]
                }
                TimelineState::Unavailable => {
                    vec![
                        name,
                        "indisponível".into(),
                        "—".into(),
                        "—".into(),
                        "—".into(),
                    ]
                }
            }
        })
        .collect();
    doc.table(
        &["Evidência", "Eventos", "Primeiro", "Último", "Avisos"],
        &rows,
    );
    doc.para("*A linha do tempo completa permanece no banco da aplicação; este documento traz o resumo e os marcos.*");
    detection_section(doc, data);
}

/// 4.3: o que a detecção automática por regras fez em cada evidência. A
/// seção existe para que o leitor saiba o que foi **avaliado por pessoa** e o
/// que não foi: candidato pendente não é achado.
fn detection_section(doc: &mut Doc, data: &DeliverableData) {
    doc.h3("4.3 Detecção automática por regras (EVTX)");
    let rows: Vec<Vec<String>> = data
        .evidence
        .iter()
        .map(|section| {
            let name = esc(&section.evidence.original_filename);
            match &section.detection {
                DetectionSummary::Discarded => vec![
                    name,
                    "evidência descartada; os candidatos saíram com ela".into(),
                    "—".into(),
                    "—".into(),
                    "—".into(),
                    "—".into(),
                ],
                DetectionSummary::NotApplicable => vec![
                    name,
                    "não se aplica (só EVTX)".into(),
                    "—".into(),
                    "—".into(),
                    "—".into(),
                    "—".into(),
                ],
                DetectionSummary::NotRun => vec![
                    name,
                    "regras não executadas".into(),
                    "—".into(),
                    "—".into(),
                    "—".into(),
                    "—".into(),
                ],
                DetectionSummary::Run {
                    confirmed,
                    rejected,
                    pending,
                    rule_sets,
                } => vec![
                    name,
                    if rule_sets.is_empty() {
                        "executadas; nenhum candidato".into()
                    } else {
                        rule_sets
                            .iter()
                            .map(|set| code(set))
                            .collect::<Vec<_>>()
                            .join(", ")
                    },
                    confirmed.to_string(),
                    rejected.to_string(),
                    pending.to_string(),
                    (confirmed + rejected + pending).to_string(),
                ],
            }
        })
        .collect();
    if rows.is_empty() {
        doc.none("nenhuma evidência citada");
        return;
    }
    doc.table(
        &[
            "Evidência",
            "Conjunto de regras",
            "Confirmados",
            "Rejeitados",
            "Pendentes",
            "Total",
        ],
        &rows,
    );
    doc.para("*As regras internas apontam hipóteses; só o analista as transforma em achado. Os candidatos **confirmados** aparecem como achados, técnicas ATT&CK e IOCs, com a referência dos eventos de origem (que permanecem no incidente mesmo depois do descarte da evidência). Os **pendentes** não foram avaliados e não constam como achado. Os **rejeitados** ficam registrados com o motivo. A ausência de candidato não prova ausência de ataque: a regra só enxerga o que o log registrou, e cada candidato declara o que não prova.*");
}

fn mapping_line(doc: &mut Doc, mapping: &AnalyticalResultResponse) -> String {
    let sub = field(mapping, "sub_technique_id");
    let technique = if sub.is_empty() {
        code(field(mapping, "technique_id"))
    } else {
        format!("{} ({})", code(field(mapping, "technique_id")), code(sub))
    };
    let basis = field(mapping, "basis");
    let basis = if basis.trim().is_empty() {
        "sem base documentada".to_string()
    } else {
        esc(basis)
    };
    format!(
        "{technique} — {basis} · confiança {} · origem {}",
        code(mapping.confidence.as_deref().unwrap_or("")),
        doc.cite(&mapping.id)
    )
}

fn tactic_sections(doc: &mut Doc, data: &DeliverableData) {
    doc.h2("5. Vetor inicial, movimentação lateral, escalonamento de privilégios e exfiltração");
    let mut mappings: Vec<&AnalyticalResultResponse> = data.attack_mappings.iter().collect();
    mappings.sort_by(|a, b| {
        tactic_rank(field(a, "tactic"))
            .cmp(&tactic_rank(field(b, "tactic")))
            .then_with(|| field(a, "technique_id").cmp(field(b, "technique_id")))
            .then_with(|| a.id.cmp(&b.id))
    });
    let groups = [
        ("INITIAL_ACCESS", "Vetor inicial"),
        ("LATERAL_MOVEMENT", "Movimentação lateral"),
        ("PRIVILEGE_ESCALATION", "Escalonamento de privilégios"),
        ("EXFILTRATION", "Possíveis exfiltrações"),
    ];
    for (tactic, heading) in groups {
        doc.h3(heading);
        let items: Vec<&&AnalyticalResultResponse> = mappings
            .iter()
            .filter(|mapping| field(mapping, "tactic") == tactic)
            .collect();
        if items.is_empty() {
            doc.none("nenhuma técnica registrada nesta tática; não identificada nas evidências analisadas ou não analisada");
            continue;
        }
        let lines: Vec<String> = items
            .into_iter()
            .map(|mapping| mapping_line(doc, mapping))
            .collect();
        doc.bullets(&lines);
    }
    let others: Vec<&&AnalyticalResultResponse> = mappings
        .iter()
        .filter(|mapping| {
            !groups
                .iter()
                .any(|(tactic, _)| *tactic == field(mapping, "tactic"))
        })
        .collect();
    if !others.is_empty() {
        doc.h3("Outras táticas");
        let rows: Vec<Vec<String>> = others
            .into_iter()
            .map(|mapping| vec![code(field(mapping, "tactic")), mapping_line(doc, mapping)])
            .collect();
        doc.table(&["Tática", "Técnica"], &rows);
    }
}

fn critical_path(doc: &mut Doc, data: &DeliverableData) {
    doc.h2("6. Caminho crítico do incidente");
    let stages = sorted_stages(data);
    if stages.is_empty() {
        doc.none("nenhum estágio do caminho crítico registrado");
        return;
    }
    let label_of = |id: &str| -> String {
        data.critical_path
            .stages
            .iter()
            .find(|stage| stage.id == id)
            .map_or_else(
                || code(&short(id)),
                |stage| esc(field(stage, "stage_label")),
            )
    };
    let mut edges: Vec<&crate::service::findings::CriticalPathEdge> =
        data.critical_path.edges.iter().collect();
    edges.sort_by(|a, b| {
        let index = |id: &str| {
            data.critical_path
                .stages
                .iter()
                .find(|stage| stage.id == id)
                .map_or(i64::MAX, stage_index)
        };
        index(&a.from)
            .cmp(&index(&b.from))
            .then_with(|| a.from.cmp(&b.from))
            .then_with(|| a.to.cmp(&b.to))
    });
    if edges.is_empty() {
        doc.para("*Os estágios acima não têm ligações de sequência registradas.*");
        return;
    }
    let lines: Vec<String> = edges
        .into_iter()
        .map(|edge| {
            format!(
                "{} → {} · {} → {}",
                label_of(&edge.from),
                label_of(&edge.to),
                doc.cite(&edge.from),
                doc.cite(&edge.to)
            )
        })
        .collect();
    doc.bullets(&lines);
}

fn impact(doc: &mut Doc, data: &DeliverableData) {
    doc.h2("7. Impacto");
    let Some(impact) = &data.impact else {
        doc.none("impacto não registrado");
        return;
    };
    let financial = impact
        .payload
        .get("financial_impact")
        .map(super::value_text)
        .unwrap_or_default();
    let lines = vec![
        format!(
            "Confidencialidade: {}",
            code(field(impact, "confidentiality_impact"))
        ),
        format!("Integridade: {}", code(field(impact, "integrity_impact"))),
        format!(
            "Disponibilidade: {}",
            code(field(impact, "availability_impact"))
        ),
        format!(
            "Ativos afetados: {}",
            esc(&field_text(impact, "affected_assets"))
        ),
        format!(
            "Dados afetados: {}",
            esc(&field_text(impact, "affected_data"))
        ),
        format!(
            "Interrupção operacional: {}",
            esc(field(impact, "operational_disruption"))
        ),
        format!(
            "Impacto financeiro: {}",
            if financial.trim().is_empty() {
                "não informado".to_string()
            } else {
                esc(&financial)
            }
        ),
        format!("Origem: {}", doc.cite(&impact.id)),
    ];
    doc.bullets(&lines);
}

fn causes(doc: &mut Doc, data: &DeliverableData) {
    doc.h2("8. Causa, confiança e lacunas");
    let causes = sorted_causes(data);
    if causes.is_empty() {
        doc.para("*Nenhuma conclusão de causa registrada: este relatório não afirma causa-raiz.*");
        return;
    }
    for cause in causes {
        doc.h3(&format!(
            "{} — confiança {}",
            cause_label(field(cause, "level")),
            cause.confidence.as_deref().unwrap_or("não informada")
        ));
        let mut lines = vec![format!("Conclusão: {}", esc(field(cause, "statement")))];
        let basis = field(cause, "basis");
        if !basis.trim().is_empty() {
            lines.push(format!("Base: {}", esc(basis)));
        }
        for id in strings(cause, "finding_ids") {
            let cited = doc.cite(&id);
            lines.push(format!(
                "Sustentada pelo achado: {} · origem {cited}",
                finding_label(data, &id)
            ));
        }
        let gaps = strings(cause, "gaps");
        if gaps.is_empty() {
            lines.push("Lacunas: nenhuma declarada pelo analista.".to_string());
        } else {
            for gap in gaps {
                lines.push(format!("Lacuna: {}", esc(&gap)));
            }
        }
        lines.push(format!("Origem: {}", doc.cite(&cause.id)));
        doc.bullets(&lines);
    }
}

fn conclusions(doc: &mut Doc, data: &DeliverableData) {
    doc.h2("9. Conclusões");
    let mut lines = Vec::new();
    let causes = sorted_causes(data);
    if causes.is_empty() {
        lines.push("Sem conclusão de causa registrada.".to_string());
    }
    for cause in causes {
        lines.push(format!(
            "{} (confiança {}): {}",
            cause_label(field(cause, "level")),
            code(cause.confidence.as_deref().unwrap_or("")),
            esc(field(cause, "statement"))
        ));
    }
    lines.push(format!(
        "Achados registrados: {} (detalhe no Entregável 2).",
        data.findings.findings.len()
    ));
    lines.push("Maturidade NIST e recomendações: Entregável 3. Hardening: Entregável 4. Criptografia e proteção de dados: Entregável 5.".to_string());
    doc.bullets(&lines);
}

fn iocs(doc: &mut Doc, data: &DeliverableData) {
    doc.h2("Apêndice A. Indicadores de comprometimento (IOCs)");
    if data.iocs.is_empty() {
        doc.none("nenhum IOC registrado");
        return;
    }
    let mut iocs: Vec<&AnalyticalResultResponse> = data.iocs.iter().collect();
    iocs.sort_by(|a, b| {
        field(a, "ioc_type")
            .cmp(field(b, "ioc_type"))
            .then_with(|| field(a, "normalized_value").cmp(field(b, "normalized_value")))
            .then_with(|| a.id.cmp(&b.id))
    });
    let mut rows = Vec::new();
    for ioc in iocs {
        rows.push(vec![
            code(field(ioc, "ioc_type")),
            code(field(ioc, "normalized_value")),
            code(field(ioc, "first_seen")),
            code(field(ioc, "last_seen")),
            field_text(ioc, "sighting_count"),
            evidence_name(data, ioc.evidence_id.as_deref()),
            doc.cite(&ioc.id),
        ]);
    }
    doc.table(
        &[
            "Tipo",
            "Valor",
            "Primeira ocorrência",
            "Última",
            "Ocorrências",
            "Evidência",
            "Origem",
        ],
        &rows,
    );
}
