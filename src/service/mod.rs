//! Camada de serviço: a lógica de negócio que antes vivia em `src/web/mod.rs`
//! (handlers Axum), agora livre de HTTP/JSON-sobre-fio. A interface nativa
//! (`src/gui/`) chama estas funções diretamente, no mesmo processo — não há
//! mais servidor local, rota, nem serialização de ida e volta.
//!
//! A extração é deliberadamente comportamental, não uma reescrita: cada
//! função aqui corresponde 1:1 a um handler antigo, com a mesma validação,
//! as mesmas transações SQLite (`BEGIN IMMEDIATE`/`COMMIT`/`ROLLBACK`) e os
//! mesmos eventos de auditoria/custódia. Ver `docs/adrs/ADR-0017-interface-nativa-eframe-egui.md`.
//!
//! `src/persistence/`, `src/application/` (engine/process) e
//! `src/infrastructure/storage.rs` não mudam: só a camada que os orquestrava
//! para servir uma requisição HTTP foi substituída por uma que os orquestra
//! para servir uma ação da interface.

pub mod deliverables;
pub mod demo;
pub mod detection;
pub mod findings;
pub mod gaps;
pub mod nist;
pub mod plan;
pub mod provenance;
#[cfg(test)]
pub(crate) mod test_support;
pub mod timeline_filter;

use crate::{
    application::engine::{self, EngineRunnerConfig},
    error::{AppError, AppResult},
    infrastructure::storage::StoragePaths,
    persistence::{analytical_results, audit, rollback_loudly, validation_error},
};
use chrono::Utc;
use dfir_protocol::NormalizedEvent;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{FromRow, SqlitePool};
use std::path::Path;
use uuid::Uuid;

#[derive(Clone)]
pub struct AppServices {
    pub db: SqlitePool,
    pub paths: StoragePaths,
    pub engine: EngineRunnerConfig,
}

async fn begin_immediate(
    pool: &SqlitePool,
) -> Result<sqlx::pool::PoolConnection<sqlx::Sqlite>, sqlx::Error> {
    let mut connection = pool.acquire().await?;
    sqlx::query("BEGIN IMMEDIATE")
        .execute(&mut *connection)
        .await?;
    Ok(connection)
}

async fn rollback(connection: &mut sqlx::pool::PoolConnection<sqlx::Sqlite>) {
    rollback_loudly(connection).await;
}

/// Descarta um artefato temporário nosso (arquivo de evidência rejeitado ou
/// diretório de job órfão) e **registra** a falha da remoção (F1-23): se o
/// arquivo ficar no disco, a falha precisa ser visível no log — o erro
/// original já é devolvido; aqui só garantimos que o descarte não passa em
/// silêncio. O caminho nunca entra no log (ADR-0022: logs não carregam nome
/// de arquivo de evidência).
async fn discard_temp_file(path: &std::path::Path) {
    if let Err(error) = tokio::fs::remove_file(path).await
        && error.kind() != std::io::ErrorKind::NotFound
    {
        tracing::warn!(%error, "failed to discard temporary artifact");
    }
}

/// Mesma regra para diretório temporário de job (F1-23).
async fn discard_temp_dir(path: &std::path::Path) {
    if let Err(error) = tokio::fs::remove_dir_all(path).await
        && error.kind() != std::io::ErrorKind::NotFound
    {
        tracing::warn!(%error, "failed to discard temporary job directory");
    }
}

// ---------------------------------------------------------------------
// Evidência
// ---------------------------------------------------------------------

/// Colunas lidas em `EvidenceRow`. Toda consulta que preenche `EvidenceRow`
/// usa esta lista: uma consulta com colunas a menos falha em tempo de
/// execução (`ColumnNotFound`), não na compilação — foi o que quebrou
/// `analyze` na Etapa 6 e o que o e2e headless (Etapa 7) pegou. É macro, e
/// não `const`, para entrar em `concat!` e manter a consulta `&'static str`.
macro_rules! evidence_columns {
    () => {
        "id,original_filename,storage_reference,format,size_bytes,blake3,sha256,imported_at,status,source,provided_by,collection_method,collection_tool,collection_tool_version,collected_at,authorization_reference,contains_personal_data,incident_id"
    };
}

#[derive(Debug, Serialize, FromRow)]
struct EvidenceRow {
    id: String,
    original_filename: String,
    storage_reference: String,
    format: String,
    size_bytes: i64,
    blake3: String,
    sha256: String,
    imported_at: String,
    status: String,
    source: String,
    provided_by: Option<String>,
    collection_method: String,
    collection_tool: Option<String>,
    collection_tool_version: Option<String>,
    collected_at: Option<String>,
    authorization_reference: Option<String>,
    contains_personal_data: i64,
    incident_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EvidenceResponse {
    pub id: String,
    pub original_filename: String,
    pub storage_reference: String,
    pub format: String,
    pub size_bytes: u64,
    pub blake3: String,
    pub sha256: String,
    pub imported_at: String,
    pub status: String,
    /// Origem declarada (sistema ou área). `NOT_PROVIDED` quando o operador
    /// não informou; linhas antigas guardam constantes de importação (ver
    /// `declared_source`).
    pub source: String,
    /// Quem ou qual área forneceu a evidência (RB-002).
    pub provided_by: Option<String>,
    pub collection_method: String,
    pub collection_tool: Option<String>,
    pub collection_tool_version: Option<String>,
    /// Data ou instante de coleta declarado; `None` quando não informado.
    pub collected_at: Option<String>,
    pub authorization_reference: Option<String>,
    /// O operador marcou que a evidência contém dado pessoal (F1-32).
    pub contains_personal_data: bool,
    /// Incidente ao qual a evidência está vinculada (F1-19); `None` quando
    /// ainda não foi vinculada a nenhum.
    pub incident_id: Option<String>,
}

impl From<EvidenceRow> for EvidenceResponse {
    fn from(row: EvidenceRow) -> Self {
        Self {
            id: row.id,
            original_filename: row.original_filename,
            storage_reference: row.storage_reference,
            format: row.format,
            size_bytes: row.size_bytes.max(0) as u64,
            blake3: row.blake3,
            sha256: row.sha256,
            imported_at: row.imported_at,
            status: row.status,
            source: row.source,
            provided_by: row.provided_by,
            collection_method: row.collection_method,
            collection_tool: row.collection_tool,
            collection_tool_version: row.collection_tool_version,
            collected_at: row.collected_at,
            authorization_reference: row.authorization_reference,
            contains_personal_data: row.contains_personal_data != 0,
            incident_id: row.incident_id,
        }
    }
}

pub async fn list_evidence(services: &AppServices) -> AppResult<Vec<EvidenceResponse>> {
    let rows = sqlx::query_as::<_, EvidenceRow>(concat!(
        "SELECT ",
        evidence_columns!(),
        " FROM evidence WHERE preservation_status != 'DESTROYED' ORDER BY imported_at DESC"
    ))
    .fetch_all(&services.db)
    .await?;
    Ok(rows.into_iter().map(EvidenceResponse::from).collect())
}

/// Todas as evidências, **inclusive as descartadas localmente**, cada uma com
/// a marca de descarte. Os documentos citam a evidência que sustentou um
/// resultado mesmo depois do descarte: o arquivo some, o hash e a custódia
/// ficam. `list_evidence` continua sendo o catálogo de trabalho (sem
/// descartadas).
pub(crate) async fn list_evidence_including_destroyed(
    services: &AppServices,
) -> AppResult<Vec<(EvidenceResponse, bool)>> {
    let rows = sqlx::query_as::<_, EvidenceRow>(concat!(
        "SELECT ",
        evidence_columns!(),
        " FROM evidence ORDER BY imported_at DESC"
    ))
    .fetch_all(&services.db)
    .await?;
    let destroyed: std::collections::HashSet<String> =
        sqlx::query_scalar("SELECT id FROM evidence WHERE preservation_status = 'DESTROYED'")
            .fetch_all(&services.db)
            .await?
            .into_iter()
            .collect();
    Ok(rows
        .into_iter()
        .map(|row| {
            let gone = destroyed.contains(&row.id);
            (EvidenceResponse::from(row), gone)
        })
        .collect())
}

pub async fn destroy_evidence(services: &AppServices, evidence_id: &str) -> AppResult<()> {
    let result = destroy_evidence_core(&services.db, &services.paths, evidence_id).await;
    match &result {
        Ok(()) => tracing::info!(evidence_id, "evidence destroyed"),
        Err(error) => tracing::warn!(evidence_id, %error, "evidence destruction failed"),
    }
    result
}

/// Núcleo do descarte local, separado de `AppServices` para ser testável sem
/// engine (F1-21, AUD-02).
///
/// Ordem deliberada: **primeiro** a transação que registra a destruição
/// (status, auditoria e custódia), **depois** a remoção física, e só então a
/// remoção das linhas de `analysis_jobs`. Assim, uma falha de banco nunca
/// deixa o arquivo apagado com a evidência ainda `ACTIVE` e sem registro, e
/// uma falha de I/O depois do commit deixa a evidência registrada como
/// destruída com restos no disco, o que é visível (erro devolvido) e
/// reexecutável: chamar a função de novo para uma evidência já `DESTROYED`
/// conclui a limpeza dos restos em vez de falhar.
async fn destroy_evidence_core(
    db: &SqlitePool,
    paths: &StoragePaths,
    evidence_id: &str,
) -> AppResult<()> {
    let evidence = sqlx::query_as::<_, (String, String, String)>(
        "SELECT original_filename,storage_reference,preservation_status FROM evidence WHERE id=?",
    )
    .bind(evidence_id)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| AppError::NotFound("evidence not found".into()))?;
    let already_destroyed = evidence.2 == "DESTROYED";

    let active_jobs: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM analysis_jobs WHERE evidence_id=? AND status IN ('PENDING','PROCESSING')",
    )
    .bind(evidence_id)
    .fetch_one(db)
    .await?;
    if active_jobs > 0 {
        return Err(AppError::Config(
            "evidence cannot be destroyed while an analysis job is pending or processing".into(),
        ));
    }

    let storage_path = paths.logical_path(&evidence.1)?;
    let job_ids: Vec<String> =
        sqlx::query_scalar("SELECT id FROM analysis_jobs WHERE evidence_id=?")
            .bind(evidence_id)
            .fetch_all(db)
            .await?;

    if already_destroyed {
        let leftover_file = tokio::fs::try_exists(&storage_path)
            .await
            .map_err(AppError::Storage)?;
        if !leftover_file && job_ids.is_empty() {
            return Err(AppError::NotFound("evidence already destroyed".into()));
        }
    } else {
        let metadata = serde_json::json!({
            "evidence_id": evidence_id,
            "original_filename": evidence.0,
            "storage_reference": evidence.1,
            "destroyed_analysis_jobs": job_ids,
            "reason": "operator_requested_local_discard",
        });
        let mut connection = begin_immediate(db).await.map_err(AppError::Database)?;
        let transaction_result = async {
            sqlx::query("UPDATE evidence SET preservation_status='DESTROYED' WHERE id=?")
                .bind(evidence_id)
                .execute(&mut *connection)
                .await?;
            audit::append_on_connection(
                &mut connection,
                None,
                "EVIDENCE_DESTROYED",
                "evidence",
                evidence_id,
                None,
                &metadata.to_string(),
            )
            .await?;
            audit::append_custody_on_connection(
                &mut connection,
                evidence_id,
                "DESTROYED",
                "local-analyst",
                "Imported evidence and analysis artifacts destruction recorded by explicit local operator request",
                &metadata.to_string(),
            )
            .await
        }
        .await;
        if let Err(error) = transaction_result {
            rollback(&mut connection).await;
            return Err(AppError::Database(error));
        }
        if let Err(error) = sqlx::query("COMMIT").execute(&mut *connection).await {
            rollback(&mut connection).await;
            return Err(AppError::Database(error));
        }
    }

    // A destruição já está registrada; agora remove os restos físicos.
    if tokio::fs::try_exists(&storage_path)
        .await
        .map_err(AppError::Storage)?
    {
        tokio::fs::remove_file(&storage_path)
            .await
            .map_err(AppError::Storage)?;
    }
    for job_id in &job_ids {
        let job_dir = paths.staging.join("analysis").join(job_id);
        if tokio::fs::try_exists(&job_dir)
            .await
            .map_err(AppError::Storage)?
        {
            tokio::fs::remove_dir_all(&job_dir)
                .await
                .map_err(AppError::Storage)?;
        }
    }
    // O índice da timeline contém dado da evidência (títulos, provedores,
    // hosts): sai junto com a evidência (RB-006).
    sqlx::query("DELETE FROM timeline_events WHERE job_id IN (SELECT id FROM analysis_jobs WHERE evidence_id=?)")
        .bind(evidence_id)
        .execute(db)
        .await?;
    sqlx::query("DELETE FROM timeline_index WHERE job_id IN (SELECT id FROM analysis_jobs WHERE evidence_id=?)")
        .bind(evidence_id)
        .execute(db)
        .await?;
    // Candidatos de detecção carregam contas, IPs e referências de evento.
    crate::persistence::detection::delete_for_evidence(db, evidence_id).await?;
    sqlx::query("DELETE FROM analysis_jobs WHERE evidence_id=?")
        .bind(evidence_id)
        .execute(db)
        .await?;
    Ok(())
}

/// Importa uma evidência a partir de um caminho local já escolhido pelo
/// operador (diálogo nativo de arquivo, `src/gui`). Diferente do antigo
/// `POST /api/evidence` (multipart via browser), a interface nativa lê o
/// arquivo diretamente do disco do próprio operador — não há mais fronteira
/// de rede nem upload em partes; o hashing em streaming e a transação de
/// importação (evidência + auditoria + custódia) continuam idênticos.
/// Importa uma evidência e registra o resultado no log (id, tamanho, formato e
/// duração; **nunca o nome do arquivo**, ADR-0022).
///
/// Grava a proveniência que o operador declarou (F1-31). O que ele não
/// informa fica como não informado: nada é preenchido por suposição.
pub async fn import_evidence_with_provenance(
    services: &AppServices,
    source_path: &Path,
    max_upload_bytes: u64,
    declared: &provenance::EvidenceProvenance,
) -> AppResult<EvidenceResponse> {
    import_evidence_classified(
        services,
        source_path,
        max_upload_bytes,
        declared,
        DATA_CLASS_FORENSIC,
    )
    .await
}

/// Classe do dado de uma evidência (seção 21.3): o que a auditoria usa para
/// nunca confundir um dataset de demonstração com evidência forense.
pub const DATA_CLASS_FORENSIC: &str = "FORENSIC";
pub const DATA_CLASS_DEMO: &str = "DEMO";

/// Como `import_evidence_with_provenance`, com a classe do dado explícita.
/// Só `FORENSIC`, `SANITIZED` e `DEMO` existem; a interface importa sempre
/// como `FORENSIC`, e só o incidente de demonstração usa `DEMO`.
pub async fn import_evidence_classified(
    services: &AppServices,
    source_path: &Path,
    max_upload_bytes: u64,
    declared: &provenance::EvidenceProvenance,
    data_class: &str,
) -> AppResult<EvidenceResponse> {
    if !matches!(data_class, "FORENSIC" | "SANITIZED" | "DEMO") {
        return Err(AppError::Config(format!(
            "unknown evidence data class: {data_class}"
        )));
    }
    // Valida antes de copiar o arquivo: dado inválido não deixa rastro.
    let clean = declared.clean()?;
    let started = std::time::Instant::now();
    let result =
        import_evidence_inner(services, source_path, max_upload_bytes, &clean, data_class).await;
    let elapsed_ms = started.elapsed().as_millis() as u64;
    match &result {
        Ok(evidence) => tracing::info!(
            evidence_id = %evidence.id,
            size_bytes = evidence.size_bytes,
            format = %evidence.format,
            elapsed_ms,
            "evidence imported"
        ),
        Err(error) => tracing::warn!(%error, elapsed_ms, "evidence import failed"),
    }
    result
}

