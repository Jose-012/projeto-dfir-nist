//! Logs em arquivo com rotação (ADR-0022).
//!
//! A GUI não tem console: sem arquivo, um erro de campo não deixa rastro. Os
//! logs vão, em JSON, para `<DFIR_DATA_ROOT>/logs/dfir.log` (rotação por
//! tamanho, 5 arquivos de até 5 MiB) e continuam indo para o stderr (a CLI).
//!
//! **O que entra nos logs:** ids (evidência, job), tamanhos, contagens,
//! durações, códigos de erro e o texto dos erros. **O que não entra:** nomes
//! de arquivo de evidência e conteúdo de eventos. O arquivo é local, com
//! permissão 0600 em Unix; é o operador quem decide enviá-lo a alguém.

use std::{
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};
use tracing_subscriber::{
    Layer as _,
    fmt::MakeWriter,
    layer::{Filter, SubscriberExt},
    util::SubscriberInitExt,
};

pub const LOG_FILE_NAME: &str = "dfir.log";
const MAX_LOG_BYTES: u64 = 5 * 1024 * 1024;
const KEEP_FILES: usize = 5;

static SUPPRESSED_EGL_PROBES: AtomicUsize = AtomicUsize::new(0);
static SUPPRESSED_XSETTINGS_WARNINGS: AtomicUsize = AtomicUsize::new(0);

/// O driver Mesa emite `ERROR` ao recusar uma configuração EGL antes de o
/// `wgpu` tentar a próxima (robust access core/ext/none). Essas sondas não são
/// falha do aplicativo quando um adaptador é obtido. O aviso de XSETTINGS tem
/// escopo semelhante em sessões sem o serviço: o tema da aplicação continua
/// funcional, só o reload dinâmico de Xft fica indisponível.
///
/// A política padrão resume somente as mensagens exatas conhecidas. Qualquer
/// `RUST_LOG` explícito desativa o resumo e preserva todos os eventos brutos.
#[derive(Clone, Copy)]
struct RecoverableProbeFilter {
    suppress: bool,
}

impl RecoverableProbeFilter {
    fn new(suppress: bool) -> Self {
        Self { suppress }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProbeKind {
    Egl,
    Xsettings,
}

fn recoverable_probe_message(
    target: &str,
    level: tracing::Level,
    message: &str,
) -> Option<ProbeKind> {
    if target == "wgpu_hal::gles::egl"
        && level == tracing::Level::ERROR
        && matches!(
            message,
            "EGL 'eglCreateContext' code 0x3009: eglCreateContext"
                | "EGL 'eglCreateContext' code 0x3005: eglCreateContext"
        )
    {
        return Some(ProbeKind::Egl);
    }
    if target == "winit::platform_impl::linux::x11::xdisplay"
        && level == tracing::Level::WARN
        && message == "error setting XSETTINGS; Xft options won't reload automatically"
    {
        return Some(ProbeKind::Xsettings);
    }
    None
}

fn recoverable_probe(metadata: &tracing::Metadata<'_>, message: &str) -> Option<ProbeKind> {
    recoverable_probe_message(metadata.target(), *metadata.level(), message)
}

#[derive(Default)]
struct MessageVisitor(String);

impl tracing::field::Visit for MessageVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.0 = format!("{value:?}");
        }
    }
}

impl<S> Filter<S> for RecoverableProbeFilter {
    fn enabled(
        &self,
        _metadata: &tracing::Metadata<'_>,
        _cx: &tracing_subscriber::layer::Context<'_, S>,
    ) -> bool {
        true
    }

    fn event_enabled(
        &self,
        event: &tracing::Event<'_>,
        _cx: &tracing_subscriber::layer::Context<'_, S>,
    ) -> bool {
        if !self.suppress {
            return true;
        }
        let mut visitor = MessageVisitor::default();
        event.record(&mut visitor);
        match recoverable_probe(event.metadata(), &visitor.0) {
            Some(ProbeKind::Egl) => {
                SUPPRESSED_EGL_PROBES.fetch_add(1, Ordering::Relaxed);
                false
            }
            Some(ProbeKind::Xsettings) => {
                SUPPRESSED_XSETTINGS_WARNINGS.fetch_add(1, Ordering::Relaxed);
                false
            }
            None => true,
        }
    }
}

/// Contadores para a UI informar, depois de obter um adaptador, que as sondas
/// recuperáveis foram resumidas. Contagens não contêm dados de evidência.
pub(crate) fn suppressed_native_probe_counts() -> (usize, usize) {
    (
        SUPPRESSED_EGL_PROBES.load(Ordering::Relaxed),
        SUPPRESSED_XSETTINGS_WARNINGS.load(Ordering::Relaxed),
    )
}

struct State {
    dir: PathBuf,
    file: Option<File>,
    size: u64,
    max_bytes: u64,
    keep: usize,
}

