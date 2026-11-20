//! Cópia verificada do banco antes de aplicar migration (RF-032, spec §29.4.4).
//!
//! As migrations 0010 e 0011 recriam a tabela de resultados analíticos; uma
//! falha no meio, ou um defeito na própria migration, não pode custar o banco
//! de um caso. Antes de aplicar qualquer migration pendente sobre um banco que
//! já tem dados, a aplicação:
//!
//! 1. cria uma cópia consistente com `VACUUM INTO` (inclui o conteúdo do WAL);
//! 2. calcula o SHA-256 da cópia;
//! 3. **verifica** a cópia: `PRAGMA integrity_check`, `PRAGMA foreign_key_check`,
//!    mesmo conjunto de migrations aplicadas e mesma contagem de linhas em cada
//!    tabela do banco de origem;
//! 4. grava o `manifest.json` (versões, SHA-256, contagens, resultado da
//!    verificação) e só então publica a pasta do backup.
//!
//! Se qualquer passo falhar, a cópia parcial é descartada e **a migration não
//! é aplicada** (falha fechada): o banco original segue como estava.
//!
//! Esta cópia **não** substitui o backup da pasta de dados inteira
//! (`docs/OPERATION.md`): ela guarda só o banco, que é o que as migrations
//! alteram. As evidências em `evidence/` não são tocadas por migration.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{
    SqlitePool,
    migrate::Migrator,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
};
use std::{
    collections::BTreeMap,
    io::Read,
    path::{Path, PathBuf},
    str::FromStr,
};

pub const MANIFEST_SCHEMA: &str = "dfir-db-backup/1";
pub const MANIFEST_FILE: &str = "manifest.json";
pub const DATABASE_FILE: &str = "dfir.sqlite";

/// Conteúdo de `manifest.json` de um backup pré-migration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackupManifest {
    pub schema: String,
    pub created_at: String,
    pub tool_version: String,
    /// Maior versão de migration aplicada no banco copiado.
    pub from_version: i64,
    /// Maior versão de migration que esta build aplica.
    pub to_version: i64,
    pub pending_versions: Vec<i64>,
    pub database_file: String,
    pub database_bytes: u64,
    pub database_sha256: String,
    /// Resultado de `PRAGMA integrity_check` na cópia (`ok` quando íntegra).
    pub integrity_check: String,
    pub foreign_key_violations: u64,
    /// Linhas por tabela no banco de origem no instante da cópia.
    pub table_row_counts: BTreeMap<String, u64>,
    pub verified: bool,
}

/// O que `migrate_with_backup` fez.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackupOutcome {
    /// Não havia o que proteger (banco novo) ou nada a migrar.
    NotNeeded(&'static str),
    Created {
        dir: PathBuf,
        manifest: Box<BackupManifest>,
    },
}

fn protocol(message: impl Into<String>) -> sqlx::Error {
    sqlx::Error::Protocol(message.into())
}

fn io_error(context: &str, error: std::io::Error) -> sqlx::Error {
    sqlx::Error::Protocol(format!("{context}: {error}"))
}

/// Versões já aplicadas com sucesso. `None` se o banco nunca foi migrado
/// (sem a tabela de controle do sqlx, ou com ela vazia).
pub async fn applied_versions(pool: &SqlitePool) -> Result<Option<Vec<i64>>, sqlx::Error> {
    let has_table: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='_sqlx_migrations'",
    )
    .fetch_one(pool)
    .await?;
    if has_table == 0 {
        return Ok(None);
    }
    let versions: Vec<i64> = sqlx::query_scalar(
        "SELECT version FROM _sqlx_migrations WHERE success = 1 ORDER BY version",
    )
    .fetch_all(pool)
    .await?;
    if versions.is_empty() {
        return Ok(None);
    }
    Ok(Some(versions))
}

/// Versões do `migrator` ainda não aplicadas, em ordem.
pub fn pending_versions(migrator: &Migrator, applied: &[i64]) -> Vec<i64> {
    let mut pending: Vec<i64> = migrator
        .iter()
        .filter(|migration| !migration.migration_type.is_down_migration())
        .map(|migration| migration.version)
        .filter(|version| !applied.contains(version))
        .collect();
    pending.sort_unstable();
    pending
}

