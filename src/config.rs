use crate::error::{AppError, AppResult};
use std::{
    env,
    ffi::OsString,
    path::{Path, PathBuf},
    time::Duration,
};

/// Mesmo valor do protocolo (ADR-0020): o engine fecha cada lote antes de passar
/// disto. Era 64 KiB, e um único registro EVTX com JSON maior derrubava o job.
pub const DEFAULT_ENGINE_MAX_LINE_BYTES: usize = dfir_protocol::MAX_LINE_BYTES;
pub const DEFAULT_ENGINE_TIMEOUT_SECONDS: u64 = 300;
pub const MIN_ENGINE_MAX_LINE_BYTES: usize = 1024;
pub const MAX_ENGINE_MAX_LINE_BYTES: usize = dfir_protocol::MAX_LINE_BYTES;
pub const MAX_ENGINE_TIMEOUT_SECONDS: u64 = 3600;
pub const DEFAULT_ENGINE_MAX_CONCURRENT_JOBS: usize = 1;
pub const MAX_ENGINE_MAX_CONCURRENT_JOBS: usize = 8;
// Bounds the engine child process's virtual address space (Linux: RLIMIT_AS via
// pre_exec on Unix). On platforms without RLIMIT_AS, memory enforcement relies on
// the operating system process limits and the configured wall-clock timeout. A crafted or corrupt EVTX
// that makes the parser allocate without bound is otherwise limited only by the
// wall-clock timeout, not by memory -- it can still exhaust host RAM within
// that window. Default is generous for single-EVTX-file parsing headroom.
pub const DEFAULT_ENGINE_MAX_MEMORY_BYTES: u64 = 1024 * 1024 * 1024; // 1 GiB
pub const MIN_ENGINE_MAX_MEMORY_BYTES: u64 = 64 * 1024 * 1024; // 64 MiB
pub const MAX_ENGINE_MAX_MEMORY_BYTES: u64 = 16 * 1024 * 1024 * 1024; // 16 GiB
pub const DEFAULT_MAX_UPLOAD_BYTES: u64 = 512 * 1024 * 1024; // 512 MiB
pub const MIN_MAX_UPLOAD_BYTES: u64 = 1024 * 1024; // 1 MiB
pub const MAX_MAX_UPLOAD_BYTES: u64 = 4 * 1024 * 1024 * 1024; // 4 GiB

#[derive(Clone)]
pub struct AppConfig {
    pub data_root: PathBuf,
    pub engine_path: PathBuf,
    pub engine_max_line_bytes: usize,
    pub engine_timeout: Duration,
    pub engine_max_concurrent_jobs: usize,
    pub engine_max_memory_bytes: u64,
    pub max_upload_bytes: usize,
    /// Margem mínima de espaço livre no volume de dados (0 desliga a guarda).
    pub min_free_disk_bytes: u64,
    /// Política do sandbox do engine (`DFIR_ENGINE_SANDBOX`): `required`,
    /// `best-effort` ou `off`. `None` deixa o engine usar o padrão da
    /// plataforma (Linux: `required`). Valor inválido é erro de configuração
    /// na partida (ADR-0024).
    pub engine_sandbox: Option<String>,
}

