//! F1-30 (RF-053) — visão NIST CSF 2.0 por Function (GV, ID, PR, DE, RS,
//! RC): contagens por status de implementação e por prioridade de gap.
//!
//! Só **contagens**. O NIST CSF 2.0 não define pontuação oficial, e a
//! especificação (§3) veta apresentar número como score: nada aqui calcula
//! média, percentual ou nota. O Tier é contexto de governança do profile
//! inteiro, informado pelo analista (`nist_profiles::Tier`): não existe um
//! Tier por Function, então esta visão não inventa um — o serviço o devolve
//! junto, uma vez por profile.
//!
//! São funções puras, sem acesso ao banco: o mesmo conjunto de entradas dá
//! sempre o mesmo resultado (RF-010), e o Entregável 3 (F1-07) reaproveita
//! exatamente isto.

use crate::persistence::nist_profiles::{ImplementationStatus, ProfileEntryRecord};
use serde::Serialize;

/// As seis Functions do CSF 2.0, na ordem oficial.
pub const FUNCTIONS: [(&str, &str); 6] = [
    ("GV", "Governar"),
    ("ID", "Identificar"),
    ("PR", "Proteger"),
    ("DE", "Detectar"),
    ("RS", "Responder"),
    ("RC", "Recuperar"),
];

/// Contagem de subcategorias do profile Current por status.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct StatusCounts {
    pub implemented: usize,
    pub partially_implemented: usize,
    pub not_implemented: usize,
    pub not_assessed: usize,
    pub not_applicable: usize,
}

impl StatusCounts {
    fn add(&mut self, status: ImplementationStatus) {
        let slot = match status {
            ImplementationStatus::Implemented => &mut self.implemented,
            ImplementationStatus::PartiallyImplemented => &mut self.partially_implemented,
            ImplementationStatus::NotImplemented => &mut self.not_implemented,
            ImplementationStatus::NotAssessed => &mut self.not_assessed,
            ImplementationStatus::NotApplicable => &mut self.not_applicable,
        };
        *slot += 1;
    }
}

/// Contagem de gaps vigentes por prioridade.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct GapCounts {
    pub high: usize,
    pub medium: usize,
    pub low: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FunctionSummary {
    pub function: String,
    pub label: String,
    /// Subcategorias do profile Current, por status.
    pub current: StatusCounts,
    /// Subcategorias que o profile Target elege (exclui `NOT_APPLICABLE`).
    pub target_subcategories: usize,
    pub gaps: GapCounts,
}

/// A Function de uma subcategoria (`PR.AA-03` → `PR`). Os ids são
/// validados na criação do profile (`is_valid_subcategory_id`), então toda
/// entrada persistida cai numa das seis.
fn function_index(subcategory_id: &str) -> Option<usize> {
    let prefix = subcategory_id.split('.').next()?;
    FUNCTIONS.iter().position(|(code, _)| *code == prefix)
}

/// Agrega por Function: o profile Current por status, o Target por
/// subcategorias elegidas e os gaps vigentes (`(subcategoria, prioridade)`)
/// por prioridade. Devolve **sempre as seis Functions**, na ordem oficial,
/// inclusive as sem dados (contagens zero são fato, não omissão).
pub fn summarize_by_function(
    current: &[ProfileEntryRecord],
    target: &[ProfileEntryRecord],
    gaps: &[(&str, &str)],
) -> Vec<FunctionSummary> {
    let mut summaries: Vec<FunctionSummary> = FUNCTIONS
        .iter()
        .map(|(code, label)| FunctionSummary {
            function: (*code).to_string(),
            label: (*label).to_string(),
            current: StatusCounts::default(),
            target_subcategories: 0,
            gaps: GapCounts::default(),
        })
        .collect();

    for entry in current {
        if let (Some(index), Some(status)) = (
            function_index(&entry.subcategory_id),
            ImplementationStatus::from_str(&entry.implementation_status),
        ) {
            summaries[index].current.add(status);
        }
    }
    for entry in target {
        if let Some(index) = function_index(&entry.subcategory_id)
            && entry.implementation_status != ImplementationStatus::NotApplicable.as_str()
        {
            summaries[index].target_subcategories += 1;
        }
    }
    for (subcategory_id, priority) in gaps {
        if let Some(index) = function_index(subcategory_id) {
            let counts = &mut summaries[index].gaps;
            match *priority {
                "HIGH" => counts.high += 1,
                "MEDIUM" => counts.medium += 1,
                "LOW" => counts.low += 1,
                _ => {}
            }
        }
    }
    summaries
}

/// Se os gaps vigentes ainda correspondem aos profiles vigentes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GapAnalysisState {
    /// Falta o Current ou o Target: não há o que comparar.
    MissingProfile,
    /// Nenhum gap vigente: a análise ainda não foi rodada **ou** o Current
    /// já atinge o Target. Os dados não distinguem os dois casos (rodar a
    /// análise sem gaps não grava resultado), então a visão diz as duas
    /// coisas em vez de escolher uma.
    NoGapsRecorded,
    /// Todos os gaps vigentes vêm dos profiles vigentes.
    UpToDate,
    /// Algum gap vigente foi calculado com uma versão anterior de um dos
    /// profiles: as contagens de gap não refletem os profiles atuais.
    Stale,
}

