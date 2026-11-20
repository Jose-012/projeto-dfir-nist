//! Renderização SVG determinística do caminho crítico (F1-39/ADR-0030).
//!
//! Não há biblioteca gráfica, JavaScript, recurso externo nem timestamp na
//! saída. A lista em `<desc>` é a alternativa textual integral ao desenho.

use super::findings::CriticalPathView;
use crate::error::{AppError, AppResult};
use crate::service::AnalyticalResultResponse;
use crate::text_safety::neutralize;
use std::collections::BTreeMap;

const WIDTH: u32 = 760;
const NODE_X: u32 = 50;
const NODE_WIDTH: u32 = 660;
const NODE_HEIGHT: u32 = 104;
const ROW_HEIGHT: u32 = 142;
const TOP: u32 = 34;
const BOTTOM: u32 = 34;

#[derive(Debug)]
struct Stage<'a> {
    id: &'a str,
    sequence_index: i64,
    label: &'a str,
    basis: &'a str,
    confidence: &'a str,
    evidence_id: Option<&'a str>,
}

fn invalid(result_id: &str, field: &str) -> AppError {
    AppError::Config(format!(
        "critical path result {result_id} has invalid or missing {field}"
    ))
}

fn stage_from_record(record: &AnalyticalResultResponse) -> AppResult<Stage<'_>> {
    if record.id.trim().is_empty() {
        return Err(invalid("(empty)", "id"));
    }
    let sequence_index = record
        .payload
        .get("sequence_index")
        .and_then(serde_json::Value::as_i64)
        .ok_or_else(|| invalid(&record.id, "sequence_index"))?;
    let label = record
        .payload
        .get("stage_label")
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| invalid(&record.id, "stage_label"))?;
    let basis = record
        .payload
        .get("basis")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("não informada");
    Ok(Stage {
        id: &record.id,
        sequence_index,
        label,
        basis,
        confidence: record.confidence.as_deref().unwrap_or("não informada"),
        evidence_id: record.evidence_id.as_deref(),
    })
}

fn valid_xml_char(ch: char) -> bool {
    matches!(
        ch,
        '\u{9}'
            | '\u{A}'
            | '\u{D}'
            | '\u{20}'..='\u{D7FF}'
            | '\u{E000}'..='\u{FFFD}'
            | '\u{10000}'..='\u{10FFFF}'
    )
}

fn xml(raw: &str) -> String {
    let neutralized = neutralize(raw);
    let mut out = String::with_capacity(neutralized.len());
    for ch in neutralized.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            ch if valid_xml_char(ch) => out.push(ch),
            ch => out.push_str(&format!("⟦U+{:04X}⟧", u32::from(ch))),
        }
    }
    out
}

fn shortened(raw: &str) -> String {
    const MAX_CHARS: usize = 72;
    let mut chars = raw.chars();
    let prefix: String = chars.by_ref().take(MAX_CHARS).collect();
    if chars.next().is_some() {
        format!("{prefix}…")
    } else {
        prefix
    }
}