impl State {
    fn path(&self, index: usize) -> PathBuf {
        if index == 0 {
            self.dir.join(LOG_FILE_NAME)
        } else {
            self.dir.join(format!("{LOG_FILE_NAME}.{index}"))
        }
    }

    fn open_current(&mut self) -> io::Result<()> {
        let mut options = OpenOptions::new();
        options.create(true).append(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let file = options.open(self.path(0))?;
        self.size = file.metadata()?.len();
        self.file = Some(file);
        Ok(())
    }

    /// `dfir.log` → `.1` → … → `.{keep-1}`; o mais antigo é apagado.
    fn rotate(&mut self) -> io::Result<()> {
        self.file = None;
        // F1-23: se o mais antigo não existe (primeira rotação), `NotFound` é
        // esperado; outra falha é registrada em vez de engolida.
        if let Err(error) = fs::remove_file(self.path(self.keep - 1))
            && error.kind() != std::io::ErrorKind::NotFound
        {
            eprintln!("log rotation: could not remove oldest log: {error}");
        }
        for index in (0..self.keep - 1).rev() {
            let from = self.path(index);
            if from.exists() {
                fs::rename(&from, self.path(index + 1))?;
            }
        }
        self.open_current()
    }

    fn write(&mut self, buf: &[u8]) -> io::Result<()> {
        if self.file.is_none() {
            self.open_current()?;
        }
        if self.size > 0 && self.size + buf.len() as u64 > self.max_bytes {
            self.rotate()?;
        }
        if let Some(file) = self.file.as_mut() {
            file.write_all(buf)?;
            self.size += buf.len() as u64;
        }
        Ok(())
    }
}

/// Arquivo de log rotativo, compartilhável entre threads.
#[derive(Clone)]
pub struct RotatingLog {
    state: Arc<Mutex<State>>,
}

impl RotatingLog {
    pub fn open(dir: &Path, max_bytes: u64, keep: usize) -> io::Result<Self> {
        fs::create_dir_all(dir)?;
        let mut state = State {
            dir: dir.to_path_buf(),
            file: None,
            size: 0,
            max_bytes,
            keep: keep.max(2),
        };
        state.open_current()?;
        Ok(Self {
            state: Arc::new(Mutex::new(state)),
        })
    }
}

pub struct LogWriter(Arc<Mutex<State>>);

impl Write for LogWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // Falha de log nunca derruba a aplicação nem vira ruído: descarta.
        // F1-23: o descarte é intencional aqui (o log não pode ser a causa da
        // falha da aplicação), então registramos em nível trace para
        // auditabilidade sem ruído.
        if let Ok(mut state) = self.0.lock()
            && let Err(error) = state.write(buf)
        {
            tracing::trace!(%error, "log write dropped");
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        if let Ok(mut state) = self.0.lock()
            && let Some(file) = state.file.as_mut()
            && let Err(error) = file.flush()
        {
            tracing::trace!(%error, "log flush dropped");
        }
        Ok(())
    }
}

impl<'a> MakeWriter<'a> for RotatingLog {
    type Writer = LogWriter;

    fn make_writer(&'a self) -> Self::Writer {
        LogWriter(Arc::clone(&self.state))
    }
}

/// Inicializa o `tracing`: stderr (JSON) e, se possível, arquivo rotativo.
/// Devolve o caminho do arquivo de log, quando ele está ativo. Filtro por
/// `RUST_LOG` (padrão `info`).
pub fn init(logs_dir: &Path) -> Option<PathBuf> {
    let filter =
        tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into());
    let suppress_recoverable_probes = std::env::var_os("RUST_LOG").is_none();
    let stderr_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_writer(std::io::stderr)
        .with_filter(RecoverableProbeFilter::new(suppress_recoverable_probes));
    let (log_path, file_error) = match RotatingLog::open(logs_dir, MAX_LOG_BYTES, KEEP_FILES) {
        Ok(log) => {
            tracing_subscriber::registry()
                .with(filter)
                .with(stderr_layer)
                .with(
                    tracing_subscriber::fmt::layer()
                        .json()
                        .with_ansi(false)
                        .with_writer(log)
                        .with_filter(RecoverableProbeFilter::new(suppress_recoverable_probes)),
                )
                .init();
            (Some(logs_dir.join(LOG_FILE_NAME)), None)
        }
        Err(error) => {
            tracing_subscriber::registry()
                .with(filter)
                .with(stderr_layer)
                .init();
            (None, Some(error))
        }
    };
    install_panic_hook();
    if let Some(error) = file_error {
        tracing::warn!(%error, "could not open the log file; logging to stderr only");
    }
    log_path
}

