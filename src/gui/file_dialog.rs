//! Diálogos fora da thread da UI (GUI-01).
//!
//! No Linux, `auto` evita a espera portal→fallback do `rfd`: uma sonda curta e
//! cacheada escolhe Zenity direto quando disponível. `DFIR_FILE_DIALOG_BACKEND`
//! aceita `auto`, `zenity` e `rfd`. O caminho escolhido continua sendo validado
//! pelos chamadores; logs nunca incluem nome/caminho de arquivo ou saída do
//! processo.

mod backend;
mod rfd_legacy;
mod zenity;

use std::{
    path::PathBuf,
    sync::mpsc::{Receiver, TryRecvError, channel},
};

#[cfg(test)]
use std::sync::mpsc::Sender;

use super::runtime::UiWake;

#[derive(Debug, Clone)]
pub enum Request {
    OpenFile {
        title: String,
    },
    SaveFile {
        title: String,
        file_name: String,
        filter_name: String,
        extensions: Vec<String>,
    },
    Folder {
        title: String,
    },
}

/// Resultado explícito do backend. O `rfd` legado tem uma limitação residual
/// para `None`, documentada em `rfd_legacy`; o Zenity distingue os quatro casos.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Picked {
    Path(PathBuf),
    Cancelled,
    Unavailable(String),
    Failed(String),
}

impl Picked {
    fn outcome_label(&self) -> &'static str {
        match self {
            Self::Path(_) => "path",
            Self::Cancelled => "cancelled",
            Self::Unavailable(_) => "unavailable",
            Self::Failed(_) => "failed",
        }
    }
}

/// Um diálogo por vez. O guard compartilhado contém somente o estado de
/// cancelamento e o filho desta instância; derrubar o picker o encerra e o recolhe.
#[derive(Default)]
pub struct FilePicker {
    receiver: Option<Receiver<Picked>>,
    child: Option<zenity::ChildGuard>,
    wake: UiWake,
    busy: bool,
}

impl FilePicker {
    #[cfg(test)]
    pub(crate) fn for_test() -> (Self, Sender<Picked>) {
        let (sender, receiver) = channel();
        (
            Self {
                receiver: Some(receiver),
                child: None,
                wake: UiWake::default(),
                busy: true,
            },
            sender,
        )
    }

    pub fn is_busy(&self) -> bool {
        self.busy
    }

    pub fn open(&mut self, request: Request, context: egui::Context) -> Result<(), String> {
        if self.busy {
            return Ok(());
        }
        let preference = backend::from_env()
            .map_err(|message| format!("seletor de arquivos indisponível: {message}"))?;
        self.wake.set_context(context);
        let (sender, receiver) = channel();
        let wake = self.wake.clone();
        let child_slot = zenity::child_slot();
        let worker_slot = child_slot.clone();
        let child = zenity::ChildGuard::new(child_slot);
        let started = std::time::Instant::now();
        let requested_at = chrono::Utc::now().to_rfc3339();
        std::thread::Builder::new()
            .name("dfir-file-dialog".into())
            .spawn(move || {
                tracing::info!(
                    requested_at = %requested_at,
                    backend = preference.label(),
                    elapsed_ms = 0,
                    outcome = "requested",
                    "file dialog requested"
                );
                let backend = preference.resolve();
                let outcome = backend.run(request, worker_slot);
                tracing::info!(
                    requested_at = %requested_at,
                    backend = backend.label(),
                    elapsed_ms = started.elapsed().as_millis() as u64,
                    outcome = outcome.outcome_label(),
                    "file dialog finished"
                );
                if sender.send(outcome).is_err() {
                    tracing::debug!("file dialog result dropped: UI no longer waiting");
                } else {
                    wake.notify();
                }
            })
            .map_err(|error| format!("não foi possível abrir o seletor de arquivos: {error}"))?;
        self.receiver = Some(receiver);
        self.child = Some(child);
        self.busy = true;
        Ok(())
    }

    /// `None` significa ainda em andamento. Canal morto é falha, não cancelamento.
    pub fn poll(&mut self) -> Option<Picked> {
        let receiver = self.receiver.as_ref()?;
        let outcome = match receiver.try_recv() {
            Ok(picked) => picked,
            Err(TryRecvError::Empty) => return None,
            Err(TryRecvError::Disconnected) => Picked::Failed(
                "o seletor de arquivos encerrou sem responder; informe o caminho manualmente"
                    .into(),
            ),
        };
        self.receiver = None;
        self.child = None;
        self.busy = false;
        Some(outcome)
    }
}

impl Drop for FilePicker {
    fn drop(&mut self) {
        self.child.take();
    }
}

#[cfg(test)]
mod tests;
