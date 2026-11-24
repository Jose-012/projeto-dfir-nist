//! Backend Zenity sem shell, sem timeout de escolha e com captura limitada.

use super::{Picked, Request};
use std::{
    io::ErrorKind,
    process::Child,
    sync::{Arc, Mutex, MutexGuard},
};
#[cfg(target_os = "linux")]
use std::{
    io::Read,
    path::PathBuf,
    process::{Command, ExitStatus, Stdio},
    sync::OnceLock,
    thread,
    time::{Duration, Instant},
};

#[cfg(target_os = "linux")]
const MAX_STDOUT_BYTES: usize = 64 * 1024;
#[cfg(target_os = "linux")]
const MAX_STDERR_BYTES: usize = 16 * 1024;
#[cfg(target_os = "linux")]
const PROBE_TIMEOUT: Duration = Duration::from_secs(2);
#[cfg(target_os = "linux")]
const WAIT_INTERVAL: Duration = Duration::from_millis(10);

#[derive(Default)]
pub(crate) struct ChildState {
    child: Option<Child>,
    cancelled: bool,
}

pub type ChildSlot = Arc<Mutex<ChildState>>;

pub fn child_slot() -> ChildSlot {
    Arc::new(Mutex::new(ChildState::default()))
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Guarda apenas o processo associado a este `FilePicker`.
pub struct ChildGuard {
    slot: ChildSlot,
}

impl ChildGuard {
    pub fn new(slot: ChildSlot) -> Self {
        Self { slot }
    }
}

impl Drop for ChildGuard {
    fn drop(&mut self) {
        terminate(&self.slot);
    }
}

fn kill_and_wait(child: &mut Child) {
    match child.try_wait() {
        Ok(Some(_)) => {}
        Ok(None) => {
            if let Err(error) = child.kill()
                && error.kind() != ErrorKind::InvalidInput
            {
                tracing::debug!(error_kind = ?error.kind(), "could not kill file dialog child");
            }
        }
        Err(error) => {
            tracing::debug!(error_kind = ?error.kind(), "could not poll file dialog child");
            if let Err(kill_error) = child.kill()
                && kill_error.kind() != ErrorKind::InvalidInput
            {
                tracing::debug!(error_kind = ?kill_error.kind(), "could not kill file dialog child after poll error");
            }
        }
    }
    if let Err(error) = child.wait() {
        tracing::warn!(error_kind = ?error.kind(), "could not reap file dialog child");
    }
}

fn terminate(slot: &ChildSlot) {
    let child = {
        let mut state = lock(slot);
        state.cancelled = true;
        state.child.take()
    };
    if let Some(mut child) = child {
        kill_and_wait(&mut child);
    }
}

#[cfg(target_os = "linux")]
fn cached_probe(cell: &OnceLock<bool>, probe: impl FnOnce() -> bool) -> bool {
    *cell.get_or_init(probe)
}

/// Sonda executada no máximo uma vez por processo. O timeout vale apenas para
/// `--version`, nunca para o tempo em que o operador escolhe um arquivo.
#[cfg(target_os = "linux")]
pub fn probe_available() -> bool {
    static AVAILABLE: OnceLock<bool> = OnceLock::new();
    cached_probe(&AVAILABLE, || probe_program("zenity"))
}

#[cfg(target_os = "linux")]
fn probe_program(program: &str) -> bool {
    let mut child = match Command::new(program)
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => child,
        Err(_) => return false,
    };
    let deadline = Instant::now() + PROBE_TIMEOUT;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return status.success(),
            Ok(None) => {}
            Err(_) => {
                kill_and_wait(&mut child);
                return false;
            }
        }
        if Instant::now() >= deadline {
            kill_and_wait(&mut child);
            return false;
        }
        thread::sleep(WAIT_INTERVAL);
    }
}

