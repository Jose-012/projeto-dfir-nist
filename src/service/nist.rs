//! F1-03 — Current/Target Profile e Gap Analysis, camada de serviço.
//! Porta direta dos handlers `create_nist_profile`/`list_nist_profiles`/
//! `run_nist_gap_analysis`/`get_nist_gap_analysis` de `web/mod.rs`.

use super::{AnalyticalResultResponse, AppServices, current_results_for_incident, incident_exists};
use crate::error::{AppError, AppResult};
use crate::persistence::{analytical_results, nist_functions, nist_profiles};
use serde::Serialize;

/// Uma entrada de profile tal como o operador a preenche na tela (strings —
/// a validação/parsing dos enums acontece em [`create_profile`], igual ao
/// `CreateProfileEntryRequest` original).
#[derive(Debug, Clone, Default)]
pub struct ProfileEntryInput {
    pub subcategory_id: String,
    pub subcategory_label: String,
    pub implementation_status: String,
    pub justification: Option<String>,
    pub evidence_reference: Option<String>,
    pub confidence: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ProfileVersionView {
    pub version_label: String,
    pub source: String,
    pub tier: Option<String>,
    pub tier_justification: Option<String>,
    pub created_at: String,
    pub entries: Vec<nist_profiles::ProfileEntryRecord>,
}

#[derive(Debug, Clone)]
pub struct ProfileHistoryView {
    pub profile_type: nist_profiles::ProfileType,
    pub versions: Vec<ProfileVersionView>,
}

#[allow(clippy::too_many_arguments)]
pub async fn create_profile(
    services: &AppServices,
    incident_id: &str,
    profile_type: &str,
    version_label: &str,
    source: &str,
    tier: Option<&str>,
    tier_justification: Option<&str>,
    entries: &[ProfileEntryInput],
) -> AppResult<String> {
    incident_exists(&services.db, incident_id).await?;

    let profile_type = nist_profiles::ProfileType::from_str(profile_type)
        .ok_or_else(|| AppError::Config("profile_type must be CURRENT or TARGET".into()))?;
    let tier = match tier {
        Some(raw) => Some(
            nist_profiles::Tier::from_str(raw)
                .ok_or_else(|| AppError::Config(format!("invalid tier: {raw}")))?,
        ),
        None => None,
    };

    let mut owned_entries = Vec::with_capacity(entries.len());
    for entry in entries {
        let implementation_status =
            nist_profiles::ImplementationStatus::from_str(&entry.implementation_status)
                .ok_or_else(|| {
                    AppError::Config(format!(
                        "invalid implementation_status: {}",
                        entry.implementation_status
                    ))
                })?;
        let confidence = match entry.confidence.as_deref() {
            Some(raw) => Some(
                analytical_results::Confidence::from_str(raw)
                    .ok_or_else(|| AppError::Config(format!("invalid confidence: {raw}")))?,
            ),
            None => None,
        };
        owned_entries.push(nist_profiles::NewProfileEntry {
            subcategory_id: &entry.subcategory_id,
            subcategory_label: &entry.subcategory_label,
            implementation_status,
            justification: entry.justification.as_deref(),
            evidence_reference: entry.evidence_reference.as_deref(),
            confidence,
        });
    }

    let profile_id = nist_profiles::insert_profile(
        &services.db,
        nist_profiles::NewProfile {
            incident_id,
            profile_type,
            version_label,
            source,
            tier,
            tier_justification,
            entries: &owned_entries,
            actor_id: None,
        },
    )
    .await
    .map_err(AppError::Database)?;

    Ok(profile_id)
}

/// Histórico completo (todas as versões, mais recente por último) de um
/// profile CURRENT ou TARGET, entradas incluídas.
pub async fn profile_history(
    services: &AppServices,
    incident_id: &str,
    profile_type: nist_profiles::ProfileType,
) -> AppResult<ProfileHistoryView> {
    incident_exists(&services.db, incident_id).await?;

    let history = nist_profiles::profile_history(&services.db, incident_id, profile_type).await?;
    let mut versions = Vec::with_capacity(history.len());
    for profile in history {
        let entries = nist_profiles::profile_entries(&services.db, &profile.id).await?;
        versions.push(ProfileVersionView {
            version_label: profile.version_label,
            source: profile.source,
            tier: profile.tier,
            tier_justification: profile.tier_justification,
            created_at: profile.created_at,
            entries,
        });
    }
    Ok(ProfileHistoryView {
        profile_type,
        versions,
    })
}

/// Recalcula a Gap Analysis a partir dos profiles CURRENT/TARGET atuais
/// (RF-010: reproduzível a partir dos dados persistidos, sem Internet) e
/// persiste cada gap via F1-01. Retorna os ids dos resultados gravados.
pub async fn run_gap_analysis(services: &AppServices, incident_id: &str) -> AppResult<Vec<String>> {
    incident_exists(&services.db, incident_id).await?;
    let ids = nist_profiles::run_gap_analysis(&services.db, incident_id, None)
        .await
        .map_err(AppError::Database)?;
    Ok(ids)
}

/// Gaps NIST atualmente vigentes (não superseded, e sem `status: "CLOSED"`
/// no payload — ver `nist_profiles::run_gap_analysis`) de um incidente.
pub async fn current_gap_analysis(
    services: &AppServices,
    incident_id: &str,
) -> AppResult<Vec<AnalyticalResultResponse>> {
    incident_exists(&services.db, incident_id).await?;
    let results = current_results_for_incident(&services.db, incident_id, "NIST_GAP").await?;
    Ok(results
        .into_iter()
        .filter(|result| {
            result
                .payload
                .get("status")
                .and_then(|value| value.as_str())
                != Some("CLOSED")
        })
        .collect())
}

/// Resumo de um profile vigente para a visão por Function. O Tier é do
/// profile inteiro (contexto de governança informado pelo analista), nunca
/// de uma Function nem uma nota.
#[derive(Debug, Clone, Serialize)]
pub struct ProfileSummary {
    pub version_label: String,
    pub tier: Option<String>,
    pub tier_justification: Option<String>,
    pub entries: usize,
}

/// Visão por Function (F1-30, RF-053): só contagens, sem pontuação.
#[derive(Debug, Clone, Serialize)]
pub struct NistFunctionView {
    pub current_profile: Option<ProfileSummary>,
    pub target_profile: Option<ProfileSummary>,
    pub functions: Vec<nist_functions::FunctionSummary>,
    pub gap_state: nist_functions::GapAnalysisState,
}

fn summarize_profile(
    profile: &nist_profiles::ProfileRecord,
    entries: &[nist_profiles::ProfileEntryRecord],
) -> ProfileSummary {
    ProfileSummary {
        version_label: profile.version_label.clone(),
        tier: profile.tier.clone(),
        tier_justification: profile.tier_justification.clone(),
        entries: entries.len(),
    }
}

/// Agrega os profiles Current e Target vigentes e os gaps vigentes por
/// Function. Não grava nada.
pub async fn function_view(
    services: &AppServices,
    incident_id: &str,
) -> AppResult<NistFunctionView> {
    incident_exists(&services.db, incident_id).await?;
    let current = nist_profiles::current_profile_for(
        &services.db,
        incident_id,
        nist_profiles::ProfileType::Current,
    )
    .await?;
    let target = nist_profiles::current_profile_for(
        &services.db,
        incident_id,
        nist_profiles::ProfileType::Target,
    )
    .await?;
    let current_entries = match &current {
        Some(profile) => nist_profiles::profile_entries(&services.db, &profile.id).await?,
        None => Vec::new(),
    };
    let target_entries = match &target {
        Some(profile) => nist_profiles::profile_entries(&services.db, &profile.id).await?,
        None => Vec::new(),
    };

    let gaps = current_gap_analysis(services, incident_id).await?;
    let gap_pairs: Vec<(&str, &str)> = gaps
        .iter()
        .filter_map(|gap| {
            Some((
                gap.payload.get("subcategory_id")?.as_str()?,
                gap.payload.get("priority")?.as_str()?,
            ))
        })
        .collect();
    let provenance: Vec<(Option<&str>, Option<&str>)> = gaps
        .iter()
        .map(|gap| {
            (
                gap.provenance
                    .get("current_profile_id")
                    .and_then(|value| value.as_str()),
                gap.provenance
                    .get("target_profile_id")
                    .and_then(|value| value.as_str()),
            )
        })
        .collect();

    Ok(NistFunctionView {
        current_profile: current
            .as_ref()
            .map(|profile| summarize_profile(profile, &current_entries)),
        target_profile: target
            .as_ref()
            .map(|profile| summarize_profile(profile, &target_entries)),
        functions: nist_functions::summarize_by_function(
            &current_entries,
            &target_entries,
            &gap_pairs,
        ),
        gap_state: nist_functions::gap_analysis_state(
            current.as_ref().map(|profile| profile.id.as_str()),
            target.as_ref().map(|profile| profile.id.as_str()),
            &provenance,
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::test_support::fixture;

    fn entry(subcategory: &str, status: &str) -> ProfileEntryInput {
        ProfileEntryInput {
            subcategory_id: subcategory.into(),
            subcategory_label: format!("rótulo {subcategory}"),
            implementation_status: status.into(),
            ..ProfileEntryInput::default()
        }
    }

    #[tokio::test]
    async fn function_view_follows_profiles_and_flags_stale_gaps() {
        let fx = fixture().await;
        let incident = crate::service::create_incident(&fx.services, "incidente")
            .await
            .expect("incident");

        // Sem profiles: a visão existe, com as seis Functions zeradas.
        let empty = function_view(&fx.services, &incident.id)
            .await
            .expect("empty view");
        assert_eq!(empty.functions.len(), 6);
        assert!(empty.current_profile.is_none());
        assert_eq!(
            empty.gap_state,
            nist_functions::GapAnalysisState::MissingProfile
        );

        create_profile(
            &fx.services,
            &incident.id,
            "CURRENT",
            "c1",
            "entrevista",
            Some("TIER_1_PARTIAL"),
            Some("governança informal"),
            &[
                entry("GV.OC-01", "IMPLEMENTED"),
                entry("PR.AA-03", "NOT_IMPLEMENTED"),
                entry("DE.CM-01", "PARTIALLY_IMPLEMENTED"),
            ],
        )
        .await
        .expect("current");
        create_profile(
            &fx.services,
            &incident.id,
            "TARGET",
            "t1",
            "alvo acordado",
            Some("TIER_3_REPEATABLE"),
            None,
            &[
                entry("GV.OC-01", "IMPLEMENTED"),
                entry("PR.AA-03", "IMPLEMENTED"),
                entry("DE.CM-01", "IMPLEMENTED"),
            ],
        )
        .await
        .expect("target");

        // Profiles sem rodada de gap: nenhum gap vigente ainda.
        let before = function_view(&fx.services, &incident.id)
            .await
            .expect("view");
        assert_eq!(
            before.gap_state,
            nist_functions::GapAnalysisState::NoGapsRecorded
        );
        assert_eq!(before.current_profile.as_ref().map(|p| p.entries), Some(3));
        assert_eq!(
            before
                .current_profile
                .as_ref()
                .and_then(|p| p.tier.as_deref()),
            Some("TIER_1_PARTIAL")
        );

        run_gap_analysis(&fx.services, &incident.id)
            .await
            .expect("gap run");
        let view = function_view(&fx.services, &incident.id)
            .await
            .expect("view");
        assert_eq!(view.gap_state, nist_functions::GapAnalysisState::UpToDate);
        let pr = &view.functions[2];
        assert_eq!(pr.function, "PR");
        assert_eq!(pr.current.not_implemented, 1);
        assert_eq!(pr.target_subcategories, 1);
        // NOT_IMPLEMENTED -> IMPLEMENTED: dois degraus, prioridade alta.
        assert_eq!(pr.gaps.high, 1);
        let de = &view.functions[3];
        assert_eq!(de.gaps.medium, 1);
        assert_eq!(view.functions[0].gaps, nist_functions::GapCounts::default());

        // Nova versão do Current sem recalcular: os gaps ficam desatualizados.
        create_profile(
            &fx.services,
            &incident.id,
            "CURRENT",
            "c2",
            "reavaliação",
            None,
            None,
            &[entry("PR.AA-03", "IMPLEMENTED")],
        )
        .await
        .expect("new current");
        let stale = function_view(&fx.services, &incident.id)
            .await
            .expect("stale view");
        assert_eq!(stale.gap_state, nist_functions::GapAnalysisState::Stale);

        run_gap_analysis(&fx.services, &incident.id)
            .await
            .expect("rerun");
        let fresh = function_view(&fx.services, &incident.id)
            .await
            .expect("fresh view");
        assert_ne!(
            fresh.gap_state,
            nist_functions::GapAnalysisState::Stale,
            "recalcular deixa os gaps em dia (ou sem gaps)"
        );
    }
}