/// Caminho do arquivo do banco principal da conexão.
async fn main_database_file(pool: &SqlitePool) -> Result<PathBuf, sqlx::Error> {
    let file: Option<String> =
        sqlx::query_scalar("SELECT file FROM pragma_database_list WHERE name='main'")
            .fetch_optional(pool)
            .await?;
    match file {
        Some(file) if !file.is_empty() => Ok(PathBuf::from(file)),
        _ => Err(protocol(
            "the database is not backed by a file; cannot create a pre-migration backup",
        )),
    }
}

/// Nomes das tabelas do usuário, em ordem.
async fn user_tables(pool: &SqlitePool) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
    )
    .fetch_all(pool)
    .await
}

async fn row_counts(pool: &SqlitePool) -> Result<BTreeMap<String, u64>, sqlx::Error> {
    let mut counts = BTreeMap::new();
    for table in user_tables(pool).await? {
        // O nome vem do próprio `sqlite_master` do banco; mesmo assim é citado
        // como identificador.
        let quoted = table.replace('"', "\"\"");
        let sql = format!("SELECT count(*) FROM \"{quoted}\"");
        let count: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(sql))
            .fetch_one(pool)
            .await?;
        counts.insert(table, count.max(0) as u64);
    }
    Ok(counts)
}

fn sha256_file(path: &Path) -> std::io::Result<(String, u64)> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; 1024 * 1024];
    let mut total = 0_u64;
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
        total += read as u64;
    }
    Ok((hex(&hasher.finalize()), total))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(unix)]
fn restrict(path: &Path, mode: u32) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))
}

#[cfg(not(unix))]
fn restrict(_path: &Path, _mode: u32) -> std::io::Result<()> {
    Ok(())
}

/// Abre a cópia **somente leitura** e confere o que o manifest afirma.
/// Devolve `(integrity_check, violações de chave estrangeira)`.
pub async fn verify_copy(
    copy: &Path,
    expected_sha256: &str,
    expected_applied: &[i64],
    expected_counts: &BTreeMap<String, u64>,
) -> Result<(String, u64), sqlx::Error> {
    let (actual_sha, _) =
        sha256_file(copy).map_err(|error| io_error("reading the backup", error))?;
    if actual_sha != expected_sha256 {
        return Err(protocol(
            "the backup file does not match its recorded SHA-256",
        ));
    }

    let options = SqliteConnectOptions::from_str(&format!("sqlite://{}", copy.display()))?
        .read_only(true)
        .journal_mode(SqliteJournalMode::Delete);
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await?;
    let result = async {
        let integrity: String = sqlx::query_scalar("PRAGMA integrity_check")
            .fetch_one(&pool)
            .await?;
        if integrity != "ok" {
            return Err(protocol(format!(
                "the backup failed PRAGMA integrity_check: {integrity}"
            )));
        }
        let violations = sqlx::query("PRAGMA foreign_key_check")
            .fetch_all(&pool)
            .await?
            .len() as u64;
        if violations > 0 {
            return Err(protocol(format!(
                "the backup has {violations} foreign key violation(s)"
            )));
        }
        let applied = applied_versions(&pool).await?.unwrap_or_default();
        if applied != expected_applied {
            return Err(protocol(
                "the backup does not hold the same applied migrations as the source",
            ));
        }
        let counts = row_counts(&pool).await?;
        if &counts != expected_counts {
            return Err(protocol(
                "the backup does not hold the same row counts as the source",
            ));
        }
        Ok((integrity, violations))
    }
    .await;
    pool.close().await;
    result
}