async fn import_evidence_inner(
    services: &AppServices,
    source_path: &Path,
    max_upload_bytes: u64,
    clean: &provenance::CleanProvenance,
    data_class: &str,
) -> AppResult<EvidenceResponse> {
    let original_filename = source_path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .filter(|name| !name.is_empty())
        .ok_or_else(|| AppError::Config("evidence path must name a file".into()))?;
    if original_filename.len() > 255 || original_filename.contains('\0') {
        return Err(AppError::Config(
            "evidence filename is invalid or too long".into(),
        ));
    }
    let source_metadata = tokio::fs::metadata(source_path)
        .await
        .map_err(AppError::Storage)?;
    if !source_metadata.is_file() {
        return Err(AppError::Config(
            "selected evidence path is not a regular file".into(),
        ));
    }
    // A cópia preservada ocupa o tamanho do arquivo; mais a margem de segurança.
    if let Some(message) = crate::infrastructure::disk::shortage_message(
        &services.paths.evidence,
        source_metadata
            .len()
            .saturating_add(services.engine.min_free_disk_bytes),
    ) {
        return Err(AppError::Config(message));
    }
    if source_metadata.len() > max_upload_bytes {
        return Err(AppError::Config(format!(
            "evidence exceeds the configured upload limit of {max_upload_bytes} bytes"
        )));
    }

    let format = detect_format(&original_filename);
    let evidence_id = format!("ev_{}", Uuid::new_v4());
    let extension = safe_extension(&original_filename);
    let storage_reference = if extension.is_empty() {
        format!("evidence/{evidence_id}")
    } else {
        format!("evidence/{evidence_id}.{extension}")
    };
    let final_path = services.paths.logical_path(&storage_reference)?;
    let temp_reference = format!("quarantine/{evidence_id}.uploading");
    let temp_path = services.paths.logical_path(&temp_reference)?;

    let (size, blake3, sha256) = copy_with_hash(source_path, &temp_path, max_upload_bytes).await?;
    tokio::fs::rename(&temp_path, &final_path)
        .await
        .map_err(AppError::Storage)?;

    let imported_at = Utc::now().to_rfc3339();
    let metadata = serde_json::json!({
        "evidence_id": evidence_id,
        "storage_reference": storage_reference,
        "original_filename": original_filename,
        "format": format,
        "size_bytes": size,
        "blake3": blake3,
        "sha256": sha256,
        "status": data_class,
        "mime_type": "application/octet-stream",
        "import_method": "LOCAL_FILE_PICKER",
        "source": clean.source_or_default(),
        "provided_by": clean.provided_by,
        "collection_method": clean.collection_method_or_default(),
        "collection_tool": clean.collection_tool,
        "collection_tool_version": clean.collection_tool_version,
        "collected_at": clean.collected_at,
        "authorization_reference": clean.authorization_reference,
        "contains_personal_data": clean.contains_personal_data,
    });

    let mut connection = match begin_immediate(&services.db).await {
        Ok(connection) => connection,
        Err(error) => {
            discard_temp_file(&final_path).await;
            return Err(AppError::Database(error));
        }
    };
    let transaction_result = async {
        sqlx::query(
            "INSERT INTO evidence(id,original_filename,storage_reference,format,size_bytes,blake3,sha256,imported_at,status,mime_type,source,collection_method,collected_at,ingested_by,preservation_status,provided_by,collection_tool,collection_tool_version,authorization_reference,contains_personal_data) VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
        )
        .bind(&evidence_id)
        .bind(&original_filename)
        .bind(&storage_reference)
        .bind(&format)
        .bind(size as i64)
        .bind(&blake3)
        .bind(&sha256)
        .bind(&imported_at)
        .bind(data_class)
        .bind("application/octet-stream")
        .bind(clean.source_or_default())
        .bind(clean.collection_method_or_default())
        .bind(&clean.collected_at)
        .bind("local-analyst")
        .bind("ACTIVE")
        .bind(&clean.provided_by)
        .bind(&clean.collection_tool)
        .bind(&clean.collection_tool_version)
        .bind(&clean.authorization_reference)
        .bind(i64::from(clean.contains_personal_data))
        .execute(&mut *connection)
        .await?;

        audit::append_import_records_on_connection(&mut connection, &evidence_id, &metadata.to_string())
            .await
    }
    .await;

    match transaction_result {
        Ok(()) => {
            if let Err(error) = sqlx::query("COMMIT").execute(&mut *connection).await {
                rollback(&mut connection).await;
                discard_temp_file(&final_path).await;
                return Err(AppError::Database(error));
            }
        }
        Err(error) => {
            rollback(&mut connection).await;
            discard_temp_file(&final_path).await;
            return Err(AppError::Database(error));
        }
    }

    Ok(EvidenceResponse {
        id: evidence_id,
        original_filename,
        storage_reference,
        format,
        size_bytes: size,
        blake3,
        sha256,
        imported_at,
        status: data_class.to_string(),
        source: clean.source_or_default().to_string(),
        provided_by: clean.provided_by.clone(),
        collection_method: clean.collection_method_or_default().to_string(),
        collection_tool: clean.collection_tool.clone(),
        collection_tool_version: clean.collection_tool_version.clone(),
        collected_at: clean.collected_at.clone(),
        authorization_reference: clean.authorization_reference.clone(),
        contains_personal_data: clean.contains_personal_data,
        incident_id: None,
    })
}

/// Copia `source` para `dest` computando BLAKE3+SHA-256 em streaming,
/// rejeitando se o total exceder `max_bytes` no meio da cópia (mesmo
/// comportamento do antigo streaming de multipart, adaptado a leitura local).
async fn copy_with_hash(
    source: &Path,
    dest: &Path,
    max_bytes: u64,
) -> AppResult<(u64, String, String)> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let mut input = tokio::fs::File::open(source)
        .await
        .map_err(AppError::Storage)?;
    let mut output = tokio::fs::File::create(dest)
        .await
        .map_err(AppError::Storage)?;
    // Heap, não pilha: um `[0_u8; 1 MiB]` dentro de um `async fn` vira parte do
    // future e estourava a pilha da thread principal em build debug
    // (achado pelo teste headless, F1-34).
    let mut buffer = vec![0_u8; 1024 * 1024];
    let mut size = 0_u64;
    let mut blake3_hasher = blake3::Hasher::new();
    let mut sha256_hasher = Sha256::new();
    loop {
        let read = input.read(&mut buffer).await.map_err(AppError::Storage)?;
        if read == 0 {
            break;
        }
        size = size.saturating_add(read as u64);
        if size > max_bytes {
            drop(output);
            discard_temp_file(dest).await;
            return Err(AppError::Config(format!(
                "evidence exceeds the configured upload limit of {max_bytes} bytes"
            )));
        }
        blake3_hasher.update(&buffer[..read]);
        sha256_hasher.update(&buffer[..read]);
        output
            .write_all(&buffer[..read])
            .await
            .map_err(AppError::Storage)?;
    }
    output.flush().await.map_err(AppError::Storage)?;
    output.sync_all().await.map_err(AppError::Storage)?;
    Ok((
        size,
        blake3_hasher.finalize().to_hex().to_string(),
        format!("{:x}", sha256_hasher.finalize()),
    ))
}

// ---------------------------------------------------------------------
// Análise
// ---------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct AnalyzeResponse {
    pub request_id: String,
    pub job_id: String,
    pub evidence_id: String,
    pub status: String,
    /// `EVTX_TIMELINE` ou `YARA_SCAN`; a GUI usa para só recarregar a timeline
    /// quando o job realmente produziu eventos (AUD-32).
    pub analysis_kind: String,
    pub output_reference: Option<String>,
    pub message_count: usize,
    pub item_count: u64,
    pub warnings: Vec<JobWarning>,
}

#[derive(Debug, FromRow)]
struct JobRow {
    id: String,
    request_id: String,
    evidence_id: String,
    status: String,
    kind: String,
    message_count: i64,
    item_count: i64,
    output_reference: Option<String>,
    error_code: Option<String>,
    error_message: Option<String>,
    created_at: String,
    started_at: Option<String>,
    finished_at: Option<String>,
    warning_count: i64,
    warnings: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct JobStatusResponse {
    pub job_id: String,
    pub request_id: String,
    pub evidence_id: String,
    pub status: String,
    pub kind: String,
    pub message_count: u64,
    pub item_count: u64,
    pub output_reference: Option<String>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub created_at: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    /// Avisos NÃO fatais (arquivo EVTX truncado, erros de registro...). Um job
    /// `COMPLETED` com avisos não é uma conclusão limpa (AUD-04/AUD-36).
    pub warning_count: u64,
    pub warnings: Vec<JobWarning>,
}

impl TryFrom<JobRow> for JobStatusResponse {
    type Error = AppError;

