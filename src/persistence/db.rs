use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
};
use std::{str::FromStr, time::Duration};

pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!();

/// Aplica as migrations pendentes **sem** copiar o banco antes. Só para testes
/// e bancos descartáveis; o código de produção usa [`migrate_with_backup`].
pub async fn migrate(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    MIGRATOR.run(pool).await.map_err(sqlx::Error::from)
}

/// Caminho de produção (RF-032): copia e verifica o banco antes de aplicar
/// qualquer migration pendente sobre um banco que já tem dados. Se a cópia
/// falhar, a migration não é aplicada. Ver `persistence::backup`.
pub async fn migrate_with_backup(
    pool: &SqlitePool,
    backups_dir: &std::path::Path,
    min_free_bytes: u64,
) -> Result<super::backup::BackupOutcome, sqlx::Error> {
    super::backup::migrate_with_backup(pool, &MIGRATOR, backups_dir, min_free_bytes).await
}

pub async fn connect(url: &str) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::from_str(url)?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Full)
        .foreign_keys(true)
        .busy_timeout(Duration::from_millis(5000));

    // The roadmap explicitly fixes a worker-oriented SQLite design; a small pool is still used
    // for request concurrency, while heavy processing remains single-job at the application layer.
    SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Migrations anteriores à 0010, em ordem. Aplicadas à mão (em vez de
    /// pelo `MIGRATOR`) para semear dados reais antes da 0010 e provar que
    /// ela os preserva.
    const BEFORE_0010: [&str; 9] = [
        include_str!("../../migrations/0001_fase0.sql"),
        include_str!("../../migrations/0002_access_visibility.sql"),
        include_str!("../../migrations/0003_local_forensic_scope.sql"),
        include_str!("../../migrations/0004_mvp_evidence_jobs.sql"),
        include_str!("../../migrations/0005_mvp_custody_fields.sql"),
        include_str!("../../migrations/0006_analytical_results.sql"),
        include_str!("../../migrations/0007_analysis_job_kind.sql"),
        include_str!("../../migrations/0008_nist_profiles.sql"),
        include_str!("../../migrations/0009_job_warnings_and_timeline_index.sql"),
    ];
    const MIGRATION_0010: &str =
        include_str!("../../migrations/0010_result_types_finding_plan_item.sql");
    const MIGRATION_0011: &str =
        include_str!("../../migrations/0011_provenance_and_source_gaps.sql");

    /// Aplica um arquivo de migration numa transação, como o sqlx faz.
    async fn apply(pool: &SqlitePool, sql: &'static str) {
        let mut tx = pool.begin().await.expect("begin migration transaction");
        sqlx::query(sql)
            .execute(&mut *tx)
            .await
            .expect("migration sql");
        tx.commit().await.expect("commit migration transaction");
    }

    async fn result_rows(pool: &SqlitePool) -> Vec<String> {
        sqlx::query_scalar(
            "SELECT sequence || '|' || id || '|' || schema_version || '|' || result_type || '|' || correlation_key || '|' || ifnull(evidence_id,'-') || '|' || ifnull(incident_id,'-') || '|' || ifnull(confidence,'-') || '|' || payload || '|' || provenance || '|' || ifnull(supersedes,'-') || '|' || created_at FROM analytical_results ORDER BY sequence",
        )
        .fetch_all(pool)
        .await
        .expect("read analytical_results")
    }

    async fn relation_rows(pool: &SqlitePool) -> Vec<String> {
        sqlx::query_scalar(
            "SELECT id || '|' || result_id || '|' || related_result_id || '|' || relation_type || '|' || created_at FROM analytical_result_relations ORDER BY id",
        )
        .fetch_all(pool)
        .await
        .expect("read analytical_result_relations")
    }

    /// Dados de partida: um incidente, uma evidência, quatro resultados (um
    /// com `supersedes`) e três relações.
    async fn seed_rows(pool: &SqlitePool) {
        sqlx::query("INSERT INTO incidents(id,name) VALUES('inc-1','incident')")
            .execute(pool)
            .await
            .expect("incident insert");
        sqlx::query(
            "INSERT INTO evidence(id,original_filename,storage_reference,format,size_bytes,blake3,sha256,imported_at,status) VALUES('ev-1','t.evtx','evidence/ev-1.evtx','evtx',1,'b3','sha','2026-08-25T00:00:00Z','FORENSIC')",
        )
        .execute(pool)
        .await
        .expect("evidence insert");

        // (id, result_type, correlation_key, evidence_id, supersedes, confidence)
        let seeds = [
            ("r1", "IOC", "k-ioc", Some("ev-1"), None, Some("HIGH")),
            ("r2", "NIST_GAP", "k-gap", None, None, Some("MEDIUM")),
            ("r3", "NIST_GAP", "k-gap", None, Some("r2"), Some("HIGH")),
            ("r4", "RECOMMENDATION", "k-rec", None, None, None),
        ];
        for (id, result_type, key, evidence_id, supersedes, confidence) in seeds {
            sqlx::query(
                "INSERT INTO analytical_results(id,schema_version,result_type,correlation_key,evidence_id,incident_id,confidence,payload,provenance,supersedes,created_at) VALUES(?,?,?,?,?,?,?,?,?,?,?)",
            )
            .bind(id)
            .bind("1")
            .bind(result_type)
            .bind(key)
            .bind(evidence_id)
            .bind("inc-1")
            .bind(confidence)
            .bind(r#"{"k":1}"#)
            .bind(r#"{"p":2}"#)
            .bind(supersedes)
            .bind("2026-10-13T00:00:00Z")
            .execute(pool)
            .await
            .expect("analytical result insert");
        }
        let relations = [
            ("rel-0", "r4", "r1", "DERIVED_FROM"),
            ("rel-1", "r4", "r3", "DERIVED_FROM"),
            ("rel-2", "r1", "r2", "SEQUENCE_NEXT"),
        ];
        for (id, result_id, related_id, relation_type) in relations {
            sqlx::query(
                "INSERT INTO analytical_result_relations(id,result_id,related_result_id,relation_type,created_at) VALUES(?,?,?,?,?)",
            )
            .bind(id)
            .bind(result_id)
            .bind(related_id)
            .bind(relation_type)
            .bind("2026-10-13T00:00:00Z")
            .execute(pool)
            .await
            .expect("relation insert");
        }
    }

    #[tokio::test]
    async fn migration_0010_keeps_rows_relations_indexes_and_append_only_guards() {
        let temp = tempfile::tempdir().expect("temporary directory");
        let url = format!("sqlite://{}", temp.path().join("m0010.sqlite").display());
        let pool = connect(&url).await.expect("database connection");

        for sql in BEFORE_0010 {
            apply(&pool, sql).await;
        }

        seed_rows(&pool).await;

        let rows_before = result_rows(&pool).await;
        let relations_before = relation_rows(&pool).await;
        assert_eq!(rows_before.len(), 4);
        assert_eq!(relations_before.len(), 3);

        apply(&pool, MIGRATION_0010).await;

        // Dados: idênticos, na mesma ordem de `sequence`.
        assert_eq!(result_rows(&pool).await, rows_before);
        assert_eq!(relation_rows(&pool).await, relations_before);

        // Integridade: nenhuma FK pendurada e arquivo íntegro.
        let violations = sqlx::query("PRAGMA foreign_key_check")
            .fetch_all(&pool)
            .await
            .expect("foreign_key_check");
        assert!(violations.is_empty(), "foreign_key_check deve ser vazio");
        let integrity: String = sqlx::query_scalar("PRAGMA integrity_check")
            .fetch_one(&pool)
            .await
            .expect("integrity_check");
        assert_eq!(integrity, "ok");

        // Índices e triggers recriados.
        let indexes: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND tbl_name='analytical_results' AND name LIKE 'idx_analytical_results_%'",
        )
        .fetch_one(&pool)
        .await
        .expect("count indexes");
        assert_eq!(indexes, 5);
        let triggers: Vec<String> = sqlx::query_scalar(
            "SELECT name FROM sqlite_master WHERE type='trigger' AND tbl_name='analytical_results' ORDER BY name",
        )
        .fetch_all(&pool)
        .await
        .expect("list triggers");
        assert_eq!(
            triggers,
            vec![
                "analytical_results_no_delete",
                "analytical_results_no_update"
            ]
        );

        // Os três tipos novos passam a ser aceitos e `sequence` continua de 4.
        for (id, result_type) in [
            ("n-1", "FINDING"),
            ("n-2", "ROOT_CAUSE"),
            ("n-3", "PLAN_ITEM"),
        ] {
            sqlx::query(
                "INSERT INTO analytical_results(id,schema_version,result_type,correlation_key,incident_id,confidence,payload,provenance,created_at) VALUES(?,?,?,?,?,?,?,?,?)",
            )
            .bind(id)
            .bind("1")
            .bind(result_type)
            .bind(format!("k-{id}"))
            .bind("inc-1")
            .bind("HIGH")
            .bind("{}")
            .bind("{}")
            .bind("2026-10-13T00:00:00Z")
            .execute(&pool)
            .await
            .expect("new result type insert");
        }
        let new_sequences: Vec<i64> = sqlx::query_scalar(
            "SELECT sequence FROM analytical_results WHERE id IN ('n-1','n-2','n-3') ORDER BY sequence",
        )
        .fetch_all(&pool)
        .await
        .expect("read new sequences");
        assert_eq!(new_sequences, vec![5, 6, 7]);

        // Tipo desconhecido continua rejeitado pelo CHECK.
        let bogus = sqlx::query(
            "INSERT INTO analytical_results(id,schema_version,result_type,correlation_key,incident_id,payload,provenance,created_at) VALUES('x','1','BOGUS','k','inc-1','{}','{}','t')",
        )
        .execute(&pool)
        .await;
        assert!(
            bogus.is_err(),
            "result_type desconhecido deve ser rejeitado"
        );

        // Append-only continua valendo.
        let update = sqlx::query("UPDATE analytical_results SET payload='{}' WHERE id='r1'")
            .execute(&pool)
            .await;
        assert!(update.is_err(), "UPDATE deve continuar bloqueado");
        let delete = sqlx::query("DELETE FROM analytical_results WHERE id='r1'")
            .execute(&pool)
            .await;
        assert!(delete.is_err(), "DELETE deve continuar bloqueado");

        // FKs continuam valendo: relação para pai inexistente e supersedes órfão.
        let orphan_relation = sqlx::query(
            "INSERT INTO analytical_result_relations(id,result_id,related_result_id,relation_type,created_at) VALUES('bad','nope','r1','DERIVED_FROM','t')",
        )
        .execute(&pool)
        .await;
        assert!(
            orphan_relation.is_err(),
            "FK das relações deve continuar ativa"
        );
        let orphan_supersedes = sqlx::query(
            "INSERT INTO analytical_results(id,schema_version,result_type,correlation_key,incident_id,confidence,payload,provenance,supersedes,created_at) VALUES('y','1','FINDING','k','inc-1','HIGH','{}','{}','nope','t')",
        )
        .execute(&pool)
        .await;
        assert!(
            orphan_supersedes.is_err(),
            "FK de supersedes deve continuar ativa"
        );
    }

    #[tokio::test]
    async fn embedded_migrator_applies_0010_on_a_fresh_database() {
        let temp = tempfile::tempdir().expect("temporary directory");
        let url = format!("sqlite://{}", temp.path().join("fresh.sqlite").display());
        let pool = connect(&url).await.expect("database connection");
        migrate(&pool).await.expect("migrations");

        let applied: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _sqlx_migrations WHERE version = 10 AND success = 1",
        )
        .fetch_one(&pool)
        .await
        .expect("read _sqlx_migrations");
        assert_eq!(applied, 1);
    }

    #[tokio::test]
    async fn migration_0011_keeps_rows_adds_source_gap_and_the_evidence_columns() {
        let temp = tempfile::tempdir().expect("temporary directory");
        let url = format!("sqlite://{}", temp.path().join("m0011.sqlite").display());
        let pool = connect(&url).await.expect("database connection");
        for sql in BEFORE_0010 {
            apply(&pool, sql).await;
        }
        apply(&pool, MIGRATION_0010).await;
        seed_rows(&pool).await;
        // Resultados dos tipos que a 0010 criou têm de sobreviver à 0011.
        for (id, result_type, confidence) in [
            ("t-1", "FINDING", Some("HIGH")),
            ("t-2", "ROOT_CAUSE", Some("MEDIUM")),
            ("t-3", "PLAN_ITEM", None),
        ] {
            sqlx::query(
                "INSERT INTO analytical_results(id,schema_version,result_type,correlation_key,incident_id,confidence,payload,provenance,created_at) VALUES(?,?,?,?,?,?,?,?,?)",
            )
            .bind(id)
            .bind("1")
            .bind(result_type)
            .bind(format!("k-{id}"))
            .bind("inc-1")
            .bind(confidence)
            .bind("{}")
            .bind("{}")
            .bind("2026-10-13T00:00:00Z")
            .execute(&pool)
            .await
            .expect("0010-era result insert");
        }
        let rows_before = result_rows(&pool).await;
        let relations_before = relation_rows(&pool).await;
        assert_eq!(rows_before.len(), 7);

        apply(&pool, MIGRATION_0011).await;

        assert_eq!(result_rows(&pool).await, rows_before);
        assert_eq!(relation_rows(&pool).await, relations_before);
        let violations = sqlx::query("PRAGMA foreign_key_check")
            .fetch_all(&pool)
            .await
            .expect("foreign_key_check");
        assert!(violations.is_empty(), "foreign_key_check deve ser vazio");
        let integrity: String = sqlx::query_scalar("PRAGMA integrity_check")
            .fetch_one(&pool)
            .await
            .expect("integrity_check");
        assert_eq!(integrity, "ok");
        let indexes: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND tbl_name='analytical_results' AND name LIKE 'idx_analytical_results_%'",
        )
        .fetch_one(&pool)
        .await
        .expect("count indexes");
        assert_eq!(indexes, 5);
        let triggers: Vec<String> = sqlx::query_scalar(
            "SELECT name FROM sqlite_master WHERE type='trigger' AND tbl_name='analytical_results' ORDER BY name",
        )
        .fetch_all(&pool)
        .await
        .expect("list triggers");
        assert_eq!(
            triggers,
            vec![
                "analytical_results_no_delete",
                "analytical_results_no_update"
            ]
        );

        // O tipo novo é aceito, a sequência continua e o inválido segue recusado.
        sqlx::query(
            "INSERT INTO analytical_results(id,schema_version,result_type,correlation_key,incident_id,payload,provenance,created_at) VALUES('g-1','1','SOURCE_GAP','k-g','inc-1','{}','{}','2026-10-16T00:00:00Z')",
        )
        .execute(&pool)
        .await
        .expect("SOURCE_GAP insert");
        let sequence: i64 =
            sqlx::query_scalar("SELECT sequence FROM analytical_results WHERE id = 'g-1'")
                .fetch_one(&pool)
                .await
                .expect("read sequence");
        assert_eq!(sequence, 8);
        let bogus = sqlx::query(
            "INSERT INTO analytical_results(id,schema_version,result_type,correlation_key,incident_id,payload,provenance,created_at) VALUES('x','1','BOGUS','k','inc-1','{}','{}','t')",
        )
        .execute(&pool)
        .await;
        assert!(
            bogus.is_err(),
            "result_type desconhecido deve ser rejeitado"
        );
        let update = sqlx::query("UPDATE analytical_results SET payload='{}' WHERE id='r1'")
            .execute(&pool)
            .await;
        assert!(update.is_err(), "UPDATE deve continuar bloqueado");

        // Colunas novas de evidência: valores de partida e restrição do flag.
        let (provided_by, personal): (Option<String>, i64) = sqlx::query_as(
            "SELECT provided_by, contains_personal_data FROM evidence WHERE id = 'ev-1'",
        )
        .fetch_one(&pool)
        .await
        .expect("read evidence columns");
        assert_eq!((provided_by, personal), (None, 0));
        let bad_flag =
            sqlx::query("UPDATE evidence SET contains_personal_data = 2 WHERE id = 'ev-1'")
                .execute(&pool)
                .await;
        assert!(bad_flag.is_err(), "o flag só aceita 0 ou 1");
    }

    #[tokio::test]
    async fn embedded_migrator_applies_0011_on_a_fresh_database() {
        let temp = tempfile::tempdir().expect("temporary directory");
        let url = format!("sqlite://{}", temp.path().join("fresh11.sqlite").display());
        let pool = connect(&url).await.expect("database connection");
        migrate(&pool).await.expect("migrations");

        let applied: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _sqlx_migrations WHERE version = 11 AND success = 1",
        )
        .fetch_one(&pool)
        .await
        .expect("read _sqlx_migrations");
        assert_eq!(applied, 1);
    }
}
