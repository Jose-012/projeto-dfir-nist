use crate::application::engine::{EngineRunError, EngineRunnerConfig};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::process::{Child, Command};

/// Rejects a symlink/reparse point on the exact leaf path.
///
/// This narrows, but does not eliminate, the canonicalize-then-open TOCTOU window
/// between this check and the point where `dfir-engine` (a separate process, spawned
/// after this check) actually opens the file: an attacker able to swap the evidence
/// file for a symlink in that window could still redirect the read. Fully closing that
/// gap needs platform-specific "identity of an already-open handle" verification
/// (e.g. resolving `/proc/self/fd/N` on Linux, `GetFinalPathNameByHandleW` on Windows)
/// instead of path-based checks, which is real engineering scope currently unjustified
/// by this product's threat model: a single local operator analyzing their own evidence
/// (ADR-0007), not a hostile co-resident process actively racing file swaps. This check
/// still blocks the common case: a pre-planted symlink sitting at the path when we look.
fn reject_symlink(path: &Path) -> Result<(), EngineRunError> {
    let metadata = std::fs::symlink_metadata(path)?;
    if is_reparse_point(&metadata) {
        return Err(EngineRunError::InvalidRequest(format!(
            "{} resolves through a symlink/reparse point and is rejected",
            path.display()
        )));
    }
    Ok(())
}

#[cfg(windows)]
fn is_reparse_point(metadata: &std::fs::Metadata) -> bool {
    // std's FileType::is_symlink() on Windows only recognizes IO_REPARSE_TAG_SYMLINK.
    // NTFS junctions (IO_REPARSE_TAG_MOUNT_POINT) are a distinct, more relevant attack
    // vector here since unprivileged users can create them without Developer Mode, so
    // this checks the general reparse-point attribute bit instead.
    use std::os::windows::fs::MetadataExt;
    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
    metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(windows))]