#[cfg(target_os = "linux")]
fn command(request: &Request) -> Command {
    let mut command = Command::new("zenity");
    command.arg("--no-markup");
    match request {
        Request::OpenFile { title } => {
            command.arg("--title").arg(title).arg("--file-selection");
        }
        Request::SaveFile {
            title,
            file_name,
            filter_name,
            extensions,
        } => {
            let patterns = extensions
                .iter()
                .map(|extension| {
                    if extension.is_empty() || extension == "*" {
                        "*".to_string()
                    } else {
                        format!("*.{extension}")
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");
            command
                .arg("--title")
                .arg(title)
                .arg("--file-selection")
                .arg("--save")
                .arg("--confirm-overwrite")
                .arg("--filename")
                .arg(file_name)
                .arg("--file-filter")
                .arg(format!("{filter_name} | {patterns}"));
        }
        Request::Folder { title } => {
            command
                .arg("--title")
                .arg(title)
                .arg("--file-selection")
                .arg("--directory");
        }
    }
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    command
}

#[cfg(target_os = "linux")]
pub fn run(request: Request, slot: ChildSlot) -> Picked {
    run_command(command(&request), slot)
}

#[cfg(not(target_os = "linux"))]
pub fn run(_request: Request, _slot: ChildSlot) -> Picked {
    Picked::Unavailable("o backend Zenity só está disponível no Linux; use RFD".into())
}

#[cfg(target_os = "linux")]
struct Capture {
    bytes: Vec<u8>,
    overflow: bool,
}

/// Drena até EOF para não bloquear o filho, mas conserva no máximo `limit`
/// bytes. Todo excedente é descartado e transforma o resultado em falha.
#[cfg(target_os = "linux")]
fn read_limited(mut reader: impl Read, limit: usize) -> Capture {
    let mut bytes = Vec::with_capacity(limit.min(8192));
    let mut overflow = false;
    let mut buffer = [0_u8; 8192];
    loop {
        let read = match reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(read) => read,
            Err(error) if error.kind() == ErrorKind::Interrupted => continue,
            Err(_) => {
                overflow = true;
                break;
            }
        };
        let remaining = limit.saturating_sub(bytes.len());
        let take = remaining.min(read);
        bytes.extend_from_slice(&buffer[..take]);
        if take < read {
            overflow = true;
        }
    }
    Capture { bytes, overflow }
}

#[cfg(target_os = "linux")]
fn spawn_reader(
    name: &str,
    reader: impl Read + Send + 'static,
    limit: usize,
) -> std::io::Result<thread::JoinHandle<Capture>> {
    thread::Builder::new()
        .name(name.into())
        .spawn(move || read_limited(reader, limit))
}

#[cfg(target_os = "linux")]
fn wait_for_exit(slot: &ChildSlot) -> Result<ExitStatus, Picked> {
    loop {
        let status = {
            let mut guard = lock(slot);
            let Some(child) = guard.child.as_mut() else {
                return Err(Picked::Failed(
                    "o seletor foi encerrado durante o fechamento da aplicação".into(),
                ));
            };
            match child.try_wait() {
                Ok(Some(status)) => Some(status),
                Ok(None) => None,
                Err(_) => {
                    return Err(Picked::Failed(
                        "não foi possível consultar o estado do seletor".into(),
                    ));
                }
            }
        };
        if let Some(status) = status {
            // `try_wait` já recolheu o processo; apenas retire o handle do slot.
            lock(slot).child.take();
            return Ok(status);
        }
        thread::sleep(WAIT_INTERVAL);
    }
}

#[cfg(target_os = "linux")]
fn run_command(mut command: Command, slot: ChildSlot) -> Picked {
    if lock(&slot).cancelled {
        return Picked::Failed("o seletor foi encerrado antes de iniciar o processo".into());
    }
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(error) if error.kind() == ErrorKind::NotFound => {
            return Picked::Unavailable(
                "Zenity não foi encontrado; instale-o, use RFD ou informe o caminho manualmente"
                    .into(),
            );
        }
        Err(error) => {
            return Picked::Failed(format!(
                "não foi possível iniciar o Zenity ({})",
                error.kind()
            ));
        }
    };
    let (stdout, stderr) = match (child.stdout.take(), child.stderr.take()) {
        (Some(stdout), Some(stderr)) => (stdout, stderr),
        _ => {
            kill_and_wait(&mut child);
            return Picked::Failed("não foi possível capturar a saída do seletor".into());
        }
    };
    let mut state = lock(&slot);
    if state.cancelled {
        drop(state);
        kill_and_wait(&mut child);
        return Picked::Failed("o seletor foi encerrado durante a inicialização".into());
    }
    state.child = Some(child);
    drop(state);
    let stdout_reader = match spawn_reader("dfir-zenity-stdout", stdout, MAX_STDOUT_BYTES) {
        Ok(reader) => reader,
        Err(_) => {
            terminate(&slot);
            return Picked::Failed("não foi possível criar o leitor do seletor".into());
        }
    };
    let stderr_reader = match spawn_reader("dfir-zenity-stderr", stderr, MAX_STDERR_BYTES) {
        Ok(reader) => reader,
        Err(_) => {
            terminate(&slot);
            let _ = stdout_reader.join();
            return Picked::Failed("não foi possível criar o leitor de diagnóstico".into());
        }
    };

    let status = wait_for_exit(&slot);
    if status.is_err() {
        // Se `try_wait` falhar com o filho ainda vivo, os pipes podem não
        // chegar a EOF. Encerre antes dos joins para não prender a thread.
        terminate(&slot);
    }
    let stdout = stdout_reader.join();
    let stderr = stderr_reader.join();
    let status = match status {
        Ok(status) => status,
        Err(picked) => return picked,
    };
    let (stdout, stderr) = match (stdout, stderr) {
        (Ok(stdout), Ok(stderr)) => (stdout, stderr),
        _ => return Picked::Failed("o leitor da saída do seletor falhou".into()),
    };
    if stdout.overflow || stderr.overflow {
        return Picked::Failed("a saída do seletor excedeu o limite de segurança".into());
    }
    interpret_output(status.code(), stdout.bytes)
}

#[cfg(target_os = "linux")]
fn interpret_output(code: Option<i32>, bytes: Vec<u8>) -> Picked {
    match code {
        Some(0) => {
            let text = match String::from_utf8(bytes) {
                Ok(text) => text,
                Err(_) => return Picked::Failed("o seletor devolveu texto inválido".into()),
            };
            let path = text
                .strip_suffix("\r\n")
                .or_else(|| text.strip_suffix('\n'))
                .unwrap_or(&text);
            if path.is_empty() || path.contains(['\n', '\r', '\0']) {
                return Picked::Failed("o seletor devolveu um caminho inválido".into());
            }
            Picked::Path(PathBuf::from(path))
        }
        Some(1) => Picked::Cancelled,
        Some(code) => Picked::Failed(format!(
            "o Zenity terminou com código inesperado {code}; informe o caminho manualmente"
        )),
        None => Picked::Failed("o Zenity terminou por sinal; informe o caminho manualmente".into()),
    }
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;

    fn arguments(request: &Request) -> Vec<String> {
        command(request)
            .get_args()
            .map(|argument| argument.to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn open_folder_and_save_have_explicit_arguments() {
        assert_eq!(
            arguments(&Request::OpenFile {
                title: "Abrir".into()
            }),
            ["--no-markup", "--title", "Abrir", "--file-selection"]
        );
        assert_eq!(
            arguments(&Request::Folder {
                title: "Pasta".into()
            }),
            [
                "--no-markup",
                "--title",
                "Pasta",
                "--file-selection",
                "--directory"
            ]
        );
        let save = arguments(&Request::SaveFile {
            title: "Salvar".into(),
            file_name: "saida.json".into(),
            filter_name: "JSON".into(),
            extensions: vec!["json".into(), "*".into()],
        });
        assert!(save.contains(&"--save".into()));
        assert!(save.contains(&"--confirm-overwrite".into()));
        assert!(save.contains(&"JSON | *.json *".into()));
    }

    #[test]
    fn reader_caps_memory_and_still_drains_the_input() {
        let input = vec![b'x'; MAX_STDOUT_BYTES * 3];
        let capture = read_limited(std::io::Cursor::new(input), MAX_STDOUT_BYTES);
        assert_eq!(capture.bytes.len(), MAX_STDOUT_BYTES);
        assert!(capture.overflow);
    }

    #[test]
    fn exact_limit_is_accepted() {
        let capture = read_limited(std::io::Cursor::new(vec![b'x'; 32]), 32);
        assert_eq!(capture.bytes.len(), 32);
        assert!(!capture.overflow);
    }

    #[test]
    fn probe_is_evaluated_only_once() {
        let cell = OnceLock::new();
        let calls = std::sync::atomic::AtomicUsize::new(0);
        for _ in 0..5 {
            assert!(cached_probe(&cell, || {
                calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                true
            }));
        }
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[test]
    fn outcomes_cover_path_cancel_and_invalid_output() {
        assert_eq!(
            interpret_output(Some(0), b"/tmp/a.evtx\n".to_vec()),
            Picked::Path("/tmp/a.evtx".into())
        );
        assert_eq!(interpret_output(Some(1), Vec::new()), Picked::Cancelled);
        assert!(matches!(
            interpret_output(Some(0), b"a\nb".to_vec()),
            Picked::Failed(_)
        ));
        assert!(matches!(
            interpret_output(Some(0), vec![0xff]),
            Picked::Failed(_)
        ));
        assert!(matches!(
            interpret_output(Some(9), Vec::new()),
            Picked::Failed(_)
        ));
        assert!(matches!(
            interpret_output(None, Vec::new()),
            Picked::Failed(_)
        ));
    }

    #[cfg(unix)]
    #[test]
    fn dropping_the_guard_kills_and_reaps_only_its_child() {
        let child = Command::new("sleep")
            .arg("30")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn sleep");
        let pid = child.id() as i32;
        let slot = child_slot();
        lock(&slot).child = Some(child);
        let guard = ChildGuard::new(slot.clone());
        drop(guard);
        assert!(lock(&slot).child.is_none());
        let result = unsafe { libc::kill(pid, 0) };
        assert_eq!(result, -1, "child should have been reaped");
        assert_eq!(
            std::io::Error::last_os_error().raw_os_error(),
            Some(libc::ESRCH)
        );
    }

    #[cfg(unix)]
    #[test]
    fn cancellation_before_spawn_prevents_an_orphan_child() {
        let slot = child_slot();
        let guard = ChildGuard::new(slot.clone());
        drop(guard);
        let mut command = Command::new("sleep");
        command
            .arg("30")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let outcome = run_command(command, slot);
        assert!(matches!(outcome, Picked::Failed(message) if message.contains("antes de iniciar")));
    }

    #[cfg(unix)]
    #[test]
    fn dropping_the_guard_unblocks_the_output_readers() {
        let slot = child_slot();
        let guard = ChildGuard::new(slot.clone());
        let worker_slot = slot.clone();
        let (sender, receiver) = std::sync::mpsc::channel();
        thread::spawn(move || {
            let mut command = Command::new("sleep");
            command
                .arg("30")
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());
            let outcome = run_command(command, worker_slot);
            let _ = sender.send(outcome);
        });

        let deadline = Instant::now() + Duration::from_secs(2);
        while lock(&slot).child.is_none() {
            assert!(Instant::now() < deadline, "child was not installed");
            thread::sleep(WAIT_INTERVAL);
        }
        drop(guard);

        let outcome = receiver
            .recv_timeout(Duration::from_secs(2))
            .expect("readers must observe EOF after the child is reaped");
        assert!(matches!(outcome, Picked::Failed(_)));
    }
}
