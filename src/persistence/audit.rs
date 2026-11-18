use blake3::Hasher as Blake3Hasher;
use serde_json::Value;
use sqlx::{SqliteConnection, SqlitePool};
use uuid::Uuid;

use super::jcs::{canonicalize_json, canonicalize_value};
use super::rollback_loudly;

struct EventHashInput<'a> {
    id: &'a str,
    timestamp: &'a str,
    actor_id: Option<&'a str>,
    action: &'a str,
    object_type: &'a str,
    object_id: &'a str,
    incident_id: Option<&'a str>,
    metadata: &'a str,
    previous_hash: &'a str,
}

const AUDIT_HASH_V2_PREFIX: &str = "BLAKE3-256-v2-JCS:";
const AUDIT_HASH_V1_PREFIX: &str = "BLAKE3-256-v1:";
const AUDIT_ENCODING_DOMAIN_V1: &[u8] = b"dfir-audit/blake3-256/v1";

fn write_field_v1(buffer: &mut Vec<u8>, value: Option<&str>) {
    match value {
        Some(value) => {
            buffer.push(1);
            let bytes = value.as_bytes();
            buffer.extend_from_slice(&(bytes.len() as u64).to_be_bytes());
            buffer.extend_from_slice(bytes);
        }
        None => buffer.push(0),
    }
}

fn event_hash_v1(input: &EventHashInput<'_>) -> String {
    let mut encoded = Vec::new();
    encoded.extend_from_slice(AUDIT_ENCODING_DOMAIN_V1);
    write_field_v1(&mut encoded, Some(input.id));
    write_field_v1(&mut encoded, Some(input.timestamp));
    write_field_v1(&mut encoded, input.actor_id);
    write_field_v1(&mut encoded, Some(input.action));
    write_field_v1(&mut encoded, Some(input.object_type));
    write_field_v1(&mut encoded, Some(input.object_id));
    write_field_v1(&mut encoded, input.incident_id);
    write_field_v1(&mut encoded, Some(input.metadata));
    write_field_v1(&mut encoded, Some(input.previous_hash));

    let mut hasher = Blake3Hasher::new();
    hasher.update(&encoded);
    format!("{AUDIT_HASH_V1_PREFIX}{}", hasher.finalize().to_hex())
}

fn event_hash(input: &EventHashInput<'_>) -> Result<String, serde_json::Error> {
    let value = serde_json::json!({
        "schema": "dfir-audit-event-v2",
        "id": input.id,
        "timestamp": input.timestamp,
        "actor_id": input.actor_id,
        "action": input.action,
        "object_type": input.object_type,
        "object_id": input.object_id,
        "incident_id": input.incident_id,
        "metadata": serde_json::from_str::<Value>(input.metadata)?,
        "previous_hash": input.previous_hash,
    });
    let canonical = canonicalize_value(&value)
        .map_err(|error| serde_json::Error::io(std::io::Error::other(error.to_string())))?;
    let mut hasher = Blake3Hasher::new();
    hasher.update(&canonical);
    Ok(format!(
        "{AUDIT_HASH_V2_PREFIX}{}",
        hasher.finalize().to_hex()
    ))
}

fn event_hash_matches(stored: &str, input: &EventHashInput<'_>) -> bool {
    if stored.starts_with(AUDIT_HASH_V2_PREFIX) {
        return event_hash(input)
            .map(|value| value == stored)
            .unwrap_or(false);
    }
    if stored.starts_with(AUDIT_HASH_V1_PREFIX) {
        return event_hash_v1(input) == stored;
    }
    false
}