fn textual_alternative(stages: &[Stage<'_>], edges: &[(usize, usize)]) -> String {
    if stages.is_empty() {
        return "Nenhum estágio do caminho crítico registrado.".into();
    }
    let mut text = format!("{} estágio(s) do caminho crítico. ", stages.len());
    for stage in stages {
        text.push_str(&format!(
            "Estágio {}: sequência {}; rótulo {}; confiança {}; evidência {}; base {}. ",
            stage.id,
            stage.sequence_index,
            stage.label,
            stage.confidence,
            stage.evidence_id.unwrap_or("não informada"),
            stage.basis
        ));
    }
    if edges.is_empty() {
        text.push_str("Sem ligações SEQUENCE_NEXT.");
    } else {
        text.push_str("Ligações SEQUENCE_NEXT: ");
        for (index, (from, to)) in edges.iter().enumerate() {
            if index > 0 {
                text.push_str("; ");
            }
            text.push_str(&format!("{} -> {}", stages[*from].id, stages[*to].id));
        }
        text.push('.');
    }
    text
}

/// Renderiza a visão atual sem consultar nem alterar o banco. Permutar
/// `stages`/`edges` na entrada não muda os bytes da saída.
pub fn render(view: &CriticalPathView) -> AppResult<String> {
    let mut stages = view
        .stages
        .iter()
        .map(stage_from_record)
        .collect::<AppResult<Vec<_>>>()?;
    stages.sort_by(|a, b| {
        a.sequence_index
            .cmp(&b.sequence_index)
            .then_with(|| a.id.cmp(b.id))
    });

    let mut positions = BTreeMap::new();
    for (index, stage) in stages.iter().enumerate() {
        if positions.insert(stage.id, index).is_some() {
            return Err(AppError::Config(format!(
                "critical path contains duplicate result id {}",
                stage.id
            )));
        }
    }
    let mut edges = Vec::new();
    for edge in &view.edges {
        let from = positions.get(edge.from.as_str()).copied().ok_or_else(|| {
            AppError::Config(format!(
                "critical path edge references missing source result {}",
                edge.from
            ))
        })?;
        let to = positions.get(edge.to.as_str()).copied().ok_or_else(|| {
            AppError::Config(format!(
                "critical path edge references missing target result {}",
                edge.to
            ))
        })?;
        edges.push((from, to));
    }
    edges.sort_by(|a, b| {
        stages[a.0]
            .id
            .cmp(stages[b.0].id)
            .then_with(|| stages[a.1].id.cmp(stages[b.1].id))
    });

    let height = if stages.is_empty() {
        180
    } else {
        TOP + BOTTOM + stages.len() as u32 * ROW_HEIGHT
    };
    let alternative = textual_alternative(&stages, &edges);
    let mut svg = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{WIDTH}\" height=\"{height}\" viewBox=\"0 0 {WIDTH} {height}\" role=\"img\" aria-labelledby=\"critical-path-title critical-path-desc\">\n<title id=\"critical-path-title\">Caminho crítico</title>\n<desc id=\"critical-path-desc\">{}</desc>\n<defs><marker id=\"arrow\" markerWidth=\"8\" markerHeight=\"8\" refX=\"7\" refY=\"4\" orient=\"auto\" markerUnits=\"strokeWidth\"><path d=\"M0,0 L8,4 L0,8 Z\" fill=\"#555555\"/></marker></defs>\n<rect width=\"100%\" height=\"100%\" fill=\"#ffffff\"/>\n",
        xml(&alternative)
    );

    for (from, to) in &edges {
        let from_y = TOP + *from as u32 * ROW_HEIGHT + NODE_HEIGHT;
        let to_y = TOP + *to as u32 * ROW_HEIGHT;
        let middle = (from_y + to_y) / 2;
        svg.push_str(&format!(
            "<path d=\"M {},{} V {} H {} V {}\" fill=\"none\" stroke=\"#555555\" stroke-width=\"2\" marker-end=\"url(#arrow)\"/>\n",
            NODE_X + NODE_WIDTH / 2,
            from_y,
            middle,
            NODE_X + NODE_WIDTH / 2,
            to_y
        ));
    }

    for (index, stage) in stages.iter().enumerate() {
        let y = TOP + index as u32 * ROW_HEIGHT;
        svg.push_str(&format!(
            "<g><title>{}</title><rect x=\"{NODE_X}\" y=\"{y}\" width=\"{NODE_WIDTH}\" height=\"{NODE_HEIGHT}\" rx=\"8\" fill=\"#f5f7fa\" stroke=\"#334455\"/><text x=\"{}\" y=\"{}\" font-family=\"monospace\" font-size=\"14\" fill=\"#334455\">{}</text><text x=\"{}\" y=\"{}\" font-family=\"sans-serif\" font-size=\"18\" font-weight=\"bold\" fill=\"#112233\">{}</text><text x=\"{}\" y=\"{}\" font-family=\"monospace\" font-size=\"13\" fill=\"#334455\">{}</text></g>\n",
            xml(&format!("{} — {}", stage.id, stage.label)),
            NODE_X + 16,
            y + 23,
            xml(&format!(
                "{} · sequência {} · confiança {}",
                stage.id, stage.sequence_index, stage.confidence
            )),
            NODE_X + 16,
            y + 53,
            xml(&shortened(stage.label)),
            NODE_X + 16,
            y + 80,
            xml(&shortened(&format!(
                "Evidência: {}",
                stage.evidence_id.unwrap_or("não informada")
            )))
        ));
    }
    if stages.is_empty() {
        svg.push_str(
            "<text x=\"50\" y=\"90\" font-family=\"sans-serif\" font-size=\"18\" fill=\"#334455\">Nenhum estágio registrado.</text>\n",
        );
    }
    svg.push_str("</svg>\n");
    Ok(svg)
}

#[cfg(test)]
mod tests {
    use super::render;
    use crate::error::AppError;
    use crate::service::AnalyticalResultResponse;
    use crate::service::findings::{CriticalPathEdge, CriticalPathView};

