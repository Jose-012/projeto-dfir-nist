//! F1-40 (ADR-0030 D-2): assinatura destacada e verificação offline.

use super::{
    FILE_CRITICAL_PATH_SVG, FILE_E1, FILE_E2, FILE_E3, FILE_E4, FILE_E5, FILE_MANIFEST,
    GeneratedFile, Manifest, sha256_hex, write_atomically,
};
use crate::error::{AppError, AppResult};
use crate::infrastructure::storage::StoragePaths;
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::io::Read;
use std::path::{Component, Path, PathBuf};

pub const FILE_SIGNATURE: &str = "manifest.json.sig";
const MAX_MANIFEST_BYTES: u64 = 16 * 1024 * 1024;
const MAX_SIGNATURE_BYTES: u64 = 64 * 1024;
const SIGNATURE_SCHEMA: &str = "dfir-deliverables-signature/1";

#[derive(Debug, Clone, Serialize)]
pub struct VerifiedDeliverables {
    pub output_dir: PathBuf,
    pub incident_id: String,
    pub generated_at: String,
    pub documents: usize,
    pub public_key_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SignatureEnvelope {
    schema: String,
    algorithm: String,
    manifest_sha256: String,
    public_key_id: String,
    signature: String,
}

fn config(message: impl Into<String>) -> AppError {
    AppError::Config(message.into())
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn decode_hex<const N: usize>(text: &str, label: &str) -> AppResult<[u8; N]> {
    if text.len() != N * 2
        || !text
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(config(format!("{label} must be lowercase hexadecimal")));
    }
    fn nibble(byte: u8) -> AppResult<u8> {
        match byte {
            b'0'..=b'9' => Ok(byte - b'0'),
            b'a'..=b'f' => Ok(byte - b'a' + 10),
            _ => Err(config("invalid hexadecimal input")),
        }
    }
    let input = text.as_bytes();
    let mut output = [0_u8; N];
    for (index, output) in output.iter_mut().enumerate() {
        *output = (nibble(input[index * 2])? << 4) | nibble(input[index * 2 + 1])?;
    }
    Ok(output)
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

fn canonical_file(path: &Path, label: &str) -> AppResult<(PathBuf, std::fs::Metadata)> {
    let original = std::fs::symlink_metadata(path)?;
    if is_reparse_point(&original) || !original.is_file() {
        return Err(config(format!(
            "{label} must be a regular file without symlink/reparse point"
        )));
    }
    let canonical = path.canonicalize()?;
    let metadata = std::fs::symlink_metadata(&canonical)?;
    if is_reparse_point(&metadata) || !metadata.is_file() {
        return Err(config(format!("{label} did not resolve to a regular file")));
    }
    Ok((canonical, metadata))
}

fn canonical_directory(path: &Path) -> AppResult<PathBuf> {
    let metadata = std::fs::symlink_metadata(path)?;
    if is_reparse_point(&metadata) || !metadata.is_dir() {
        return Err(config(
            "deliverables directory must be a real directory without symlink/reparse point",
        ));
    }
    Ok(path.canonicalize()?)
}

fn load_private_key(
    path: &Path,
    paths: &StoragePaths,
    output_dir: Option<&Path>,
) -> AppResult<SigningKey> {
    let (canonical, metadata) = canonical_file(path, "private key")?;
    let storage_root = paths
        .evidence
        .parent()
        .ok_or_else(|| config("could not resolve the trusted storage root"))?
        .canonicalize()?;
    if canonical.starts_with(&storage_root) {
        return Err(config(
            "private key must be outside the application's internal storage",
        ));
    }
    for protected in [
        &paths.db,
        &paths.runtime,
        &paths.evidence,
        &paths.staging,
        &paths.sanitized,
        &paths.exports,
        &paths.quarantine,
        &paths.logs,
        &paths.backups,
    ] {
        let protected = protected.canonicalize().map_err(|_| {
            config("could not canonicalize internal storage while validating the private key")
        })?;
        if canonical.starts_with(protected) {
            return Err(config(
                "private key must be outside the application's internal storage",
            ));
        }
    }
    if let Some(output_dir) = output_dir
        && canonical.starts_with(output_dir)
    {
        return Err(config(
            "private key must be outside the exported deliverables directory",
        ));
    }
    if !matches!(metadata.len(), 64 | 65) {
        return Err(config(
            "private key must contain exactly 32 bytes as lowercase hexadecimal",
        ));
    }
    let bytes = std::fs::read(canonical)?;
    let text = std::str::from_utf8(&bytes).map_err(|_| config("private key is not valid UTF-8"))?;
    let text = text.strip_suffix('\n').unwrap_or(text);
    let seed = decode_hex::<32>(text, "private key")?;
    Ok(SigningKey::from_bytes(&seed))
}

pub fn public_key_hex(private_key: &Path, paths: &StoragePaths) -> AppResult<String> {
    let key = load_private_key(private_key, paths, None)?;
    Ok(hex_encode(&key.verifying_key().to_bytes()))
}

fn public_key_id(key: &VerifyingKey) -> String {
    format!("sha256:{}", hex_encode(&Sha256::digest(key.to_bytes())))
}

fn read_bounded(root: &Path, file_name: &str, maximum: u64, label: &str) -> AppResult<Vec<u8>> {
    let (path, metadata) = canonical_file(&root.join(file_name), label)?;
    if !path.starts_with(root) || metadata.len() > maximum {
        return Err(config(format!(
            "{label} is outside the export or exceeds the size limit"
        )));
    }
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    std::fs::File::open(path)?
        .take(maximum + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > maximum {
        return Err(config(format!("{label} exceeds the size limit")));
    }
    Ok(bytes)
}

fn verify_manifest_and_documents(root: &Path) -> AppResult<(Manifest, [u8; 32])> {
    let manifest_bytes = read_bounded(root, FILE_MANIFEST, MAX_MANIFEST_BYTES, "manifest")?;
    let manifest: Manifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| config(format!("invalid deliverable manifest: {error}")))?;
    if manifest.schema != "dfir-deliverables/1" {
        return Err(config("unsupported deliverable manifest schema"));
    }
    let expected: BTreeSet<&str> = [
        FILE_E1,
        FILE_E2,
        FILE_E3,
        FILE_E4,
        FILE_E5,
        FILE_CRITICAL_PATH_SVG,
    ]
    .into_iter()
    .collect();
    let mut actual = BTreeSet::new();
    for document in &manifest.documents {
        let mut components = Path::new(&document.file).components();
        if !matches!(components.next(), Some(Component::Normal(_)))
            || components.next().is_some()
            || !expected.contains(document.file.as_str())
            || !actual.insert(document.file.as_str())
        {
            return Err(config(
                "manifest contains an unsafe, duplicate or unexpected document path",
            ));
        }
        let (path, metadata) = canonical_file(&root.join(&document.file), "manifest document")?;
        if !path.starts_with(root) || metadata.len() != document.bytes {
            return Err(config("document size does not match the manifest"));
        }
        let mut file = std::fs::File::open(path)?;
        let mut sha256 = Sha256::new();
        let mut blake3 = blake3::Hasher::new();
        let mut count = 0_u64;
        let mut buffer = [0_u8; 64 * 1024];
        loop {
            let read = file.read(&mut buffer)?;
            if read == 0 {
                break;
            }
            sha256.update(&buffer[..read]);
            blake3.update(&buffer[..read]);
            count = count
                .checked_add(read as u64)
                .ok_or_else(|| config("document size overflow"))?;
        }
        if count != document.bytes
            || hex_encode(&sha256.finalize()) != document.sha256
            || blake3.finalize().to_hex().to_string() != document.blake3
        {
            return Err(config("document hash or size does not match the manifest"));
        }
    }
    if actual != expected {
        return Err(config(
            "manifest does not contain all documents and the SVG",
        ));
    }
    Ok((manifest, Sha256::digest(&manifest_bytes).into()))
}

pub(super) fn sign_directory(
    output_dir: &Path,
    private_key: &Path,
    paths: &StoragePaths,
) -> AppResult<GeneratedFile> {
    let root = canonical_directory(output_dir)?;
    let key = load_private_key(private_key, paths, Some(&root))?;
    let (_, digest) = verify_manifest_and_documents(&root)?;
    let verifying_key = key.verifying_key();
    let signature = key.sign(&digest);
    let envelope = SignatureEnvelope {
        schema: SIGNATURE_SCHEMA.into(),
        algorithm: "Ed25519".into(),
        manifest_sha256: hex_encode(&digest),
        public_key_id: public_key_id(&verifying_key),
        signature: hex_encode(&signature.to_bytes()),
    };
    let bytes = serde_json::to_vec_pretty(&envelope)
        .map_err(|error| config(format!("could not serialize signature: {error}")))?;
    write_atomically(&root, FILE_SIGNATURE, &bytes)?;
    Ok(GeneratedFile {
        file_name: FILE_SIGNATURE.into(),
        bytes: bytes.len() as u64,
        sha256: sha256_hex(&bytes),
    })
}

pub fn verify_directory(
    output_dir: &Path,
    expected_public_key_hex: &str,
) -> AppResult<VerifiedDeliverables> {
    let root = canonical_directory(output_dir)?;
    let public_bytes = decode_hex::<32>(expected_public_key_hex, "public key")?;
    let verifying_key = VerifyingKey::from_bytes(&public_bytes)
        .map_err(|_| config("public key is not a valid Ed25519 key"))?;
    let (manifest, digest) = verify_manifest_and_documents(&root)?;
    let signature_bytes = read_bounded(&root, FILE_SIGNATURE, MAX_SIGNATURE_BYTES, "signature")?;
    let envelope: SignatureEnvelope = serde_json::from_slice(&signature_bytes)
        .map_err(|error| config(format!("invalid signature artifact: {error}")))?;
    if envelope.schema != SIGNATURE_SCHEMA || envelope.algorithm != "Ed25519" {
        return Err(config("unsupported signature schema or algorithm"));
    }
    if decode_hex::<32>(&envelope.manifest_sha256, "manifest digest")? != digest {
        return Err(config("signed manifest digest does not match the manifest"));
    }
    let key_id = public_key_id(&verifying_key);
    if envelope.public_key_id != key_id {
        return Err(config("signature was made by a different public key"));
    }
    let signature = Signature::from_bytes(&decode_hex::<64>(&envelope.signature, "signature")?);
    verifying_key
        .verify_strict(&digest, &signature)
        .map_err(|_| config("Ed25519 signature verification failed"))?;
    Ok(VerifiedDeliverables {
        output_dir: root,
        incident_id: manifest.incident_id,
        generated_at: manifest.generated_at,
        documents: manifest.documents.len(),
        public_key_id: key_id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::deliverables::{
        DeliverableSet, FILE_CRITICAL_PATH_SVG, FILE_E1, FILE_E2, FILE_E3, FILE_E4, FILE_E5,
        FILE_MANIFEST, RenderedDocument, write_set,
    };
    use ed25519_dalek::{Signature, VerifyingKey};
    use sha2::{Digest, Sha256};
    use tempfile::TempDir;

    // Chave sintética fixa, exclusiva de testes. Nunca usar fora deste módulo.
    const TEST_PRIVATE_KEY_HEX: &str =
        "4242424242424242424242424242424242424242424242424242424242424242";
    const OTHER_PUBLIC_KEY_HEX: &str =
        "4343434343434343434343434343434343434343434343434343434343434343";

    struct Fixture {
        _storage: TempDir,
        _output: TempDir,
        _keys: TempDir,
        paths: StoragePaths,
        output_dir: PathBuf,
        private_key: PathBuf,
    }

    fn fixture() -> Fixture {
        let storage = tempfile::tempdir().expect("storage");
        let paths = StoragePaths::from_root(storage.path());
        paths.ensure().expect("storage layout");
        let output = tempfile::tempdir().expect("output");
        let keys = tempfile::tempdir().expect("keys");
        let private_key = keys.path().join("synthetic-ed25519-private.key");
        std::fs::write(&private_key, format!("{TEST_PRIVATE_KEY_HEX}\n")).expect("private key");

        let documents = [FILE_E1, FILE_E2, FILE_E3, FILE_E4, FILE_E5]
            .into_iter()
            .enumerate()
            .map(|(index, file_name)| RenderedDocument {
                file_name,
                title: "Documento sintético",
                content: format!("# Documento {index}\n\nconteúdo sintético\n"),
                source_result_ids: vec![],
            })
            .collect();
        let set = DeliverableSet {
            incident_id: "inc-signing-test".into(),
            incident_name: "Incidente sintético".into(),
            documents,
            critical_path_svg: RenderedDocument {
                file_name: FILE_CRITICAL_PATH_SVG,
                title: "SVG sintético",
                content: "<svg xmlns=\"http://www.w3.org/2000/svg\"></svg>\n".into(),
                source_result_ids: vec![],
            },
            redaction: None,
            personal_data_evidence: 0,
        };
        let written =
            write_set(&set, output.path(), &paths, "2026-11-20T00:00:00Z").expect("written set");
        Fixture {
            _storage: storage,
            _output: output,
            _keys: keys,
            paths,
            output_dir: written.output_dir,
            private_key,
        }
    }

    fn public_key(fixture: &Fixture) -> String {
        public_key_hex(&fixture.private_key, &fixture.paths).expect("public key")
    }

    fn sign(fixture: &Fixture) -> GeneratedFile {
        sign_directory(&fixture.output_dir, &fixture.private_key, &fixture.paths)
            .expect("signature")
    }

    fn signature_json(fixture: &Fixture) -> serde_json::Value {
        serde_json::from_slice(
            &std::fs::read(fixture.output_dir.join(FILE_SIGNATURE)).expect("signature bytes"),
        )
        .expect("signature json")
    }

    fn decode_hex<const N: usize>(text: &str) -> [u8; N] {
        assert_eq!(text.len(), N * 2);
        let mut bytes = [0_u8; N];
        for (index, byte) in bytes.iter_mut().enumerate() {
            *byte = u8::from_str_radix(&text[index * 2..index * 2 + 2], 16).expect("hex");
        }
        bytes
    }

    #[test]
    fn roundtrip_is_deterministic_and_signs_exactly_the_manifest_digest() {
        let fixture = fixture();
        let public_key = public_key(&fixture);
        let first = sign(&fixture);
        let first_bytes = std::fs::read(fixture.output_dir.join(FILE_SIGNATURE)).expect("sig");
        let second = sign(&fixture);
        let second_bytes = std::fs::read(fixture.output_dir.join(FILE_SIGNATURE)).expect("sig");
        assert_eq!(first_bytes, second_bytes);
        assert_eq!(first.file_name, FILE_SIGNATURE);
        assert_eq!(first.bytes, second.bytes);
        assert_eq!(first.sha256, second.sha256);

        let verified = verify_directory(&fixture.output_dir, &public_key).expect("roundtrip");
        assert_eq!(verified.documents, 6);
        assert_eq!(verified.incident_id, "inc-signing-test");
        assert!(verified.public_key_id.starts_with("sha256:"));

        let manifest_bytes =
            std::fs::read(fixture.output_dir.join(FILE_MANIFEST)).expect("manifest");
        let envelope = signature_json(&fixture);
        assert_eq!(envelope["schema"], "dfir-deliverables-signature/1");
        assert_eq!(envelope["algorithm"], "Ed25519");
        assert_eq!(envelope["manifest_sha256"], {
            let digest = Sha256::digest(&manifest_bytes);
            digest
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        });
        let verifying_key = VerifyingKey::from_bytes(&decode_hex::<32>(&public_key)).expect("key");
        let signature = Signature::from_bytes(&decode_hex::<64>(
            envelope["signature"].as_str().expect("signature hex"),
        ));
        let digest = Sha256::digest(&manifest_bytes);
        verifying_key
            .verify_strict(&digest, &signature)
            .expect("digest signature");
        assert!(
            verifying_key
                .verify_strict(&manifest_bytes, &signature)
                .is_err()
        );
        let hex_digest = format!("{digest:x}");
        assert!(
            verifying_key
                .verify_strict(hex_digest.as_bytes(), &signature)
                .is_err()
        );
    }

    #[test]
    fn verification_rejects_tampered_manifest_document_and_signature() {
        for artifact in ["manifest", "document", "signature"] {
            let fixture = fixture();
            let public_key = public_key(&fixture);
            sign(&fixture);
            match artifact {
                "manifest" => {
                    let path = fixture.output_dir.join(FILE_MANIFEST);
                    let mut bytes = std::fs::read(&path).expect("manifest");
                    let index = bytes.iter().position(|byte| *byte == b'I').unwrap_or(0);
                    bytes[index] ^= 1;
                    std::fs::write(path, bytes).expect("tamper manifest");
                }
                "document" => std::fs::write(fixture.output_dir.join(FILE_E1), b"adulterado\n")
                    .expect("tamper document"),
                "signature" => {
                    let path = fixture.output_dir.join(FILE_SIGNATURE);
                    let mut envelope = signature_json(&fixture);
                    let signature = envelope["signature"].as_str().expect("hex").to_string();
                    let replacement = if signature.starts_with('0') { "1" } else { "0" };
                    envelope["signature"] = format!("{replacement}{}", &signature[1..]).into();
                    std::fs::write(path, serde_json::to_vec_pretty(&envelope).expect("json"))
                        .expect("tamper signature");
                }
                _ => unreachable!(),
            }
            assert!(
                verify_directory(&fixture.output_dir, &public_key).is_err(),
                "{artifact} adulterado foi aceito"
            );
        }
    }

    #[test]
    fn verification_rejects_missing_invalid_and_wrong_key_artifacts() {
        for artifact in [FILE_MANIFEST, FILE_E1, FILE_SIGNATURE] {
            let fixture = fixture();
            let public_key = public_key(&fixture);
            sign(&fixture);
            std::fs::remove_file(fixture.output_dir.join(artifact)).expect("remove artifact");
            assert!(verify_directory(&fixture.output_dir, &public_key).is_err());
        }

        let fixture = fixture();
        sign(&fixture);
        assert!(verify_directory(&fixture.output_dir, OTHER_PUBLIC_KEY_HEX).is_err());
        assert!(verify_directory(&fixture.output_dir, "not-a-key").is_err());
        std::fs::write(fixture.output_dir.join(FILE_SIGNATURE), b"{invalid")
            .expect("invalid signature");
        assert!(verify_directory(&fixture.output_dir, &public_key(&fixture)).is_err());
    }

    #[test]
    fn private_key_must_exist_outside_storage_and_export_and_be_a_regular_leaf() {
        let fixture = fixture();
        let missing = fixture._keys.path().join("missing.key");
        assert!(public_key_hex(&missing, &fixture.paths).is_err());
        assert!(!missing.exists(), "não pode gerar chave silenciosamente");
        assert!(public_key_hex(fixture._keys.path(), &fixture.paths).is_err());

        let internal = fixture.paths.evidence.join("private.key");
        std::fs::write(&internal, TEST_PRIVATE_KEY_HEX).expect("internal key");
        assert!(public_key_hex(&internal, &fixture.paths).is_err());
        let root_key = fixture._storage.path().join("root-private.key");
        std::fs::write(&root_key, TEST_PRIVATE_KEY_HEX).expect("root key");
        assert!(public_key_hex(&root_key, &fixture.paths).is_err());

        let exported = fixture.output_dir.join("private.key");
        std::fs::write(&exported, TEST_PRIVATE_KEY_HEX).expect("exported key");
        assert!(sign_directory(&fixture.output_dir, &exported, &fixture.paths).is_err());
        assert!(!fixture.output_dir.join(FILE_SIGNATURE).exists());

        #[cfg(unix)]
        {
            let link = fixture._keys.path().join("private-link.key");
            std::os::unix::fs::symlink(&fixture.private_key, &link).expect("key symlink");
            assert!(public_key_hex(&link, &fixture.paths).is_err());
        }
    }

    #[test]
    fn unsafe_manifest_paths_and_document_symlinks_are_rejected() {
        let unsafe_fixture = fixture();
        let manifest_path = unsafe_fixture.output_dir.join(FILE_MANIFEST);
        let mut manifest: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&manifest_path).expect("manifest"))
                .expect("json");
        manifest["documents"][0]["file"] = "../fora.md".into();
        std::fs::write(
            &manifest_path,
            serde_json::to_vec_pretty(&manifest).expect("json"),
        )
        .expect("unsafe manifest");
        assert!(
            sign_directory(
                &unsafe_fixture.output_dir,
                &unsafe_fixture.private_key,
                &unsafe_fixture.paths
            )
            .is_err()
        );
        assert!(!unsafe_fixture.output_dir.join(FILE_SIGNATURE).exists());

        let another = fixture();
        let public_key = public_key(&another);
        sign(&another);
        #[cfg(unix)]
        {
            let external = another._keys.path().join("external-document.md");
            std::fs::write(&external, "conteúdo externo".as_bytes()).expect("external document");
            let document = another.output_dir.join(FILE_E1);
            std::fs::remove_file(&document).expect("remove document");
            std::os::unix::fs::symlink(external, &document).expect("document symlink");
            assert!(verify_directory(&another.output_dir, &public_key).is_err());
        }
    }
}