impl AppConfig {
    pub fn from_env() -> AppResult<Self> {
        let data_root = env::var_os("DFIR_DATA_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(default_data_root);
        let engine_path = select_engine_path(
            current_host_platform(),
            env::var_os("DFIR_ENGINE_PATH"),
            current_exe_dir().as_deref(),
            &data_root,
            |candidate| candidate.is_file(),
        );
        let engine_max_line_bytes = parse_usize_bounded(
            "DFIR_ENGINE_MAX_LINE_BYTES",
            DEFAULT_ENGINE_MAX_LINE_BYTES,
            MIN_ENGINE_MAX_LINE_BYTES,
            MAX_ENGINE_MAX_LINE_BYTES,
        )?;
        let timeout_seconds = parse_u64_bounded(
            "DFIR_ENGINE_TIMEOUT_SECONDS",
            DEFAULT_ENGINE_TIMEOUT_SECONDS,
            1,
            MAX_ENGINE_TIMEOUT_SECONDS,
        )?;
        let engine_max_concurrent_jobs = parse_usize_bounded(
            "DFIR_ENGINE_MAX_CONCURRENT_JOBS",
            DEFAULT_ENGINE_MAX_CONCURRENT_JOBS,
            1,
            MAX_ENGINE_MAX_CONCURRENT_JOBS,
        )?;
        let engine_max_memory_bytes = parse_u64_bounded(
            "DFIR_ENGINE_MAX_MEMORY_BYTES",
            DEFAULT_ENGINE_MAX_MEMORY_BYTES,
            MIN_ENGINE_MAX_MEMORY_BYTES,
            MAX_ENGINE_MAX_MEMORY_BYTES,
        )?;

        let max_upload_bytes = parse_u64_bounded(
            "DFIR_MAX_UPLOAD_BYTES",
            DEFAULT_MAX_UPLOAD_BYTES,
            MIN_MAX_UPLOAD_BYTES,
            MAX_MAX_UPLOAD_BYTES,
        )? as usize;

        let min_free_disk_bytes = parse_u64_bounded(
            "DFIR_MIN_FREE_DISK_BYTES",
            crate::infrastructure::disk::DEFAULT_MIN_FREE_DISK_BYTES,
            0,
            crate::infrastructure::disk::MAX_MIN_FREE_DISK_BYTES,
        )?;

        let engine_sandbox = parse_sandbox_policy(env::var("DFIR_ENGINE_SANDBOX").ok())?;

        Ok(Self {
            data_root,
            engine_path,
            engine_max_line_bytes,
            engine_timeout: Duration::from_secs(timeout_seconds),
            engine_max_concurrent_jobs,
            engine_max_memory_bytes,
            max_upload_bytes,
            min_free_disk_bytes,
            engine_sandbox,
        })
    }
}

/// Valida `DFIR_ENGINE_SANDBOX`. Vazio é "não definido"; um valor desconhecido
/// **recusa a partida** em vez de ser ignorado: um erro de digitação não pode
/// parecer que a contenção está ligada.
pub fn parse_sandbox_policy(raw: Option<String>) -> AppResult<Option<String>> {
    let Some(raw) = raw else {
        return Ok(None);
    };
    let value = raw.trim().to_ascii_lowercase();
    match value.as_str() {
        "" => Ok(None),
        "required" => Ok(Some("required".into())),
        "best-effort" | "best_effort" | "besteffort" => Ok(Some("best-effort".into())),
        "off" | "none" | "disabled" => Ok(Some("off".into())),
        _ => Err(AppError::Config(
            "DFIR_ENGINE_SANDBOX must be one of: required, best-effort, off".into(),
        )),
    }
}

/// Qual convenção de diretório de dados do usuário aplicar. Um enum
/// explícito (em vez de `cfg!` espalhado pela lógica de seleção) para que
/// `select_data_root` seja testável para as três plataformas a partir de
/// qualquer host de CI, não só da plataforma que efetivamente compila os
/// testes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HostPlatform {
    Windows,
    MacOs,
    OtherUnix,
}

fn current_host_platform() -> HostPlatform {
    if cfg!(windows) {
        HostPlatform::Windows
    } else if cfg!(target_os = "macos") {
        HostPlatform::MacOs
    } else {
        HostPlatform::OtherUnix
    }
}

/// Falls back to a stable per-user application-data location on every
/// platform, em vez de um `./storage` relativo ao diretório de trabalho.
///
/// Isso não é só uma questão de conveniência: um default relativo ao CWD é
/// um risco real de exposição de evidência forense. Se o binário for
/// iniciado a partir da Área de Trabalho ou de Documentos — exatamente as
/// pastas que o OneDrive "Known Folder Move" e o iCloud Drive costumam
/// sincronizar para a nuvem por padrão —, `./storage` criaria o banco e os
/// arquivos de evidência ali dentro, sincronizando tudo silenciosamente. Os
/// diretórios de "dados de aplicativo" usados aqui (`LOCALAPPDATA`,
/// `XDG_DATA_HOME`/`.local/share`, `Library/Application Support`) são
/// justamente os que essas mesmas ferramentas de sincronização **não**
/// redirecionam por padrão — a escolha não é arbitrária, é a que reduz esse
/// risco específico.
fn default_data_root() -> PathBuf {
    select_data_root(
        current_host_platform(),
        env::var_os("LOCALAPPDATA"),
        env::var_os("XDG_DATA_HOME"),
        env::var_os("HOME"),
    )
}