pub(crate) async fn append_on_connection(
    connection: &mut SqliteConnection,
    actor_id: Option<&str>,
    action: &str,
    object_type: &str,
    object_id: &str,
    incident_id: Option<&str>,
    metadata_json: &str,
) -> Result<(), sqlx::Error> {
    let metadata = canonicalize_json(metadata_json)?;
    let previous_hash: Option<String> =
        sqlx::query_scalar("SELECT event_hash FROM audit_events ORDER BY sequence DESC LIMIT 1")
            .fetch_optional(&mut *connection)
            .await?;
    let previous = previous_hash.as_deref().unwrap_or("");
    let id = Uuid::new_v4().to_string();
    let timestamp = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true);
    let hash = event_hash(&EventHashInput {
        id: &id,
        timestamp: &timestamp,
        actor_id,
        action,
        object_type,
        object_id,
        incident_id,
        metadata: &metadata,
        previous_hash: previous,
    })
    .map_err(|error| sqlx::Error::Protocol(error.to_string()))?;

    sqlx::query(
        "INSERT INTO audit_events (id,timestamp,actor_id,action,object_type,object_id,incident_id,metadata,previous_hash,event_hash) VALUES (?,?,?,?,?,?,?,?,?,?)",
    )
    .bind(&id)
    .bind(&timestamp)
    .bind(actor_id)
    .bind(action)
    .bind(object_type)
    .bind(object_id)
    .bind(incident_id)
    .bind(&metadata)
    .bind(previous_hash)
    .bind(&hash)
    .execute(&mut *connection)
    .await?;

    sqlx::query("UPDATE audit_state SET event_count=event_count+1, head_hash=? WHERE id=1")
        .bind(&hash)
        .execute(&mut *connection)
        .await?;

    Ok(())
}

/// Appends an audit event in its own serialized SQLite transaction.
///
/// Callers that need several database mutations to commit atomically should
/// use `append_on_connection` from their surrounding transaction instead.
pub async fn append(
    pool: &SqlitePool,
    actor_id: Option<&str>,
    action: &str,
    object_type: &str,
    object_id: &str,
    incident_id: Option<&str>,
    metadata_json: &str,
) -> Result<(), sqlx::Error> {
    let started = std::time::Instant::now();
    let mut connection = pool.acquire().await?;
    sqlx::query("BEGIN IMMEDIATE")
        .execute(&mut *connection)
        .await?;
    let result = append_on_connection(
        &mut connection,
        actor_id,
        action,
        object_type,
        object_id,
        incident_id,
        metadata_json,
    )
    .await;
    match result {
        Ok(()) => {
            sqlx::query("COMMIT").execute(&mut *connection).await?;
            tracing::debug!(
                elapsed_ms = started.elapsed().as_millis(),
                "audit append completed"
            );
            Ok(())
        }
        Err(error) => {
            rollback_loudly(&mut connection).await;
            Err(error)
        }
    }
}

pub async fn verify(pool: &SqlitePool) -> Result<bool, sqlx::Error> {
    const BATCH_SIZE: i64 = 512;
    let stored = sqlx::query_as::<_, (i64, Option<String>)>(
        "SELECT event_count, head_hash FROM audit_state WHERE id=1",
    )
    .fetch_one(pool)
    .await?;

    let mut previous = String::new();
    let mut verified_count = 0_i64;
    let mut last_sequence = 0_i64;

    loop {
        let rows = sqlx::query_as::<_, (i64, String, String, Option<String>, String, String, String, Option<String>, String, Option<String>, String)>(
            "SELECT sequence,id,timestamp,actor_id,action,object_type,object_id,incident_id,metadata,previous_hash,event_hash FROM audit_events WHERE sequence>? ORDER BY sequence LIMIT ?",
        )
        .bind(last_sequence)
        .bind(BATCH_SIZE)
        .fetch_all(pool)
        .await?;

        if rows.is_empty() {
            break;
        }

        for (
            sequence,
            id,
            timestamp,
            actor,
            action,
            object_type,
            object_id,
            incident_id,
            metadata,
            previous_hash,
            event_hash_value,
        ) in rows
        {
            if sequence <= 0
                || sequence != last_sequence.saturating_add(1)
                || previous_hash.as_deref().unwrap_or("") != previous
            {
                return Ok(false);
            }
            let hash_input = EventHashInput {
                id: &id,
                timestamp: &timestamp,
                actor_id: actor.as_deref(),
                action: &action,
                object_type: &object_type,
                object_id: &object_id,
                incident_id: incident_id.as_deref(),
                metadata: &metadata,
                previous_hash: &previous,
            };
            if !event_hash_matches(&event_hash_value, &hash_input) {
                return Ok(false);
            }
            last_sequence = sequence;
            previous = event_hash_value;
            verified_count = verified_count.saturating_add(1);
        }
    }

    if stored.0 != verified_count {
        return Ok(false);
    }
    if stored.1.as_deref().unwrap_or("") != previous {
        return Ok(false);
    }
    let actual_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM audit_events")
        .fetch_one(pool)
        .await?;
    Ok(actual_count == verified_count)
}