fn is_reparse_point(metadata: &std::fs::Metadata) -> bool {
    metadata.file_type().is_symlink()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProcessMode {
    Stdio,
    Smoke,
}

#[derive(Debug, Clone)]
pub(crate) enum ProcessInput {
    Directory(PathBuf),
    File { root: PathBuf, file: PathBuf },
}

pub(crate) struct ProcessHandle {
    child: Child,
    stdin: Option<Box<dyn AsyncWrite + Send + Unpin>>,
    stdout: Option<Box<dyn AsyncRead + Send + Unpin>>,
    stderr: Option<Box<dyn AsyncRead + Send + Unpin>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ProcessExitStatus {
    pub success: bool,
    pub code: u32,
}

impl ProcessHandle {
    pub(crate) fn from_child(mut child: Child) -> Self {
        let stdin = child
            .stdin
            .take()
            .map(|stream| Box::new(stream) as Box<dyn AsyncWrite + Send + Unpin>);
        let stdout = child
            .stdout
            .take()
            .map(|stream| Box::new(stream) as Box<dyn AsyncRead + Send + Unpin>);
        let stderr = child
            .stderr
            .take()
            .map(|stream| Box::new(stream) as Box<dyn AsyncRead + Send + Unpin>);
        Self {
            child,
            stdin,
            stdout,
            stderr,
        }
    }

    pub(crate) fn take_stdin(&mut self) -> Option<Box<dyn AsyncWrite + Send + Unpin>> {
        self.stdin.take()
    }

    pub(crate) fn take_stdout(&mut self) -> Option<Box<dyn AsyncRead + Send + Unpin>> {
        self.stdout.take()
    }

    pub(crate) fn take_stderr(&mut self) -> Option<Box<dyn AsyncRead + Send + Unpin>> {
        self.stderr.take()
    }

    pub(crate) async fn wait(
        &mut self,
        timeout: std::time::Duration,
    ) -> Result<ProcessExitStatus, EngineRunError> {
        match tokio::time::timeout(timeout, self.child.wait()).await {
            Ok(Ok(status)) => Ok(ProcessExitStatus {
                success: status.success(),
                code: status.code().unwrap_or_default() as u32,
            }),
            Ok(Err(error)) => Err(EngineRunError::Io(error)),
            Err(_) => Err(EngineRunError::Timeout),
        }
    }

    /// Pid do processo, enquanto ele existe.
    #[cfg(test)]
    pub(crate) fn id(&self) -> Option<u32> {
        self.child.id()
    }

    pub(crate) async fn kill(&mut self) {
        // F1-23: kill/wait podem falhar se o processo já saiu; registrar em
        // vez de engolir — o objetivo (processo morto) continua valendo.
        if let Err(error) = self.child.kill().await {
            tracing::debug!(%error, "engine kill: process already gone or failed");
        }
        if let Err(error) = self.child.wait().await {
            tracing::debug!(%error, "engine wait after kill failed");
        }
        self.stdin.take();
        self.stdout.take();
        self.stderr.take();
    }
}

/// Ambiente que o engine recebe do ambiente do app (F1-22, RF-033).
///
/// O engine processa entrada hostil: ele **não** herda o ambiente do app, que
/// pode conter chaves, tokens e caminhos do usuário. Em Unix o engine não
/// precisa de nada do ambiente herdado. No Windows ele precisa de `SystemRoot`
/// e `SystemDrive` para carregar bibliotecas do sistema e de `TEMP`/`TMP`; o
/// `PATH` é refeito apontando só para `System32`, para o engine não procurar
/// DLL em pastas do usuário. As variáveis `DFIR_ENGINE_*` são definidas à
/// parte, por `launch_process`.
pub(crate) fn reduced_environment(
    ambient: impl IntoIterator<Item = (OsString, OsString)>,
    host_is_windows: bool,
) -> Vec<(OsString, OsString)> {
    if !host_is_windows {
        return Vec::new();
    }
    let mut kept: Vec<(OsString, OsString)> = Vec::new();
    let mut system_root: Option<OsString> = None;
    for (name, value) in ambient {
        let upper = name.to_string_lossy().to_ascii_uppercase();
        match upper.as_str() {
            "SYSTEMROOT" => {
                system_root = Some(value.clone());
                kept.push((name, value));
            }
            "SYSTEMDRIVE" | "TEMP" | "TMP" => kept.push((name, value)),
            _ => {}
        }
    }
    if let Some(root) = system_root {
        let mut path = PathBuf::from(root);
        path.push("System32");
        kept.push((OsString::from("PATH"), path.into_os_string()));
    }
    kept
}

pub(crate) fn launch_process(
    config: &EngineRunnerConfig,
    input: &ProcessInput,
    output_dir: &Path,
    probe_mode: bool,
) -> Result<ProcessHandle, EngineRunError> {
    let trusted_root = std::fs::canonicalize(&config.trusted_data_root).map_err(|error| {
        EngineRunError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("trusted storage root is unavailable: {error}"),
        ))
    })?;
    let canonical_output = std::fs::canonicalize(output_dir).map_err(|error| {
        EngineRunError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("process output root is unavailable: {error}"),
        ))
    })?;
    if !canonical_output.starts_with(&trusted_root) {
        return Err(EngineRunError::InvalidRequest(
            "output path must remain under the trusted storage root".into(),
        ));
    }

    let (input_root, input_file) = match input {
        ProcessInput::Directory(path) => {
            let root = std::fs::canonicalize(path)?;
            if !root.starts_with(&trusted_root) || !root.is_dir() {
                return Err(EngineRunError::InvalidRequest(
                    "input root must remain under the trusted storage root".into(),
                ));
            }
            (root, None)
        }
        ProcessInput::File { root, file } => {
            let canonical_root = std::fs::canonicalize(root)?;
            let canonical_file = std::fs::canonicalize(file)?;
            if !canonical_root.starts_with(&trusted_root)
                || !canonical_file.starts_with(&canonical_root)
                || !canonical_file.is_file()
            {
                return Err(EngineRunError::InvalidRequest(
                    "input file must remain under the trusted input root".into(),
                ));
            }
            // BUG FIX: must check the pre-canonicalize leaf, not `canonical_file`.
            // `canonicalize()` already follows symlinks, so by the time a path has
            // been canonicalized it can never itself look like a symlink to
            // `symlink_metadata` — checking `canonical_file` here made this call a
            // permanent no-op against exactly the "pre-planted symlink" case the
            // doc comment on `reject_symlink` says it defends against. Checking the
            // original `file` (still the on-disk leaf as given) is what actually
            // catches it, while the trusted-root containment check above keeps
            // relying on the canonicalized paths as before.
            reject_symlink(file)?;
            (canonical_root, Some(canonical_file))
        }
    };

    let executable = std::fs::canonicalize(&config.engine_path)?;
    if !executable.is_file() || !executable.is_absolute() {
        return Err(EngineRunError::InvalidRequest(
            "engine executable is unavailable".into(),
        ));
    }

    let mut command = Command::new(&executable);
    // O processo do engine não sobrevive ao app nem a uma tarefa abandonada:
    // se o handle é solto sem `kill` explícito, o filho é encerrado.
    command.kill_on_drop(true);
    // Ambiente reduzido: nada do ambiente do app passa para o engine.
    command.env_clear();
    for (name, value) in reduced_environment(std::env::vars_os(), cfg!(windows)) {
        command.env(name, value);
    }
    command.arg(if probe_mode {
        "--process-smoke"
    } else {
        "--stdio"
    });
    command.env("DFIR_ENGINE_INPUT_ROOT", &input_root);
    command.env("DFIR_ENGINE_OUTPUT_ROOT", &canonical_output);
    // O arquivo de entrada é mapeado em memória (YARA-X faz `mmap`), e `RLIMIT_AS`
    // conta espaço de endereçamento, não RAM: com o teto fixo, um arquivo
    // maior que ele não podia ser varrido (`Cannot allocate memory`, medido
    // com 1,1 GiB). O teto vale, então, para tudo MENOS o arquivo avaliado.
    let input_size = input_file
        .as_ref()
        .and_then(|file| std::fs::metadata(file).ok())
        .map_or(0, |metadata| metadata.len());
    // Sem `DFIR_ENGINE_INPUT_FILE` o engine cai no modo manual/QA; o app só
    // chega aqui sem arquivo no smoke de processo (`ProcessInput::Directory`).
    if let Some(file) = input_file {
        command.env("DFIR_ENGINE_INPUT_FILE", file);
    }
    if let Some(policy) = &config.sandbox_policy {
        command.env("DFIR_ENGINE_SANDBOX", policy);
    }
    // No Windows não existe `pre_exec`/`RLIMIT_AS`: o teto de memória vai por
    // variável e o próprio engine o aplica ao seu Job Object, antes de ler a
    // entrada. No Linux o supervisor aplica `RLIMIT_AS` e a variável não é usada.
    if cfg!(windows) {
        command.env(
            "DFIR_ENGINE_MEMORY_LIMIT_BYTES",
            memory_limit_for(config.max_memory_bytes, input_size).to_string(),
        );
    }
    command.current_dir(&trusted_root);
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    apply_memory_limit(
        &mut command,
        memory_limit_for(config.max_memory_bytes, input_size),
    );
    Ok(ProcessHandle::from_child(command.spawn()?))
}