/// Registra panics no log (antes só iam para o stderr, invisível na GUI).
fn install_panic_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let location = info
            .location()
            .map(|location| format!("{}:{}", location.file(), location.line()));
        let message = info
            .payload()
            .downcast_ref::<&str>()
            .map(|text| (*text).to_string())
            .or_else(|| info.payload().downcast_ref::<String>().cloned());
        tracing::error!(
            location = ?location,
            message = ?message,
            thread = ?std::thread::current().name(),
            "panic"
        );
        default_hook(info);
    }));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_line(log: &RotatingLog, text: &str) {
        let mut writer = log.make_writer();
        writer.write_all(text.as_bytes()).expect("write");
    }

    #[test]
    fn lines_are_appended_to_the_current_file() {
        let dir = tempfile::tempdir().expect("dir");
        let log = RotatingLog::open(dir.path(), 1024, 3).expect("open");
        write_line(&log, "a\n");
        write_line(&log, "b\n");
        let text = fs::read_to_string(dir.path().join(LOG_FILE_NAME)).expect("read");
        assert_eq!(text, "a\nb\n");
    }

    #[test]
    fn the_file_rotates_by_size_and_keeps_a_bounded_number_of_files() {
        let dir = tempfile::tempdir().expect("dir");
        let log = RotatingLog::open(dir.path(), 20, 3).expect("open");
        for index in 0..10 {
            write_line(&log, &format!("linha-{index:02}-xxxxx\n")); // 15 bytes
        }
        let names: Vec<String> = fs::read_dir(dir.path())
            .expect("list")
            .map(|entry| {
                entry
                    .expect("entry")
                    .file_name()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();
        assert!(names.contains(&LOG_FILE_NAME.to_string()));
        assert!(names.contains(&format!("{LOG_FILE_NAME}.1")));
        assert!(names.contains(&format!("{LOG_FILE_NAME}.2")));
        assert!(
            !names.contains(&format!("{LOG_FILE_NAME}.3")),
            "no máximo `keep` arquivos: {names:?}"
        );
        // A linha mais recente está no arquivo atual; a mais antiga já saiu.
        let current = fs::read_to_string(dir.path().join(LOG_FILE_NAME)).expect("read");
        assert!(current.contains("linha-09"));
        let all: String = names
            .iter()
            .map(|name| fs::read_to_string(dir.path().join(name)).unwrap_or_default())
            .collect();
        assert!(
            !all.contains("linha-00"),
            "o log mais antigo foi descartado"
        );
    }

    #[cfg(unix)]
    #[test]
    fn the_log_file_is_private_to_the_owner() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().expect("dir");
        let _log = RotatingLog::open(dir.path(), 1024, 3).expect("open");
        let mode = fs::metadata(dir.path().join(LOG_FILE_NAME))
            .expect("metadata")
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600);
    }

    #[test]
    fn an_existing_log_is_appended_not_truncated_across_restarts() {
        let dir = tempfile::tempdir().expect("dir");
        write_line(
            &RotatingLog::open(dir.path(), 1024, 3).expect("open"),
            "um\n",
        );
        write_line(
            &RotatingLog::open(dir.path(), 1024, 3).expect("reopen"),
            "dois\n",
        );
        let text = fs::read_to_string(dir.path().join(LOG_FILE_NAME)).expect("read");
        assert_eq!(text, "um\ndois\n");
    }

    #[test]
    fn only_the_two_known_egl_probe_codes_are_summarized() {
        for code in ["0x3009", "0x3005"] {
            let message = format!("EGL 'eglCreateContext' code {code}: eglCreateContext");
            assert_eq!(
                recoverable_probe_message("wgpu_hal::gles::egl", tracing::Level::ERROR, &message,),
                Some(ProbeKind::Egl)
            );
        }
        assert_eq!(
            recoverable_probe_message(
                "wgpu_hal::gles::egl",
                tracing::Level::ERROR,
                "EGL 'eglCreateContext' code 0x3001: eglCreateContext",
            ),
            None
        );
        assert_eq!(
            recoverable_probe_message(
                "wgpu_hal::gles::egl",
                tracing::Level::ERROR,
                "EGL 'eglCreateWindowSurface' code 0x3009: eglCreateWindowSurface",
            ),
            None
        );
    }

    #[test]
    fn unrelated_targets_levels_and_xsettings_messages_remain_visible() {
        assert_eq!(
            recoverable_probe_message(
                "dfir_app::gui",
                tracing::Level::ERROR,
                "EGL 'eglCreateContext' code 0x3009: eglCreateContext",
            ),
            None
        );
        assert_eq!(
            recoverable_probe_message(
                "wgpu_hal::gles::egl",
                tracing::Level::WARN,
                "EGL 'eglCreateContext' code 0x3009: eglCreateContext",
            ),
            None
        );
        assert_eq!(
            recoverable_probe_message(
                "winit::platform_impl::linux::x11::xdisplay",
                tracing::Level::WARN,
                "error setting XSETTINGS; Xft options won't reload automatically",
            ),
            Some(ProbeKind::Xsettings)
        );
        assert_eq!(
            recoverable_probe_message(
                "winit::platform_impl::linux::x11::xdisplay",
                tracing::Level::WARN,
                "another XSETTINGS failure",
            ),
            None
        );
    }
}
