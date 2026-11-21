//! Persistência local e versionada das preferências da interface nativa.
//!
//! O arquivo é tratado como entrada não confiável: tamanho limitado, UTF-8,
//! JSON estrito, versão e validação de domínio. Mensagens de erro são
//! sanitizadas e nunca incluem caminhos ou conteúdo do arquivo.

use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    process,
};

use serde::{
    Deserialize, Serialize,
    de::{MapAccess, SeqAccess, Visitor},
};

use super::theme::{CustomAccent, ThemePreset, ThemeTokens};

pub(crate) const PREFERENCES_VERSION: u32 = 1;
const PREFERENCES_FILE_NAME: &str = "ui-preferences.json";
const MAX_PREFERENCES_BYTES: u64 = 64 * 1024;
const MAX_TEMP_ATTEMPTS: usize = 32;

/// Limites de domínio do acento sob medida no esquema versionado 1.
const MAX_HUE_DEGREES: f32 = 360.0;
const MAX_CHROMA: f32 = 0.32;

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct UiPreferences {
    pub version: u32,
    pub theme: ThemePreset,
    pub custom: CustomAccent,
    pub reduced_motion: bool,
}

/// Campos do acento sob medida no esquema versionado 1 das preferências.
///
/// O esquema do arquivo é versionado, portanto a lista é explícita e estável.
/// Qualquer alteração futura no formato exige incremento de
/// `PREFERENCES_VERSION` e uma decisão de migração.
const DOCUMENTED_CUSTOM_ACCENT_FIELDS: [&str; 3] = ["base", "hue_degrees", "chroma"];

impl<'de> Deserialize<'de> for UiPreferences {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct RawUiPreferences {
            version: u32,
            theme: ThemePreset,
            custom: serde_json::Value,
            reduced_motion: bool,
        }

        let raw = RawUiPreferences::deserialize(deserializer)?;
        let custom =
            strict_custom_accent(raw.custom).map_err(<D::Error as serde::de::Error>::custom)?;
        Ok(Self {
            version: raw.version,
            theme: raw.theme,
            custom,
            reduced_motion: raw.reduced_motion,
        })
    }
}

fn strict_custom_accent(value: serde_json::Value) -> Result<CustomAccent, String> {
    if has_unknown_custom_field(&value)? {
        return Err("custom accent contains an unknown field".to_owned());
    }
    serde_json::from_value(value)
        .map_err(|_| "custom accent does not match the expected schema".to_owned())
}

/// Indica se o objeto de acento possui algum campo fora do esquema.
///
/// Campos desconhecidos são rejeitados para que o arquivo de preferências
/// permaneça estrito e sem ambiguidade.
fn has_unknown_custom_field(value: &serde_json::Value) -> Result<bool, String> {
    let object = value
        .as_object()
        .ok_or_else(|| "custom accent must be a JSON object".to_owned())?;
    let canonical_fields = canonical_custom_fields();
    Ok(object.keys().any(|key| {
        !DOCUMENTED_CUSTOM_ACCENT_FIELDS.contains(&key.as_str())
            && !canonical_fields.iter().any(|field| field == key)
    }))
}

/// Nomes de campos efetivamente serializados pelo tipo de acento do tema.
///
/// Complementa a lista documentada para acompanhar renomeações ou omissões
/// de campos definidas por `theme.rs`.
fn canonical_custom_fields() -> Vec<String> {
    serde_json::to_value(CustomAccent::default())
        .ok()
        .and_then(|value| {
            value
                .as_object()
                .map(|object| object.keys().cloned().collect::<Vec<String>>())
        })
        .unwrap_or_default()
}

/// Marcador usado para distinguir chaves duplicadas dos demais erros de dados.
const DUPLICATE_KEY_MARKER: &str = "duplicate object key";

/// Valor JSON já validado como não duplicado.
struct StrictJson(serde_json::Value);

impl<'de> Deserialize<'de> for StrictJson {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(StrictJsonVisitor)
    }
}

