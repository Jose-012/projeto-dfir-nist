//! F1-31 (RF-054, RB-001, RB-002) — proveniência da evidência.
//!
//! O operador declara na importação de onde a evidência veio, quem a
//! forneceu, como e quando foi coletada e com que autorização. O que ele
//! não informa é gravado como **não informado**; nada é preenchido por
//! suposição. (Antes desta etapa a importação gravava constantes —
//! `LOCAL_OPERATOR`, `LOCAL_FILE_PICKER` — e a data de **importação** no
//! lugar da data de coleta; linhas antigas ainda carregam esses valores, e
//! os métodos `declared_*` os tratam como não informados.)

use super::EvidenceResponse;
use crate::error::{AppError, AppResult};
use crate::persistence::source_gaps::parse_instant;

/// Valor gravado em `source` e `collection_method` quando o operador não
/// informa.
pub const NOT_PROVIDED: &str = "NOT_PROVIDED";
const MAX_FIELD_CHARS: usize = 500;

/// Constantes que linhas anteriores à Etapa 6 gravavam nos campos de origem
/// e método; descrevem a importação, não a origem da evidência.
const LEGACY_SOURCES: [&str; 3] = [NOT_PROVIDED, "LOCAL_OPERATOR", "LOCAL_BROWSER"];
const LEGACY_METHODS: [&str; 3] = [NOT_PROVIDED, "LOCAL_FILE_PICKER", "MANUAL_UPLOAD"];

/// Proveniência como a interface e a CLI a recebem.
#[derive(Debug, Clone, Default)]
pub struct EvidenceProvenance {
    /// Sistema ou área de origem (ex.: "servidor de arquivos FS02", "TI do cliente").
    pub source: Option<String>,
    /// Quem ou qual área forneceu a evidência.
    pub provided_by: Option<String>,
    /// Como foi coletada (ex.: "exportação com wevtutil").
    pub collection_method: Option<String>,
    pub collection_tool: Option<String>,
    pub collection_tool_version: Option<String>,
    /// Data (`AAAA-MM-DD`) ou instante RFC 3339 da coleta.
    pub collected_at: Option<String>,
    /// Referência da autorização para coletar (ex.: ofício, ticket).
    pub authorization_reference: Option<String>,
    /// A evidência contém dado pessoal (F1-32).
    pub contains_personal_data: bool,
}

/// Valores já validados e normalizados, prontos para gravar.
#[derive(Debug, Clone, Default)]
pub(super) struct CleanProvenance {
    pub source: Option<String>,
    pub provided_by: Option<String>,
    pub collection_method: Option<String>,
    pub collection_tool: Option<String>,
    pub collection_tool_version: Option<String>,
    pub collected_at: Option<String>,
    pub authorization_reference: Option<String>,
    pub contains_personal_data: bool,
}

impl CleanProvenance {
    pub(super) fn source_or_default(&self) -> &str {
        self.source.as_deref().unwrap_or(NOT_PROVIDED)
    }

    pub(super) fn collection_method_or_default(&self) -> &str {
        self.collection_method.as_deref().unwrap_or(NOT_PROVIDED)
    }
}

/// Campo de uma linha: sem espaços nas pontas; em branco vira ausente;
/// recusa texto longo demais e caracteres de controle.
fn clean_text(value: &Option<String>, what: &str) -> AppResult<Option<String>> {
    let Some(raw) = value else {
        return Ok(None);
    };
    let text = raw.trim();
    if text.is_empty() {
        return Ok(None);
    }
    if text.chars().count() > MAX_FIELD_CHARS {
        return Err(AppError::Config(format!(
            "{what} is longer than {MAX_FIELD_CHARS} characters"
        )));
    }
    if text.chars().any(char::is_control) {
        return Err(AppError::Config(format!(
            "{what} must be a single line without control characters"
        )));
    }
    Ok(Some(text.to_string()))
}

impl EvidenceProvenance {
    pub(super) fn clean(&self) -> AppResult<CleanProvenance> {
        let collected_at = clean_text(&self.collected_at, "collected_at")?;
        if let Some(value) = &collected_at
            && parse_instant(value).is_none()
        {
            return Err(AppError::Config(format!(
                "collected_at must be a date (YYYY-MM-DD) or an RFC 3339 instant: {value}"
            )));
        }
        Ok(CleanProvenance {
            source: clean_text(&self.source, "source")?,
            provided_by: clean_text(&self.provided_by, "provided_by")?,
            collection_method: clean_text(&self.collection_method, "collection_method")?,
            collection_tool: clean_text(&self.collection_tool, "collection_tool")?,
            collection_tool_version: clean_text(
                &self.collection_tool_version,
                "collection_tool_version",
            )?,
            collected_at,
            authorization_reference: clean_text(
                &self.authorization_reference,
                "authorization_reference",
            )?,
            contains_personal_data: self.contains_personal_data,
        })
    }
}

