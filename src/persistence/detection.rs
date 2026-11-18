//! Candidatos de detecção produzidos pelas regras do engine (F1-28, ADR-0025).
//!
//! Um candidato é uma **hipótese**: tem a regra, a técnica ATT&CK sugerida, os
//! eventos de origem e o que a regra não prova. Só o analista o transforma em
//! mapeamento, achado e IOC. O que a regra afirmou fica imutável no banco; só a
//! decisão (e os vínculos criados ao confirmar) mudam, uma vez.

use super::validation_error;
use chrono::Utc;
use dfir_protocol::{CandidateIoc, DetectionCandidate};
use serde::Serialize;
use sqlx::{FromRow, SqliteConnection, SqlitePool};
use uuid::Uuid;

pub const STATUS_PENDING: &str = "PENDING";
pub const STATUS_CONFIRMED: &str = "CONFIRMED";
pub const STATUS_REJECTED: &str = "REJECTED";

#[derive(Debug, Clone, FromRow)]
pub struct CandidateRow {
    pub id: String,
    pub evidence_id: String,
    pub job_id: String,
    pub candidate_key: String,
    pub rule_id: String,
    pub rule_version: String,
    pub rule_set_version: String,
    pub title: String,
    pub description: String,
    pub tactic: String,
    pub technique_id: String,
    pub sub_technique_id: Option<String>,
    pub severity: String,
    pub confidence: String,
    pub aggregation: String,
    pub event_count: i64,
    pub first_seen: String,
    pub last_seen: String,
    pub event_refs: String,
    pub refs_truncated: i64,
    pub host: Option<String>,
    pub user_name: Option<String>,
    pub src_ip: Option<String>,
    pub iocs: String,
    pub limitations: String,
    pub status: String,
    pub created_at: String,
    pub decided_at: Option<String>,
    pub decision_note: Option<String>,
    pub incident_id: Option<String>,
    pub attack_mapping_id: Option<String>,
    pub finding_id: Option<String>,
}

impl CandidateRow {
    pub fn event_refs(&self) -> Result<Vec<String>, sqlx::Error> {
        serde_json::from_str(&self.event_refs).map_err(|error| {
            validation_error(format!(
                "stored detection candidate event_refs are malformed JSON: {error}"
            ))
        })
    }

    pub fn iocs(&self) -> Result<Vec<CandidateIoc>, sqlx::Error> {
        serde_json::from_str(&self.iocs).map_err(|error| {
            validation_error(format!(
                "stored detection candidate iocs are malformed JSON: {error}"
            ))
        })
    }
}

/// Contagem por situação, para a tela e para os entregáveis.
#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
pub struct CandidateCounts {
    pub pending: u64,
    pub confirmed: u64,
    pub rejected: u64,
}

impl CandidateCounts {
    pub fn total(&self) -> u64 {
        self.pending + self.confirmed + self.rejected
    }
}

const COLUMNS: &str = "id,evidence_id,job_id,candidate_key,rule_id,rule_version,rule_set_version,title,description,tactic,technique_id,sub_technique_id,severity,confidence,aggregation,event_count,first_seen,last_seen,event_refs,refs_truncated,host,user_name,src_ip,iocs,limitations,status,created_at,decided_at,decision_note,incident_id,attack_mapping_id,finding_id";