    fn try_from(row: JobRow) -> Result<Self, Self::Error> {
        let warnings: Vec<JobWarning> = match row.warnings.as_deref() {
            Some(json) => serde_json::from_str(json).map_err(|error| {
                AppError::Config(format!(
                    "stored analysis job warnings are malformed JSON: {error}"
                ))
            })?,
            None => Vec::new(),
        };
        let warning_count = u64::try_from(row.warning_count).map_err(|_| {
            AppError::Config("stored analysis job warning_count is negative".into())
        })?;
        if warning_count != warnings.len() as u64 {
            return Err(AppError::Config(format!(
                "stored analysis job warning_count does not match decoded warnings: {warning_count} != {}",
                warnings.len()
            )));
        }
        Ok(Self {
            job_id: row.id,
            request_id: row.request_id,
            evidence_id: row.evidence_id,
            status: row.status,
            kind: row.kind,
            message_count: row.message_count.max(0) as u64,
            item_count: row.item_count.max(0) as u64,
            output_reference: row.output_reference,
            error_code: row.error_code,
            error_message: row.error_message,
            created_at: row.created_at,
            started_at: row.started_at,
            finished_at: row.finished_at,
            warning_count,
            warnings,
        })
    }
}

/// Executa uma análise e registra o resultado no log (job, tipo, itens, avisos
/// e duração total, ou o erro).
pub async fn analyze(
    services: &AppServices,
    evidence_id: &str,
    kind: &str,
    max_output_bytes: Option<u64>,
    max_items: Option<u64>,
) -> AppResult<AnalyzeResponse> {
    let started = std::time::Instant::now();
    let result = analyze_inner(services, evidence_id, kind, max_output_bytes, max_items).await;
    let elapsed_ms = started.elapsed().as_millis() as u64;
    match &result {
        Ok(response) => tracing::info!(
            evidence_id,
            kind,
            job_id = %response.job_id,
            items = response.item_count,
            warnings = response.warnings.len(),
            elapsed_ms,
            "analysis completed"
        ),
        Err(error) => tracing::warn!(evidence_id, kind, %error, elapsed_ms, "analysis failed"),
    }
    result
}

async fn analyze_inner(
    services: &AppServices,
    evidence_id: &str,
    kind: &str,
    max_output_bytes: Option<u64>,
    max_items: Option<u64>,
) -> AppResult<AnalyzeResponse> {
    let evidence = sqlx::query_as::<_, EvidenceRow>(concat!(
        "SELECT ",
        evidence_columns!(),
        " FROM evidence WHERE id=? AND preservation_status != 'DESTROYED'"
    ))
    .bind(evidence_id)
    .fetch_optional(&services.db)
    .await?
    .ok_or_else(|| AppError::NotFound("evidence not found".into()))?;

    // Não começa uma análise com o disco quase cheio (ADR-0021): sem teto de
    // saída, a margem de espaço livre é o freio. Falha antes de criar o job.
    if let Some(message) = crate::infrastructure::disk::shortage_message(
        &services.paths.staging,
        services.engine.min_free_disk_bytes,
    ) {
        return Err(AppError::Config(message));
    }

    match kind {
        dfir_protocol::ANALYSIS_KIND_EVTX_TIMELINE => {
            if evidence.format != "evtx" {
                return Err(AppError::Config(
                    "only EVTX evidence is analyzable with analysis_kind=EVTX_TIMELINE".into(),
                ));
            }
        }
        dfir_protocol::ANALYSIS_KIND_EVTX_DETECTION => {
            if evidence.format != "evtx" {
                return Err(AppError::Config(
                    "only EVTX evidence is analyzable with analysis_kind=EVTX_DETECTION".into(),
                ));
            }
        }
        dfir_protocol::ANALYSIS_KIND_YARA_SCAN => {}
        other => {
            return Err(AppError::Config(format!(
                "unsupported analysis_kind: {other}"
            )));
        }
    }

    let max_output_bytes = max_output_bytes.unwrap_or(dfir_protocol::MAX_OUTPUT_BYTES);
    let max_items = max_items.unwrap_or(dfir_protocol::MAX_ITEMS);
    if max_output_bytes == 0 || max_output_bytes > dfir_protocol::MAX_OUTPUT_BYTES {
        return Err(AppError::Config(format!(
            "max_output_bytes must be between 1 and {}",
            dfir_protocol::MAX_OUTPUT_BYTES
        )));
    }
    if max_items == 0 || max_items > dfir_protocol::MAX_ITEMS {
        return Err(AppError::Config(format!(
            "max_items must be between 1 and {}",
            dfir_protocol::MAX_ITEMS
        )));
    }

    let input = services
        .paths
        .logical_path(&evidence.storage_reference)
        .map_err(AppError::Storage)?;
    let metadata = tokio::fs::metadata(&input)
        .await
        .map_err(AppError::Storage)?;
    if !metadata.is_file() {
        return Err(AppError::Config(
            "evidence storage reference is not a regular file".into(),
        ));
    }
    if metadata.len() != evidence.size_bytes.max(0) as u64 {
        let metadata_json = serde_json::json!({
            "evidence_id": &evidence.id,
            "expected_size_bytes": evidence.size_bytes,
            "actual_size_bytes": metadata.len(),
        });
        return Err(integrity_failure_error(
            &services.db,
            &evidence.id,
            "evidence size changed after import",
            &metadata_json.to_string(),
            "evidence changed after import; analysis refused",
        )
        .await);
    }
    let (actual_blake3, actual_sha256) = hash_file(&input).await?;
    if actual_blake3 != evidence.blake3 || actual_sha256 != evidence.sha256 {
        let metadata_json = serde_json::json!({
            "evidence_id": &evidence.id,
            "expected_blake3": &evidence.blake3,
            "actual_blake3": actual_blake3,
            "expected_sha256": &evidence.sha256,
            "actual_sha256": actual_sha256,
        });
        return Err(integrity_failure_error(
            &services.db,
            &evidence.id,
            "evidence digest changed after import",
            &metadata_json.to_string(),
            "evidence digest mismatch; analysis refused",
        )
        .await);
    }

    let request = engine::new_request(
        evidence.storage_reference.clone(),
        max_output_bytes,
        max_items,
        kind,
    );
    let job_id = request.job_id.clone();
    let now = Utc::now().to_rfc3339();
    let job_dir = services.paths.staging.join("analysis").join(&job_id);
    let output_dir = job_dir.join("output");
    tokio::fs::create_dir_all(&output_dir)
        .await
        .map_err(AppError::Storage)?;

    let mut connection = match begin_immediate(&services.db).await {
        Ok(connection) => connection,
        Err(error) => {
            discard_temp_dir(&job_dir).await;
            return Err(AppError::Database(error));
        }
    };
    let started_at = Utc::now().to_rfc3339();
    let action_label = match kind {
        dfir_protocol::ANALYSIS_KIND_YARA_SCAN => "YARA scan",
        dfir_protocol::ANALYSIS_KIND_EVTX_DETECTION => "EVTX detection",
        _ => "EVTX analysis",
    };
    let transaction_result = async {
        sqlx::query(
            "INSERT INTO analysis_jobs(id,request_id,evidence_id,status,max_output_bytes,max_items,kind,created_at,started_at) VALUES(?,?,?,?,?,?,?,?,?)",
        )
        .bind(&job_id)
        .bind(&request.request_id)
        .bind(&evidence.id)
        .bind("PROCESSING")
        .bind(max_output_bytes as i64)
        .bind(max_items as i64)
        .bind(kind)
        .bind(&now)
        .bind(&started_at)
        .execute(&mut *connection)
        .await?;

        let started_metadata = serde_json::json!({
            "request_id": request.request_id,
            "job_id": job_id,
            "evidence_id": evidence.id,
            "input_reference": evidence.storage_reference,
            "analysis_kind": kind,
            "max_output_bytes": max_output_bytes,
            "max_items": max_items,
        });
        audit::append_on_connection(
            &mut connection,
            None,
            "ANALYSIS_STARTED",
            "analysis_job",
            &job_id,
            None,
            &started_metadata.to_string(),
        )
        .await?;
        audit::append_custody_on_connection(
            &mut connection,
            &evidence.id,
            "ANALYSIS_STARTED",
            "local-analyst",
            &format!("{action_label} started"),
            &started_metadata.to_string(),
        )
        .await
    }
    .await;
    if let Err(error) = transaction_result {
        rollback(&mut connection).await;
        discard_temp_dir(&job_dir).await;
        return Err(AppError::Database(error));
    }
    if let Err(error) = sqlx::query("COMMIT").execute(&mut *connection).await {
        rollback(&mut connection).await;
        discard_temp_dir(&job_dir).await;
        return Err(AppError::Database(error));
    }

    let result = engine::run_engine(
        request.clone(),
        &services.paths.evidence,
        &input,
        &output_dir,
        &services.engine,
    )
    .await;
    match result {
        Ok(summary) => {
            let output_reference = format!("staging/analysis/{job_id}/output/results.jsonl");
            let finished_at = Utc::now().to_rfc3339();
            let mut completed_metadata = serde_json::json!({
                "request_id": &request.request_id,
                "job_id": &job_id,
                "evidence_id": &evidence.id,
                "analysis_kind": kind,
                "output_reference": &output_reference,
                "message_count": summary.message_count,
                "item_count": summary.item_count,
                "warning_count": summary.diagnostics.len(),
                "sandbox": &summary.sandbox,
            });
            // Itens que viram linhas no banco ao concluir: casamentos do YARA ou
            // candidatos de detecção (poucos, ao contrário dos eventos da timeline).
            let yara_items: Vec<serde_json::Value> = if kind
                == dfir_protocol::ANALYSIS_KIND_YARA_SCAN
                || kind == dfir_protocol::ANALYSIS_KIND_EVTX_DETECTION
            {
                match read_result_items(&output_dir.join("results.jsonl")).await {
                    Ok(items) => items,
                    Err(error) => {
                        mark_job_failed_loudly(
                            &services.db,
                            &job_id,
                            "PERSISTENCE_ERROR",
                            &error.to_string(),
                        )
                        .await;
                        return Err(error);
                    }
                }
            } else {
                Vec::new()
            };
            if kind == dfir_protocol::ANALYSIS_KIND_EVTX_DETECTION {
                let candidates = yara_items
                    .iter()
                    .filter(|item| dfir_protocol::DetectionCandidate::from_item(item).is_some())
                    .count();
                completed_metadata["candidates"] = serde_json::json!(candidates);
            }
            // O índice usa a própria transação de escrita: precisa vir ANTES de
            // abrir a transação de conclusão (SQLite tem um único escritor).
            let mut warnings: Vec<JobWarning> = summary
                .diagnostics
                .iter()
                .map(|diagnostic| JobWarning {
                    code: diagnostic.code.clone(),
                    count: diagnostic.count,
                    sample: diagnostic.sample.clone(),
                })
                .collect();
            // F1-22: a contenção do engine é registrada em TODO job. Se alguma
            // camada não foi aplicada (política `best-effort`/`off`, ou sistema
            // sem suporte), isso aparece como aviso do job, nunca em silêncio.
            match summary.sandbox.as_deref() {
                Some(state) if engine::sandbox_fully_applied(state) => {}
                Some(state) => warnings.push(JobWarning {
                    code: "SANDBOX_NOT_FULLY_APPLIED".into(),
                    count: 1,
                    sample: state.chars().take(300).collect(),
                }),
                None => warnings.push(JobWarning {
                    code: "SANDBOX_STATE_UNKNOWN".into(),
                    count: 1,
                    sample: "the engine did not report its sandbox state".into(),
                }),
            }
            if kind == dfir_protocol::ANALYSIS_KIND_EVTX_TIMELINE {
                match index_timeline(&services.db, &job_id, &output_dir.join("results.jsonl")).await
                {
                    Ok((_, skipped)) if skipped > 0 => warnings.push(JobWarning {
                        code: "TIMELINE_ITEMS_SKIPPED".into(),
                        count: skipped,
                        sample: "items without a valid timestamp are not shown in the timeline"
                            .into(),
                    }),
                    Ok(_) => {}
                    Err(error) => {
                        mark_job_failed_loudly(
                            &services.db,
                            &job_id,
                            "PERSISTENCE_ERROR",
                            &error.to_string(),
                        )
                        .await;
                        return Err(error);
                    }
                }
            }
            let mut connection = match begin_immediate(&services.db).await {
                Ok(connection) => connection,
                Err(error) => {
                    // AUD-03: sem isto o job ficaria `PROCESSING` para sempre.
                    mark_job_failed_loudly(
                        &services.db,
                        &job_id,
                        "PERSISTENCE_ERROR",
                        &error.to_string(),
                    )
                    .await;
                    return Err(AppError::Database(error));
                }
            };
            let transaction_result: Result<(), sqlx::Error> = async {
                let warnings_json = serde_json::to_string(&warnings).map_err(|error| {
                    validation_error(format!(
                        "could not encode analysis job warnings as JSON: {error}"
                    ))
                })?;
                sqlx::query("UPDATE analysis_jobs SET status='COMPLETED',message_count=?,item_count=?,output_reference=?,finished_at=?,warning_count=?,warnings=? WHERE id=?")
                    .bind(summary.message_count as i64)
                    .bind(summary.item_count as i64)
                    .bind(&output_reference)
                    .bind(&finished_at)
                    .bind(warnings.len() as i64)
                    .bind(&warnings_json)
                    .bind(&job_id)
                    .execute(&mut *connection)
                    .await?;
                audit::append_on_connection(
                    &mut connection,
                    None,
                    "ANALYSIS_COMPLETED",
                    "analysis_job",
                    &job_id,
                    None,
                    &completed_metadata.to_string(),
                )
                .await?;
                audit::append_custody_on_connection(
                    &mut connection,
                    &evidence.id,
                    "ANALYSIS_COMPLETED",
                    "local-analyst",
                    &format!("{action_label} completed"),
                    &completed_metadata.to_string(),
                )
                .await?;

                for item in yara_items.iter() {
                    if item.get("kind").and_then(serde_json::Value::as_str) != Some("YARA_MATCH") {
                        continue;
                    }
                    let rule_id = item.get("rule_id").and_then(serde_json::Value::as_str);
                    let ruleset_version = item
                        .get("ruleset_version")
                        .and_then(serde_json::Value::as_str);
                    let confidence = item
                        .get("confidence")
                        .and_then(serde_json::Value::as_str)
                        .and_then(analytical_results::Confidence::from_str);
                    let correlation_key = format!("yara:{}:{}", rule_id.unwrap_or("unknown"), evidence.id);
                    // Reexecutar o scan não pode duplicar o resultado vigente
                    // (AUD-07): o novo resultado substitui o vigente da mesma
                    // chave, e o histórico continua no ledger append-only.
                    let previous_current: Option<String> = sqlx::query_scalar(
                        "SELECT ar.id FROM analytical_results ar \
                         WHERE ar.correlation_key = ? \
                         AND NOT EXISTS (SELECT 1 FROM analytical_results newer WHERE newer.supersedes = ar.id) \
                         ORDER BY ar.sequence DESC LIMIT 1",
                    )
                    .bind(&correlation_key)
                    .fetch_optional(&mut *connection)
                    .await?;
                    let payload_json = item.to_string();
                    let provenance_json = serde_json::json!({
                        "producer": "dfir-engine",
                        "ruleset_id": item.get("ruleset_id"),
                        "ruleset_version": item.get("ruleset_version"),
                        "ruleset_sha256": item.get("ruleset_sha256"),
                        "ruleset_license": item.get("ruleset_license"),
                        "ruleset_source_origin": item.get("ruleset_source_origin"),
                        "engine_compatibility": item.get("engine_compatibility"),
                        "analysis_job_id": &job_id,
                    })
                    .to_string();
                    analytical_results::insert_on_connection(
                        &mut connection,
                        analytical_results::NewAnalyticalResult {
                            result_type: analytical_results::AnalyticalResultType::YaraMatch,
                            correlation_key: &correlation_key,
                            evidence_id: Some(evidence.id.as_str()),
                            incident_id: None,
                            analysis_job_id: Some(job_id.as_str()),
                            rule_or_model_id: rule_id,
                            rule_or_model_version: ruleset_version,
                            confidence,
                            payload_json: &payload_json,
                            provenance_json: &provenance_json,
                            supersedes: previous_current.as_deref(),
                            related: &[],
                            actor_id: None,
                        },
                    )
                    .await?;
                }
                // Candidatos de detecção: hipóteses pendentes, nunca achados.
                for item in yara_items.iter() {
                    if let Some(candidate) = dfir_protocol::DetectionCandidate::from_item(item) {
                        crate::persistence::detection::insert_on_connection(
                            &mut connection,
                            &evidence.id,
                            &job_id,
                            &candidate,
                        )
                        .await?;
                    }
                }
                Ok(())
            }
            .await;
            if let Err(error) = transaction_result {
                rollback(&mut connection).await;
                mark_job_failed_loudly(
                    &services.db,
                    &job_id,
                    "PERSISTENCE_ERROR",
                    &error.to_string(),
                )
                .await;
                return Err(AppError::Database(error));
            }
            if let Err(error) = sqlx::query("COMMIT").execute(&mut *connection).await {
                rollback(&mut connection).await;
                mark_job_failed_loudly(
                    &services.db,
                    &job_id,
                    "PERSISTENCE_ERROR",
                    &error.to_string(),
                )
                .await;
                return Err(AppError::Database(error));
            }
            Ok(AnalyzeResponse {
                request_id: request.request_id,
                job_id,
                evidence_id: evidence.id,
                status: "COMPLETED".into(),
                analysis_kind: kind.to_string(),
                output_reference: Some(output_reference),
                message_count: summary.message_count,
                item_count: summary.item_count,
                warnings,
            })
        }
        Err(error) => {
            let code = engine_error_code(&error);
            let message = error.to_string();
            let finished_at = Utc::now().to_rfc3339();
            let failed_metadata = serde_json::json!({
                "request_id": &request.request_id,
                "job_id": &job_id,
                "evidence_id": &evidence.id,
                "analysis_kind": kind,
                "error_code": code,
                "error": &message,
            });
            let mut connection = match begin_immediate(&services.db).await {
                Ok(connection) => connection,
                Err(db_error) => {
                    // AUD-03: registra a falha real do engine no job mesmo sem
                    // conseguir abrir a transação de auditoria.
                    mark_job_failed_loudly(&services.db, &job_id, code, &message).await;
                    return Err(AppError::Database(db_error));
                }
            };
            let transaction_result = async {
                sqlx::query("UPDATE analysis_jobs SET status='FAILED',error_code=?,error_message=?,finished_at=? WHERE id=?")
                    .bind(code)
                    .bind(&message)
                    .bind(&finished_at)
                    .bind(&job_id)
                    .execute(&mut *connection)
                    .await?;
                audit::append_on_connection(
                    &mut connection,
                    None,
                    "ANALYSIS_FAILED",
                    "analysis_job",
                    &job_id,
                    None,
                    &failed_metadata.to_string(),
                )
                .await?;
                audit::append_custody_on_connection(
                    &mut connection,
                    &evidence.id,
                    "ANALYSIS_FAILED",
                    "local-analyst",
                    &format!("{action_label} failed"),
                    &failed_metadata.to_string(),
                )
                .await
            }
            .await;
            if let Err(db_error) = transaction_result {
                rollback(&mut connection).await;
                mark_job_failed_loudly(
                    &services.db,
                    &job_id,
                    "PERSISTENCE_ERROR",
                    &db_error.to_string(),
                )
                .await;
                return Err(AppError::Database(db_error));
            }
            if let Err(db_error) = sqlx::query("COMMIT").execute(&mut *connection).await {
                rollback(&mut connection).await;
                mark_job_failed_loudly(
                    &services.db,
                    &job_id,
                    "PERSISTENCE_ERROR",
                    &db_error.to_string(),
                )
                .await;
                return Err(AppError::Database(db_error));
            }
            Err(AppError::Engine(message))
        }
    }
}

/// Registra uma falha de integridade da evidência na auditoria e na
/// custódia. Devolve `Err` se **qualquer** dos dois registros falhar: uma
/// falha de integridade que não ficou registrada precisa ser visível, não
/// engolida (AUD-25, AGENTS §6).
async fn record_integrity_failure(
    pool: &SqlitePool,
    evidence_id: &str,
    custody_reason: &str,
    metadata_json: &str,
) -> Result<(), sqlx::Error> {
    audit::append(
        pool,
        None,
        "EVIDENCE_INTEGRITY_FAILURE",
        "evidence",
        evidence_id,
        None,
        metadata_json,
    )
    .await?;
    audit::append_custody(
        pool,
        evidence_id,
        "INTEGRITY_FAILURE",
        "local-analyst",
        custody_reason,
        metadata_json,
    )
    .await
}

/// Constrói o erro devolvido ao operador quando a integridade falha: a
/// mensagem original, mais um aviso explícito quando o registro também falhou.
async fn integrity_failure_error(
    pool: &SqlitePool,
    evidence_id: &str,
    custody_reason: &str,
    metadata_json: &str,
    message: &str,
) -> AppError {
    match record_integrity_failure(pool, evidence_id, custody_reason, metadata_json).await {
        Ok(()) => AppError::Config(message.into()),
        Err(error) => {
            tracing::error!(%error, evidence_id, "failed to record evidence integrity failure");
            AppError::Config(format!(
                "{message} (the failure could NOT be recorded in the audit log)"
            ))
        }
    }
}

async fn mark_job_failed(
    pool: &SqlitePool,
    job_id: &str,
    code: &str,
    message: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE analysis_jobs SET status='FAILED',error_code=?,error_message=?,finished_at=? WHERE id=?")
        .bind(code)
        .bind(message)
        .bind(Utc::now().to_rfc3339())
        .bind(job_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Marca o job como falho e **registra** se essa marcação também falhar
/// (F1-23): o job não pode ficar `PROCESSING` para sempre sem rastro no log
/// (AUD-03). O erro original do fluxo continua sendo devolvido pelo chamador.
async fn mark_job_failed_loudly(pool: &SqlitePool, job_id: &str, code: &str, message: &str) {
    if let Err(error) = mark_job_failed(pool, job_id, code, message).await {
        tracing::warn!(job_id, code, %error, "failed to mark analysis job as failed");
    }
}

/// Recupera jobs que ficaram `PENDING`/`PROCESSING` porque o app anterior foi
/// fechado ou morreu no meio da análise (F1-10, AUD-03). Sem isto o job ficava
/// `PROCESSING` para sempre e `destroy_evidence` recusava a evidência
/// eternamente.
///
/// **Só deve ser chamada na partida do app, antes de qualquer análise nova**:
/// chamada com outro processo analisando, marcaria um job vivo como órfão.
/// Cada job recuperado vira `FAILED` com `error_code = 'ORPHANED'` e gera um
/// evento de auditoria. Devolve quantos jobs foram recuperados.
pub async fn recover_orphaned_jobs(db: &SqlitePool) -> AppResult<u64> {
    let orphans: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT id,evidence_id,status FROM analysis_jobs WHERE status IN ('PENDING','PROCESSING') ORDER BY created_at",
    )
    .fetch_all(db)
    .await?;
    let mut recovered = 0_u64;
    for (job_id, evidence_id, previous_status) in orphans {
        mark_job_failed(
            db,
            &job_id,
            "ORPHANED",
            "analysis was interrupted: the application closed or stopped before the job finished",
        )
        .await?;
        let metadata = serde_json::json!({
            "job_id": job_id,
            "evidence_id": evidence_id,
            "previous_status": previous_status,
        });
        audit::append(
            db,
            None,
            "ANALYSIS_JOB_ORPHANED",
            "analysis_job",
            &job_id,
            None,
            &metadata.to_string(),
        )
        .await?;
        recovered += 1;
    }
    if recovered > 0 {
        tracing::warn!(recovered, "recovered orphaned analysis jobs");
    }
    Ok(recovered)
}

fn engine_error_code(error: &engine::EngineRunError) -> &'static str {
    match error {
        engine::EngineRunError::Timeout => "TIMEOUT",
        engine::EngineRunError::Protocol(_) => "ENGINE_PROTOCOL_ERROR",
        engine::EngineRunError::OutputLimitExceeded => "OUTPUT_LIMIT",
        engine::EngineRunError::DiskSpaceLow => "DISK_SPACE_LOW",
        engine::EngineRunError::EngineFailed(_) => "ENGINE_FAILED",
        engine::EngineRunError::InvalidRequest(_) => "INVALID_REQUEST",
        engine::EngineRunError::Io(_) => "ENGINE_IO_ERROR",
    }
}

// ---------------------------------------------------------------------
// Timeline
// ---------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct TimelineEvent {
    pub id: String,
    pub timestamp: String,
    pub title: String,
    pub description: String,
    pub category: String,
    pub severity: String,
    pub event_id: Option<String>,
    pub provider: Option<String>,
    pub channel: Option<String>,
    pub evidence_id: Option<String>,
    pub raw_reference: Option<String>,
    pub record_id: Option<String>,
    // Campos estruturais (F1-26). Texto vindo de log: já neutralizado para exibição.
    pub host: Option<String>,
    pub user: Option<String>,
    pub src_ip: Option<String>,
    pub dst_ip: Option<String>,
    pub process: Option<String>,
    pub action: Option<String>,
    pub outcome: Option<String>,
    pub source_type: Option<String>,
    #[serde(skip)]
    sort_timestamp: chrono::DateTime<Utc>,
}