/// Lógica de seleção pura — recebe a plataforma como parâmetro em vez de
/// consultar `cfg!`/env vars diretamente, para que cada ramo (Windows,
/// macOS, Unix genérico, e o fallback final) seja testável a partir de
/// qualquer host, com valores sintéticos, sem mutar env vars reais do
/// processo.
fn select_data_root(
    platform: HostPlatform,
    local_app_data: Option<std::ffi::OsString>,
    xdg_data_home: Option<std::ffi::OsString>,
    home: Option<std::ffi::OsString>,
) -> PathBuf {
    match platform {
        HostPlatform::Windows => {
            if let Some(local_app_data) = local_app_data {
                return windows_data_root(&local_app_data);
            }
        }
        HostPlatform::MacOs => {
            if let Some(home) = home {
                return macos_data_root(&home);
            }
        }
        HostPlatform::OtherUnix => {
            if let Some(xdg_data_home) = xdg_data_home {
                return linux_xdg_data_root(&xdg_data_home);
            }
            if let Some(home) = home {
                return linux_home_data_root(&home);
            }
        }
    }
    PathBuf::from("./storage")
}

/// Pure join logic factored out so it can be unit-tested on any host platform,
/// independent of `cfg(windows)` and without mutating process-global env vars.
fn windows_data_root(local_app_data: &std::ffi::OsStr) -> PathBuf {
    PathBuf::from(local_app_data).join("DFIR").join("storage")
}

fn macos_data_root(home: &std::ffi::OsStr) -> PathBuf {
    PathBuf::from(home)
        .join("Library")
        .join("Application Support")
        .join("DFIR")
        .join("storage")
}

fn linux_xdg_data_root(xdg_data_home: &std::ffi::OsStr) -> PathBuf {
    PathBuf::from(xdg_data_home).join("dfir").join("storage")
}

fn linux_home_data_root(home: &std::ffi::OsStr) -> PathBuf {
    PathBuf::from(home)
        .join(".local")
        .join("share")
        .join("dfir")
        .join("storage")
}

/// Nome do binário do engine na plataforma dada.
fn engine_file_name(platform: HostPlatform) -> &'static str {
    match platform {
        HostPlatform::Windows => "dfir-engine.exe",
        HostPlatform::MacOs | HostPlatform::OtherUnix => "dfir-engine",
    }
}

/// Pasta onde o próprio `dfir-app` está rodando (base da resolução padrão do
/// engine). Em Unix o caminho do executável é canonicalizado para seguir um
/// symlink até o local real de instalação; no Windows o valor de
/// `current_exe()` é usado como está (o `process.rs` canonicaliza o caminho
/// final do engine antes de executá-lo).
fn current_exe_dir() -> Option<PathBuf> {
    let exe = env::current_exe().ok()?;
    #[cfg(not(windows))]
    let exe = std::fs::canonicalize(&exe).unwrap_or(exe);
    exe.parent().map(Path::to_path_buf)
}

/// Lógica pura de resolução do caminho do `dfir-engine` (ADR-0015). Recebe
/// plataforma, valor de `DFIR_ENGINE_PATH`, pasta do executável e o teste de
/// existência como parâmetros — mesmo motivo de `select_data_root`: cada
/// ramo é testável de qualquer host, sem tocar o disco nem o ambiente real.
///
/// Ordem:
/// 1. `DFIR_ENGINE_PATH` (valor vazio é ignorado);
/// 2. `<pasta do dfir-app>/dfir-engine[.exe]`, se for um arquivo — é o layout
///    de `target/{debug,release}/` e de uma distribuição autocontida, sem
///    exigir instalação de sistema nem privilégio de administrador;
/// 3. alternativa por plataforma: Unix (Linux/macOS) usa
///    `/usr/local/bin/dfir-engine`; no Windows, que não tem local de sistema
///    equivalente, devolve o caminho ao lado do executável mesmo sem o
///    arquivo (a mensagem de erro do launcher mostra onde ele foi procurado).
///    Se nem a pasta do executável for conhecida, o Windows cai no local
///    legado `<data_root>/runtime/`, nunca em um caminho relativo ao CWD.
fn select_engine_path(
    platform: HostPlatform,
    explicit: Option<OsString>,
    exe_dir: Option<&Path>,
    data_root: &Path,
    is_file: impl Fn(&Path) -> bool,
) -> PathBuf {
    if let Some(explicit) = explicit.filter(|value| !value.is_empty()) {
        return PathBuf::from(explicit);
    }
    let name = engine_file_name(platform);
    let sibling = exe_dir.map(|dir| dir.join(name));
    if let Some(candidate) = &sibling
        && is_file(candidate)
    {
        return candidate.clone();
    }
    match platform {
        HostPlatform::Windows => sibling.unwrap_or_else(|| data_root.join("runtime").join(name)),
        HostPlatform::MacOs | HostPlatform::OtherUnix => {
            PathBuf::from("/usr/local/bin/dfir-engine")
        }
    }
}