/// Cria, verifica e publica o backup. `backups_dir` é criada se não existir.
async fn create_backup(
    pool: &SqlitePool,
    backups_dir: &Path,
    applied: &[i64],
    pending: &[i64],
    now: chrono::DateTime<chrono::Utc>,
) -> Result<(PathBuf, BackupManifest), sqlx::Error> {
    let from_version = applied.iter().copied().max().unwrap_or(0);
    let to_version = pending.iter().copied().max().unwrap_or(from_version);
    let name = format!(
        "pre-migration-{}-v{from_version}-to-v{to_version}",
        now.format("%Y%m%dT%H%M%SZ")
    );
    std::fs::create_dir_all(backups_dir)
        .map_err(|error| io_error("creating backups dir", error))?;
    restrict(backups_dir, 0o700).map_err(|error| io_error("restricting backups dir", error))?;
    let final_dir = backups_dir.join(&name);
    if final_dir.exists() {
        return Err(protocol(format!("backup folder already exists: {name}")));
    }
    let partial_dir = backups_dir.join(format!("{name}.partial"));
    if partial_dir.exists() {
        std::fs::remove_dir_all(&partial_dir)
            .map_err(|error| io_error("clearing a stale partial backup", error))?;
    }
    std::fs::create_dir(&partial_dir).map_err(|error| io_error("creating the backup", error))?;
    restrict(&partial_dir, 0o700).map_err(|error| io_error("restricting the backup", error))?;

    let outcome = async {
        let copy = partial_dir.join(DATABASE_FILE);
        sqlx::query("VACUUM INTO ?")
            .bind(copy.to_string_lossy().into_owned())
            .execute(pool)
            .await?;
        restrict(&copy, 0o600).map_err(|error| io_error("restricting the backup file", error))?;

        let counts = row_counts(pool).await?;
        let (sha256, bytes) =
            sha256_file(&copy).map_err(|error| io_error("hashing the backup", error))?;
        let (integrity, violations) = verify_copy(&copy, &sha256, applied, &counts).await?;

        let manifest = BackupManifest {
            schema: MANIFEST_SCHEMA.into(),
            created_at: now.to_rfc3339(),
            tool_version: env!("CARGO_PKG_VERSION").into(),
            from_version,
            to_version,
            pending_versions: pending.to_vec(),
            database_file: DATABASE_FILE.into(),
            database_bytes: bytes,
            database_sha256: sha256,
            integrity_check: integrity,
            foreign_key_violations: violations,
            table_row_counts: counts,
            verified: true,
        };
        let json = serde_json::to_string_pretty(&manifest)
            .map_err(|error| protocol(format!("encoding the backup manifest: {error}")))?;
        let manifest_path = partial_dir.join(MANIFEST_FILE);
        std::fs::write(&manifest_path, json)
            .map_err(|error| io_error("writing the backup manifest", error))?;
        restrict(&manifest_path, 0o600)
            .map_err(|error| io_error("restricting the backup manifest", error))?;
        Ok::<BackupManifest, sqlx::Error>(manifest)
    }
    .await;

    match outcome {
        Ok(manifest) => {
            std::fs::rename(&partial_dir, &final_dir)
                .map_err(|error| io_error("publishing the backup", error))?;
            Ok((final_dir, manifest))
        }
        Err(error) => {
            // A cópia parcial é artefato nosso, nunca evidência: descartá-la
            // evita que uma cópia inválida passe por backup disponível.
            // F1-23: registrar a falha de descarte em vez de engolir.
            if let Err(cleanup_error) = std::fs::remove_dir_all(&partial_dir)
                && cleanup_error.kind() != std::io::ErrorKind::NotFound
            {
                tracing::warn!(%cleanup_error, "failed to discard partial backup copy");
            }
            Err(error)
        }
    }
}