impl EvidenceResponse {
    /// Só EVTX é analisado automaticamente; o resto é preservado (hash,
    /// custódia, descarte), mas nenhum conteúdo é interpretado.
    pub fn is_analyzed_automatically(&self) -> bool {
        self.format.eq_ignore_ascii_case("evtx")
    }

    /// Origem declarada, ou `None` quando não foi informada (inclusive as
    /// constantes de importação de linhas antigas).
    pub fn declared_source(&self) -> Option<&str> {
        let value = self.source.trim();
        (!value.is_empty() && !LEGACY_SOURCES.contains(&value)).then_some(value)
    }

    pub fn declared_collection_method(&self) -> Option<&str> {
        let value = self.collection_method.trim();
        (!value.is_empty() && !LEGACY_METHODS.contains(&value)).then_some(value)
    }

    /// Data de coleta declarada. Linhas antigas gravavam aqui a data de
    /// importação; igual a ela, é tratada como não informada.
    pub fn declared_collected_at(&self) -> Option<&str> {
        self.collected_at
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty() && *value != self.imported_at)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::test_support::fixture;
    use crate::service::{import_evidence_with_provenance, list_evidence};

    async fn sample_file(dir: &std::path::Path, name: &str) -> std::path::PathBuf {
        let path = dir.join(name);
        tokio::fs::write(&path, b"conteudo da evidencia")
            .await
            .expect("write sample");
        path
    }

    fn full() -> EvidenceProvenance {
        EvidenceProvenance {
            source: Some("  servidor de arquivos FS02 ".into()),
            provided_by: Some("TI do cliente".into()),
            collection_method: Some("exportação com wevtutil".into()),
            collection_tool: Some("wevtutil".into()),
            collection_tool_version: Some("10.0.22621".into()),
            collected_at: Some("2026-09-30".into()),
            authorization_reference: Some("Ofício 123/2026".into()),
            contains_personal_data: true,
        }
    }

