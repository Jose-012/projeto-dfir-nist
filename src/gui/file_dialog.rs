//! Diálogos nativos de arquivo que **nunca bloqueiam a thread da UI**
//! (GUI-01).
//!
//! `rfd::FileDialog` é uma chamada bloqueante. No Linux ela depende do
//! `xdg-desktop-portal` (via D-Bus) e, quando o portal não responde, cai para
//! `zenity`; se nenhum dos dois existe, a chamada pode esperar o timeout do
//! D-Bus (~25 s) antes de devolver `None`. Executada dentro de
//! `eframe::App::ui()`, isso congela a janela inteira — e o operador não vê
//! erro nenhum, porque `rfd` devolve `None` tanto para cancelamento quanto
//! para falha do seletor.
//!
//! Por isso o diálogo roda em uma thread própria e o resultado volta por
//! canal, drenado a cada quadro. A interface continua responsiva mesmo sem
//! portal/zenity, e há dois caminhos que não dependem de diálogo: digitar o
//! caminho e arrastar o arquivo para a janela.
//!
//! A thread é criada com [`std::thread::Builder`] e seu erro é propagado
//! (nunca engolido): se o sistema não puder criar a thread, o chamador mostra
//! o motivo em vez de deixar o botão "não fazer nada".

use std::{
    path::PathBuf,
    sync::mpsc::{Receiver, TryRecvError, channel},
};

/// Título do diálogo e modo de escolha.
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

impl Request {
    fn run(self) -> Option<PathBuf> {
        match self {
            Self::OpenFile { title } => rfd::FileDialog::new().set_title(title).pick_file(),
            Self::SaveFile {
                title,
                file_name,
                filter_name,
                extensions,
            } => {
                let extensions: Vec<&str> = extensions.iter().map(String::as_str).collect();
                rfd::FileDialog::new()
                    .set_title(title)
                    .set_file_name(file_name)
                    .add_filter(filter_name, &extensions)
                    .save_file()
            }
            Self::Folder { title } => rfd::FileDialog::new().set_title(title).pick_folder(),
        }
    }
}

/// Resultado de um diálogo já concluído.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Picked {
    Path(PathBuf),
    /// Cancelado pelo operador **ou** seletor indisponível: `rfd` não
    /// distingue os dois casos, então a interface trata ambos como "nada
    /// escolhido" e oferece os caminhos alternativos.
    Nothing,
}

/// Um diálogo por vez, executado fora da thread da UI.
#[derive(Default)]
pub struct FilePicker {
    receiver: Option<Receiver<Option<PathBuf>>>,
    /// O diálogo foi pedido mas o resultado ainda não chegou. Diferente de
    /// `receiver.is_some()` só em caso de falha ao criar a thread.
    busy: bool,
}

impl FilePicker {
    /// Canal controlado pelo teste, sem abrir um diálogo nativo nem exigir
    /// display. O emissor permite simular escolha, cancelamento e thread morta.
    #[cfg(test)]
    pub(crate) fn for_test() -> (Self, std::sync::mpsc::Sender<Option<PathBuf>>) {
        let (sender, receiver) = channel();
        (
            Self {
                receiver: Some(receiver),
                busy: true,
            },
            sender,
        )
    }

    pub fn is_busy(&self) -> bool {
        self.busy
    }

    /// Pede um diálogo. Devolve `Err` se a thread não pôde ser criada.
    pub fn open(&mut self, request: Request) -> Result<(), String> {
        if self.busy {
            return Ok(());
        }
        let (sender, receiver) = channel();
        std::thread::Builder::new()
            .name("dfir-file-dialog".into())
            .spawn(move || {
                // O operador cancelar ou o seletor falhar levam ao mesmo
                // `None`; o `send` pode falhar apenas se a interface já não
                // estiver interessada no resultado (F1-23: registrar em
                // vez de engolir).
                if sender.send(request.run()).is_err() {
                    tracing::debug!("file dialog result dropped: UI no longer waiting");
                }
            })
            .map_err(|error| format!("não foi possível abrir o seletor de arquivos: {error}"))?;
        self.receiver = Some(receiver);
        self.busy = true;
        Ok(())
    }

    /// Drena o resultado, se houver. `None` = ainda em andamento.
    pub fn poll(&mut self) -> Option<Picked> {
        let receiver = self.receiver.as_ref()?;
        match receiver.try_recv() {
            Ok(Some(path)) => {
                self.receiver = None;
                self.busy = false;
                Some(Picked::Path(path))
            }
            Ok(None) => {
                self.receiver = None;
                self.busy = false;
                Some(Picked::Nothing)
            }
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => {
                // A thread do diálogo morreu sem enviar: não é cancelamento
                // silencioso, é falha; o mesmo caminho alternativo se aplica.
                self.receiver = None;
                self.busy = false;
                Some(Picked::Nothing)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::FilePicker;

    #[test]
    fn an_idle_picker_has_nothing_to_drain() {
        let mut picker = FilePicker::default();
        assert!(!picker.is_busy());
        assert_eq!(picker.poll(), None);
    }

    #[test]
    fn a_finished_request_is_drained_exactly_once() {
        let (sender, receiver) = std::sync::mpsc::channel();
        sender
            .send(Some(std::path::PathBuf::from("/tmp/a.evtx")))
            .expect("send");
        let mut picker = FilePicker {
            receiver: Some(receiver),
            busy: true,
        };
        assert!(picker.is_busy(), "em andamento antes do resultado");
        assert_eq!(
            picker.poll(),
            Some(super::Picked::Path("/tmp/a.evtx".into()))
        );
        assert!(!picker.is_busy(), "concluído");
        assert_eq!(picker.poll(), None, "o resultado é consumido uma só vez");
    }

    #[test]
    fn an_empty_channel_is_pending_and_cancellation_finishes_without_a_path() {
        let (sender, receiver) = std::sync::mpsc::channel();
        let mut picker = FilePicker {
            receiver: Some(receiver),
            busy: true,
        };
        assert_eq!(picker.poll(), None, "sem resultado, continua em andamento");
        assert!(picker.is_busy());

        sender.send(None).expect("cancelamento");
        assert_eq!(picker.poll(), Some(super::Picked::Nothing));
        assert!(!picker.is_busy());
        assert_eq!(
            picker.poll(),
            None,
            "cancelamento também é consumido uma vez"
        );
    }

    #[test]
    fn a_dead_worker_ends_the_request_without_inventing_a_path() {
        let (sender, receiver) = std::sync::mpsc::channel::<Option<std::path::PathBuf>>();
        drop(sender);
        let mut picker = FilePicker {
            receiver: Some(receiver),
            busy: true,
        };
        assert_eq!(picker.poll(), Some(super::Picked::Nothing));
        assert!(!picker.is_busy());
    }
}