/// Aplica as migrations pendentes **depois** de copiar e verificar o banco.
///
/// - Banco novo (nunca migrado): nada a proteger, migra direto.
/// - Nada pendente: não copia; roda o migrator só para a checagem de integridade
///   das migrations já aplicadas (checksum).
/// - Pendente sobre banco com dados: copia, verifica e só então migra. Se a
///   cópia ou a verificação falhar, devolve erro **sem migrar**.
///
/// `min_free_bytes` é a margem de espaço livre que deve sobrar no volume depois
/// da cópia (`0` desliga a checagem de espaço).
pub async fn migrate_with_backup(
    pool: &SqlitePool,
    migrator: &Migrator,
    backups_dir: &Path,
    min_free_bytes: u64,
) -> Result<BackupOutcome, sqlx::Error> {
    let Some(applied) = applied_versions(pool).await? else {
        migrator.run(pool).await?;
        return Ok(BackupOutcome::NotNeeded("new database"));
    };
    let pending = pending_versions(migrator, &applied);
    if pending.is_empty() {
        migrator.run(pool).await?;
        return Ok(BackupOutcome::NotNeeded("no pending migrations"));
    }

    let database_file = main_database_file(pool).await?;
    let database_bytes = std::fs::metadata(&database_file)
        .map(|metadata| metadata.len())
        .map_err(|error| io_error("reading the database size", error))?;
    // O VACUUM INTO escreve uma cópia do tamanho do banco (no máximo).
    let required = database_bytes.saturating_add(min_free_bytes);
    std::fs::create_dir_all(backups_dir)
        .map_err(|error| io_error("creating backups dir", error))?;
    if let Some(message) = crate::infrastructure::disk::shortage_message(backups_dir, required) {
        return Err(protocol(format!(
            "cannot create the pre-migration backup, so the migration was not applied: {message}"
        )));
    }

    let (dir, manifest) = create_backup(pool, backups_dir, &applied, &pending, chrono::Utc::now())
        .await
        .map_err(|error| {
            protocol(format!(
                "the pre-migration backup failed, so the migration was not applied: {error}"
            ))
        })?;
    tracing::info!(
        backup = %dir.display(),
        from_version = manifest.from_version,
        to_version = manifest.to_version,
        bytes = manifest.database_bytes,
        "pre-migration database backup created and verified"
    );

    migrator.run(pool).await?;

    // Registra na cadeia de auditoria **depois** da migration (a tabela de
    // auditoria precisa existir na versão nova). Só nome da pasta, versões,
    // hash e tamanho: nenhum caminho absoluto.
    let metadata = serde_json::json!({
        "backup_folder": dir.file_name().map(|name| name.to_string_lossy().into_owned()),
        "from_version": manifest.from_version,
        "to_version": manifest.to_version,
        "database_sha256": manifest.database_sha256,
        "database_bytes": manifest.database_bytes,
        "verified": manifest.verified,
    });
    if let Err(error) = crate::persistence::audit::append(
        pool,
        None,
        "DB_PRE_MIGRATION_BACKUP",
        "database",
        &manifest.database_sha256,
        None,
        &metadata.to_string(),
    )
    .await
    {
        // A migration já foi aplicada e o backup existe: falhar aqui não
        // desfaz nada, mas não pode passar em silêncio.
        tracing::error!(%error, "could not record the pre-migration backup in the audit chain");
    }
    Ok(BackupOutcome::Created {
        dir,
        manifest: Box::new(manifest),
    })
}