#[cfg(test)]
impl TimelineEvent {
    /// Construtor mínimo para testes de fora do módulo (o campo de ordenação
    /// é privado).
    pub fn for_test(title: &str, provider: Option<&str>) -> Self {
        Self {
            id: title.to_string(),
            timestamp: "2026-01-01T00:00:00+00:00".into(),
            title: title.to_string(),
            description: String::new(),
            category: "TEST".into(),
            severity: "INFO".into(),
            event_id: None,
            provider: provider.map(str::to_string),
            channel: None,
            evidence_id: None,
            raw_reference: None,
            record_id: None,
            host: None,
            user: None,
            src_ip: None,
            dst_ip: None,
            process: None,
            action: None,
            outcome: None,
            source_type: None,
            sort_timestamp: Utc::now(),
        }
    }
}

/// Uma página da timeline de um job (ADR-0021). Os eventos vêm do índice no
/// SQLite, já ordenados por UTC e filtrados; nada é carregado inteiro na
/// memória.
#[derive(Debug, Serialize)]
pub struct TimelinePage {
    pub job: JobStatusResponse,
    pub offset: u64,
    /// Total de eventos que passam no filtro. Só é calculado quando a chamada
    /// pede (`want_total`); uma página qualquer não paga o `COUNT`.
    pub total: Option<u64>,
    pub events: Vec<TimelineEvent>,
}

#[cfg(test)]
impl TimelinePage {
    /// Página sintética para testes de GUI.
    pub fn for_test(
        job_id: &str,
        total: Option<u64>,
        offset: u64,
        events: Vec<TimelineEvent>,
        warnings: Vec<JobWarning>,
    ) -> Self {
        Self {
            job: JobStatusResponse {
                job_id: job_id.to_string(),
                request_id: "req".into(),
                evidence_id: "ev".into(),
                status: "COMPLETED".into(),
                kind: dfir_protocol::ANALYSIS_KIND_EVTX_TIMELINE.into(),
                message_count: 1,
                item_count: total.unwrap_or(0),
                output_reference: None,
                error_code: None,
                error_message: None,
                created_at: "2026-01-01T00:00:00Z".into(),
                started_at: None,
                finished_at: None,
                warning_count: warnings.len() as u64,
                warnings,
            },
            offset,
            total,
            events,
        }
    }
}

/// Aviso não fatal de um job (ex.: arquivo EVTX truncado), exibido ao analista.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JobWarning {
    pub code: String,
    pub count: u64,
    pub sample: String,
}

pub const TIMELINE_MAX_PAGE: u64 = 5_000;

/// Lê o `results.jsonl` linha a linha e entrega cada item ao `visit`, sem
/// guardar o `Value` da linha: a memória fica proporcional ao que o chamador
/// retém, não ao tamanho do arquivo. Não há limite de linhas (ADR-0020).
async fn for_each_result_item(
    path: &Path,
    mut visit: impl FnMut(&serde_json::Value),
) -> AppResult<()> {
    let file = tokio::fs::File::open(path)
        .await
        .map_err(AppError::Storage)?;
    let reader = tokio::io::BufReader::new(file);
    let mut lines = tokio::io::AsyncBufReadExt::lines(reader);
    while let Some(line) = lines.next_line().await.map_err(AppError::Storage)? {
        if dfir_protocol::json_depth_exceeds_limit(line.as_bytes(), dfir_protocol::MAX_JSON_DEPTH) {
            return Err(AppError::Config(
                "stored analysis result exceeds JSON nesting depth limit".into(),
            ));
        }
        let value: serde_json::Value = serde_json::from_str(&line).map_err(|error| {
            AppError::Config(format!("stored analysis result is malformed JSON: {error}"))
        })?;
        if let Some(array) = value.get("items").and_then(serde_json::Value::as_array) {
            array.iter().for_each(&mut visit);
        }
    }
    Ok(())
}

/// Todos os itens do resultado (usado por YARA, cujos itens são poucos).
async fn read_result_items(path: &Path) -> AppResult<Vec<serde_json::Value>> {
    let mut items = Vec::new();
    for_each_result_item(path, |item| items.push(item.clone())).await?;
    Ok(items)
}

#[derive(Debug, FromRow)]
struct TimelineRow {
    sort_ts: i64,
    id: String,
    timestamp: String,
    title: String,
    description: String,
    category: String,
    severity: String,
    event_id: Option<String>,
    provider: Option<String>,
    channel: Option<String>,
    evidence_id: Option<String>,
    raw_reference: Option<String>,
    record_id: Option<String>,
    host: Option<String>,
    user_name: Option<String>,
    src_ip: Option<String>,
    dst_ip: Option<String>,
    process: Option<String>,
    action: Option<String>,
    outcome: Option<String>,
    source_type: Option<String>,
}

/// Texto que veio de log, pronto para exibir: caracteres invisíveis e de
/// direção de texto viram marcadores visíveis (`text_safety`).
fn display_text(value: String) -> String {
    match crate::text_safety::neutralize(&value) {
        std::borrow::Cow::Borrowed(_) => value,
        std::borrow::Cow::Owned(safe) => safe,
    }
}

fn display_option(value: Option<String>) -> Option<String> {
    value.map(display_text)
}

impl From<TimelineRow> for TimelineEvent {
    fn from(row: TimelineRow) -> Self {
        Self {
            id: row.id,
            timestamp: row.timestamp,
            title: display_text(row.title),
            description: display_text(row.description),
            category: row.category,
            severity: row.severity,
            event_id: display_option(row.event_id),
            provider: display_option(row.provider),
            channel: display_option(row.channel),
            evidence_id: row.evidence_id,
            raw_reference: row.raw_reference,
            record_id: row.record_id,
            host: display_option(row.host),
            user: display_option(row.user_name),
            src_ip: display_option(row.src_ip),
            dst_ip: display_option(row.dst_ip),
            process: display_option(row.process),
            action: row.action,
            outcome: row.outcome,
            source_type: row.source_type,
            sort_timestamp: chrono::DateTime::from_timestamp_micros(row.sort_ts)
                .unwrap_or_default(),
        }
    }
}

/// Indexa o `results.jsonl` de um job no SQLite, em fluxo: a memória não
/// cresce com o número de eventos (ADR-0021). Idempotente (refaz o índice).
/// Devolve `(indexados, ignorados)`; "ignorados" são itens sem timestamp
/// válido, que antes sumiam em silêncio e agora viram aviso do job.
async fn index_timeline(db: &SqlitePool, job_id: &str, path: &Path) -> AppResult<(u64, u64)> {
    let started = std::time::Instant::now();
    let result = index_timeline_inner(db, job_id, path).await;
    let elapsed_ms = started.elapsed().as_millis() as u64;
    match &result {
        Ok((indexed, skipped)) => {
            tracing::info!(job_id, indexed, skipped, elapsed_ms, "timeline indexed")
        }
        Err(error) => tracing::error!(job_id, %error, elapsed_ms, "timeline indexing failed"),
    }
    result
}

async fn index_timeline_inner(db: &SqlitePool, job_id: &str, path: &Path) -> AppResult<(u64, u64)> {
    let file = tokio::fs::File::open(path)
        .await
        .map_err(AppError::Storage)?;
    let mut lines = tokio::io::AsyncBufReadExt::lines(tokio::io::BufReader::new(file));
    let mut connection = begin_immediate(db).await.map_err(AppError::Database)?;
    let outcome: Result<(u64, u64), AppError> = async {
        sqlx::query("DELETE FROM timeline_events WHERE job_id=?")
            .bind(job_id)
            .execute(&mut *connection)
            .await?;
        sqlx::query("DELETE FROM timeline_index WHERE job_id=?")
            .bind(job_id)
            .execute(&mut *connection)
            .await?;
        let mut ord: i64 = 0;
        let (mut indexed, mut skipped) = (0_u64, 0_u64);
        while let Some(line) = lines.next_line().await.map_err(AppError::Storage)? {
            if dfir_protocol::json_depth_exceeds_limit(line.as_bytes(), dfir_protocol::MAX_JSON_DEPTH) {
                continue;
            }
            let Ok(value) = serde_json::from_str::<serde_json::Value>(&line) else {
                continue;
            };
            let Some(array) = value.get("items").and_then(serde_json::Value::as_array) else {
                continue;
            };
            for item in array {
                ord += 1;
                let Some(event) = timeline_event_from_item(item) else {
                    skipped += 1;
                    continue;
                };
                sqlx::query(
                    "INSERT INTO timeline_events (job_id,sort_ts,ord,id,timestamp,title,description,category,severity,event_id,provider,channel,evidence_id,raw_reference,record_id,host,user_name,src_ip,dst_ip,process,action,outcome,source_type) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
                )
                .bind(job_id)
                .bind(event.sort_timestamp.timestamp_micros())
                .bind(ord)
                .bind(&event.id)
                .bind(&event.timestamp)
                .bind(&event.title)
                .bind(&event.description)
                .bind(&event.category)
                .bind(&event.severity)
                .bind(&event.event_id)
                .bind(&event.provider)
                .bind(&event.channel)
                .bind(&event.evidence_id)
                .bind(&event.raw_reference)
                .bind(&event.record_id)
                .bind(&event.host)
                .bind(&event.user)
                .bind(&event.src_ip)
                .bind(&event.dst_ip)
                .bind(&event.process)
                .bind(&event.action)
                .bind(&event.outcome)
                .bind(&event.source_type)
                .execute(&mut *connection)
                .await?;
                indexed += 1;
            }
        }
        sqlx::query(
            "INSERT INTO timeline_index (job_id,event_count,skipped_count,indexed_at) VALUES (?,?,?,?)",
        )
        .bind(job_id)
        .bind(indexed as i64)
        .bind(skipped as i64)
        .bind(Utc::now().to_rfc3339())
        .execute(&mut *connection)
        .await?;
        Ok((indexed, skipped))
    }
    .await;
    match outcome {
        Ok(counts) => {
            if let Err(error) = sqlx::query("COMMIT").execute(&mut *connection).await {
                rollback(&mut connection).await;
                return Err(AppError::Database(error));
            }
            Ok(counts)
        }
        Err(error) => {
            rollback(&mut connection).await;
            Err(error)
        }
    }
}

/// Escapa `%`, `_` e `\` para uso em `LIKE ... ESCAPE '\'`.
fn like_pattern(text: &str) -> String {
    let mut pattern = String::with_capacity(text.len() + 2);
    pattern.push('%');
    for character in text.chars() {
        if matches!(character, '%' | '_' | '\\') {
            pattern.push('\\');
        }
        pattern.push(character);
    }
    pattern.push('%');
    pattern
}

/// O job EVTX concluído mais recente da evidência, se houver (a GUI o usa para
/// mostrar a timeline ao selecionar a evidência, sem reanalisar).
pub async fn latest_timeline_job(
    services: &AppServices,
    evidence_id: &str,
) -> AppResult<Option<String>> {
    Ok(sqlx::query_scalar(
        "SELECT id FROM analysis_jobs WHERE evidence_id=? AND kind='EVTX_TIMELINE' AND status='COMPLETED' ORDER BY created_at DESC LIMIT 1",
    )
    .bind(evidence_id)
    .fetch_optional(&services.db)
    .await?)
}

/// Uma página da timeline de um job, ordenada por UTC. `filter` busca (sem
/// diferenciar maiúsculas ASCII) em título, descrição, provedor e canal.
pub async fn timeline_page(
    services: &AppServices,
    job_id: &str,
    offset: u64,
    limit: u64,
    filter: Option<&str>,
    want_total: bool,
) -> AppResult<TimelinePage> {
    let started = std::time::Instant::now();
    let page = timeline_page_core(
        &services.db,
        &services.paths,
        job_id,
        offset,
        limit,
        filter,
        want_total,
    )
    .await;
    if let Ok(page) = &page {
        tracing::debug!(
            job_id,
            offset,
            limit,
            filtered = filter.is_some_and(|text| !text.trim().is_empty()),
            total = ?page.total,
            rows = page.events.len(),
            elapsed_ms = started.elapsed().as_millis() as u64,
            "timeline page served"
        );
    }
    page
}