/// Visitor que reconstrói o valor JSON e rejeita chaves de objeto duplicadas.
///
/// O parser padrão aceita chaves repetidas e fica com o último valor, o que
/// permitiria que um arquivo malformado tivesse leituras ambíguas.
struct StrictJsonVisitor;

impl<'de> Visitor<'de> for StrictJsonVisitor {
    type Value = StrictJson;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a JSON value without duplicate object keys")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(StrictJson(serde_json::Value::Bool(value)))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(StrictJson(serde_json::Value::Number(value.into())))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(StrictJson(serde_json::Value::Number(value.into())))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let number = serde_json::Number::from_f64(value)
            .ok_or_else(|| <E as serde::de::Error>::custom("non-finite number"))?;
        Ok(StrictJson(serde_json::Value::Number(number)))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(StrictJson(serde_json::Value::String(value.to_owned())))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(StrictJson(serde_json::Value::String(value)))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(StrictJson(serde_json::Value::Null))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(StrictJson(serde_json::Value::Null))
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        StrictJson::deserialize(deserializer)
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut items = Vec::new();
        while let Some(item) = sequence.next_element::<StrictJson>()? {
            items.push(item.0);
        }
        Ok(StrictJson(serde_json::Value::Array(items)))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut entries = serde_json::Map::new();
        while let Some(key) = map.next_key::<String>()? {
            if entries.get(&key).is_some() {
                return Err(<A::Error as serde::de::Error>::custom(DUPLICATE_KEY_MARKER));
            }
            let value = map.next_value::<StrictJson>()?;
            entries.insert(key, value.0);
        }
        Ok(StrictJson(serde_json::Value::Object(entries)))
    }
}

/// Valida o acento sob medida contra os limites do esquema versionado 1.
///
/// Os limites locais espelham o contrato de `theme.rs` para que o arquivo de
/// preferências permaneça estrito mesmo que a validação do tema evolua.
/// Qualquer estreitamento dos limites no tema deve ser refletido aqui e
/// acompanhado de incremento de `PREFERENCES_VERSION`.
///
/// A validação do tema também é consultada, para que o arquivo seja rejeitado
/// quando o tema considerar um valor inválido. A mensagem de erro do tema não
/// é propagada, para não introduzir conteúdo externo na resposta.
fn validate_custom_accent(custom: &CustomAccent) -> Result<(), String> {
    if !custom.hue_degrees.is_finite()
        || custom.hue_degrees < 0.0
        || custom.hue_degrees > MAX_HUE_DEGREES
    {
        return Err("invalid custom accent".to_owned());
    }
    if !custom.chroma.is_finite() || custom.chroma < 0.0 || custom.chroma > MAX_CHROMA {
        return Err("invalid custom accent".to_owned());
    }
    custom
        .clone()
        .validate()
        .map_err(|_| "invalid custom accent".to_owned())
}

impl Default for UiPreferences {
    fn default() -> Self {
        Self {
            version: PREFERENCES_VERSION,
            theme: ThemePreset::GoldenWind,
            custom: CustomAccent::default(),
            reduced_motion: false,
        }
    }
}

impl UiPreferences {
    #[allow(dead_code)]
    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.version != PREFERENCES_VERSION {
            return Err("unsupported version".to_owned());
        }
        validate_custom_accent(&self.custom)
    }

    #[allow(clippy::clone_on_copy)]
    #[allow(dead_code)]
    pub(crate) fn tokens(&self) -> ThemeTokens {
        ThemeTokens::new(self.theme.clone(), self.custom.clone(), self.reduced_motion)
    }
}

/// Armazenamento das preferências sob uma raiz confiável.
#[allow(dead_code)]
#[derive(Clone)]
pub(crate) struct PreferenceStore {
    path: PathBuf,
    #[cfg(test)]
    rename_failure: bool,
}