/// Lê e confere um backup já publicado (para `verify` manual e testes).
pub async fn verify_published(dir: &Path) -> Result<BackupManifest, sqlx::Error> {
    let manifest_text = std::fs::read_to_string(dir.join(MANIFEST_FILE))
        .map_err(|error| io_error("reading the backup manifest", error))?;
    let manifest: BackupManifest = serde_json::from_str(&manifest_text)
        .map_err(|error| protocol(format!("invalid backup manifest: {error}")))?;
    if manifest.schema != MANIFEST_SCHEMA {
        return Err(protocol("unsupported backup manifest schema"));
    }
    let copy = dir.join(&manifest.database_file);
    // O hash vem ANTES de abrir o arquivo: um arquivo adulterado nem chega ao
    // SQLite.
    let (actual_sha, _) =
        sha256_file(&copy).map_err(|error| io_error("reading the backup", error))?;
    if actual_sha != manifest.database_sha256 {
        return Err(protocol(
            "the backup file does not match its recorded SHA-256",
        ));
    }
    // O manifest guarda as contagens do banco de origem e a lista de
    // migrations aplicadas é a de 1..=from_version presente na cópia: a
    // comparação usa o que a própria cópia declara.
    let options = SqliteConnectOptions::from_str(&format!("sqlite://{}", copy.display()))?
        .read_only(true)
        .journal_mode(SqliteJournalMode::Delete);
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await?;
    let applied = applied_versions(&pool).await?.unwrap_or_default();
    pool.close().await;
    verify_copy(
        &copy,
        &manifest.database_sha256,
        &applied,
        &manifest.table_row_counts,
    )
    .await?;
    Ok(manifest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::db::MIGRATOR;

    async fn file_pool(path: &Path) -> SqlitePool {
        crate::persistence::db::connect(&format!("sqlite://{}", path.display()))
            .await
            .expect("connect")
    }

    /// Migrations 0001..=0009 aplicadas à mão, para ter pendentes 0010+.
    async fn migrate_up_to(pool: &SqlitePool, last_version: i64) {
        let mut tx = pool.begin().await.expect("begin");
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS _sqlx_migrations (version BIGINT PRIMARY KEY, description TEXT NOT NULL, installed_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP, success BOOLEAN NOT NULL, checksum BLOB NOT NULL, execution_time BIGINT NOT NULL)",
        )
        .execute(&mut *tx)
        .await
        .expect("control table");
        for migration in MIGRATOR.iter() {
            if migration.version > last_version {
                continue;
            }
            sqlx::query(migration.sql.clone())
                .execute(&mut *tx)
                .await
                .expect("migration sql");
            sqlx::query(
                "INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time) VALUES (?,?,1,?,0)",
            )
            .bind(migration.version)
            .bind(migration.description.to_string())
            .bind(migration.checksum.to_vec())
            .execute(&mut *tx)
            .await
            .expect("record migration");
        }
        tx.commit().await.expect("commit");
    }

    async fn seed(pool: &SqlitePool) {
        sqlx::query("INSERT INTO incidents(id,name) VALUES('inc-1','incident')")
            .execute(pool)
            .await
            .expect("incident");
        sqlx::query(
            "INSERT INTO evidence(id,original_filename,storage_reference,format,size_bytes,blake3,sha256,imported_at,status) VALUES('ev-1','t.evtx','evidence/ev-1.evtx','evtx',1,'b3','sha','2026-08-25T00:00:00Z','FORENSIC')",
        )
        .execute(pool)
        .await
        .expect("evidence");
    }

    #[tokio::test]
    async fn a_new_database_is_migrated_without_a_backup() {
        let dir = tempfile::tempdir().expect("dir");
        let pool = file_pool(&dir.path().join("dfir.sqlite")).await;
        let backups = dir.path().join("backups");
        let outcome = migrate_with_backup(&pool, &MIGRATOR, &backups, 0)
            .await
            .expect("migrate");
        assert_eq!(outcome, BackupOutcome::NotNeeded("new database"));
        assert!(!backups.exists(), "nada a proteger num banco novo");
        assert!(applied_versions(&pool).await.expect("applied").is_some());
    }

    #[tokio::test]
    async fn an_up_to_date_database_is_not_copied_again() {
        let dir = tempfile::tempdir().expect("dir");
        let pool = file_pool(&dir.path().join("dfir.sqlite")).await;
        let backups = dir.path().join("backups");
        migrate_with_backup(&pool, &MIGRATOR, &backups, 0)
            .await
            .expect("first");
        let outcome = migrate_with_backup(&pool, &MIGRATOR, &backups, 0)
            .await
            .expect("second");
        assert_eq!(outcome, BackupOutcome::NotNeeded("no pending migrations"));
        assert!(!backups.exists());
    }

    #[tokio::test]
    async fn pending_migrations_over_data_create_a_verified_backup_first() {
        let dir = tempfile::tempdir().expect("dir");
        let pool = file_pool(&dir.path().join("dfir.sqlite")).await;
        migrate_up_to(&pool, 9).await;
        seed(&pool).await;
        let backups = dir.path().join("backups");

        let outcome = migrate_with_backup(&pool, &MIGRATOR, &backups, 0)
            .await
            .expect("migrate with backup");
        let BackupOutcome::Created {
            dir: folder,
            manifest,
        } = outcome
        else {
            panic!("expected a backup, got {outcome:?}");
        };
        assert_eq!(manifest.from_version, 9);
        assert!(manifest.to_version >= 11);
        assert_eq!(manifest.pending_versions.first().copied(), Some(10));
        assert!(manifest.verified);
        assert_eq!(manifest.integrity_check, "ok");
        assert_eq!(manifest.table_row_counts.get("evidence"), Some(&1));
        assert_eq!(manifest.table_row_counts.get("incidents"), Some(&1));
        assert!(folder.join(DATABASE_FILE).is_file());
        assert!(folder.join(MANIFEST_FILE).is_file());

        // O backup guarda o banco **antes** da migration; o banco vivo, depois.
        let copy_pool = SqlitePoolOptions::new()
            .connect_with(
                SqliteConnectOptions::from_str(&format!(
                    "sqlite://{}",
                    folder.join(DATABASE_FILE).display()
                ))
                .expect("options")
                .read_only(true)
                .journal_mode(SqliteJournalMode::Delete),
            )
            .await
            .expect("open copy");
        let copy_versions = applied_versions(&copy_pool)
            .await
            .expect("applied")
            .expect("some");
        assert_eq!(copy_versions.iter().copied().max(), Some(9));
        copy_pool.close().await;
        let live_versions = applied_versions(&pool)
            .await
            .expect("applied")
            .expect("some");
        assert!(live_versions.iter().copied().max() >= Some(11));

        // Auditoria: sem caminho absoluto, com o hash do arquivo.
        let metadata: String = sqlx::query_scalar(
            "SELECT metadata FROM audit_events WHERE action='DB_PRE_MIGRATION_BACKUP'",
        )
        .fetch_one(&pool)
        .await
        .expect("audit event");
        assert!(metadata.contains(&manifest.database_sha256));
        assert!(!metadata.contains(dir.path().to_string_lossy().as_ref()));

        // O backup publicado é reverificável depois, de forma independente.
        let reverified = verify_published(&folder).await.expect("reverify");
        assert_eq!(reverified.database_sha256, manifest.database_sha256);
    }

    #[tokio::test]
    async fn a_tampered_backup_is_rejected_by_verification() {
        let dir = tempfile::tempdir().expect("dir");
        let pool = file_pool(&dir.path().join("dfir.sqlite")).await;
        migrate_up_to(&pool, 9).await;
        seed(&pool).await;
        let backups = dir.path().join("backups");
        let outcome = migrate_with_backup(&pool, &MIGRATOR, &backups, 0)
            .await
            .expect("migrate with backup");
        let BackupOutcome::Created { dir: folder, .. } = outcome else {
            panic!("expected a backup");
        };

        // Troca um byte do arquivo: o SHA-256 do manifest deixa de bater.
        let file = folder.join(DATABASE_FILE);
        let mut bytes = std::fs::read(&file).expect("read");
        let last = bytes.len() - 1;
        bytes[last] ^= 0xFF;
        std::fs::write(&file, bytes).expect("tamper");
        let error = verify_published(&folder).await.expect_err("tampered");
        assert!(error.to_string().contains("SHA-256"), "{error}");
    }

    #[tokio::test]
    async fn a_backup_that_cannot_be_made_stops_the_migration() {
        let dir = tempfile::tempdir().expect("dir");
        let pool = file_pool(&dir.path().join("dfir.sqlite")).await;
        migrate_up_to(&pool, 9).await;
        seed(&pool).await;
        // `backups` existe como ARQUIVO: não dá para criar a pasta do backup.
        let blocked = dir.path().join("backups");
        std::fs::write(&blocked, b"not a directory").expect("blocker");

        let error = migrate_with_backup(&pool, &MIGRATOR, &blocked, 0)
            .await
            .expect_err("must refuse");
        assert!(error.to_string().contains("backup"), "{error}");
        // A migration NÃO foi aplicada: o banco continua na versão 9.
        let versions = applied_versions(&pool)
            .await
            .expect("applied")
            .expect("some");
        assert_eq!(versions.iter().copied().max(), Some(9));
    }

    #[tokio::test]
    async fn a_shortage_of_free_space_stops_the_migration_before_copying() {
        let dir = tempfile::tempdir().expect("dir");
        let pool = file_pool(&dir.path().join("dfir.sqlite")).await;
        migrate_up_to(&pool, 9).await;
        seed(&pool).await;
        let backups = dir.path().join("backups");
        // Margem absurda (1 TiB): a checagem recusa antes de copiar.
        let error = migrate_with_backup(&pool, &MIGRATOR, &backups, 1024 * 1024 * 1024 * 1024)
            .await
            .expect_err("no space");
        if cfg!(unix) {
            assert!(
                error.to_string().contains("not enough free disk space"),
                "{error}"
            );
            let versions = applied_versions(&pool)
                .await
                .expect("applied")
                .expect("some");
            assert_eq!(versions.iter().copied().max(), Some(9));
        }
    }

    /// A 0012 recria `analysis_jobs` para aceitar `EVTX_DETECTION`: os jobs
    /// existentes (com avisos e tipo) precisam sobreviver, e o índice de
    /// timeline deve ser invalidado para ser refeito com os campos novos.
    #[tokio::test]
    async fn migration_0012_keeps_existing_jobs_and_accepts_the_detection_kind() {
        let dir = tempfile::tempdir().expect("dir");
        let pool = file_pool(&dir.path().join("dfir.sqlite")).await;
        migrate_up_to(&pool, 11).await;
        seed(&pool).await;
        sqlx::query(
            "INSERT INTO analysis_jobs(id,request_id,evidence_id,status,max_output_bytes,max_items,item_count,created_at,kind,warning_count,warnings) VALUES('job-1','req-1','ev-1','COMPLETED',100,100,7,'2026-10-01T00:00:00Z','YARA_SCAN',1,'[{\"code\":\"X\"}]'), ('job-2','req-2','ev-1','FAILED',100,100,0,'2026-10-02T00:00:00Z','EVTX_TIMELINE',0,NULL)",
        )
        .execute(&pool)
        .await
        .expect("jobs");
        sqlx::query("INSERT INTO timeline_index(job_id,event_count,skipped_count,indexed_at) VALUES('job-2',5,0,'2026-10-02T00:00:00Z')")
            .execute(&pool)
            .await
            .expect("index");
        let before = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM timeline_index")
            .fetch_one(&pool)
            .await
            .expect("count");
        assert_eq!(before, 1);

        let backups = dir.path().join("backups");
        let outcome = migrate_with_backup(&pool, &MIGRATOR, &backups, 0)
            .await
            .expect("migrate");
        assert!(matches!(outcome, BackupOutcome::Created { .. }));

        // Linhas, tipo, contadores e avisos preservados.
        let (kind, items, warnings): (String, i64, Option<String>) =
            sqlx::query_as("SELECT kind,item_count,warnings FROM analysis_jobs WHERE id='job-1'")
                .fetch_one(&pool)
                .await
                .expect("job-1");
        assert_eq!((kind.as_str(), items), ("YARA_SCAN", 7));
        assert_eq!(warnings.as_deref(), Some("[{\"code\":\"X\"}]"));
        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM analysis_jobs")
            .fetch_one(&pool)
            .await
            .expect("count");
        assert_eq!(total, 2);

        // O tipo novo é aceito; um tipo desconhecido continua recusado.
        let detection = sqlx::query(
            "INSERT INTO analysis_jobs(id,request_id,evidence_id,status,max_output_bytes,max_items,created_at,kind) VALUES('job-3','req-3','ev-1','PROCESSING',1,1,'2026-10-03T00:00:00Z','EVTX_DETECTION')",
        )
        .execute(&pool)
        .await;
        assert!(detection.is_ok(), "{detection:?}");
        let bogus = sqlx::query(
            "INSERT INTO analysis_jobs(id,request_id,evidence_id,status,max_output_bytes,max_items,created_at,kind) VALUES('job-4','req-4','ev-1','PROCESSING',1,1,'2026-10-03T00:00:00Z','QUALQUER')",
        )
        .execute(&pool)
        .await;
        assert!(bogus.is_err());

        // Índices recriados e a chave estrangeira para `evidence` segue valendo.
        let indexes: Vec<String> = sqlx::query_scalar(
            "SELECT name FROM sqlite_master WHERE type='index' AND tbl_name='analysis_jobs' AND name LIKE 'idx_%' ORDER BY name",
        )
        .fetch_all(&pool)
        .await
        .expect("indexes");
        assert_eq!(
            indexes,
            [
                "idx_analysis_jobs_evidence",
                "idx_analysis_jobs_kind",
                "idx_analysis_jobs_status"
            ]
        );
        let orphan = sqlx::query(
            "INSERT INTO analysis_jobs(id,request_id,evidence_id,status,max_output_bytes,max_items,created_at) VALUES('job-5','req-5','nao-existe','PENDING',1,1,'2026-10-03T00:00:00Z')",
        )
        .execute(&pool)
        .await;
        assert!(orphan.is_err(), "FK para evidence preservada");

        // O índice de timeline foi invalidado: será refeito na primeira consulta.
        let after: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM timeline_index")
            .fetch_one(&pool)
            .await
            .expect("count");
        assert_eq!(after, 0);
        let violations = sqlx::query("PRAGMA foreign_key_check")
            .fetch_all(&pool)
            .await
            .expect("fk check");
        assert!(violations.is_empty());
    }

    /// A 0014 preserva os jobs existentes, aceita CANCELLED somente com a
    /// requisição persistida e mantém índices/FK da tabela recriada.
    #[tokio::test]
    async fn migration_0014_preserves_jobs_and_adds_guarded_cancellation() {
        let dir = tempfile::tempdir().expect("dir");
        let pool = file_pool(&dir.path().join("dfir.sqlite")).await;
        migrate_up_to(&pool, 13).await;
        seed(&pool).await;
        sqlx::query(
            "INSERT INTO analysis_jobs(id,request_id,evidence_id,status,max_output_bytes,max_items,item_count,created_at,kind,warning_count,warnings) VALUES('job-cancel-1','req-cancel-1','ev-1','COMPLETED',100,100,7,'2026-10-01T00:00:00Z','YARA_SCAN',1,'[{\"code\":\"X\"}]'), ('job-cancel-2','req-cancel-2','ev-1','FAILED',100,100,0,'2026-10-02T00:00:00Z','EVTX_TIMELINE',0,NULL)",
        )
        .execute(&pool)
        .await
        .expect("legacy jobs");
        let backups = dir.path().join("backups");
        let outcome = migrate_with_backup(&pool, &MIGRATOR, &backups, 0)
            .await
            .expect("migrate 0014");
        assert!(matches!(outcome, BackupOutcome::Created { .. }));

        let preserved: (String, i64, Option<String>) = sqlx::query_as(
            "SELECT kind,item_count,warnings FROM analysis_jobs WHERE id='job-cancel-1'",
        )
        .fetch_one(&pool)
        .await
        .expect("preserved job");
        assert_eq!(
            (preserved.0.as_str(), preserved.1, preserved.2.as_deref()),
            ("YARA_SCAN", 7, Some("[{\"code\":\"X\"}]"))
        );
        let cancelled = sqlx::query(
            "INSERT INTO analysis_jobs(id,request_id,evidence_id,status,max_output_bytes,max_items,created_at,cancellation_requested_at,error_code) VALUES('job-cancel-3','req-cancel-3','ev-1','CANCELLED',1,1,'2026-10-03T00:00:00Z','2026-10-03T00:01:00Z','CANCELLED')",
        )
        .execute(&pool)
        .await;
        assert!(cancelled.is_ok(), "{cancelled:?}");
        let unrequested = sqlx::query(
            "INSERT INTO analysis_jobs(id,request_id,evidence_id,status,max_output_bytes,max_items,created_at) VALUES('job-cancel-4','req-cancel-4','ev-1','CANCELLED',1,1,'2026-10-03T00:00:00Z')",
        )
        .execute(&pool)
        .await;
        assert!(unrequested.is_err(), "CANCELLED sem requisição foi aceito");
        let orphan = sqlx::query(
            "INSERT INTO analysis_jobs(id,request_id,evidence_id,status,max_output_bytes,max_items,created_at) VALUES('job-cancel-5','req-cancel-5','missing','PENDING',1,1,'2026-10-03T00:00:00Z')",
        )
        .execute(&pool)
        .await;
        assert!(orphan.is_err(), "FK para evidence não foi preservada");
        let indexes: Vec<String> = sqlx::query_scalar(
            "SELECT name FROM sqlite_master WHERE type='index' AND tbl_name='analysis_jobs' AND name LIKE 'idx_%' ORDER BY name",
        )
        .fetch_all(&pool)
        .await
        .expect("indexes");
        assert_eq!(
            indexes,
            [
                "idx_analysis_jobs_evidence",
                "idx_analysis_jobs_kind",
                "idx_analysis_jobs_status"
            ]
        );
        let violations = sqlx::query("PRAGMA foreign_key_check")
            .fetch_all(&pool)
            .await
            .expect("fk check");
        assert!(violations.is_empty());
    }

    #[test]
    fn pending_versions_lists_only_what_is_missing_in_order() {
        let all: Vec<i64> = MIGRATOR.iter().map(|migration| migration.version).collect();
        let applied: Vec<i64> = all.iter().copied().filter(|v| *v <= 4).collect();
        let pending = pending_versions(&MIGRATOR, &applied);
        assert_eq!(pending.first().copied(), Some(5));
        assert_eq!(pending.len(), all.len() - applied.len());
        assert!(pending.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(pending_versions(&MIGRATOR, &all).is_empty());
    }
}