async fn timeline_page_core(
    db: &SqlitePool,
    paths: &StoragePaths,
    job_id: &str,
    offset: u64,
    limit: u64,
    filter: Option<&str>,
    want_total: bool,
) -> AppResult<TimelinePage> {
    let row = sqlx::query_as::<_, JobRow>("SELECT id,request_id,evidence_id,status,kind,message_count,item_count,output_reference,error_code,error_message,created_at,started_at,finished_at,warning_count,warnings FROM analysis_jobs WHERE id=?")
        .bind(job_id)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| AppError::NotFound("analysis job not found".into()))?;
    let job = JobStatusResponse::try_from(row)?;
    let produces_timeline =
        job.kind == dfir_protocol::ANALYSIS_KIND_EVTX_TIMELINE && job.status == "COMPLETED";
    let Some(reference) = job.output_reference.clone().filter(|_| produces_timeline) else {
        return Ok(TimelinePage {
            job,
            offset,
            total: Some(0),
            events: Vec::new(),
        });
    };

    // Jobs concluídos antes do índice existir (ou com índice perdido) são
    // indexados na primeira consulta.
    let indexed_count: Option<i64> =
        sqlx::query_scalar("SELECT event_count FROM timeline_index WHERE job_id=?")
            .bind(job_id)
            .fetch_optional(db)
            .await?;
    let indexed_count = match indexed_count {
        Some(count) => count,
        None => {
            let path = paths.logical_path(&reference).map_err(AppError::Storage)?;
            index_timeline(db, job_id, &path).await?.0 as i64
        }
    };

    let limit = limit.clamp(1, TIMELINE_MAX_PAGE);
    // Texto livre + termos `campo:valor` (ver `timeline_filter`). Um filtro
    // inválido (data malformada) é erro para o analista, nunca texto livre.
    let parsed = timeline_filter::parse(filter.unwrap_or("")).map_err(AppError::Config)?;
    let total = if !want_total {
        None
    } else if parsed.is_empty() {
        Some(indexed_count.max(0) as u64)
    } else {
        let mut query = sqlx::QueryBuilder::<sqlx::Sqlite>::new(
            "SELECT COUNT(*) FROM timeline_events WHERE job_id=",
        );
        query.push_bind(job_id);
        push_timeline_filter(&mut query, &parsed);
        let count: i64 = query.build_query_scalar().fetch_one(db).await?;
        Some(count.max(0) as u64)
    };

    let offset_sql = offset.min(i64::MAX as u64) as i64;
    let mut query = sqlx::QueryBuilder::<sqlx::Sqlite>::new(TIMELINE_SELECT);
    query.push(" FROM timeline_events WHERE job_id=");
    query.push_bind(job_id);
    push_timeline_filter(&mut query, &parsed);
    query.push(" ORDER BY sort_ts, ord LIMIT ");
    query.push_bind(limit as i64);
    query.push(" OFFSET ");
    query.push_bind(offset_sql);
    let rows: Vec<TimelineRow> = query.build_query_as().fetch_all(db).await?;

    Ok(TimelinePage {
        job,
        offset,
        total,
        events: rows.into_iter().map(Into::into).collect(),
    })
}

/// Colunas lidas da timeline (a ordem não importa: o mapeamento é por nome).
const TIMELINE_SELECT: &str = "SELECT sort_ts,id,timestamp,title,description,category,severity,event_id,provider,channel,evidence_id,raw_reference,record_id,host,user_name,src_ip,dst_ip,process,action,outcome,source_type";

/// `AND (coluna LIKE ? OR ...)` para os valores informados (contém, sem
/// diferenciar maiúsculas ASCII). Só texto fixo; os valores vão ligados.
fn push_any_like(query: &mut sqlx::QueryBuilder<sqlx::Sqlite>, column: &str, values: &[String]) {
    if values.is_empty() {
        return;
    }
    query.push(" AND (");
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            query.push(" OR ");
        }
        query.push(column);
        query.push(" LIKE ");
        query.push_bind(like_pattern(value));
        query.push(" ESCAPE '\\'");
    }
    query.push(")");
}

/// `AND coluna IN (?, ...)` (igualdade exata).
fn push_any_equal(
    query: &mut sqlx::QueryBuilder<sqlx::Sqlite>,
    columns: &[&str],
    values: &[String],
) {
    if values.is_empty() {
        return;
    }
    query.push(" AND (");
    for (column_index, column) in columns.iter().enumerate() {
        if column_index > 0 {
            query.push(" OR ");
        }
        query.push(*column);
        query.push(" IN (");
        let mut separated = query.separated(", ");
        for value in values {
            separated.push_bind(value.clone());
        }
        separated.push_unseparated(")");
    }
    query.push(")");
}

fn push_timeline_filter(
    query: &mut sqlx::QueryBuilder<sqlx::Sqlite>,
    filter: &timeline_filter::TimelineFilter,
) {
    if let Some(text) = &filter.text {
        let pattern = like_pattern(text);
        query.push(" AND (title LIKE ");
        query.push_bind(pattern.clone());
        query.push(" ESCAPE '\\' OR description LIKE ");
        query.push_bind(pattern.clone());
        query.push(" ESCAPE '\\' OR provider LIKE ");
        query.push_bind(pattern.clone());
        query.push(" ESCAPE '\\' OR channel LIKE ");
        query.push_bind(pattern);
        query.push(" ESCAPE '\\')");
    }
    push_any_equal(query, &["event_id"], &filter.event_ids);
    push_any_like(query, "user_name", &filter.users);
    push_any_like(query, "host", &filter.hosts);
    push_any_equal(query, &["src_ip", "dst_ip"], &filter.ips);
    push_any_equal(query, &["src_ip"], &filter.src_ips);
    push_any_equal(query, &["dst_ip"], &filter.dst_ips);
    push_any_like(query, "process", &filter.processes);
    push_any_like(query, "action", &filter.actions);
    push_any_equal(query, &["outcome"], &filter.outcomes);
    push_any_like(query, "provider", &filter.providers);
    push_any_like(query, "channel", &filter.channels);
    if let Some(from) = filter.from_micros {
        query.push(" AND sort_ts >= ");
        query.push_bind(from);
    }
    if let Some(to) = filter.to_micros {
        query.push(" AND sort_ts < ");
        query.push_bind(to);
    }
}

fn timeline_event_from_item(item: &serde_json::Value) -> Option<TimelineEvent> {
    let normalized = NormalizedEvent::from_item(item)?;
    let sort_timestamp = chrono::DateTime::parse_from_rfc3339(&normalized.timestamp)
        .ok()?
        .with_timezone(&Utc);
    let record_id = normalized.event_record_id.to_string();
    let data = &normalized.data;

    let event_id = data
        .pointer("/Event/System/EventID")
        .map(value_to_string)
        .filter(|v| !v.is_empty());
    let provider = data
        .pointer("/Event/System/Provider/Name")
        .or_else(|| data.pointer("/Event/System/Provider/#attributes/Name"))
        .map(value_to_string)
        .filter(|v| !v.is_empty());
    let channel = data
        .pointer("/Event/System/Channel")
        .map(value_to_string)
        .filter(|v| !v.is_empty());
    let severity = data
        .pointer("/Event/System/Level")
        .map(value_to_string)
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "INFO".into());
    let title = match (&event_id, &provider) {
        (Some(id), Some(provider)) => format!("Event ID {id} · {provider}"),
        (Some(id), None) => format!("Event ID {id}"),
        (None, _) => format!("EVTX record {record_id}"),
    };
    let computer = data.pointer("/Event/System/Computer").map(value_to_string);
    let description = build_event_description(
        data,
        event_id.as_deref().unwrap_or("unknown"),
        provider.as_deref(),
        channel.as_deref(),
        computer.as_deref(),
    );

    let id = format!("{}:{}", normalized.evidence_id, normalized.raw_reference);

    // Resultados da versão 3 do esquema não trazem os campos estruturais: o
    // app os extrai do próprio registro, com o mesmo extrator do engine.
    let structured = if normalized.source_type.is_some() {
        dfir_protocol::EventFields {
            event_id: normalized.event_id.clone(),
            provider: normalized.provider.clone(),
            channel: normalized.channel.clone(),
            host: normalized.host.clone(),
            user: normalized.user.clone(),
            src_ip: normalized.src_ip.clone(),
            dst_ip: normalized.dst_ip.clone(),
            process: normalized.process.clone(),
            command_line: normalized.command_line.clone(),
            action: normalized.action.clone(),
            outcome: normalized.outcome.clone(),
            source_type: normalized.source_type.clone(),
        }
    } else {
        dfir_protocol::normalize::event_fields(data)
    };

    Some(TimelineEvent {
        id,
        timestamp: normalized.timestamp,
        title,
        description,
        category: "EVTX".into(),
        severity,
        event_id,
        provider,
        channel,
        evidence_id: Some(normalized.evidence_id),
        raw_reference: Some(normalized.raw_reference),
        record_id: Some(record_id),
        host: structured.host,
        user: structured.user,
        src_ip: structured.src_ip,
        dst_ip: structured.dst_ip,
        process: structured.process,
        action: structured.action,
        outcome: structured.outcome,
        source_type: structured.source_type,
        sort_timestamp,
    })
}

fn build_event_description(
    data: &serde_json::Value,
    event_id: &str,
    provider: Option<&str>,
    channel: Option<&str>,
    computer: Option<&str>,
) -> String {
    let mut parts = vec![format!("Event ID {event_id}")];
    if let Some(provider) = provider {
        parts.push(format!("Provider={provider}"));
    }
    if let Some(channel) = channel {
        parts.push(format!("Channel={channel}"));
    }
    if let Some(computer) = computer {
        parts.push(format!("Computer={computer}"));
    }
    let mut fields = Vec::new();
    if let Some(event_data) = data.pointer("/Event/EventData") {
        collect_event_fields(event_data, "", &mut fields);
    }
    if !fields.is_empty() {
        parts.push(fields.join("; "));
    }
    parts.join(" · ")
}

fn collect_event_fields(value: &serde_json::Value, prefix: &str, fields: &mut Vec<String>) {
    if fields.len() >= 32 {
        return;
    }
    match value {
        serde_json::Value::Object(map) => {
            if let Some(name) = map.get("@Name").or_else(|| map.get("Name"))
                && let Some(text) = map
                    .get("#text")
                    .or_else(|| map.get("Value"))
                    .map(value_to_string)
            {
                fields.push(format!("{}={}", value_to_string(name), text));
                return;
            }
            for (key, child) in map {
                if matches!(key.as_str(), "@Name" | "Name" | "#text" | "Value") {
                    continue;
                }
                let next_prefix = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{prefix}.{key}")
                };
                if child.is_object() || child.is_array() {
                    collect_event_fields(child, &next_prefix, fields);
                } else {
                    fields.push(format!("{}={}", next_prefix, value_to_string(child)));
                }
                if fields.len() >= 32 {
                    return;
                }
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                collect_event_fields(item, prefix, fields);
                if fields.len() >= 32 {
                    return;
                }
            }
        }
        _ if !prefix.is_empty() => fields.push(format!("{}={}", prefix, value_to_string(value))),
        _ => {}
    }
}

fn value_to_string(value: &serde_json::Value) -> String {
    value
        .as_str()
        .map(str::to_owned)
        .unwrap_or_else(|| value.to_string())
}

async fn hash_file(path: &Path) -> AppResult<(String, String)> {
    use tokio::io::AsyncReadExt;
    let mut file = tokio::fs::File::open(path)
        .await
        .map_err(AppError::Storage)?;
    // Heap, não pilha: um `[0_u8; 1 MiB]` dentro de um `async fn` vira parte do
    // future e estourava a pilha da thread principal em build debug
    // (achado pelo teste headless, F1-34).
    let mut buffer = vec![0_u8; 1024 * 1024];
    let mut blake3_hasher = blake3::Hasher::new();
    let mut sha256_hasher = Sha256::new();
    loop {
        let read = file.read(&mut buffer).await.map_err(AppError::Storage)?;
        if read == 0 {
            break;
        }
        blake3_hasher.update(&buffer[..read]);
        sha256_hasher.update(&buffer[..read]);
    }
    Ok((
        blake3_hasher.finalize().to_hex().to_string(),
        format!("{:x}", sha256_hasher.finalize()),
    ))
}

fn detect_format(filename: &str) -> String {
    filename
        .rsplit_once('.')
        .map(|(_, ext)| ext.to_ascii_lowercase())
        .filter(|ext| !ext.is_empty())
        .unwrap_or_else(|| "unknown".into())
}

fn safe_extension(filename: &str) -> String {
    let extension = filename
        .rsplit_once('.')
        .map(|(_, ext)| ext)
        .unwrap_or_default();
    let extension = extension.to_ascii_lowercase();
    if extension.len() <= 12 && extension.bytes().all(|byte| byte.is_ascii_alphanumeric()) {
        extension
    } else {
        String::new()
    }
}

// ---------------------------------------------------------------------
// Resultados analíticos (compartilhado entre evidência e incidentes)
// ---------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticalResultResponse {
    pub id: String,
    pub result_type: String,
    pub correlation_key: String,
    pub evidence_id: Option<String>,
    pub incident_id: Option<String>,
    pub analysis_job_id: Option<String>,
    pub rule_or_model_id: Option<String>,
    pub rule_or_model_version: Option<String>,
    pub confidence: Option<String>,
    pub payload: serde_json::Value,
    pub provenance: serde_json::Value,
    pub supersedes: Option<String>,
    pub created_at: String,
}

impl From<analytical_results::AnalyticalResultRecord> for AnalyticalResultResponse {
    fn from(record: analytical_results::AnalyticalResultRecord) -> Self {
        let payload = serde_json::from_str(&record.payload).unwrap_or(serde_json::Value::Null);
        let provenance =
            serde_json::from_str(&record.provenance).unwrap_or(serde_json::Value::Null);
        Self {
            id: record.id,
            result_type: record.result_type,
            correlation_key: record.correlation_key,
            evidence_id: record.evidence_id,
            incident_id: record.incident_id,
            analysis_job_id: record.analysis_job_id,
            rule_or_model_id: record.rule_or_model_id,
            rule_or_model_version: record.rule_or_model_version,
            confidence: record.confidence,
            payload,
            provenance,
            supersedes: record.supersedes,
            created_at: record.created_at,
        }
    }
}

/// Resultados analíticos **vigentes** (não superseded) de um incidente,
/// filtrados por `result_type`. Reaproveitado por NIST gap, ATT&CK mapping,
/// caminho crítico e matriz de impacto. Mesma limitação de janela de
/// `MAX_LISTED_RESULTS` (500, ver `analytical_results.rs`) do handler
/// original — aceitável para o volume esperado no MVP (ADR-0011).
pub(crate) async fn current_results_for_incident(
    pool: &SqlitePool,
    incident_id: &str,
    result_type: &str,
) -> AppResult<Vec<AnalyticalResultResponse>> {
    let records = analytical_results::list_by_incident(pool, incident_id).await?;
    let superseded_ids: std::collections::HashSet<&str> = records
        .iter()
        .filter_map(|record| record.supersedes.as_deref())
        .collect();
    Ok(records
        .iter()
        .filter(|record| {
            record.result_type == result_type && !superseded_ids.contains(record.id.as_str())
        })
        .cloned()
        .map(Into::into)
        .collect())
}

pub async fn evidence_analytical_results(
    services: &AppServices,
    evidence_id: &str,
) -> AppResult<Vec<AnalyticalResultResponse>> {
    sqlx::query_scalar::<_, String>(
        "SELECT id FROM evidence WHERE id=? AND preservation_status != 'DESTROYED'",
    )
    .bind(evidence_id)
    .fetch_optional(&services.db)
    .await?
    .ok_or_else(|| AppError::NotFound("evidence not found".into()))?;

    let records = analytical_results::list_current_by_evidence(&services.db, evidence_id).await?;
    Ok(records.into_iter().map(Into::into).collect())
}