/// `gap_provenance` traz, por gap vigente, os ids dos profiles Current e
/// Target com que foi calculado (campos da proveniência gravada em
/// `run_gap_analysis`). Toda rodada regrava todos os gaps, então depois de
/// recalcular todos carregam os ids vigentes.
pub fn gap_analysis_state(
    current_profile_id: Option<&str>,
    target_profile_id: Option<&str>,
    gap_provenance: &[(Option<&str>, Option<&str>)],
) -> GapAnalysisState {
    let (Some(current_id), Some(target_id)) = (current_profile_id, target_profile_id) else {
        return GapAnalysisState::MissingProfile;
    };
    if gap_provenance.is_empty() {
        return GapAnalysisState::NoGapsRecorded;
    }
    let all_match = gap_provenance
        .iter()
        .all(|(current, target)| *current == Some(current_id) && *target == Some(target_id));
    if all_match {
        GapAnalysisState::UpToDate
    } else {
        GapAnalysisState::Stale
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(subcategory_id: &str, status: &str) -> ProfileEntryRecord {
        ProfileEntryRecord {
            subcategory_id: subcategory_id.into(),
            subcategory_label: "rótulo".into(),
            implementation_status: status.into(),
            confidence: None,
        }
    }

    #[test]
    fn counts_current_status_target_and_gaps_per_function_in_official_order() {
        let current = [
            entry("GV.OC-01", "IMPLEMENTED"),
            entry("GV.OC-02", "NOT_IMPLEMENTED"),
            entry("PR.AA-03", "PARTIALLY_IMPLEMENTED"),
            entry("PR.AA-05", "NOT_ASSESSED"),
            entry("PR.DS-01", "NOT_APPLICABLE"),
            entry("DE.CM-01", "NOT_IMPLEMENTED"),
        ];
        let target = [
            entry("GV.OC-01", "IMPLEMENTED"),
            entry("GV.OC-02", "IMPLEMENTED"),
            entry("PR.AA-03", "IMPLEMENTED"),
            entry("PR.DS-01", "NOT_APPLICABLE"),
            entry("DE.CM-01", "IMPLEMENTED"),
        ];
        let gaps = [
            ("GV.OC-02", "MEDIUM"),
            ("PR.AA-03", "MEDIUM"),
            ("DE.CM-01", "HIGH"),
        ];

        let summary = summarize_by_function(&current, &target, &gaps);
        let order: Vec<&str> = summary.iter().map(|s| s.function.as_str()).collect();
        assert_eq!(order, vec!["GV", "ID", "PR", "DE", "RS", "RC"]);

        let gv = &summary[0];
        assert_eq!(gv.label, "Governar");
        assert_eq!(gv.current.implemented, 1);
        assert_eq!(gv.current.not_implemented, 1);
        assert_eq!(gv.target_subcategories, 2);
        assert_eq!(
            gv.gaps,
            GapCounts {
                high: 0,
                medium: 1,
                low: 0
            }
        );

        let pr = &summary[2];
        assert_eq!(pr.current.partially_implemented, 1);
        assert_eq!(pr.current.not_assessed, 1);
        assert_eq!(pr.current.not_applicable, 1);
        assert_eq!(
            pr.target_subcategories, 1,
            "NOT_APPLICABLE não é elegida pelo Target"
        );

        let de = &summary[3];
        assert_eq!(de.gaps.high, 1);
        assert_eq!(de.current.not_implemented, 1);
    }

    #[test]
    fn functions_without_data_are_listed_with_zero_counts() {
        let summary = summarize_by_function(&[], &[], &[]);
        assert_eq!(summary.len(), 6);
        for function in &summary {
            assert_eq!(function.current, StatusCounts::default());
            assert_eq!(function.target_subcategories, 0);
            assert_eq!(function.gaps, GapCounts::default());
        }
    }

    #[test]
    fn unknown_prefixes_and_priorities_are_ignored_not_misfiled() {
        let summary = summarize_by_function(
            &[entry("ZZ.AA-01", "IMPLEMENTED")],
            &[entry("ZZ.AA-01", "IMPLEMENTED")],
            &[("ZZ.AA-01", "HIGH"), ("GV.OC-01", "URGENT")],
        );
        let total_current: usize = summary.iter().map(|s| s.current.implemented).sum();
        let total_gaps: usize = summary
            .iter()
            .map(|s| s.gaps.high + s.gaps.medium + s.gaps.low)
            .sum();
        assert_eq!((total_current, total_gaps), (0, 0));
    }

    #[test]
    fn gap_state_distinguishes_missing_profile_no_gaps_up_to_date_and_stale() {
        assert_eq!(
            gap_analysis_state(None, Some("t1"), &[]),
            GapAnalysisState::MissingProfile
        );
        assert_eq!(
            gap_analysis_state(Some("c1"), None, &[(Some("c1"), Some("t1"))]),
            GapAnalysisState::MissingProfile
        );
        assert_eq!(
            gap_analysis_state(Some("c1"), Some("t1"), &[]),
            GapAnalysisState::NoGapsRecorded
        );
        assert_eq!(
            gap_analysis_state(
                Some("c1"),
                Some("t1"),
                &[(Some("c1"), Some("t1")), (Some("c1"), Some("t1"))]
            ),
            GapAnalysisState::UpToDate
        );
        // Um gap calculado com a versão anterior do Current.
        assert_eq!(
            gap_analysis_state(
                Some("c2"),
                Some("t1"),
                &[(Some("c2"), Some("t1")), (Some("c1"), Some("t1"))]
            ),
            GapAnalysisState::Stale
        );
        // Proveniência ausente não é "em dia".
        assert_eq!(
            gap_analysis_state(Some("c1"), Some("t1"), &[(None, None)]),
            GapAnalysisState::Stale
        );
    }
}