pub(crate) async fn append_custody_on_connection(
    connection: &mut SqliteConnection,
    evidence_id: &str,
    action: &str,
    actor: &str,
    reason: &str,
    metadata_json: &str,
) -> Result<(), sqlx::Error> {
    let metadata = canonicalize_json(metadata_json)?;
    let previous: Option<String> = sqlx::query_scalar(
        "SELECT entry_hash FROM chain_of_custody WHERE evidence_id=? ORDER BY sequence DESC LIMIT 1",
    )
    .bind(evidence_id)
    .fetch_optional(&mut *connection)
    .await?;
    let entry_id = Uuid::new_v4().to_string();
    let timestamp = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true);
    let value = serde_json::json!({
        "schema": "dfir-custody-entry-v1",
        "entry_id": entry_id,
        "evidence_id": evidence_id,
        "action": action,
        "actor": actor,
        "timestamp": timestamp,
        "reason": reason,
        "metadata": serde_json::from_str::<Value>(&metadata)
            .map_err(|error| sqlx::Error::Protocol(error.to_string()))?,
        "previous_entry_hash": previous,
    });
    let canonical = canonicalize_value(&value)?;
    let mut hasher = Blake3Hasher::new();
    hasher.update(&canonical);
    let entry_hash = format!("BLAKE3-256-v1:{}", hasher.finalize().to_hex());
    sqlx::query(
        "INSERT INTO chain_of_custody(entry_id,evidence_id,action,actor,timestamp,reason,metadata,previous_entry_hash,entry_hash) VALUES(?,?,?,?,?,?,?,?,?)",
    )
    .bind(entry_id)
    .bind(evidence_id)
    .bind(action)
    .bind(actor)
    .bind(timestamp)
    .bind(reason)
    .bind(metadata)
    .bind(previous)
    .bind(entry_hash)
    .execute(&mut *connection)
    .await?;
    Ok(())
}

pub async fn append_custody(
    pool: &SqlitePool,
    evidence_id: &str,
    action: &str,
    actor: &str,
    reason: &str,
    metadata_json: &str,
) -> Result<(), sqlx::Error> {
    let mut connection = pool.acquire().await?;
    sqlx::query("BEGIN IMMEDIATE")
        .execute(&mut *connection)
        .await?;
    let result = append_custody_on_connection(
        &mut connection,
        evidence_id,
        action,
        actor,
        reason,
        metadata_json,
    )
    .await;
    match result {
        Ok(()) => {
            sqlx::query("COMMIT").execute(&mut *connection).await?;
            Ok(())
        }
        Err(error) => {
            rollback_loudly(&mut connection).await;
            Err(error)
        }
    }
}

/// Atomically persists evidence plus its first audit and custody records.
/// The caller must have already started `BEGIN IMMEDIATE` on `connection`.
pub(crate) async fn append_import_records_on_connection(
    connection: &mut SqliteConnection,
    evidence_id: &str,
    metadata_json: &str,
) -> Result<(), sqlx::Error> {
    append_on_connection(
        connection,
        None,
        "EVIDENCE_IMPORTED",
        "evidence",
        evidence_id,
        None,
        metadata_json,
    )
    .await?;
    append_custody_on_connection(
        connection,
        evidence_id,
        "IMPORTED",
        "local-analyst",
        "manual local evidence import",
        metadata_json,
    )
    .await
}

/// Entrada da cadeia de custódia pronta para exibir/listar (F1-24): os
/// campos são exatamente os da tabela; `metadata` vai como JSON parseado.
#[derive(Debug, Clone, serde::Serialize)]
pub struct CustodyEntry {
    pub sequence: i64,
    pub entry_id: String,
    pub evidence_id: String,
    pub action: String,
    pub actor: String,
    pub timestamp: String,
    pub reason: String,
    pub metadata: Value,
    pub previous_entry_hash: Option<String>,
    pub entry_hash: String,
}

/// Lista a cadeia de custódia de uma evidência em ordem de sequência.
pub async fn list_custody(
    pool: &SqlitePool,
    evidence_id: &str,
) -> Result<Vec<CustodyEntry>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (i64, String, String, String, String, String, String, Option<String>, String)>(
        "SELECT sequence,entry_id,action,actor,timestamp,reason,metadata,previous_entry_hash,entry_hash FROM chain_of_custody WHERE evidence_id=? ORDER BY sequence",
    )
    .bind(evidence_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(
            |(
                sequence,
                entry_id,
                action,
                actor,
                timestamp,
                reason,
                metadata,
                previous_entry_hash,
                entry_hash,
            )| {
                let metadata = serde_json::from_str(&metadata).unwrap_or(Value::Null);
                CustodyEntry {
                    sequence,
                    entry_id,
                    evidence_id: evidence_id.to_string(),
                    action,
                    actor,
                    timestamp,
                    reason,
                    metadata,
                    previous_entry_hash,
                    entry_hash,
                }
            },
        )
        .collect())
}

