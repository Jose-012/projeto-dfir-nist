//! Resolução segura de `input_reference` para um caminho real dentro da raiz
//! de entrada confiável.
//!
//! Este é o código de segurança de caminhos do engine (traversal, escape da
//! raiz, symlink/reparse point no componente final). A lógica é a mesma de
//! antes de o módulo existir; a única mudança é que a raiz é um parâmetro em
//! vez de ser lida do ambiente a cada chamada, o que permite testar o caminho
//! feliz e os escapes com diretórios temporários.

use crate::error::EngineFailure;
use std::path::{Path, PathBuf};

const INPUT_ROOT: &str = "/input";

pub fn input_root() -> PathBuf {
    std::env::var_os("DFIR_ENGINE_INPUT_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(INPUT_ROOT))
}

/// Arquivo de evidência fixado para este processo (`DFIR_ENGINE_INPUT_FILE`,
/// F1-22). O supervisor define a variável com o caminho do arquivo do job; o
/// engine só aceita esse arquivo, mesmo que a raiz de entrada contenha outros.
/// Resolvido **antes** do sandbox. Se o caminho não resolve, devolve o texto
/// cru: nenhum arquivo resolvido será igual a ele, então tudo é recusado.
pub fn pinned_input_file() -> Option<PathBuf> {
    let raw = std::env::var_os("DFIR_ENGINE_INPUT_FILE")?;
    let raw = PathBuf::from(raw);
    Some(std::fs::canonicalize(&raw).unwrap_or(raw))
}

pub fn resolve_input(root: &Path, reference: &str) -> Result<PathBuf, EngineFailure> {
    let relative_reference = if let Some(relative) = reference.strip_prefix("evidence/") {
        relative
    } else if !reference.contains('/') {
        // The CLI smoke harness uses a root already scoped to a single job input.
        reference
    } else {
        return Err((
            "INPUT_REJECTED",
            "input reference must use the evidence/ logical namespace".into(),
        ));
    };
    if relative_reference.is_empty() {
        return Err((
            "INPUT_REJECTED",
            "input reference must identify a file".into(),
        ));
    }
    let path = root.join(relative_reference);
    let canonical_root = std::fs::canonicalize(root).map_err(|error| {
        (
            "INPUT_UNAVAILABLE",
            format!("input root unavailable: {error}"),
        )
    })?;
    let canonical_path = std::fs::canonicalize(&path).map_err(|error| {
        (
            "INPUT_UNAVAILABLE",
            format!("input file unavailable: {error}"),
        )
    })?;
    if !canonical_path.starts_with(&canonical_root) {
        return Err((
            "INPUT_REJECTED",
            "input path escapes the trusted input root".into(),
        ));
    }
    if !canonical_path.is_file() {
        return Err((
            "INPUT_REJECTED",
            "input reference is not a regular file".into(),
        ));
    }
    // BUG FIX: check the pre-canonicalize leaf (`path`), not `canonical_path`.
    // `std::fs::canonicalize` already follows symlinks, so by the time a path
    // has been canonicalized it can never itself look like a symlink to
    // `symlink_metadata` — checking `canonical_path` here made this call a
    // permanent no-op against exactly the "pre-planted symlink" case the
    // comment below describes. `path` (still the on-disk leaf as given,
    // before resolution) is what actually exposes a symlink sitting at that
    // spot; the containment check above still relies on the canonicalized
    // paths, unchanged.
    //
    // Narrows (does not eliminate) the canonicalize-then-open TOCTOU window: this is
    // the last check before EvtxParser::from_path actually opens the file. See the
    // matching comment on dfir-app's process::reject_symlink for the accepted residual
    // risk under this product's single-local-operator threat model (ADR-0007).
    reject_symlink(&path)?;
    Ok(canonical_path)
}

#[cfg(windows)]
fn is_reparse_point(metadata: &std::fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
    metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(windows))]
fn is_reparse_point(metadata: &std::fs::Metadata) -> bool {
    metadata.file_type().is_symlink()
}