// ---------------------------------------------------------------------
// Incidentes
// ---------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct IncidentResponse {
    pub id: String,
    pub name: String,
    pub state: String,
    pub created_at: String,
}

pub async fn create_incident(services: &AppServices, name: &str) -> AppResult<IncidentResponse> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 200 {
        return Err(AppError::Config(
            "incident name must be 1..=200 characters".into(),
        ));
    }
    let id = Uuid::new_v4().to_string();
    let created_at = Utc::now().to_rfc3339();

    let mut connection = begin_immediate(&services.db).await?;
    let outcome: Result<(), sqlx::Error> = async {
        sqlx::query("INSERT INTO incidents(id,name,state,created_at) VALUES(?,?,?,?)")
            .bind(&id)
            .bind(name)
            .bind("OPEN")
            .bind(&created_at)
            .execute(&mut *connection)
            .await?;
        let metadata = serde_json::json!({ "incident_id": &id, "name": name }).to_string();
        audit::append_on_connection(
            &mut connection,
            None,
            "INCIDENT_CREATED",
            "incident",
            &id,
            Some(id.as_str()),
            &metadata,
        )
        .await
    }
    .await;
    match outcome {
        Ok(()) => sqlx::query("COMMIT").execute(&mut *connection).await?,
        Err(error) => {
            rollback(&mut connection).await;
            return Err(AppError::Database(error));
        }
    };

    Ok(IncidentResponse {
        id,
        name: name.to_string(),
        state: "OPEN".to_string(),
        created_at,
    })
}

pub async fn list_incidents(services: &AppServices) -> AppResult<Vec<IncidentResponse>> {
    let rows = sqlx::query_as::<_, IncidentResponse>(
        "SELECT id,name,state,created_at FROM incidents ORDER BY created_at DESC",
    )
    .fetch_all(&services.db)
    .await?;
    Ok(rows)
}

pub(crate) async fn incident_exists(pool: &SqlitePool, incident_id: &str) -> AppResult<()> {
    sqlx::query_scalar::<_, String>("SELECT id FROM incidents WHERE id=?")
        .bind(incident_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("incident not found".into()))?;
    Ok(())
}

/// Lista as evidências vinculadas a um incidente (F1-19). Evidência
/// descartada continua listada aqui: o vínculo histórico não pode sumir com
/// o arquivo (mesma regra dos documentos, que citam a evidência pelo hash).
pub async fn evidence_for_incident(
    services: &AppServices,
    incident_id: &str,
) -> AppResult<Vec<EvidenceResponse>> {
    incident_exists(&services.db, incident_id).await?;
    let rows = sqlx::query_as::<_, EvidenceRow>(concat!(
        "SELECT ",
        evidence_columns!(),
        " FROM evidence WHERE incident_id=? ORDER BY imported_at DESC"
    ))
    .bind(incident_id)
    .fetch_all(&services.db)
    .await?;
    Ok(rows.into_iter().map(EvidenceResponse::from).collect())
}

/// Vincula uma evidência a um incidente (F1-19). O vínculo é gravado na
/// própria linha da evidência (`evidence.incident_id`, migration 0005) e na
/// auditoria; trocar de incidente é permitido (o vínculo anterior fica no
/// log de auditoria, a coluna guarda só o vigente). Repetir o mesmo vínculo
/// é operação idempotente.
pub async fn link_evidence_to_incident(
    services: &AppServices,
    evidence_id: &str,
    incident_id: &str,
) -> AppResult<EvidenceResponse> {
    incident_exists(&services.db, incident_id).await?;
    let row = sqlx::query_as::<_, EvidenceRow>(concat!(
        "SELECT ",
        evidence_columns!(),
        " FROM evidence WHERE id=?"
    ))
    .bind(evidence_id)
    .fetch_optional(&services.db)
    .await?
    .ok_or_else(|| AppError::NotFound("evidence not found".into()))?;
    if row.incident_id.as_deref() == Some(incident_id) {
        return Ok(EvidenceResponse::from(row));
    }

    let mut connection = begin_immediate(&services.db).await?;
    let outcome: Result<(), sqlx::Error> = async {
        sqlx::query("UPDATE evidence SET incident_id=? WHERE id=?")
            .bind(incident_id)
            .bind(evidence_id)
            .execute(&mut *connection)
            .await?;
        let metadata = serde_json::json!({
            "evidence_id": evidence_id,
            "incident_id": incident_id,
            "previous_incident_id": row.incident_id,
        })
        .to_string();
        audit::append_on_connection(
            &mut connection,
            None,
            "EVIDENCE_LINKED_TO_INCIDENT",
            "evidence",
            evidence_id,
            Some(incident_id),
            &metadata,
        )
        .await
    }
    .await;
    match outcome {
        Ok(()) => {
            sqlx::query("COMMIT").execute(&mut *connection).await?;
        }
        Err(error) => {
            rollback(&mut connection).await;
            return Err(AppError::Database(error));
        }
    }

    let updated = sqlx::query_as::<_, EvidenceRow>(concat!(
        "SELECT ",
        evidence_columns!(),
        " FROM evidence WHERE id=?"
    ))
    .bind(evidence_id)
    .fetch_one(&services.db)
    .await?;
    Ok(EvidenceResponse::from(updated))
}