fn parse_usize_bounded(name: &str, default: usize, min: usize, max: usize) -> AppResult<usize> {
    let value = env::var(name).unwrap_or_else(|_| default.to_string());
    let parsed = value
        .parse::<usize>()
        .map_err(|error| AppError::Config(format!("{name} must be an integer: {error}")))?;
    if !(min..=max).contains(&parsed) {
        return Err(AppError::Config(format!(
            "{name} must be between {min} and {max}"
        )));
    }
    Ok(parsed)
}

fn parse_u64_bounded(name: &str, default: u64, min: u64, max: u64) -> AppResult<u64> {
    let value = env::var(name).unwrap_or_else(|_| default.to_string());
    let parsed = value
        .parse::<u64>()
        .map_err(|error| AppError::Config(format!("{name} must be an integer: {error}")))?;
    if !(min..=max).contains(&parsed) {
        return Err(AppError::Config(format!(
            "{name} must be between {min} and {max}"
        )));
    }
    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::{
        HostPlatform, current_exe_dir, default_data_root, engine_file_name, linux_home_data_root,
        linux_xdg_data_root, macos_data_root, parse_sandbox_policy, parse_u64_bounded,
        parse_usize_bounded, select_data_root, select_engine_path, windows_data_root,
    };
    use std::path::{Path, PathBuf};

    #[test]
    fn windows_data_root_joins_under_local_app_data() {
        // `PathBuf::join` usa o separador do host: em Linux/macOS `\` não é
        // separador, então a string literal do Windows só é comparável no
        // próprio Windows. A estrutura (base + DFIR + storage) vale em qualquer host.
        let base = r"C:\Users\analyst\AppData\Local";
        let root = windows_data_root(std::ffi::OsStr::new(base));
        assert_eq!(root, PathBuf::from(base).join("DFIR").join("storage"));
        #[cfg(windows)]
        assert_eq!(
            root,
            PathBuf::from(r"C:\Users\analyst\AppData\Local\DFIR\storage")
        );
    }

    #[test]
    fn macos_data_root_joins_under_application_support() {
        assert_eq!(
            macos_data_root(std::ffi::OsStr::new("/Users/analyst")),
            PathBuf::from("/Users/analyst/Library/Application Support/DFIR/storage")
        );
    }

    #[test]
    fn linux_xdg_data_root_joins_under_xdg_data_home() {
        assert_eq!(
            linux_xdg_data_root(std::ffi::OsStr::new("/home/analyst/.data")),
            PathBuf::from("/home/analyst/.data/dfir/storage")
        );
    }

    #[test]
    fn linux_home_data_root_joins_under_local_share() {
        assert_eq!(
            linux_home_data_root(std::ffi::OsStr::new("/home/analyst")),
            PathBuf::from("/home/analyst/.local/share/dfir/storage")
        );
    }

    #[test]
    fn select_data_root_picks_the_right_join_per_platform() {
        let windows_base = r"C:\Users\analyst\AppData\Local";
        let windows_root =
            select_data_root(HostPlatform::Windows, Some(windows_base.into()), None, None);
        // Ver `windows_data_root_joins_under_local_app_data`: o separador é do host.
        assert_eq!(
            windows_root,
            PathBuf::from(windows_base).join("DFIR").join("storage")
        );
        #[cfg(windows)]
        assert_eq!(
            windows_root,
            PathBuf::from(r"C:\Users\analyst\AppData\Local\DFIR\storage")
        );
        assert_eq!(
            select_data_root(
                HostPlatform::MacOs,
                None,
                None,
                Some("/Users/analyst".into())
            ),
            PathBuf::from("/Users/analyst/Library/Application Support/DFIR/storage")
        );
        assert_eq!(
            select_data_root(
                HostPlatform::OtherUnix,
                None,
                Some("/home/analyst/.data".into()),
                Some("/home/analyst".into())
            ),
            PathBuf::from("/home/analyst/.data/dfir/storage"),
            "XDG_DATA_HOME takes precedence over HOME when both are set"
        );
        assert_eq!(
            select_data_root(
                HostPlatform::OtherUnix,
                None,
                None,
                Some("/home/analyst".into())
            ),
            PathBuf::from("/home/analyst/.local/share/dfir/storage")
        );
    }

    #[test]
    fn select_data_root_falls_back_to_relative_storage_when_nothing_is_set() {
        for platform in [
            HostPlatform::Windows,
            HostPlatform::MacOs,
            HostPlatform::OtherUnix,
        ] {
            assert_eq!(
                select_data_root(platform, None, None, None),
                PathBuf::from("./storage"),
                "{platform:?} must fall back to ./storage when its expected env vars are absent"
            );
        }
    }

    #[test]
    fn default_data_root_resolves_to_some_path_on_the_current_host() {
        // Smoke test only: the exact value is host-environment-dependent by
        // design (real env vars). The deterministic, per-platform behavior
        // is covered above via select_data_root with synthetic inputs.
        assert!(!default_data_root().as_os_str().is_empty());
    }

    #[test]
    fn engine_line_limit_bounds_are_enforced() {
        assert!(parse_usize_bounded("TEST", 64 * 1024, 1024, 1024 * 1024).is_ok());
        assert!(parse_usize_bounded("TEST", 1023, 1024, 1024 * 1024).is_err());
        assert!(parse_usize_bounded("TEST", 1024 * 1024 + 1, 1024, 1024 * 1024).is_err());
    }

    #[test]
    fn engine_timeout_bounds_are_enforced() {
        assert!(parse_u64_bounded("TEST", 300, 1, 3600).is_ok());
        assert!(parse_u64_bounded("TEST", 0, 1, 3600).is_err());
        assert!(parse_u64_bounded("TEST", 3601, 1, 3600).is_err());
    }

    #[test]
    fn engine_concurrency_bounds_are_enforced() {
        assert!(parse_usize_bounded("TEST", 1, 1, 8).is_ok());
        assert!(parse_usize_bounded("TEST", 0, 1, 8).is_err());
        assert!(parse_usize_bounded("TEST", 9, 1, 8).is_err());
    }

    #[test]
    fn engine_memory_bounds_are_enforced() {
        use super::{
            DEFAULT_ENGINE_MAX_MEMORY_BYTES, MAX_ENGINE_MAX_MEMORY_BYTES,
            MIN_ENGINE_MAX_MEMORY_BYTES,
        };
        assert!(
            parse_u64_bounded(
                "TEST",
                DEFAULT_ENGINE_MAX_MEMORY_BYTES,
                MIN_ENGINE_MAX_MEMORY_BYTES,
                MAX_ENGINE_MAX_MEMORY_BYTES
            )
            .is_ok()
        );
        assert!(
            parse_u64_bounded(
                "TEST",
                MIN_ENGINE_MAX_MEMORY_BYTES - 1,
                MIN_ENGINE_MAX_MEMORY_BYTES,
                MAX_ENGINE_MAX_MEMORY_BYTES
            )
            .is_err()
        );
        assert!(
            parse_u64_bounded(
                "TEST",
                MAX_ENGINE_MAX_MEMORY_BYTES + 1,
                MIN_ENGINE_MAX_MEMORY_BYTES,
                MAX_ENGINE_MAX_MEMORY_BYTES
            )
            .is_err()
        );
    }
    const APP_DIR: &str = "/opt/dfir";
    const DATA_ROOT: &str = "/data/dfir/storage";

    fn resolve(
        platform: HostPlatform,
        explicit: Option<&str>,
        exe_dir: Option<&str>,
        sibling_exists: bool,
    ) -> PathBuf {
        select_engine_path(
            platform,
            explicit.map(Into::into),
            exe_dir.map(Path::new),
            Path::new(DATA_ROOT),
            |_| sibling_exists,
        )
    }

    #[test]
    fn engine_path_env_override_wins_over_everything() {
        for platform in [
            HostPlatform::Windows,
            HostPlatform::MacOs,
            HostPlatform::OtherUnix,
        ] {
            assert_eq!(
                resolve(platform, Some("/custom/engine"), Some(APP_DIR), true),
                PathBuf::from("/custom/engine"),
                "{platform:?}"
            );
        }
    }

    #[test]
    fn engine_path_empty_env_override_is_ignored() {
        assert_eq!(
            resolve(HostPlatform::OtherUnix, Some(""), Some(APP_DIR), true),
            PathBuf::from(APP_DIR).join("dfir-engine")
        );
    }

    #[test]
    fn engine_path_defaults_to_the_binary_next_to_dfir_app() {
        assert_eq!(
            resolve(HostPlatform::OtherUnix, None, Some(APP_DIR), true),
            PathBuf::from(APP_DIR).join("dfir-engine")
        );
        assert_eq!(
            resolve(HostPlatform::MacOs, None, Some(APP_DIR), true),
            PathBuf::from(APP_DIR).join("dfir-engine")
        );
        assert_eq!(
            resolve(HostPlatform::Windows, None, Some(APP_DIR), true),
            PathBuf::from(APP_DIR).join("dfir-engine.exe")
        );
    }

    #[test]
    fn engine_path_on_unix_falls_back_to_usr_local_bin() {
        for platform in [HostPlatform::MacOs, HostPlatform::OtherUnix] {
            assert_eq!(
                resolve(platform, None, Some(APP_DIR), false),
                PathBuf::from("/usr/local/bin/dfir-engine"),
                "sibling missing, {platform:?}"
            );
            assert_eq!(
                resolve(platform, None, None, false),
                PathBuf::from("/usr/local/bin/dfir-engine"),
                "executable directory unknown, {platform:?}"
            );
        }
    }

    #[test]
    fn engine_path_on_windows_reports_the_sibling_even_when_it_is_missing() {
        assert_eq!(
            resolve(HostPlatform::Windows, None, Some(APP_DIR), false),
            PathBuf::from(APP_DIR).join("dfir-engine.exe")
        );
    }

    #[test]
    fn engine_path_on_windows_never_falls_back_to_a_cwd_relative_path() {
        let resolved = resolve(HostPlatform::Windows, None, None, false);
        assert_eq!(
            resolved,
            PathBuf::from(DATA_ROOT)
                .join("runtime")
                .join("dfir-engine.exe")
        );
        assert!(resolved.starts_with(DATA_ROOT));
    }

    #[test]
    fn engine_file_name_matches_the_platform() {
        assert_eq!(engine_file_name(HostPlatform::Windows), "dfir-engine.exe");
        assert_eq!(engine_file_name(HostPlatform::MacOs), "dfir-engine");
        assert_eq!(engine_file_name(HostPlatform::OtherUnix), "dfir-engine");
    }

    #[test]
    fn current_exe_dir_resolves_on_the_current_host() {
        // Smoke test: o valor exato depende de onde o runner de testes está.
        assert!(current_exe_dir().is_some_and(|dir| dir.is_absolute()));
    }

    #[test]
    fn the_sandbox_policy_accepts_only_known_names_and_rejects_typos() {
        assert_eq!(parse_sandbox_policy(None).unwrap(), None);
        assert_eq!(parse_sandbox_policy(Some("  ".into())).unwrap(), None);
        assert_eq!(
            parse_sandbox_policy(Some("Required".into()))
                .unwrap()
                .as_deref(),
            Some("required")
        );
        assert_eq!(
            parse_sandbox_policy(Some("best_effort".into()))
                .unwrap()
                .as_deref(),
            Some("best-effort")
        );
        assert_eq!(
            parse_sandbox_policy(Some("OFF".into())).unwrap().as_deref(),
            Some("off")
        );
        // Um erro de digitação não pode passar por "ligado".
        assert!(parse_sandbox_policy(Some("requird".into())).is_err());
        assert!(parse_sandbox_policy(Some("true".into())).is_err());
    }
}