pub fn reject_symlink(path: &Path) -> Result<(), EngineFailure> {
    let metadata = std::fs::symlink_metadata(path).map_err(|error| {
        (
            "INPUT_UNAVAILABLE",
            format!("input file unavailable: {error}"),
        )
    })?;
    if is_reparse_point(&metadata) {
        return Err((
            "INPUT_REJECTED",
            "input path resolves through a symlink/reparse point and is rejected".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{reject_symlink, resolve_input};
    use std::path::Path;

    #[test]
    fn resolve_input_rejects_traversal_and_unmounted_namespaces() {
        let root = Path::new("/nonexistent-root");
        assert!(resolve_input(root, "../secret.evtx").is_err());
        assert!(resolve_input(root, "staging/result.jsonl").is_err());
        assert!(resolve_input(root, "evidence/").is_err());
    }

    #[test]
    fn resolve_input_accepts_the_logical_evidence_namespace() {
        // The test intentionally reaches canonicalization after stripping the
        // logical namespace; a missing file must therefore report INPUT_UNAVAILABLE,
        // not INPUT_REJECTED or a path-namespace error.
        let error = resolve_input(Path::new("/nonexistent-root"), "evidence/missing.evtx")
            .expect_err("missing fixture");
        assert_eq!(error.0, "INPUT_UNAVAILABLE");
        assert!(
            error.1.contains("input root unavailable")
                || error.1.contains("input file unavailable")
        );
    }

    #[test]
    fn resolve_input_returns_the_canonical_path_of_a_regular_file_under_the_root() {
        let dir = tempfile::tempdir().expect("temp dir");
        let file = dir.path().join("security.evtx");
        std::fs::write(&file, b"not real evtx bytes").expect("write fixture");

        let resolved = resolve_input(dir.path(), "evidence/security.evtx").expect("resolves");
        assert_eq!(resolved, std::fs::canonicalize(&file).expect("canonical"));

        // The CLI smoke harness passes a bare file name against a job-scoped root.
        let bare = resolve_input(dir.path(), "security.evtx").expect("bare name resolves");
        assert_eq!(bare, resolved);
    }

    #[test]
    fn resolve_input_rejects_a_directory() {
        let dir = tempfile::tempdir().expect("temp dir");
        std::fs::create_dir(dir.path().join("folder")).expect("create dir");
        let error = resolve_input(dir.path(), "evidence/folder").expect_err("not a file");
        assert_eq!(error.0, "INPUT_REJECTED");
    }

    #[cfg(unix)]
    #[test]
    fn resolve_input_rejects_a_symlink_that_escapes_the_root() {
        let outside = tempfile::tempdir().expect("outside dir");
        let secret = outside.path().join("secret.evtx");
        std::fs::write(&secret, b"secret").expect("write secret");

        let root = tempfile::tempdir().expect("root dir");
        std::os::unix::fs::symlink(&secret, root.path().join("link.evtx")).expect("symlink");

        let error = resolve_input(root.path(), "evidence/link.evtx").expect_err("escape rejected");
        assert_eq!(error.0, "INPUT_REJECTED");
        assert!(error.1.contains("escapes the trusted input root"));
    }

    // Regression test for a symlink-check-ordering bug: `reject_symlink` was
    // being called on the already-canonicalized path, which can never look
    // like a symlink (canonicalize has, by definition, already resolved it),
    // so a symlink whose *target* stayed inside the trusted root slipped
    // through both the containment check (target is in-bounds) and the old
    // `reject_symlink(&canonical_path)` call (nothing left to detect). This
    // is exactly the "pre-planted symlink" case the function's docs claim to
    // reject, and it was not exercised by any existing test — the
    // escapes-the-root case above only exercises the containment check.
    #[cfg(unix)]
    #[test]
    fn resolve_input_rejects_a_symlink_whose_target_stays_inside_the_root() {
        let root = tempfile::tempdir().expect("root dir");
        let real = root.path().join("real.evtx");
        std::fs::write(&real, b"real evidence bytes").expect("write real file");
        std::os::unix::fs::symlink(&real, root.path().join("link.evtx")).expect("symlink");

        let error =
            resolve_input(root.path(), "evidence/link.evtx").expect_err("in-root symlink rejected");
        assert_eq!(error.0, "INPUT_REJECTED");
        assert!(error.1.contains("symlink/reparse point"));
    }

    #[test]
    fn reject_symlink_accepts_a_regular_file() {
        let dir = tempfile::tempdir().expect("temp dir");
        let file = dir.path().join("security.evtx");
        std::fs::write(&file, b"not real evtx bytes").expect("write fixture");
        assert!(reject_symlink(&file).is_ok());
    }

    #[cfg(unix)]
    #[test]
    fn reject_symlink_rejects_a_symlinked_leaf() {
        let dir = tempfile::tempdir().expect("temp dir");
        let target = dir.path().join("outside-target");
        std::fs::write(&target, b"secret").expect("write target");
        let link = dir.path().join("security.evtx");
        std::os::unix::fs::symlink(&target, &link).expect("create symlink");
        let error = reject_symlink(&link).expect_err("symlink must be rejected");
        assert_eq!(error.0, "INPUT_REJECTED");
    }
}