/// Teto de espaço de endereçamento do engine: o configurado mais o tamanho do
/// arquivo de entrada (mapeado em memória pelo scanner).
pub(crate) fn memory_limit_for(base: u64, input_size: u64) -> u64 {
    base.saturating_add(input_size)
}

#[cfg(unix)]
fn apply_memory_limit(command: &mut Command, max_memory_bytes: u64) {
    // `Command` é o `tokio::process::Command`, cujo `pre_exec` é um método
    // inerente (unsafe) em Unix — não precisa do trait `CommandExt` da std.
    let limit = libc::rlimit {
        rlim_cur: max_memory_bytes as libc::rlim_t,
        rlim_max: max_memory_bytes as libc::rlim_t,
    };
    unsafe {
        command.pre_exec(move || {
            if libc::setrlimit(libc::RLIMIT_AS, &limit) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
}

#[cfg(not(unix))]
fn apply_memory_limit(_command: &mut Command, _max_memory_bytes: u64) {}

#[cfg(test)]
mod tests {
    use super::reject_symlink;

    #[test]
    fn accepts_a_regular_file() {
        let dir = tempfile::tempdir().expect("temp dir");
        let file = dir.path().join("evidence.evtx");
        std::fs::write(&file, b"not real evtx bytes").expect("write fixture");
        assert!(reject_symlink(&file).is_ok());
    }

    #[cfg(unix)]
    #[test]
    fn rejects_a_symlinked_leaf() {
        let dir = tempfile::tempdir().expect("temp dir");
        let target = dir.path().join("outside-target");
        std::fs::write(&target, b"secret").expect("write target");
        let link = dir.path().join("evidence.evtx");
        std::os::unix::fs::symlink(&target, &link).expect("create symlink");
        assert!(reject_symlink(&link).is_err());
    }

    #[test]
    fn rejects_a_missing_path() {
        let dir = tempfile::tempdir().expect("temp dir");
        assert!(reject_symlink(&dir.path().join("missing.evtx")).is_err());
    }
}

#[cfg(test)]
mod memory_limit_tests {
    use super::memory_limit_for;

    #[test]
    fn the_limit_covers_the_mapped_input_file_without_overflowing() {
        const GIB: u64 = 1024 * 1024 * 1024;
        assert_eq!(memory_limit_for(GIB, 0), GIB);
        assert_eq!(
            memory_limit_for(GIB, 1100 * 1024 * 1024),
            GIB + 1100 * 1024 * 1024
        );
        assert_eq!(memory_limit_for(u64::MAX, 5), u64::MAX);
    }
}

#[cfg(test)]
mod isolation_tests {
    use super::{ProcessInput, launch_process, reduced_environment};
    use crate::application::engine::EngineRunnerConfig;
    use crate::config::AppConfig;
    use std::ffi::OsString;
    use std::time::Duration;

    fn pairs(items: &[(&str, &str)]) -> Vec<(OsString, OsString)> {
        items
            .iter()
            .map(|(name, value)| (OsString::from(name), OsString::from(value)))
            .collect()
    }

    #[test]
    fn on_unix_the_engine_inherits_nothing() {
        let ambient = pairs(&[
            ("HOME", "/home/jose"),
            ("PATH", "/usr/bin"),
            ("AWS_SECRET_ACCESS_KEY", "segredo"),
            ("SystemRoot", "ignorado fora do Windows"),
        ]);
        assert!(reduced_environment(ambient, false).is_empty());
    }

    #[test]
    fn on_windows_only_the_system_variables_survive_and_path_is_rebuilt() {
        let ambient = pairs(&[
            ("Path", "C:\\Users\\jose\\bin;C:\\Windows"),
            ("SYSTEMROOT", "C:\\Windows"),
            ("SystemDrive", "C:"),
            ("temp", "C:\\Temp"),
            ("USERPROFILE", "C:\\Users\\jose"),
            ("GITHUB_TOKEN", "segredo"),
        ]);
        let reduced = reduced_environment(ambient, true);
        let names: Vec<String> = reduced
            .iter()
            .map(|(name, _)| name.to_string_lossy().to_ascii_uppercase())
            .collect();
        assert!(names.contains(&"SYSTEMROOT".to_string()));
        assert!(names.contains(&"SYSTEMDRIVE".to_string()));
        assert!(names.contains(&"TEMP".to_string()));
        assert!(!names.contains(&"GITHUB_TOKEN".to_string()));
        assert!(!names.contains(&"USERPROFILE".to_string()));
        // O PATH original (com a pasta do usuário) é descartado e refeito.
        let path = reduced
            .iter()
            .find(|(name, _)| name == "PATH")
            .map(|(_, value)| value.to_string_lossy().into_owned())
            .expect("PATH is rebuilt");
        assert!(path.ends_with("System32"), "{path}");
        assert!(!path.contains("jose"), "{path}");
    }

    #[cfg(unix)]
    fn test_services(root: &std::path::Path, engine: &std::path::Path) -> EngineRunnerConfig {
        let config = AppConfig {
            data_root: root.to_path_buf(),
            engine_path: engine.to_path_buf(),
            engine_max_line_bytes: 1024 * 1024,
            engine_timeout: Duration::from_secs(5),
            engine_max_concurrent_jobs: 1,
            // 0 aplicaria RLIMIT_AS = 0 e o `exec` do filho falharia.
            engine_max_memory_bytes: 1024 * 1024 * 1024,
            max_upload_bytes: 1024 * 1024,
            min_free_disk_bytes: 0,
            engine_sandbox: Some("best-effort".into()),
        };
        EngineRunnerConfig::from_config(&config)
    }

    #[cfg(unix)]
    fn write_script(path: &std::path::Path, body: &str) {
        use std::os::unix::fs::PermissionsExt;
        std::fs::write(path, format!("#!/bin/sh\n{body}\n")).expect("write script");
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).expect("chmod");
    }

    /// O engine de verdade recebe só o que o app define, nada do ambiente do app.
    #[cfg(unix)]
    #[tokio::test]
    async fn the_launched_process_does_not_see_the_apps_environment() {
        use tokio::io::AsyncReadExt;
        let root = tempfile::tempdir().expect("root");
        let root_path = root.path().canonicalize().expect("canonical root");
        let input = root_path.join("input");
        let output = root_path.join("output");
        std::fs::create_dir_all(&input).expect("input");
        std::fs::create_dir_all(&output).expect("output");
        let script = root_path.join("fake-engine.sh");
        // Imprime o ambiente recebido e sai.
        write_script(&script, "/usr/bin/env");
        let config = test_services(&root_path, &script);

        let mut handle = launch_process(&config, &ProcessInput::Directory(input), &output, true)
            .expect("launch");
        let mut stdout = handle.take_stdout().expect("stdout");
        let mut text = String::new();
        stdout.read_to_string(&mut text).await.expect("read env");
        let _ = handle.wait(Duration::from_secs(5)).await;

        // O ambiente de quem roda o teste tem PATH e HOME; o filho não os vê.
        assert!(
            std::env::var_os("PATH").is_some(),
            "o teste precisa de PATH no ambiente"
        );
        for forbidden in ["PATH=", "HOME=", "USER=", "SHELL=", "LANG=", "CARGO"] {
            assert!(
                !text.lines().any(|line| line.starts_with(forbidden)),
                "o engine não deve herdar {forbidden}: {text}"
            );
        }
        assert!(text.contains("DFIR_ENGINE_INPUT_ROOT="), "{text}");
        assert!(text.contains("DFIR_ENGINE_OUTPUT_ROOT="), "{text}");
        assert!(text.contains("DFIR_ENGINE_SANDBOX=best-effort"), "{text}");
        assert!(
            !text.contains("DFIR_ENGINE_INPUT_FILE="),
            "sem arquivo, a variável não existe: {text}"
        );
    }

    /// Soltar o handle sem `kill` explícito encerra o filho.
    #[cfg(unix)]
    #[tokio::test]
    async fn dropping_the_handle_terminates_the_engine_process() {
        let root = tempfile::tempdir().expect("root");
        let root_path = root.path().canonicalize().expect("canonical root");
        let input = root_path.join("input");
        let output = root_path.join("output");
        std::fs::create_dir_all(&input).expect("input");
        std::fs::create_dir_all(&output).expect("output");
        let script = root_path.join("sleepy-engine.sh");
        write_script(&script, "exec /bin/sleep 60");
        let config = test_services(&root_path, &script);

        let handle = launch_process(&config, &ProcessInput::Directory(input), &output, true)
            .expect("launch");
        let pid = handle.id().expect("pid while running") as libc::pid_t;
        // SAFETY: sinal 0 só testa a existência do processo.
        assert_eq!(unsafe { libc::kill(pid, 0) }, 0, "o filho deve estar vivo");
        drop(handle);

        let mut gone = false;
        for _ in 0..100 {
            tokio::time::sleep(Duration::from_millis(50)).await;
            // SAFETY: idem; ESRCH significa que o processo foi encerrado e colhido.
            let alive = unsafe { libc::kill(pid, 0) } == 0;
            if !alive {
                gone = true;
                break;
            }
        }
        assert!(gone, "o filho continuou vivo depois de o handle ser solto");
    }
}