    #[tokio::test]
    async fn import_without_provenance_records_not_provided_and_no_invented_collection_date() {
        let fx = fixture().await;
        let dir = tempfile::tempdir().expect("temporary directory");
        let path = sample_file(dir.path(), "Security.evtx").await;
        let evidence = import_evidence_with_provenance(
            &fx.services,
            &path,
            1024 * 1024,
            &EvidenceProvenance::default(),
        )
        .await
        .expect("import");

        assert_eq!(evidence.source, NOT_PROVIDED);
        assert_eq!(evidence.collection_method, NOT_PROVIDED);
        assert_eq!(
            evidence.collected_at, None,
            "a data de importação não é a de coleta"
        );
        assert_eq!(evidence.provided_by, None);
        assert!(!evidence.contains_personal_data);
        assert!(evidence.declared_source().is_none());
        assert!(evidence.declared_collection_method().is_none());
        assert!(evidence.declared_collected_at().is_none());

        // O que foi gravado é o que a listagem devolve.
        let listed = list_evidence(&fx.services).await.expect("list");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].source, NOT_PROVIDED);
        assert_eq!(listed[0].collected_at, None);
    }

    #[tokio::test]
    async fn import_with_provenance_stores_and_returns_what_was_declared() {
        let fx = fixture().await;
        let dir = tempfile::tempdir().expect("temporary directory");
        let path = sample_file(dir.path(), "Security.evtx").await;
        let evidence = import_evidence_with_provenance(&fx.services, &path, 1024 * 1024, &full())
            .await
            .expect("import");
        assert_eq!(evidence.source, "servidor de arquivos FS02");
        assert_eq!(evidence.provided_by.as_deref(), Some("TI do cliente"));
        assert_eq!(evidence.collected_at.as_deref(), Some("2026-09-30"));
        assert!(evidence.contains_personal_data);
        assert_eq!(
            evidence.declared_source(),
            Some("servidor de arquivos FS02")
        );
        assert_eq!(evidence.declared_collected_at(), Some("2026-09-30"));

        let listed = list_evidence(&fx.services).await.expect("list");
        let stored = &listed[0];
        assert_eq!(stored.collection_tool.as_deref(), Some("wevtutil"));
        assert_eq!(
            stored.collection_tool_version.as_deref(),
            Some("10.0.22621")
        );
        assert_eq!(
            stored.authorization_reference.as_deref(),
            Some("Ofício 123/2026")
        );
        assert!(stored.contains_personal_data);

        // A proveniência vai para a trilha de auditoria da importação.
        let metadata: String = sqlx::query_scalar(
            "SELECT metadata FROM audit_events WHERE object_id = ? ORDER BY sequence LIMIT 1",
        )
        .bind(&evidence.id)
        .fetch_one(&fx.services.db)
        .await
        .expect("audit metadata");
        assert!(metadata.contains("TI do cliente"));
        assert!(metadata.contains("\"import_method\":\"LOCAL_FILE_PICKER\""));
    }

    #[tokio::test]
    async fn invalid_provenance_is_refused_before_anything_is_copied() {
        let fx = fixture().await;
        let dir = tempfile::tempdir().expect("temporary directory");
        let path = sample_file(dir.path(), "Security.evtx").await;
        for bad in [
            EvidenceProvenance {
                collected_at: Some("ontem".into()),
                ..EvidenceProvenance::default()
            },
            EvidenceProvenance {
                source: Some("linha 1\nlinha 2".into()),
                ..EvidenceProvenance::default()
            },
            EvidenceProvenance {
                provided_by: Some("x".repeat(501)),
                ..EvidenceProvenance::default()
            },
        ] {
            assert!(
                import_evidence_with_provenance(&fx.services, &path, 1024 * 1024, &bad)
                    .await
                    .is_err()
            );
        }
        assert!(list_evidence(&fx.services).await.expect("list").is_empty());
        let preserved = std::fs::read_dir(&fx.services.paths.evidence)
            .expect("evidence dir")
            .count();
        assert_eq!(preserved, 0, "nada foi copiado para o armazenamento");
    }

    /// RF-003 (preservação): a importação lê o arquivo de origem e copia para
    /// o storage controlado; o original jamais é aberto para escrita nem
    /// alterado. Este teste fixa o critério de aceitação com bytes e hash
    /// antes/depois idênticos, além de mtime inalterado.
    #[tokio::test]
    async fn import_leaves_the_source_file_byte_for_byte_untouched() {
        let fx = fixture().await;
        let dir = tempfile::tempdir().expect("temporary directory");
        let path = sample_file(dir.path(), "Security.evtx").await;
        // Conteúdo com bytes que não são UTF-8 válido, para barrar qualquer
        // caminho que reescreva o arquivo como texto.
        let original_bytes: Vec<u8> = (0..=255_u8).chain(b"\n\x00\xff".iter().copied()).collect();
        tokio::fs::write(&path, &original_bytes)
            .await
            .expect("rewrite sample");
        let before_metadata = std::fs::metadata(&path).expect("source metadata");
        let before_hash = {
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(&original_bytes);
            format!("{:x}", hasher.finalize())
        };

        let evidence = import_evidence_with_provenance(
            &fx.services,
            &path,
            1024 * 1024,
            &EvidenceProvenance::default(),
        )
        .await
        .expect("import");

        let after_bytes = tokio::fs::read(&path)
            .await
            .expect("read source after import");
        assert_eq!(after_bytes, original_bytes, "conteúdo do original mudou");
        let after_metadata = std::fs::metadata(&path).expect("source metadata after");
        assert_eq!(
            before_metadata.modified().ok(),
            after_metadata.modified().ok(),
            "mtime do original mudou"
        );
        assert_eq!(
            evidence.sha256, before_hash,
            "o SHA-256 gravado deve ser o do arquivo de origem intacto"
        );
        // A cópia preservada no storage é que guarda os bytes.
        let preserved = fx
            .services
            .paths
            .logical_path(&evidence.storage_reference)
            .expect("logical path");
        let stored_bytes = tokio::fs::read(&preserved).await.expect("read preserved");
        assert_eq!(stored_bytes, original_bytes);
    }

    #[test]
    fn blank_fields_are_absent_and_the_instant_must_parse() {
        let clean = EvidenceProvenance {
            source: Some("   ".into()),
            collected_at: Some("2026-09-30T10:00:00Z".into()),
            ..EvidenceProvenance::default()
        }
        .clean()
        .expect("valid");
        assert!(clean.source.is_none());
        assert_eq!(clean.source_or_default(), NOT_PROVIDED);
        assert_eq!(clean.collected_at.as_deref(), Some("2026-09-30T10:00:00Z"));
    }

    #[test]
    fn legacy_constants_and_import_dates_are_not_reported_as_declared() {
        let legacy = EvidenceResponse {
            id: "ev-old".into(),
            original_filename: "Old.evtx".into(),
            storage_reference: "evidence/ev-old.evtx".into(),
            format: "EVTX".into(),
            size_bytes: 1,
            blake3: String::new(),
            sha256: String::new(),
            imported_at: "2026-08-01T10:00:00+00:00".into(),
            status: "FORENSIC".into(),
            source: "LOCAL_OPERATOR".into(),
            provided_by: None,
            collection_method: "LOCAL_FILE_PICKER".into(),
            collection_tool: None,
            collection_tool_version: None,
            collected_at: Some("2026-08-01T10:00:00+00:00".into()),
            authorization_reference: None,
            contains_personal_data: false,
            incident_id: None,
        };
        assert!(legacy.declared_source().is_none());
        assert!(legacy.declared_collection_method().is_none());
        assert!(legacy.declared_collected_at().is_none());
        assert!(
            legacy.is_analyzed_automatically(),
            "EVTX em maiúsculas conta"
        );

        let preserved_only = EvidenceResponse {
            format: "pcap".into(),
            ..legacy
        };
        assert!(!preserved_only.is_analyzed_automatically());
    }
}