/// Evento de auditoria pronto para exibir/listar (F1-24).
#[derive(Debug, Clone, serde::Serialize)]
pub struct AuditEventView {
    pub sequence: i64,
    pub id: String,
    pub timestamp: String,
    pub actor_id: Option<String>,
    pub action: String,
    pub object_type: String,
    pub object_id: String,
    pub incident_id: Option<String>,
    pub metadata: Value,
    pub event_hash: String,
}

/// Lista eventos de auditoria (mais recentes primeiro, até `limit`).
/// Com `incident_id`, só os daquele incidente.
pub async fn list_events(
    pool: &SqlitePool,
    incident_id: Option<&str>,
    limit: u64,
) -> Result<Vec<AuditEventView>, sqlx::Error> {
    let limit = limit.clamp(1, 10_000) as i64;
    let rows = match incident_id {
        Some(incident) => sqlx::query_as::<_, (i64, String, String, Option<String>, String, String, String, Option<String>, String, String)>(
            "SELECT sequence,id,timestamp,actor_id,action,object_type,object_id,incident_id,metadata,event_hash FROM audit_events WHERE incident_id=? ORDER BY sequence DESC LIMIT ?",
        )
        .bind(incident)
        .bind(limit)
        .fetch_all(pool)
        .await?,
        None => sqlx::query_as::<_, (i64, String, String, Option<String>, String, String, String, Option<String>, String, String)>(
            "SELECT sequence,id,timestamp,actor_id,action,object_type,object_id,incident_id,metadata,event_hash FROM audit_events ORDER BY sequence DESC LIMIT ?",
        )
        .bind(limit)
        .fetch_all(pool)
        .await?,
    };
    Ok(rows
        .into_iter()
        .map(
            |(
                sequence,
                id,
                timestamp,
                actor_id,
                action,
                object_type,
                object_id,
                incident_id,
                metadata,
                event_hash,
            )| {
                let metadata = serde_json::from_str(&metadata).unwrap_or(Value::Null);
                AuditEventView {
                    sequence,
                    id,
                    timestamp,
                    actor_id,
                    action,
                    object_type,
                    object_id,
                    incident_id,
                    metadata,
                    event_hash,
                }
            },
        )
        .collect())
}