/// Relatório de diagnóstico para suporte (ADR-0022): versões, contagens do
/// banco, falhas recentes por **código**, espaço livre e arquivos de log.
/// Deliberadamente não inclui nomes de arquivo, caminhos de evidência, nem
/// mensagens de erro (podem conter caminhos): só números, ids e códigos.
pub async fn diagnostics_report(
    db: &SqlitePool,
    paths: &StoragePaths,
    settings: serde_json::Value,
) -> AppResult<serde_json::Value> {
    let migrations: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations WHERE success=1")
            .fetch_one(db)
            .await?;
    let evidence_active: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM evidence WHERE preservation_status != 'DESTROYED'",
    )
    .fetch_one(db)
    .await?;
    let evidence_destroyed: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM evidence WHERE preservation_status = 'DESTROYED'")
            .fetch_one(db)
            .await?;
    let by_status: Vec<(String, i64)> =
        sqlx::query_as("SELECT status,COUNT(*) FROM analysis_jobs GROUP BY status ORDER BY status")
            .fetch_all(db)
            .await?;
    let with_warnings: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM analysis_jobs WHERE warning_count > 0")
            .fetch_one(db)
            .await?;
    let failed: Vec<(String, String, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT id,kind,error_code,finished_at FROM analysis_jobs WHERE status='FAILED' ORDER BY created_at DESC LIMIT 10",
    )
    .fetch_all(db)
    .await?;
    let mut log_files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&paths.logs) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                log_files.push(serde_json::json!({
                    "name": entry.file_name().to_string_lossy(),
                    "size_bytes": metadata.len(),
                }));
            }
        }
    }
    let jobs_by_status: serde_json::Map<String, serde_json::Value> = by_status
        .into_iter()
        .map(|(status, count)| (status, serde_json::json!(count)))
        .collect();
    Ok(serde_json::json!({
        "generated_at": Utc::now().to_rfc3339(),
        "app": {
            "version": env!("CARGO_PKG_VERSION"),
            "os": std::env::consts::OS,
            "arch": std::env::consts::ARCH,
        },
        "settings": settings,
        "database": {
            "migrations_applied": migrations,
            "evidence_active": evidence_active,
            "evidence_destroyed": evidence_destroyed,
            "jobs_by_status": jobs_by_status,
            "jobs_with_warnings": with_warnings,
            "recent_failed_jobs": failed
                .into_iter()
                .map(|(job_id, kind, error_code, finished_at)| serde_json::json!({
                    "job_id": job_id,
                    "kind": kind,
                    "error_code": error_code,
                    "finished_at": finished_at,
                }))
                .collect::<Vec<_>>(),
        },
        "disk": {
            "data_volume_free_bytes": crate::infrastructure::disk::available_space(&paths.evidence).ok(),
        },
        "logs": { "dir": paths.logs, "files": log_files },
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::db;

    struct Fixture {
        pool: SqlitePool,
        paths: StoragePaths,
        _root: tempfile::TempDir,
    }

    async fn fixture() -> Fixture {
        let root = tempfile::tempdir().expect("temporary directory");
        let paths = StoragePaths::from_root(root.path());
        paths.ensure().expect("storage layout");
        let url = format!(
            "sqlite://{}",
            paths.db.join("service-test.sqlite").display()
        );
        let pool = db::connect(&url).await.expect("database connection");
        db::migrate(&pool).await.expect("migrations");
        Fixture {
            pool,
            paths,
            _root: root,
        }
    }

    async fn seed_evidence(fx: &Fixture, id: &str, with_job: bool) -> std::path::PathBuf {
        let reference = format!("evidence/{id}.evtx");
        let file = fx.paths.logical_path(&reference).expect("logical path");
        tokio::fs::write(&file, b"evidence bytes")
            .await
            .expect("write evidence file");
        sqlx::query(
            "INSERT INTO evidence(id,original_filename,storage_reference,format,size_bytes,blake3,sha256,imported_at,status,mime_type,source,collection_method,collected_at,ingested_by,preservation_status) VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
        )
        .bind(id)
        .bind("sample.evtx")
        .bind(&reference)
        .bind("EVTX")
        .bind(14_i64)
        .bind("b3")
        .bind("s256")
        .bind("2026-10-02T00:00:00Z")
        .bind("FORENSIC")
        .bind("application/octet-stream")
        .bind("LOCAL_OPERATOR")
        .bind("LOCAL_FILE_PICKER")
        .bind("2026-10-02T00:00:00Z")
        .bind("local-analyst")
        .bind("ACTIVE")
        .execute(&fx.pool)
        .await
        .expect("insert evidence");
        if with_job {
            sqlx::query(
                "INSERT INTO analysis_jobs(id,request_id,evidence_id,status,max_output_bytes,max_items,created_at) VALUES(?,?,?,?,?,?,?)",
            )
            .bind(format!("job_{id}"))
            .bind(format!("req_{id}"))
            .bind(id)
            .bind("COMPLETED")
            .bind(1_i64)
            .bind(1_i64)
            .bind("2026-10-02T00:00:00Z")
            .execute(&fx.pool)
            .await
            .expect("insert job");
            let job_dir = fx.paths.staging.join("analysis").join(format!("job_{id}"));
            tokio::fs::create_dir_all(&job_dir).await.expect("job dir");
            tokio::fs::write(job_dir.join("results.jsonl"), b"{}")
                .await
                .expect("job output");
        }
        file
    }

    async fn status_of(fx: &Fixture, id: &str) -> String {
        sqlx::query_scalar("SELECT preservation_status FROM evidence WHERE id=?")
            .bind(id)
            .fetch_one(&fx.pool)
            .await
            .expect("status")
    }

    async fn audit_count(fx: &Fixture, action: &str) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*) FROM audit_events WHERE action=?")
            .bind(action)
            .fetch_one(&fx.pool)
            .await
            .expect("audit count")
    }

    fn evtx_item(record_id: u64, timestamp: &str) -> serde_json::Value {
        dfir_protocol::NormalizedEvent::new(
            record_id,
            timestamp,
            serde_json::json!({"Event": {"System": {"EventID": 4624, "Channel": "Security"}}}),
            "ev_t",
        )
        .into_item()
        .expect("serializable event")
    }

    /// Escreve um `results.jsonl` no formato do engine (lotes de 128 itens por
    /// linha) com `count` eventos em ordem cronológica **inversa**.
    async fn write_results_file(path: &std::path::Path, count: u64) {
        use tokio::io::AsyncWriteExt;
        let base = chrono::DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z")
            .expect("base timestamp")
            .with_timezone(&Utc);
        let mut file = tokio::fs::File::create(path).await.expect("create results");
        let mut batch: Vec<serde_json::Value> = Vec::with_capacity(128);
        for index in (0..count).rev() {
            let stamp = (base + chrono::Duration::seconds(index as i64)).to_rfc3339();
            batch.push(evtx_item(index + 1, &stamp));
            if batch.len() == 128 {
                let line = serde_json::json!({"message_type": "RESULT", "items": batch});
                file.write_all(format!("{line}\n").as_bytes())
                    .await
                    .expect("write line");
                batch.clear();
            }
        }
        if !batch.is_empty() {
            let line = serde_json::json!({"message_type": "RESULT", "items": batch});
            file.write_all(format!("{line}\n").as_bytes())
                .await
                .expect("write line");
        }
        file.flush().await.expect("flush");
    }

    /// Semeia um job EVTX concluído cujo `results.jsonl` tem `count` eventos
    /// em ordem cronológica inversa; devolve o id do job.
    async fn seed_timeline_job(fx: &Fixture, id: &str, count: u64) -> String {
        seed_evidence(fx, id, true).await;
        let job_id = format!("job_{id}");
        let reference = format!("staging/analysis/{job_id}/output/results.jsonl");
        let path = fx.paths.logical_path(&reference).expect("logical path");
        tokio::fs::create_dir_all(path.parent().expect("parent"))
            .await
            .expect("output dir");
        write_results_file(&path, count).await;
        sqlx::query("UPDATE analysis_jobs SET output_reference=? WHERE id=?")
            .bind(&reference)
            .bind(&job_id)
            .execute(&fx.pool)
            .await
            .expect("set output reference");
        job_id
    }

    /// Regressão da Etapa 6: `analyze` lia a evidência com menos colunas do
    /// que `EvidenceRow` exige e falhava com `ColumnNotFound("source")` antes
    /// de qualquer validação. Sem engine nos testes unitários, o que se prova
    /// aqui é que a linha carrega: o erro esperado é o do tipo de análise
    /// inválido, que só vem depois da leitura da evidência.
    #[tokio::test]
    async fn analyze_loads_the_evidence_row_before_validating_the_kind() {
        let fx = crate::service::test_support::fixture().await;
        let dir = tempfile::tempdir().expect("temporary directory");
        let file = dir.path().join("amostra.evtx");
        std::fs::write(&file, b"conteudo de teste").expect("sample file");
        let evidence = import_evidence_with_provenance(
            &fx.services,
            &file,
            1024 * 1024,
            &provenance::EvidenceProvenance::default(),
        )
        .await
        .expect("import");

        let error = analyze(&fx.services, &evidence.id, "TIPO_INEXISTENTE", None, None)
            .await
            .expect_err("an unknown analysis kind must be refused");
        assert!(
            matches!(&error, AppError::Config(message) if message.contains("unsupported analysis_kind")),
            "expected the analysis-kind validation error, got: {error:?}"
        );
    }

    #[tokio::test]
    async fn timeline_pages_cover_every_event_in_utc_order() {
        // ADR-0021: 100 mil eventos, escritos de trás para frente, voltam todos,
        // em ordem cronológica, por páginas — sem carregar tudo na memória.
        let fx = fixture().await;
        let count: u64 = 100_000;
        let job_id = seed_timeline_job(&fx, "ev_pg", count).await;

        let first = timeline_page_core(&fx.pool, &fx.paths, &job_id, 0, 5, None, true)
            .await
            .expect("first page");
        assert_eq!(first.total, Some(count));
        let stamps: Vec<&str> = first.events.iter().map(|e| e.timestamp.as_str()).collect();
        assert_eq!(stamps[0], "2026-01-01T00:00:00+00:00");
        assert!(stamps.windows(2).all(|pair| pair[0] < pair[1]));

        let mut seen: u64 = 0;
        let mut previous: Option<chrono::DateTime<Utc>> = None;
        let mut offset = 0;
        while offset < count {
            let page = timeline_page_core(&fx.pool, &fx.paths, &job_id, offset, 5_000, None, false)
                .await
                .expect("page");
            assert_eq!(page.total, None, "só paga o COUNT quem pede");
            for event in &page.events {
                if let Some(prev) = previous {
                    assert!(prev <= event.sort_timestamp, "ordem cronológica global");
                }
                previous = Some(event.sort_timestamp);
            }
            seen += page.events.len() as u64;
            offset += 5_000;
        }
        assert_eq!(seen, count, "nenhum evento pode faltar");

        let beyond = timeline_page_core(&fx.pool, &fx.paths, &job_id, count + 10, 50, None, true)
            .await
            .expect("beyond the end");
        assert!(beyond.events.is_empty());
        let tail = timeline_page_core(&fx.pool, &fx.paths, &job_id, count - 2, 50, None, false)
            .await
            .expect("tail");
        assert_eq!(tail.events.len(), 2);
    }

    #[tokio::test]
    async fn timeline_filter_is_case_insensitive_and_treats_wildcards_literally() {
        use tokio::io::AsyncWriteExt;
        let fx = fixture().await;
        seed_evidence(&fx, "ev_fl", true).await;
        let job_id = "job_ev_fl".to_string();
        let reference = format!("staging/analysis/{job_id}/output/results.jsonl");
        let path = fx.paths.logical_path(&reference).expect("path");
        tokio::fs::create_dir_all(path.parent().expect("parent"))
            .await
            .expect("dir");
        let item = |record: u64, stamp: &str, event_id: u64, channel: &str| {
            dfir_protocol::NormalizedEvent::new(
                record,
                stamp,
                serde_json::json!({"Event": {"System": {"EventID": event_id, "Channel": channel,
                    "Provider": {"#attributes": {"Name": "Contoso_100%"}}}}}),
                "ev_fl",
            )
            .into_item()
            .expect("item")
        };
        let items = vec![
            item(1, "2026-01-01T00:00:01Z", 4624, "Security"),
            item(2, "2026-01-01T00:00:02Z", 4625, "Security"),
            item(3, "2026-01-01T00:00:03Z", 7045, "System"),
        ];
        let line = serde_json::json!({"message_type": "RESULT", "items": items});
        let mut file = tokio::fs::File::create(&path).await.expect("create");
        file.write_all(format!("{line}\n").as_bytes())
            .await
            .expect("write");
        file.flush().await.expect("flush");
        sqlx::query("UPDATE analysis_jobs SET output_reference=? WHERE id=?")
            .bind(&reference)
            .bind(&job_id)
            .execute(&fx.pool)
            .await
            .expect("reference");

        let count = |filter: &'static str| {
            let (pool, paths, job) = (fx.pool.clone(), fx.paths.clone(), job_id.clone());
            async move {
                timeline_page_core(&pool, &paths, &job, 0, 50, Some(filter), true)
                    .await
                    .expect("page")
                    .total
            }
        };
        assert_eq!(count("4624").await, Some(1), "título");
        assert_eq!(
            count("SECURITY").await,
            Some(2),
            "canal, sem diferenciar maiúsculas"
        );
        assert_eq!(
            count("contoso_100%").await,
            Some(3),
            "provedor com % e _ literais"
        );
        assert_eq!(
            count("%").await,
            Some(3),
            "% casa o provedor, não é curinga"
        );
        assert_eq!(count("zzz").await, Some(0));
        assert_eq!(
            count("  ").await,
            Some(3),
            "filtro só de espaços = sem filtro"
        );
        // `_` não pode funcionar como "qualquer caractere": "4_24" não casa "4624".
        assert_eq!(count("4_24").await, Some(0));
    }

    /// Semeia um job EVTX concluído com os itens dados (uma linha de resultado).
    async fn seed_items_job(fx: &Fixture, id: &str, items: Vec<serde_json::Value>) -> String {
        use tokio::io::AsyncWriteExt;
        seed_evidence(fx, id, true).await;
        let job_id = format!("job_{id}");
        let reference = format!("staging/analysis/{job_id}/output/results.jsonl");
        let path = fx.paths.logical_path(&reference).expect("path");
        tokio::fs::create_dir_all(path.parent().expect("parent"))
            .await
            .expect("dir");
        let line = serde_json::json!({"message_type": "RESULT", "items": items});
        let mut file = tokio::fs::File::create(&path).await.expect("create");
        file.write_all(format!("{line}\n").as_bytes())
            .await
            .expect("write");
        file.flush().await.expect("flush");
        sqlx::query("UPDATE analysis_jobs SET output_reference=? WHERE id=?")
            .bind(&reference)
            .bind(&job_id)
            .execute(&fx.pool)
            .await
            .expect("reference");
        job_id
    }

    fn security_record(id: u64, data: serde_json::Value) -> serde_json::Value {
        serde_json::json!({"Event": {
            "System": {
                "Provider": {"#attributes": {"Name": "Microsoft-Windows-Security-Auditing"}},
                "EventID": id, "Channel": "Security", "Computer": "DC01"
            },
            "EventData": data
        }})
    }

    #[tokio::test]
    async fn the_structured_filter_finds_events_by_field_and_backfills_old_results() {
        let fx = fixture().await;
        // Item novo (v4): o engine já trouxe os campos estruturais.
        let new_data = security_record(
            4625,
            serde_json::json!({"TargetUserName": "fsir", "IpAddress": "10.0.0.5"}),
        );
        let new_item = dfir_protocol::NormalizedEvent::new(
            1,
            "2026-01-01T00:00:01Z",
            new_data.clone(),
            "ev_sf",
        )
        .with_fields(dfir_protocol::normalize::event_fields(&new_data))
        .into_item()
        .expect("item");
        // Item antigo (v3): sem campo estrutural; o app extrai do registro.
        let old_data = security_record(
            4624,
            serde_json::json!({"TargetUserName": "maria", "IpAddress": "10.0.0.9", "LogonType": 3}),
        );
        let old_item =
            dfir_protocol::NormalizedEvent::new(2, "2026-01-02T00:00:00Z", old_data, "ev_sf")
                .into_item()
                .expect("item");
        assert!(
            old_item.get("user").is_none(),
            "o item antigo não tem campo novo"
        );
        let other = dfir_protocol::NormalizedEvent::new(
            3,
            "2026-01-03T00:00:00Z",
            security_record(4672, serde_json::json!({"SubjectUserName": "fsir"})),
            "ev_sf",
        )
        .into_item()
        .expect("item");
        let job_id = seed_items_job(&fx, "ev_sf", vec![new_item, old_item, other]).await;

        let count = |filter: &'static str| {
            let (pool, paths, job) = (fx.pool.clone(), fx.paths.clone(), job_id.clone());
            async move {
                timeline_page_core(&pool, &paths, &job, 0, 50, Some(filter), true)
                    .await
                    .map(|page| page.total)
            }
        };
        assert_eq!(count("event:4625").await.unwrap(), Some(1));
        assert_eq!(
            count("event:4624,4625").await.unwrap(),
            Some(2),
            "vírgula = OU"
        );
        assert_eq!(count("user:fsir").await.unwrap(), Some(2), "4625 e 4672");
        assert_eq!(
            count("user:MARIA").await.unwrap(),
            Some(1),
            "campo do item antigo, sem diferenciar maiúsculas"
        );
        assert_eq!(count("ip:10.0.0.5").await.unwrap(), Some(1), "IP exato");
        assert_eq!(
            count("ip:10.0.0").await.unwrap(),
            Some(0),
            "IP não é busca por trecho"
        );
        assert_eq!(count("outcome:failure").await.unwrap(), Some(1));
        assert_eq!(count("outcome:success").await.unwrap(), Some(1));
        assert_eq!(count("host:dc01").await.unwrap(), Some(3));
        assert_eq!(
            count("action:logon").await.unwrap(),
            Some(2),
            "logon e logon_failed"
        );
        assert_eq!(
            count("user:fsir event:4672").await.unwrap(),
            Some(1),
            "termos diferentes = E"
        );
        assert_eq!(
            count("user:fsir Security").await.unwrap(),
            Some(2),
            "estruturado + texto livre"
        );
        assert_eq!(count("from:2026-01-02").await.unwrap(), Some(2));
        assert_eq!(
            count("from:2026-01-02 to:2026-01-02").await.unwrap(),
            Some(1),
            "to de dia inclui o dia inteiro"
        );
        assert_eq!(count("to:2026-01-01").await.unwrap(), Some(1));
        let error = count("from:ontem").await.expect_err("data inválida é erro");
        assert!(matches!(error, AppError::Config(message) if message.contains("invalid date")));
    }

    #[tokio::test]
    async fn timeline_text_from_logs_is_neutralized_for_display_but_stays_intact_in_the_result() {
        let fx = fixture().await;
        let hostile = "relatorio\u{202E}txt.exe";
        let data = security_record(
            4688,
            serde_json::json!({"SubjectUserName": hostile, "NewProcessName": hostile, "CommandLine": hostile}),
        );
        let item = dfir_protocol::NormalizedEvent::new(1, "2026-01-01T00:00:01Z", data, "ev_bd")
            .into_item()
            .expect("item");
        let job_id = seed_items_job(&fx, "ev_bd", vec![item]).await;
        let page = timeline_page_core(&fx.pool, &fx.paths, &job_id, 0, 10, None, true)
            .await
            .expect("page");
        let event = &page.events[0];
        for shown in [
            event.description.as_str(),
            event.user.as_deref().unwrap_or(""),
            event.process.as_deref().unwrap_or(""),
        ] {
            assert!(!shown.contains('\u{202E}'), "{shown}");
            assert!(shown.contains("⟦U+202E⟧"), "{shown}");
        }
        // O que foi gravado no banco continua sendo o dado original.
        let stored: String =
            sqlx::query_scalar("SELECT user_name FROM timeline_events WHERE job_id=?")
                .bind(&job_id)
                .fetch_one(&fx.pool)
                .await
                .expect("stored");
        assert!(stored.contains('\u{202E}'));
    }

    #[tokio::test]
    async fn timeline_page_exposes_the_job_warnings() {
        let fx = fixture().await;
        let job_id = seed_timeline_job(&fx, "ev_w", 3).await;
        sqlx::query("UPDATE analysis_jobs SET warning_count=1, warnings=? WHERE id=?")
            .bind(r#"[{"code":"EVTX_TRUNCATED","count":1,"sample":"x"}]"#)
            .bind(&job_id)
            .execute(&fx.pool)
            .await
            .expect("warnings");

        let page = timeline_page_core(&fx.pool, &fx.paths, &job_id, 0, 10, None, true)
            .await
            .expect("page");

        assert_eq!(page.job.warning_count, 1);
        assert_eq!(page.job.warnings[0].code, "EVTX_TRUNCATED");
    }

    #[tokio::test]
    async fn timeline_page_rejects_malformed_stored_job_warnings() {
        let fx = fixture().await;
        let job_id = seed_timeline_job(&fx, "ev_w_bad", 1).await;
        sqlx::query("UPDATE analysis_jobs SET warning_count=1, warnings='not-json' WHERE id=?")
            .bind(&job_id)
            .execute(&fx.pool)
            .await
            .expect("malformed warnings fixture");

        let error = timeline_page_core(&fx.pool, &fx.paths, &job_id, 0, 10, None, true)
            .await
            .expect_err("JSON inválido não pode apagar os avisos da resposta");
        assert!(
            matches!(&error, AppError::Config(message) if message.contains("stored analysis job warnings are malformed JSON")),
            "{error:?}"
        );
    }

    #[tokio::test]
    async fn timeline_page_rejects_inconsistent_stored_job_warning_count() {
        let fx = fixture().await;
        for (index, warning_count) in [1_i64, -1].into_iter().enumerate() {
            let job_id = seed_timeline_job(&fx, &format!("ev_w_count_{index}"), 1).await;
            sqlx::query("UPDATE analysis_jobs SET warning_count=?, warnings='[]' WHERE id=?")
                .bind(warning_count)
                .bind(&job_id)
                .execute(&fx.pool)
                .await
                .expect("warning count fixture");

            let error = timeline_page_core(&fx.pool, &fx.paths, &job_id, 0, 10, None, true)
                .await
                .expect_err("contagem inválida não pode ser escondida");
            let expected = if warning_count < 0 {
                "warning_count is negative"
            } else {
                "warning_count does not match"
            };
            assert!(
                matches!(&error, AppError::Config(message) if message.contains(expected)),
                "count={warning_count}: {error:?}"
            );
        }
    }

    #[tokio::test]
    async fn destroying_the_evidence_removes_its_timeline_index() {
        let fx = fixture().await;
        let job_id = seed_timeline_job(&fx, "ev_di", 50).await;
        timeline_page_core(&fx.pool, &fx.paths, &job_id, 0, 10, None, true)
            .await
            .expect("indexes lazily");
        let rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM timeline_events")
            .fetch_one(&fx.pool)
            .await
            .expect("count");
        assert_eq!(rows, 50);

        destroy_evidence_core(&fx.pool, &fx.paths, "ev_di")
            .await
            .expect("destroy");

        let rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM timeline_events")
            .fetch_one(&fx.pool)
            .await
            .expect("count");
        let marks: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM timeline_index")
            .fetch_one(&fx.pool)
            .await
            .expect("count");
        assert_eq!(
            (rows, marks),
            (0, 0),
            "dado da evidência não pode sobrar no banco"
        );
    }

    /// Medição manual (`cargo test -- --ignored --nocapture`): 1 milhão de
    /// eventos indexados e paginados.
    #[tokio::test]
    #[ignore = "medição de desempenho; rode manualmente"]
    async fn timeline_one_million_events_index_and_paging_timing() {
        let fx = fixture().await;
        let started = std::time::Instant::now();
        let job_id = seed_timeline_job(&fx, "ev_1m", 1_000_000).await;
        println!("gerar results.jsonl: {:?}", started.elapsed());
        let started = std::time::Instant::now();
        let first = timeline_page_core(&fx.pool, &fx.paths, &job_id, 0, 200, None, true)
            .await
            .expect("index + first page");
        println!(
            "indexar 1M + 1ª página: {:?} (total {:?})",
            started.elapsed(),
            first.total
        );
        for offset in [0_u64, 500_000, 999_800] {
            let started = std::time::Instant::now();
            let page = timeline_page_core(&fx.pool, &fx.paths, &job_id, offset, 200, None, false)
                .await
                .expect("page");
            println!(
                "página @{offset}: {:?} ({} eventos)",
                started.elapsed(),
                page.events.len()
            );
        }
        let started = std::time::Instant::now();
        let filtered = timeline_page_core(&fx.pool, &fx.paths, &job_id, 0, 200, Some("4624"), true)
            .await
            .expect("filtered");
        println!(
            "filtro + COUNT: {:?} (total {:?})",
            started.elapsed(),
            filtered.total
        );
    }

    #[tokio::test]
    async fn the_diagnostics_report_has_counts_and_codes_but_no_filenames_or_messages() {
        let fx = fixture().await;
        seed_evidence(&fx, "ev_dg", true).await; // arquivo original: sample.evtx
        sqlx::query(
            "UPDATE analysis_jobs SET status='FAILED', error_code='TIMEOUT', error_message=? WHERE evidence_id='ev_dg'",
        )
        .bind("engine failed reading /home/analista/segredo/sample.evtx")
        .execute(&fx.pool)
        .await
        .expect("fail the job");
        tokio::fs::write(fx.paths.logs.join("dfir.log"), b"{}\n")
            .await
            .expect("log file");

        let report = diagnostics_report(&fx.pool, &fx.paths, serde_json::json!({"k": 1}))
            .await
            .expect("report");

        assert_eq!(report["database"]["evidence_active"], 1);
        assert_eq!(report["database"]["jobs_by_status"]["FAILED"], 1);
        assert_eq!(
            report["database"]["recent_failed_jobs"][0]["error_code"],
            "TIMEOUT"
        );
        assert!(
            report["database"]["migrations_applied"]
                .as_i64()
                .unwrap_or(0)
                >= 9
        );
        assert_eq!(report["logs"]["files"][0]["name"], "dfir.log");
        assert_eq!(report["settings"]["k"], 1);
        let text = report.to_string();
        assert!(
            !text.contains("sample.evtx"),
            "o nome do arquivo não pode vazar"
        );
        assert!(
            !text.contains("segredo"),
            "mensagens de erro não entram no relatório"
        );
    }

    #[tokio::test]
    async fn default_limits_fit_in_the_audit_chain_metadata() {
        // Regressão: com o teto em i64::MAX, a canonicalização RFC 8785 da
        // cadeia de auditoria rejeitava o pedido e TODA análise falhava.
        let fx = fixture().await;
        let metadata = serde_json::json!({
            "max_output_bytes": dfir_protocol::MAX_OUTPUT_BYTES,
            "max_items": dfir_protocol::MAX_ITEMS,
        });
        audit::append(
            &fx.pool,
            None,
            "ANALYSIS_STARTED",
            "evidence",
            "ev_limits",
            None,
            &metadata.to_string(),
        )
        .await
        .expect("os limites padrão precisam caber na auditoria");
    }

    #[tokio::test]
    async fn orphaned_jobs_are_recovered_and_unblock_destruction() {
        // AUD-03: job preso em PROCESSING impedia destroy_evidence para sempre.
        let fx = fixture().await;
        let file = seed_evidence(&fx, "ev_o", true).await;
        sqlx::query("UPDATE analysis_jobs SET status='PROCESSING' WHERE evidence_id='ev_o'")
            .execute(&fx.pool)
            .await
            .expect("simulate crash");
        assert!(
            destroy_evidence_core(&fx.pool, &fx.paths, "ev_o")
                .await
                .is_err()
        );

        let recovered = recover_orphaned_jobs(&fx.pool).await.expect("recovery");

        assert_eq!(recovered, 1);
        let (status, code): (String, Option<String>) =
            sqlx::query_as("SELECT status,error_code FROM analysis_jobs WHERE evidence_id='ev_o'")
                .fetch_one(&fx.pool)
                .await
                .expect("job row");
        assert_eq!(status, "FAILED");
        assert_eq!(code.as_deref(), Some("ORPHANED"));
        assert_eq!(audit_count(&fx, "ANALYSIS_JOB_ORPHANED").await, 1);
        destroy_evidence_core(&fx.pool, &fx.paths, "ev_o")
            .await
            .expect("agora o descarte é permitido");
        assert!(!file.exists());
    }

    #[tokio::test]
    async fn recovery_leaves_finished_jobs_alone_and_is_idempotent() {
        let fx = fixture().await;
        seed_evidence(&fx, "ev_p", true).await; // job COMPLETED
        assert_eq!(recover_orphaned_jobs(&fx.pool).await.expect("recovery"), 0);
        let status: String =
            sqlx::query_scalar("SELECT status FROM analysis_jobs WHERE evidence_id='ev_p'")
                .fetch_one(&fx.pool)
                .await
                .expect("status");
        assert_eq!(status, "COMPLETED");
        assert_eq!(audit_count(&fx, "ANALYSIS_JOB_ORPHANED").await, 0);
    }

    #[tokio::test]
    async fn integrity_failure_is_recorded_in_audit_and_custody() {
        let fx = fixture().await;
        seed_evidence(&fx, "ev_i", false).await;

        let error = integrity_failure_error(
            &fx.pool,
            "ev_i",
            "evidence digest changed after import",
            "{}",
            "evidence digest mismatch; analysis refused",
        )
        .await;

        assert_eq!(
            error.user_message(),
            "evidence digest mismatch; analysis refused"
        );
        assert_eq!(audit_count(&fx, "EVIDENCE_INTEGRITY_FAILURE").await, 1);
        let custody: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM chain_of_custody WHERE evidence_id='ev_i' AND action='INTEGRITY_FAILURE'",
        )
        .fetch_one(&fx.pool)
        .await
        .expect("custody count");
        assert_eq!(custody, 1);
    }

    #[tokio::test]
    async fn integrity_failure_that_cannot_be_recorded_says_so() {
        // Regressão de AUD-25: antes, `let _ =` escondia a falha de auditoria.
        let fx = fixture().await;
        seed_evidence(&fx, "ev_j", false).await;
        fx.pool.close().await;

        let error = integrity_failure_error(
            &fx.pool,
            "ev_j",
            "evidence size changed after import",
            "{}",
            "evidence changed after import; analysis refused",
        )
        .await;

        let message = error.user_message();
        assert!(
            message.contains("analysis refused"),
            "keeps the original message"
        );
        assert!(
            message.contains("could NOT be recorded"),
            "must say the audit write failed, got: {message}"
        );
    }

    #[tokio::test]
    async fn destroy_removes_file_jobs_and_records_destruction() {
        let fx = fixture().await;
        let file = seed_evidence(&fx, "ev_a", true).await;

        destroy_evidence_core(&fx.pool, &fx.paths, "ev_a")
            .await
            .expect("destroy succeeds");

        assert!(!file.exists(), "evidence file must be gone");
        assert!(
            !fx.paths.staging.join("analysis").join("job_ev_a").exists(),
            "job staging dir must be gone"
        );
        assert_eq!(status_of(&fx, "ev_a").await, "DESTROYED");
        assert_eq!(audit_count(&fx, "EVIDENCE_DESTROYED").await, 1);
        let jobs: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM analysis_jobs WHERE evidence_id='ev_a'")
                .fetch_one(&fx.pool)
                .await
                .expect("jobs");
        assert_eq!(jobs, 0);
    }

    #[tokio::test]
    async fn destroy_never_leaves_active_record_after_file_removal_failure() {
        // Regressão de AUD-02: a falha de remoção física acontece DEPOIS do
        // registro. Simulada com um diretório no lugar do arquivo, onde
        // `remove_file` falha.
        let fx = fixture().await;
        let file = seed_evidence(&fx, "ev_b", false).await;
        tokio::fs::remove_file(&file).await.expect("remove file");
        tokio::fs::create_dir(&file)
            .await
            .expect("dir in place of file");

        let result = destroy_evidence_core(&fx.pool, &fx.paths, "ev_b").await;

        assert!(result.is_err(), "removal failure must be reported");
        assert_eq!(
            status_of(&fx, "ev_b").await,
            "DESTROYED",
            "the destruction must already be recorded"
        );
        assert_eq!(audit_count(&fx, "EVIDENCE_DESTROYED").await, 1);
    }

    #[tokio::test]
    async fn destroy_can_be_retried_to_finish_the_purge() {
        let fx = fixture().await;
        let file = seed_evidence(&fx, "ev_c", true).await;
        // Primeira tentativa falha na remoção física (diretório no lugar do arquivo).
        tokio::fs::remove_file(&file).await.expect("remove file");
        tokio::fs::create_dir(&file)
            .await
            .expect("dir in place of file");
        assert!(
            destroy_evidence_core(&fx.pool, &fx.paths, "ev_c")
                .await
                .is_err()
        );

        // O operador corrige o problema; a segunda chamada conclui a limpeza
        // sem registrar uma segunda destruição.
        tokio::fs::remove_dir(&file).await.expect("clear obstacle");
        tokio::fs::write(&file, b"leftover")
            .await
            .expect("leftover file");
        destroy_evidence_core(&fx.pool, &fx.paths, "ev_c")
            .await
            .expect("retry completes the purge");

        assert!(!file.exists());
        assert_eq!(audit_count(&fx, "EVIDENCE_DESTROYED").await, 1);

        // Sem restos, uma terceira chamada é recusada como antes.
        let third = destroy_evidence_core(&fx.pool, &fx.paths, "ev_c").await;
        assert!(matches!(third, Err(AppError::NotFound(_))));
    }

    #[tokio::test]
    async fn destroy_refuses_while_a_job_is_processing() {
        let fx = fixture().await;
        let file = seed_evidence(&fx, "ev_d", true).await;
        sqlx::query("UPDATE analysis_jobs SET status='PROCESSING' WHERE evidence_id='ev_d'")
            .execute(&fx.pool)
            .await
            .expect("mark processing");

        let result = destroy_evidence_core(&fx.pool, &fx.paths, "ev_d").await;

        assert!(result.is_err());
        assert!(file.exists(), "nothing may be removed");
        assert_eq!(status_of(&fx, "ev_d").await, "ACTIVE");
    }

    #[tokio::test]
    async fn destroy_records_destruction_before_touching_the_file() {
        // F1-21: ordem segura — a transação (status DESTROYED + auditoria +
        // custódia) faz COMMIT antes de qualquer remoção física. Se o banco
        // falha, nada é apagado.
        let fx = fixture().await;
        let file = seed_evidence(&fx, "ev_e", false).await;

        // Fecha o pool para forçar falha de banco na transação.
        fx.pool.close().await;
        let result = destroy_evidence_core(&fx.pool, &fx.paths, "ev_e").await;

        assert!(result.is_err(), "database failure must be reported");
        assert!(
            file.exists(),
            "file must NOT be removed when the recording transaction fails"
        );
    }

    #[tokio::test]
    async fn destroy_removes_detection_candidates_with_the_evidence() {
        // F1-21: ordem completa — arquivo físico, diretórios de job, índice
        // da timeline, candidatos de detecção e linhas de analysis_jobs saem
        // junto; custódia e auditoria ficam.
        let fx = fixture().await;
        seed_evidence(&fx, "ev_f", true).await;
        sqlx::query(
            "INSERT INTO detection_candidates(id,evidence_id,job_id,candidate_key,rule_id,rule_version,rule_set_version,title,description,tactic,technique_id,severity,confidence,aggregation,event_count,first_seen,last_seen,event_refs,iocs,limitations,status,created_at) VALUES('cand_1','ev_f','job_ev_f','key_1','rule_x','1','1','t','d','TA0001','T1000','HIGH','HIGH','none',1,'2026-10-02T00:00:00Z','2026-10-02T00:00:00Z','[]','[]','none','PENDING','2026-10-02T00:00:00Z')",
        )
        .execute(&fx.pool)
        .await
        .expect("seed candidate");

        destroy_evidence_core(&fx.pool, &fx.paths, "ev_f")
            .await
            .expect("destroy succeeds");

        let candidates: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM detection_candidates WHERE job_id='job_ev_f'")
                .fetch_one(&fx.pool)
                .await
                .expect("count");
        assert_eq!(candidates, 0, "detection candidates must be purged");
        // Custódia e auditoria sobrevivem à destruição (RF-004).
        let custody: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM chain_of_custody WHERE evidence_id='ev_f'")
                .fetch_one(&fx.pool)
                .await
                .expect("custody count");
        assert!(custody >= 1, "custody entries must survive destruction");
        assert_eq!(audit_count(&fx, "EVIDENCE_DESTROYED").await, 1);
    }

    #[tokio::test]
    async fn linking_evidence_to_incident_records_the_link_and_audit() {
        let fx = crate::service::test_support::fixture().await;
        let dir = tempfile::tempdir().expect("temporary directory");
        let file = dir.path().join("amostra.evtx");
        std::fs::write(&file, b"conteudo de teste").expect("sample file");
        let evidence = import_evidence_with_provenance(
            &fx.services,
            &file,
            1024 * 1024,
            &provenance::EvidenceProvenance::default(),
        )
        .await
        .expect("import");
        let services = &fx.services;
        let incident = create_incident(services, "incidente teste")
            .await
            .expect("incident");

        let linked = link_evidence_to_incident(services, &evidence.id, &incident.id)
            .await
            .expect("link");
        assert_eq!(linked.incident_id.as_deref(), Some(incident.id.as_str()));

        // Idempotente: repetir o mesmo vínculo não gera auditoria nova.
        let again = link_evidence_to_incident(services, &evidence.id, &incident.id)
            .await
            .expect("link again");
        assert_eq!(again.incident_id.as_deref(), Some(incident.id.as_str()));
        let audit_links: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM audit_events WHERE action='EVIDENCE_LINKED_TO_INCIDENT'",
        )
        .fetch_one(&fx.services.db)
        .await
        .expect("audit count");
        assert_eq!(audit_links, 1);

        // A listagem por incidente traz a evidência; a do catálogo também.
        let by_incident = evidence_for_incident(services, &incident.id)
            .await
            .expect("by incident");
        assert_eq!(by_incident.len(), 1);
        assert_eq!(by_incident[0].id, evidence.id);
        let listed = list_evidence(services).await.expect("list");
        assert_eq!(
            listed
                .iter()
                .find(|item| item.id == evidence.id)
                .and_then(|item| item.incident_id.clone()),
            Some(incident.id.clone())
        );

        // Incidente inexistente é recusado; evidência inexistente também.
        assert!(
            link_evidence_to_incident(services, &evidence.id, "inc_nao_existe")
                .await
                .is_err()
        );
        assert!(
            link_evidence_to_incident(services, "ev_nao_existe", &incident.id)
                .await
                .is_err()
        );
        assert!(
            evidence_for_incident(services, "inc_nao_existe")
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn discard_temp_file_removes_existing_and_tolerates_missing() {
        // F1-23: o helper precisa remover o arquivo existente e não falhar
        // (nem logar warn) quando o caminho já não existe.
        let root = tempfile::tempdir().expect("temp dir");
        let file = root.path().join("artifact.tmp");
        tokio::fs::write(&file, b"x").await.expect("write");
        super::discard_temp_file(&file).await;
        assert!(!file.exists());
        // Segunda chamada: NotFound é esperado e não é falha.
        super::discard_temp_file(&file).await;
        assert!(!file.exists());
    }

    #[tokio::test]
    async fn discard_temp_dir_removes_existing_and_tolerates_missing() {
        let root = tempfile::tempdir().expect("temp dir");
        let dir = root.path().join("job-dir");
        tokio::fs::create_dir_all(dir.join("nested"))
            .await
            .expect("create dirs");
        super::discard_temp_dir(&dir).await;
        assert!(!dir.exists());
        super::discard_temp_dir(&dir).await;
        assert!(!dir.exists());
    }

    #[tokio::test]
    async fn mark_job_failed_loudly_records_failure_and_tolerates_double_call() {
        // F1-23: a primeira chamada marca o job FAILED; a segunda (erro já
        // registrado) não pode falhar nem deixar o job em estado inconsistente.
        let fx = fixture().await;
        seed_evidence(&fx, "ev_1", false).await;
        sqlx::query(
            "INSERT INTO analysis_jobs(id,request_id,evidence_id,status,max_output_bytes,max_items,created_at) VALUES('job_1','req_1','ev_1','PROCESSING',1024,10,'2026-10-02T00:00:00Z')",
        )
        .execute(&fx.pool)
        .await
        .expect("seed job");
        super::mark_job_failed_loudly(&fx.pool, "job_1", "E_TEST", "boom").await;
        let status: (String,) = sqlx::query_as("SELECT status FROM analysis_jobs WHERE id='job_1'")
            .fetch_one(&fx.pool)
            .await
            .expect("job row");
        assert_eq!(status.0, "FAILED");
        // Segunda marcação no mesmo job: não pode falhar nem mudar o estado.
        super::mark_job_failed_loudly(&fx.pool, "job_1", "E_TEST", "boom again").await;
        let status: (String,) = sqlx::query_as("SELECT status FROM analysis_jobs WHERE id='job_1'")
            .fetch_one(&fx.pool)
            .await
            .expect("job row");
        assert_eq!(status.0, "FAILED");
    }
}