    fn stage(id: &str, sequence: i64, label: &str) -> AnalyticalResultResponse {
        AnalyticalResultResponse {
            id: id.into(),
            result_type: "CRITICAL_PATH_LINK".into(),
            correlation_key: format!("critical-path-stage:inc-1:{sequence}"),
            evidence_id: Some("ev-fixture".into()),
            incident_id: Some("inc-fixture".into()),
            analysis_job_id: None,
            rule_or_model_id: None,
            rule_or_model_version: None,
            confidence: Some("HIGH".into()),
            payload: serde_json::json!({
                "kind": "CRITICAL_PATH_STAGE",
                "sequence_index": sequence,
                "stage_label": label,
                "basis": "base <analítica> & revisada",
                "evidence_status": "DOCUMENTED"
            }),
            provenance: serde_json::Value::Null,
            supersedes: None,
            created_at: "2026-10-16T00:00:00Z".into(),
        }
    }

    fn edge(from: &str, to: &str) -> CriticalPathEdge {
        CriticalPathEdge {
            from: from.into(),
            to: to.into(),
        }
    }

    #[test]
    fn permutations_have_identical_bytes_and_xml_is_escaped() {
        let first = stage(
            "stage-a",
            1,
            "Phishing <script>alert('x')</script> & \"abertura\"",
        );
        let second = stage("stage-b", 2, "Credencial\u{202E}capturada");
        let forward = CriticalPathView {
            stages: vec![second.clone(), first.clone()],
            edges: vec![edge("stage-b", "stage-a"), edge("stage-a", "stage-b")],
        };
        let reverse = CriticalPathView {
            stages: vec![first, second],
            edges: vec![edge("stage-a", "stage-b"), edge("stage-b", "stage-a")],
        };

        let left = render(&forward).expect("render forward");
        let right = render(&reverse).expect("render reverse");
        assert_eq!(left, right, "ordem de stages/edges não pode mudar bytes");
        assert!(left.contains("&lt;script&gt;alert(&apos;x&apos;)&lt;/script&gt;"));
        assert!(left.contains("&amp;"));
        assert!(left.contains("&quot;abertura&quot;"));
        assert!(left.contains("⟦U+202E⟧"));
        assert!(!left.contains("<script>"));
        assert!(!left.contains("href="));
        assert!(left.contains("<desc id=\"critical-path-desc\">"));
        assert!(left.contains("stage-a -&gt; stage-b"));
    }

    #[test]
    fn malformed_stage_and_missing_edge_target_fail_with_result_ids() {
        let malformed = stage("stage-good", 1, "válido");
        let mut missing_label = stage("stage-bad", 2, "removido");
        missing_label
            .payload
            .as_object_mut()
            .expect("payload object")
            .remove("stage_label");
        let error = render(&CriticalPathView {
            stages: vec![malformed.clone(), missing_label],
            edges: vec![],
        })
        .expect_err("missing label must fail");
        assert!(
            matches!(&error, AppError::Config(message) if message.contains("stage-bad") && message.contains("stage_label")),
            "{error:?}"
        );

        let error = render(&CriticalPathView {
            stages: vec![malformed],
            edges: vec![edge("stage-good", "stage-missing")],
        })
        .expect_err("missing target must fail");
        assert!(
            matches!(&error, AppError::Config(message) if message.contains("stage-missing")),
            "{error:?}"
        );
    }

    #[test]
    fn duplicate_ids_and_invalid_xml_controls_are_not_hidden() {
        let duplicate = stage("stage-a", 1, "primeiro");
        let error = render(&CriticalPathView {
            stages: vec![duplicate.clone(), duplicate],
            edges: vec![],
        })
        .expect_err("duplicate id must fail");
        assert!(
            matches!(&error, AppError::Config(message) if message.contains("duplicate result id stage-a")),
            "{error:?}"
        );

        let control = stage("stage-control", 1, "inválido\u{0001}");
        let svg = render(&CriticalPathView {
            stages: vec![control],
            edges: vec![],
        })
        .expect("control becomes visible marker");
        assert!(svg.contains("⟦U+0001⟧"));
        assert!(!svg.contains('\u{0001}'));
    }

    #[test]
    fn empty_graph_has_a_textual_alternative() {
        let svg = render(&CriticalPathView {
            stages: vec![],
            edges: vec![],
        })
        .expect("empty graph");
        assert!(svg.contains("Nenhum estágio do caminho crítico registrado."));
        assert!(svg.contains("Nenhum estágio registrado."));
        assert!(!svg.contains("<path d=\"M "));
    }
}