/// Grava um candidato. `false` quando a mesma chave (regra + versão +
/// agrupamento) já existe para a evidência: rodar a detecção de novo não
/// duplica nem desfaz decisões do analista.
pub async fn insert_on_connection(
    connection: &mut SqliteConnection,
    evidence_id: &str,
    job_id: &str,
    candidate: &DetectionCandidate,
) -> Result<bool, sqlx::Error> {
    let event_refs = serde_json::to_string(&candidate.event_refs).map_err(|error| {
        validation_error(format!(
            "could not encode detection candidate event_refs as JSON: {error}"
        ))
    })?;
    let iocs = serde_json::to_string(&candidate.iocs).map_err(|error| {
        validation_error(format!(
            "could not encode detection candidate iocs as JSON: {error}"
        ))
    })?;
    let result = sqlx::query(
        "INSERT OR IGNORE INTO detection_candidates (id,evidence_id,job_id,candidate_key,rule_id,rule_version,rule_set_version,title,description,tactic,technique_id,sub_technique_id,severity,confidence,aggregation,event_count,first_seen,last_seen,event_refs,refs_truncated,host,user_name,src_ip,iocs,limitations,status,created_at) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,'PENDING',?)",
    )
    .bind(format!("cand_{}", Uuid::new_v4()))
    .bind(evidence_id)
    .bind(job_id)
    .bind(&candidate.candidate_key)
    .bind(&candidate.rule_id)
    .bind(&candidate.rule_version)
    .bind(&candidate.rule_set_version)
    .bind(&candidate.title)
    .bind(&candidate.description)
    .bind(&candidate.tactic)
    .bind(&candidate.technique_id)
    .bind(&candidate.sub_technique_id)
    .bind(&candidate.severity)
    .bind(&candidate.confidence)
    .bind(&candidate.aggregation)
    .bind(candidate.count.max(1) as i64)
    .bind(&candidate.first_seen)
    .bind(&candidate.last_seen)
    .bind(event_refs)
    .bind(i64::from(candidate.refs_truncated))
    .bind(&candidate.host)
    .bind(&candidate.user)
    .bind(&candidate.src_ip)
    .bind(iocs)
    .bind(&candidate.limitations)
    .bind(Utc::now().to_rfc3339())
    .execute(&mut *connection)
    .await?;
    Ok(result.rows_affected() == 1)
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<Option<CandidateRow>, sqlx::Error> {
    let sql = format!("SELECT {COLUMNS} FROM detection_candidates WHERE id=?");
    sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// Candidatos da evidência, do mais antigo ao mais novo, filtrados por situação.
pub async fn list(
    pool: &SqlitePool,
    evidence_id: &str,
    status: Option<&str>,
) -> Result<Vec<CandidateRow>, sqlx::Error> {
    let sql = format!(
        "SELECT {COLUMNS} FROM detection_candidates WHERE evidence_id=? AND (? IS NULL OR status=?) ORDER BY first_seen, rule_id, candidate_key"
    );
    sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(evidence_id)
        .bind(status)
        .bind(status)
        .fetch_all(pool)
        .await
}

pub async fn counts(pool: &SqlitePool, evidence_id: &str) -> Result<CandidateCounts, sqlx::Error> {
    let rows: Vec<(String, i64)> = sqlx::query_as(
        "SELECT status, COUNT(*) FROM detection_candidates WHERE evidence_id=? GROUP BY status",
    )
    .bind(evidence_id)
    .fetch_all(pool)
    .await?;
    let mut counts = CandidateCounts::default();
    for (status, count) in rows {
        let count = count.max(0) as u64;
        match status.as_str() {
            STATUS_PENDING => counts.pending = count,
            STATUS_CONFIRMED => counts.confirmed = count,
            STATUS_REJECTED => counts.rejected = count,
            _ => {}
        }
    }
    Ok(counts)
}

/// Quantos candidatos a execução `job_id` criou (os já existentes não contam).
pub async fn count_for_job(pool: &SqlitePool, job_id: &str) -> Result<u64, sqlx::Error> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM detection_candidates WHERE job_id=?")
        .bind(job_id)
        .fetch_one(pool)
        .await?;
    Ok(count.max(0) as u64)
}

/// Guarda o mapeamento criado, ainda pendente (a confirmação pode ser retomada).
pub async fn link_attack_mapping(
    pool: &SqlitePool,
    id: &str,
    attack_mapping_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE detection_candidates SET attack_mapping_id=? WHERE id=? AND status='PENDING'",
    )
    .bind(attack_mapping_id)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn link_finding(
    pool: &SqlitePool,
    id: &str,
    finding_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE detection_candidates SET finding_id=? WHERE id=? AND status='PENDING'")
        .bind(finding_id)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Registra a decisão. `false` se o candidato já tinha sido decidido (a
/// decisão é final: o gatilho do banco também a protege).
pub async fn decide(
    pool: &SqlitePool,
    id: &str,
    status: &str,
    note: Option<&str>,
    incident_id: Option<&str>,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE detection_candidates SET status=?, decided_at=?, decision_note=?, incident_id=COALESCE(?, incident_id) WHERE id=? AND status='PENDING'",
    )
    .bind(status)
    .bind(Utc::now().to_rfc3339())
    .bind(note)
    .bind(incident_id)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() == 1)
}

/// Apaga os candidatos de uma evidência destruída: eles carregam dado da
/// evidência (contas, IPs, referências de evento) e saem com ela (RB-006).
pub async fn delete_for_evidence(pool: &SqlitePool, evidence_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM detection_candidates WHERE evidence_id=?")
        .bind(evidence_id)
        .execute(pool)
        .await?;
    Ok(())
}