impl PreferenceStore {
    #[allow(dead_code)]
    pub(crate) fn new(root: &Path) -> Self {
        let root = if root.as_os_str().is_empty() {
            Path::new(".")
        } else {
            root
        };
        Self {
            path: root.join(PREFERENCES_FILE_NAME),
            #[cfg(test)]
            rename_failure: false,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn load(&self) -> Result<UiPreferences, String> {
        let parent = self
            .path
            .parent()
            .ok_or_else(|| "preferences directory is unavailable".to_owned())?;
        match fs::symlink_metadata(parent) {
            Ok(metadata) => {
                if is_link(&metadata) {
                    return Err(directory_link_error());
                }
                if !metadata.is_dir() {
                    return Err("preferences directory is not a directory".to_owned());
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(UiPreferences::default());
            }
            Err(_) => return Err("could not inspect preferences directory".to_owned()),
        }

        let metadata = match fs::symlink_metadata(&self.path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(UiPreferences::default());
            }
            Err(_) => return Err("could not inspect preferences file".to_owned()),
        };

        reject_link(&metadata)?;
        if !metadata.is_file() {
            return Err("preferences path is not a regular file".to_owned());
        }
        if metadata.len() > MAX_PREFERENCES_BYTES {
            return Err("preferences file exceeds the size limit".to_owned());
        }

        let mut bytes = Vec::with_capacity(metadata.len() as usize);
        File::open(&self.path)
            .map_err(|_| "could not open preferences file".to_owned())?
            .take(MAX_PREFERENCES_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| "could not read preferences file".to_owned())?;
        if bytes.len() as u64 > MAX_PREFERENCES_BYTES {
            return Err("preferences file exceeds the size limit".to_owned());
        }

        let text = std::str::from_utf8(&bytes)
            .map_err(|_| "preferences file is not valid UTF-8".to_owned())?;
        let strict: StrictJson = serde_json::from_str(text).map_err(|error| {
            if error.to_string().contains(DUPLICATE_KEY_MARKER) {
                "preferences file contains duplicate object keys".to_owned()
            } else {
                "preferences file is not valid JSON".to_owned()
            }
        })?;
        let preferences: UiPreferences = serde_json::from_value(strict.0)
            .map_err(|_| "preferences file does not match the expected schema".to_owned())?;
        preferences
            .validate()
            .map_err(|error| format!("preferences file contains {error}"))?;
        Ok(preferences)
    }

    #[allow(dead_code)]
    pub(crate) fn save(&self, preferences: &UiPreferences) -> Result<(), String> {
        preferences
            .validate()
            .map_err(|error| format!("cannot save preferences: {error}"))?;
        let bytes = serde_json::to_vec_pretty(preferences)
            .map_err(|_| "could not serialize preferences".to_owned())?;
        if bytes.len() as u64 > MAX_PREFERENCES_BYTES {
            return Err("preferences payload exceeds the size limit".to_owned());
        }

        let parent = self
            .path
            .parent()
            .ok_or_else(|| "preferences directory is unavailable".to_owned())?;
        ensure_directory(parent)?;

        let (temporary_path, mut temporary_file) = create_temporary(parent)?;
        let result = write_and_sync(&mut temporary_file, &bytes);
        drop(temporary_file);
        let result = result.and_then(|()| self.replace_target(&temporary_path));

        if result.is_err() {
            let _ = fs::remove_file(&temporary_path);
        }
        result
    }

    #[allow(dead_code)]
    fn replace_target(&self, temporary_path: &Path) -> Result<(), String> {
        match fs::symlink_metadata(&self.path) {
            Ok(metadata) => {
                reject_link(&metadata)?;
                if !metadata.is_file() {
                    return Err("preferences path is not a regular file".to_owned());
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err("could not inspect preferences file".to_owned()),
        }

        #[cfg(test)]
        if self.rename_failure {
            return Err("could not replace preferences file".to_owned());
        }

        fs::rename(temporary_path, &self.path)
            .map_err(|_| "could not replace preferences file".to_owned())
    }
}

/// Garante que o diretório de destino exista e não seja um link.
///
/// O diretório é inspecionado antes de qualquer criação, para que um link
/// apontando para fora da raiz confiável nunca seja atravessado.
#[allow(dead_code)]
fn ensure_directory(parent: &Path) -> Result<(), String> {
    match fs::symlink_metadata(parent) {
        Ok(metadata) => {
            if is_link(&metadata) {
                return Err(directory_link_error());
            }
            if !metadata.is_dir() {
                return Err("preferences directory is not a directory".to_owned());
            }
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            fs::create_dir_all(parent)
                .map_err(|_| "could not create preferences directory".to_owned())?;
            match fs::symlink_metadata(parent) {
                Ok(metadata) => {
                    if is_link(&metadata) {
                        return Err(directory_link_error());
                    }
                    if !metadata.is_dir() {
                        return Err("preferences directory is not a directory".to_owned());
                    }
                    Ok(())
                }
                Err(_) => Err("could not inspect preferences directory".to_owned()),
            }
        }
        Err(_) => Err("could not inspect preferences directory".to_owned()),
    }
}

/// Cria um arquivo temporário único no mesmo diretório do destino.
///
/// O nome combina PID, contador monotônico e tentativa para evitar colisão
/// entre processos e entre chamadas concorrentes no mesmo processo.
#[allow(dead_code)]
fn create_temporary(parent: &Path) -> Result<(PathBuf, File), String> {
    for attempt in 0..MAX_TEMP_ATTEMPTS {
        let candidate = parent.join(format!(
            ".{PREFERENCES_FILE_NAME}.{}.{}.{attempt}.tmp",
            process::id(),
            TEMP_FILE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        match create_temporary_file(&candidate) {
            Ok(file) => return Ok((candidate, file)),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(_) => return Err("could not create temporary preferences file".to_owned()),
        }
    }
    Err("could not allocate a unique temporary preferences file".to_owned())
}

/// Escreve o conteúdo completo e sincroniza o arquivo temporário em disco.
#[allow(dead_code)]
fn write_and_sync(file: &mut File, bytes: &[u8]) -> Result<(), String> {
    file.write_all(bytes)
        .map_err(|_| "could not write temporary preferences file".to_owned())?;
    file.sync_all()
        .map_err(|_| "could not synchronize temporary preferences file".to_owned())
}

#[allow(dead_code)]
fn create_temporary_file(path: &Path) -> std::io::Result<File> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;

        OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(path)
    }
    #[cfg(not(unix))]
    {
        OpenOptions::new().write(true).create_new(true).open(path)
    }
}

#[allow(dead_code)]
fn reject_link(metadata: &fs::Metadata) -> Result<(), String> {
    if is_link(metadata) {
        return Err(link_error());
    }
    Ok(())
}

/// Indica se a entrada é um link simbólico ou um reparse point.
///
/// No Windows, o bit genérico de reparse point é verificado para cobrir
/// junctions, que não usam `IO_REPARSE_TAG_SYMLINK`.
#[allow(dead_code)]
fn is_link(metadata: &fs::Metadata) -> bool {
    #[cfg(unix)]
    {
        metadata.file_type().is_symlink()
    }
    #[cfg(windows)]
    {
        is_reparse_point(metadata)
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = metadata;
        false
    }
}

#[cfg(windows)]
#[allow(dead_code)]
fn link_error() -> String {
    "preferences path must not be a reparse point".to_owned()
}

#[cfg(not(windows))]
#[allow(dead_code)]
fn link_error() -> String {
    "preferences path must not be a symbolic link".to_owned()
}

#[cfg(windows)]
#[allow(dead_code)]
fn directory_link_error() -> String {
    "preferences directory must not be a reparse point".to_owned()
}

#[cfg(not(windows))]
#[allow(dead_code)]
fn directory_link_error() -> String {
    "preferences directory must not be a symbolic link".to_owned()
}

#[cfg(windows)]
fn is_reparse_point(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;

    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
    metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[allow(dead_code)]
static TEMP_FILE_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

#[cfg(test)]
mod tests {
    use super::super::theme::{CustomAccent, ThemePreset, ThemeTokens};
    use super::{PREFERENCES_VERSION, PreferenceStore, UiPreferences, validate_custom_accent};
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use std::{fs, io::Write, path::Path};
    use tempfile::tempdir;

    fn valid_json() -> serde_json::Value {
        json!({
            "version": 1,
            "theme": "custom",
            "custom": {
                "base": "light",
                "hue_degrees": 85.0,
                "chroma": 0.14
            },
            "reduced_motion": false
        })
    }

    fn write_json(path: &Path, value: &serde_json::Value) {
        fs::write(
            path,
            serde_json::to_vec(value).expect("serialize test fixture"),
        )
        .expect("write test fixture");
    }

    #[test]
    fn preferences_implement_required_traits() {
        fn assert_traits<T>()
        where
            T: Clone + std::fmt::Debug + Serialize + for<'de> Deserialize<'de> + PartialEq,
        {
        }
        assert_traits::<UiPreferences>();
        assert_traits::<ThemePreset>();
        assert_traits::<CustomAccent>();
    }

    #[test]
    fn theme_token_constructor_has_expected_signature() {
        let constructor: fn(ThemePreset, CustomAccent, bool) -> ThemeTokens = ThemeTokens::new;
        let _ = constructor;
    }

    #[test]
    fn custom_accent_validation_accepts_default() {
        assert!(validate_custom_accent(&CustomAccent::default()).is_ok());
    }

    #[test]
    fn custom_accent_default_has_expected_signature() {
        let constructor: fn() -> CustomAccent = CustomAccent::default;
        let _ = constructor;
    }

    #[test]
    fn golden_wind_preset_exists() {
        let _ = ThemePreset::GoldenWind;
    }

    #[test]
    fn custom_accent_uses_snake_case_base_variants() {
        for base in ["light", "dark", "golden_wind"] {
            let value = json!({
                "base": base,
                "hue_degrees": 85.0,
                "chroma": 0.14
            });
            let accent: CustomAccent =
                serde_json::from_value(value).expect("deserialize custom accent fixture");
            assert_eq!(
                serde_json::to_value(accent).expect("serialize custom accent fixture")["base"],
                base
            );
        }
    }

    #[test]
    fn theme_preset_uses_snake_case_variants() {
        for preset in ["light", "dark", "golden_wind", "custom"] {
            let value = json!({
                "version": 1,
                "theme": preset,
                "custom": {
                    "base": "golden_wind",
                    "hue_degrees": 85.0,
                    "chroma": 0.14
                },
                "reduced_motion": false
            });
            let preferences: UiPreferences =
                serde_json::from_value(value).expect("deserialize preferences fixture");
            assert_eq!(
                serde_json::to_value(preferences).expect("serialize preferences fixture")["theme"],
                preset
            );
        }
    }

    #[test]
    fn reduced_motion_flag_roundtrips() {
        let preferences = UiPreferences {
            reduced_motion: true,
            ..Default::default()
        };
        let value = serde_json::to_value(&preferences).expect("serialize preferences fixture");
        assert_eq!(value["reduced_motion"], json!(true));
        let loaded: UiPreferences =
            serde_json::from_value(value).expect("deserialize preferences fixture");
        assert!(loaded.reduced_motion);
    }

    #[test]
    fn preference_version_constant_matches_payload() {
        assert_eq!(PREFERENCES_VERSION, 1);
        let value =
            serde_json::to_value(UiPreferences::default()).expect("serialize preferences fixture");
        assert_eq!(value["version"], json!(PREFERENCES_VERSION));
    }

    #[test]
    fn preference_store_uses_expected_filename() {
        let directory = tempdir().expect("temporary directory");
        let store = PreferenceStore::new(directory.path());
        assert_eq!(
            store.path.file_name().and_then(|name| name.to_str()),
            Some("ui-preferences.json")
        );
    }

    #[test]
    fn preference_store_constructor_accepts_root_path() {
        let constructor: fn(&Path) -> PreferenceStore = PreferenceStore::new;
        let _ = constructor;
    }

    #[test]
    fn preference_store_methods_have_expected_signatures() {
        let loader: fn(&PreferenceStore) -> Result<UiPreferences, String> = PreferenceStore::load;
        let saver: fn(&PreferenceStore, &UiPreferences) -> Result<(), String> =
            PreferenceStore::save;
        let _ = (loader, saver);
    }

    #[test]
    fn preference_methods_have_expected_signatures() {
        let validator: fn(&UiPreferences) -> Result<(), String> = UiPreferences::validate;
        let tokens: fn(&UiPreferences) -> ThemeTokens = UiPreferences::tokens;
        let _ = (validator, tokens);
    }

    #[test]
    fn roundtrip_preserves_preferences() {
        let directory = tempdir().expect("temporary directory");
        let store = PreferenceStore::new(directory.path());
        let preferences = UiPreferences::default();

        store.save(&preferences).expect("save preferences");
        let loaded = store.load().expect("load preferences");

        assert_eq!(loaded, preferences);
    }

    #[test]
    fn missing_file_returns_defaults() {
        let directory = tempdir().expect("temporary directory");
        let store = PreferenceStore::new(directory.path());

        assert_eq!(
            store.load().expect("load defaults"),
            UiPreferences::default()
        );
    }

    #[test]
    fn invalid_json_is_rejected() {
        let directory = tempdir().expect("temporary directory");
        let store = PreferenceStore::new(directory.path());
        fs::write(directory.path().join("ui-preferences.json"), b"{not-json")
            .expect("write test fixture");

        let error = store.load().expect_err("invalid JSON must fail");
        assert!(error.contains("JSON"));
        assert!(!error.contains(directory.path().to_string_lossy().as_ref()));
    }

    #[test]
    fn invalid_utf8_is_rejected() {
        let directory = tempdir().expect("temporary directory");
        let store = PreferenceStore::new(directory.path());
        fs::write(&store.path, [0xff, 0xfe, 0x00]).expect("write test fixture");

        let error = store.load().expect_err("invalid UTF-8 must fail");
        assert!(error.contains("UTF-8"));
    }

    #[test]
    fn wrong_version_is_rejected() {
        let directory = tempdir().expect("temporary directory");
        let store = PreferenceStore::new(directory.path());
        let mut value = valid_json();
        value["version"] = json!(PREFERENCES_VERSION + 1);
        write_json(&store.path, &value);

        let error = store.load().expect_err("wrong version must fail");
        assert!(error.contains("unsupported version"));
    }

    #[test]
    fn unknown_field_is_rejected() {
        let directory = tempdir().expect("temporary directory");
        let store = PreferenceStore::new(directory.path());
        let mut value = valid_json();
        value["unexpected"] = json!(true);
        write_json(&store.path, &value);

        let error = store.load().expect_err("unknown field must fail");
        assert!(error.contains("schema"));
    }

    #[test]
    fn unknown_nested_custom_field_is_rejected() {
        let directory = tempdir().expect("temporary directory");
        let store = PreferenceStore::new(directory.path());
        let mut value = valid_json();
        value["custom"]["unexpected"] = json!(true);
        write_json(&store.path, &value);

        let error = store.load().expect_err("unknown nested field must fail");
        assert!(error.contains("schema"));
    }

    #[test]
    fn duplicate_object_key_is_rejected() {
        let directory = tempdir().expect("temporary directory");
        let store = PreferenceStore::new(directory.path());
        let content = concat!(
            r#"{"version":1,"theme":"golden_wind","custom":{"base":"golden_wind","#,
            r#""hue_degrees":85.0,"chroma":0.14},"reduced_motion":false,"#,
            r#""version":1}"#
        );
        fs::write(&store.path, content).expect("write test fixture");

        let error = store.load().expect_err("duplicate key must fail");
        assert!(error.contains("duplicate"));
    }

    #[test]
    fn oversized_file_is_rejected_without_parsing() {
        let directory = tempdir().expect("temporary directory");
        let store = PreferenceStore::new(directory.path());
        let mut file = fs::File::create(&store.path).expect("create test fixture");
        file.write_all(&vec![b'x'; 64 * 1024 + 1])
            .expect("write test fixture");

        let error = store.load().expect_err("oversized file must fail");
        assert!(error.contains("size limit"));
    }

    #[test]
    fn invalid_custom_accent_is_rejected() {
        let directory = tempdir().expect("temporary directory");
        let store = PreferenceStore::new(directory.path());
        let mut value = valid_json();
        value["custom"]["chroma"] = json!(0.5);
        write_json(&store.path, &value);

        let error = store.load().expect_err("invalid custom accent must fail");
        assert!(error.contains("invalid custom accent"));
    }

    #[test]
    fn invalid_custom_accent_is_not_saved() {
        let directory = tempdir().expect("temporary directory");
        let store = PreferenceStore::new(directory.path());
        let mut preferences = UiPreferences::default();
        let mut value = valid_json();
        value["custom"]["chroma"] = json!(0.5);
        let invalid_custom: CustomAccent = serde_json::from_value(value["custom"].clone())
            .expect("deserialize custom accent fixture");
        preferences.custom = invalid_custom;

        let error = store
            .save(&preferences)
            .expect_err("invalid custom accent must fail");
        assert!(error.contains("invalid custom accent"));
        assert!(!store.path.exists());
    }

    #[test]
    fn non_file_target_is_rejected() {
        let directory = tempdir().expect("temporary directory");
        let store = PreferenceStore::new(directory.path());
        fs::create_dir(&store.path).expect("create directory fixture");

        let error = store.load().expect_err("directory target must fail");
        assert!(error.contains("regular file"));
    }

    #[test]
    fn failed_rename_preserves_previous_file_and_removes_temporary() {
        let directory = tempdir().expect("temporary directory");
        let mut store = PreferenceStore::new(directory.path());
        let previous = UiPreferences::default();
        store.save(&previous).expect("save previous preferences");
        let previous_bytes = fs::read(&store.path).expect("read previous preferences");

        store.rename_failure = true;
        let result = store.save(&UiPreferences::default());

        assert!(result.is_err());
        assert_eq!(
            fs::read(&store.path).expect("read preserved preferences"),
            previous_bytes
        );
        let temporary_files = fs::read_dir(directory.path())
            .expect("read temporary directory")
            .filter_map(Result::ok)
            .filter(|entry| entry.file_name().to_string_lossy().ends_with(".tmp"))
            .count();
        assert_eq!(temporary_files, 0);
    }

    #[cfg(unix)]
    #[test]
    fn symbolic_link_is_rejected() {
        use std::os::unix::fs::symlink;

        let directory = tempdir().expect("temporary directory");
        let store = PreferenceStore::new(directory.path());
        let target = directory.path().join("target");
        write_json(&target, &valid_json());
        symlink(&target, &store.path).expect("create symbolic link");

        let error = store.load().expect_err("symbolic link must fail");
        assert!(error.contains("symbolic link"));
    }

    #[cfg(unix)]
    #[test]
    fn save_rejects_symbolic_link_destination() {
        use std::os::unix::fs::symlink;

        let directory = tempdir().expect("temporary directory");
        let store = PreferenceStore::new(directory.path());
        let target = directory.path().join("target");
        write_json(&target, &valid_json());
        symlink(&target, &store.path).expect("create symbolic link");

        let error = store
            .save(&UiPreferences::default())
            .expect_err("symbolic link must fail");
        assert!(error.contains("symbolic link"));
        assert!(
            fs::symlink_metadata(&store.path)
                .expect("inspect symbolic link")
                .file_type()
                .is_symlink()
        );
    }

    #[cfg(unix)]
    #[test]
    fn symbolic_link_parent_directory_is_rejected() {
        use std::os::unix::fs::symlink;

        let directory = tempdir().expect("temporary directory");
        let target = directory.path().join("target");
        fs::create_dir(&target).expect("create directory fixture");
        let linked = directory.path().join("linked");
        symlink(&target, &linked).expect("create symbolic link");
        let store = PreferenceStore::new(&linked);

        let load_error = store.load().expect_err("symbolic link parent must fail");
        assert!(load_error.contains("symbolic link"));
        let save_error = store
            .save(&UiPreferences::default())
            .expect_err("symbolic link parent must fail");
        assert!(save_error.contains("symbolic link"));
    }

    #[cfg(unix)]
    #[test]
    fn saved_file_has_owner_only_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let directory = tempdir().expect("temporary directory");
        let store = PreferenceStore::new(directory.path());
        store
            .save(&UiPreferences::default())
            .expect("save preferences");

        let mode = fs::metadata(&store.path)
            .expect("inspect preferences metadata")
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o600);
    }
}