pub async fn verify_custody(pool: &SqlitePool, evidence_id: &str) -> Result<bool, sqlx::Error> {
    let rows = sqlx::query_as::<_, (i64, String, String, String, String, String, String, Option<String>, String)>(
        "SELECT sequence,entry_id,action,actor,timestamp,reason,metadata,previous_entry_hash,entry_hash FROM chain_of_custody WHERE evidence_id=? ORDER BY sequence",
    )
    .bind(evidence_id)
    .fetch_all(pool)
    .await?;

    let mut previous: Option<String> = None;
    for (
        sequence,
        entry_id,
        action,
        actor,
        timestamp,
        reason,
        metadata,
        previous_hash,
        stored_hash,
    ) in rows
    {
        if sequence <= 0 || previous_hash != previous {
            return Ok(false);
        }
        let metadata_value: Value = match serde_json::from_str(&metadata) {
            Ok(value) => value,
            Err(_) => return Ok(false),
        };
        let value = serde_json::json!({
            "schema": "dfir-custody-entry-v1",
            "entry_id": entry_id,
            "evidence_id": evidence_id,
            "action": action,
            "actor": actor,
            "timestamp": timestamp,
            "reason": reason,
            "metadata": metadata_value,
            "previous_entry_hash": previous,
        });
        let canonical = match canonicalize_value(&value) {
            Ok(canonical) => canonical,
            Err(_) => return Ok(false),
        };
        let mut hasher = Blake3Hasher::new();
        hasher.update(&canonical);
        let expected = format!("BLAKE3-256-v1:{}", hasher.finalize().to_hex());
        if expected != stored_hash {
            return Ok(false);
        }
        previous = Some(stored_hash);
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::{
        AUDIT_HASH_V2_PREFIX, EventHashInput, append, append_custody,
        append_import_records_on_connection, canonicalize_json, event_hash, event_hash_v1, verify,
        verify_custody,
    };

    #[test]
    fn blake3_hashes_are_versioned_and_unambiguous() {
        let first = EventHashInput {
            id: "ab",
            timestamp: "c",
            actor_id: None,
            action: "action",
            object_type: "object",
            object_id: "id",
            incident_id: None,
            metadata: "{}",
            previous_hash: "",
        };
        let second = EventHashInput {
            id: "a",
            timestamp: "bc",
            actor_id: None,
            action: "action",
            object_type: "object",
            object_id: "id",
            incident_id: None,
            metadata: "{}",
            previous_hash: "",
        };
        let first_hash = event_hash(&first).expect("hash first event");
        let second_hash = event_hash(&second).expect("hash second event");
        assert!(first_hash.starts_with(AUDIT_HASH_V2_PREFIX));
        assert_ne!(first_hash, second_hash);
    }

    #[test]
    fn audit_hash_v1_and_v2_match_known_vectors() {
        let input = EventHashInput {
            id: "evt-01",
            timestamp: "2026-01-02T03:04:05Z",
            actor_id: Some("analyst"),
            action: "EVIDENCE_IMPORTED",
            object_type: "evidence",
            object_id: "ev-01",
            incident_id: None,
            metadata: r#"{"a":[true,"x"],"z":1}"#,
            previous_hash: "",
        };
        assert_eq!(
            event_hash_v1(&input),
            "BLAKE3-256-v1:e83bf79c5ee23b0efafa2371b6c907b31c8dc1fefd55ad9dbf6dd56057bb9176"
        );
        assert_eq!(
            event_hash(&input).expect("v2 hash"),
            "BLAKE3-256-v2-JCS:191e3a21f14920b53d1218fcbbcb0b5a3aead657ef496ae50af768abbad1183a"
        );
    }

    #[test]
    fn canonicalize_json_is_key_order_independent() {
        let a = canonicalize_json(r#"{"z":1,"a":2}"#).expect("valid metadata");
        let b = canonicalize_json(r#"{"a":2,"z":1}"#).expect("valid metadata");
        assert_eq!(a, b);
    }

    #[tokio::test]
    async fn import_provenance_rolls_back_as_one_unit() {
        let temp = tempfile::tempdir().expect("temporary directory");
        let db_path = temp.path().join("provenance.sqlite");
        let url = format!("sqlite://{}", db_path.display());
        let pool = crate::persistence::db::connect(&url)
            .await
            .expect("database connection");
        crate::persistence::db::migrate(&pool)
            .await
            .expect("migrations");

        let mut connection = pool.acquire().await.expect("connection");
        sqlx::query("BEGIN IMMEDIATE")
            .execute(&mut *connection)
            .await
            .expect("begin transaction");
        sqlx::query("INSERT INTO evidence(id,original_filename,storage_reference,format,size_bytes,blake3,sha256,imported_at,status) VALUES(?,?,?,?,?,?,?,?,?)")
            .bind("ev-test")
            .bind("test.evtx")
            .bind("evidence/ev-test.evtx")
            .bind("evtx")
            .bind(1_i64)
            .bind("b3")
            .bind("sha")
            .bind("2026-08-20T00:00:00Z")
            .bind("FORENSIC")
            .execute(&mut *connection)
            .await
            .expect("evidence insert");

        let result =
            append_import_records_on_connection(&mut connection, "ev-test", "not-json").await;
        assert!(result.is_err());
        sqlx::query("ROLLBACK")
            .execute(&mut *connection)
            .await
            .expect("rollback");

        let evidence_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM evidence WHERE id='ev-test'")
                .fetch_one(&pool)
                .await
                .expect("evidence count");
        let audit_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM audit_events WHERE object_id='ev-test'")
                .fetch_one(&pool)
                .await
                .expect("audit count");
        let custody_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM chain_of_custody WHERE evidence_id='ev-test'")
                .fetch_one(&pool)
                .await
                .expect("custody count");
        assert_eq!((evidence_count, audit_count, custody_count), (0, 0, 0));
    }

    #[tokio::test]
    async fn custody_chain_verification_detects_tampering() {
        let temp = tempfile::tempdir().expect("temporary directory");
        let db_path = temp.path().join("custody.sqlite");
        let url = format!("sqlite://{}", db_path.display());
        let pool = crate::persistence::db::connect(&url)
            .await
            .expect("database connection");
        crate::persistence::db::migrate(&pool)
            .await
            .expect("migrations");
        sqlx::query("INSERT INTO evidence(id,original_filename,storage_reference,format,size_bytes,blake3,sha256,imported_at,status) VALUES(?,?,?,?,?,?,?,?,?)")
            .bind("ev-test")
            .bind("test.evtx")
            .bind("evidence/ev-test.evtx")
            .bind("evtx")
            .bind(1_i64)
            .bind("b3")
            .bind("sha")
            .bind("2026-08-20T00:00:00Z")
            .bind("FORENSIC")
            .execute(&pool)
            .await
            .expect("evidence insert");
        append_custody(&pool, "ev-test", "IMPORTED", "local-analyst", "test", "{}")
            .await
            .expect("custody append");
        assert!(
            verify_custody(&pool, "ev-test")
                .await
                .expect("verify custody")
        );
        sqlx::query("DROP TRIGGER custody_no_update")
            .execute(&pool)
            .await
            .expect("drop update trigger for tamper simulation");
        sqlx::query("UPDATE chain_of_custody SET reason='tampered' WHERE evidence_id='ev-test'")
            .execute(&pool)
            .await
            .expect("tamper simulation");
        assert!(
            !verify_custody(&pool, "ev-test")
                .await
                .expect("verify custody")
        );
    }

    #[tokio::test]
    async fn concurrent_appends_preserve_one_auditable_chain() {
        let temp = tempfile::tempdir().expect("temporary directory");
        let db_path = temp.path().join("audit.sqlite");
        let url = format!("sqlite://{}", db_path.display());
        let pool = crate::persistence::db::connect(&url)
            .await
            .expect("database connection");
        crate::persistence::db::migrate(&pool)
            .await
            .expect("migrations");

        let mut tasks = Vec::new();
        for _ in 0..32 {
            let pool = pool.clone();
            tasks.push(tokio::spawn(async move {
                append(&pool, None, "TEST", "test", "object", None, "{}").await
            }));
        }

        for task in tasks {
            task.await
                .expect("append task did not panic")
                .expect("append failed");
        }

        assert!(verify(&pool).await.expect("audit verification"));
        let count: i64 = sqlx::query_scalar("SELECT event_count FROM audit_state WHERE id=1")
            .fetch_one(&pool)
            .await
            .expect("audit state");
        assert_eq!(count, 32);
    }

    #[tokio::test]
    async fn tampered_event_count_is_detected() {
        let temp = tempfile::tempdir().expect("temporary directory");
        let db_path = temp.path().join("audit-count.sqlite");
        let url = format!("sqlite://{}", db_path.display());
        let pool = crate::persistence::db::connect(&url)
            .await
            .expect("database connection");
        crate::persistence::db::migrate(&pool)
            .await
            .expect("migrations");
        append(&pool, None, "TEST", "test", "object", None, "{}")
            .await
            .expect("append");
        sqlx::query("UPDATE audit_state SET event_count=0 WHERE id=1")
            .execute(&pool)
            .await
            .expect("tamper event count");
        assert!(!verify(&pool).await.expect("audit verification"));
    }

    #[tokio::test]
    async fn tampering_with_audit_head_is_detected() {
        let temp = tempfile::tempdir().expect("temporary directory");
        let db_path = temp.path().join("audit-tamper.sqlite");
        let url = format!("sqlite://{}", db_path.display());
        let pool = crate::persistence::db::connect(&url)
            .await
            .expect("database connection");
        crate::persistence::db::migrate(&pool)
            .await
            .expect("migrations");
        append(&pool, None, "TEST", "test", "object", None, "{}")
            .await
            .expect("append");
        assert!(verify(&pool).await.expect("audit verification"));

        sqlx::query("UPDATE audit_state SET head_hash='tampered' WHERE id=1")
            .execute(&pool)
            .await
            .expect("tamper state");
        assert!(
            !verify(&pool)
                .await
                .expect("audit verification after tamper")
        );
    }
    #[test]
    fn jcs_metadata_is_canonical_and_rejects_large_integers() {
        let canonical = canonicalize_json(r#"{"b":2,"a":1}"#).unwrap();
        assert_eq!(canonical, r#"{"a":1,"b":2}"#);
        assert!(canonicalize_json(r#"{"too_large":9007199254740993}"#).is_err());
    }
}
