//! Wakeup compartilhado entre produtores assíncronos e a thread do egui.
//!
//! `UiWake` é `Send + Sync`: backend e seletor notificam depois de entregar
//! um resultado no canal, e a UI chama [`UiWake::acknowledge`] imediatamente
//! antes de drená-lo. Não há timer periódico em idle nem `ControlFlow::Poll`.
//!
//! Corrida acknowledge/dreno: `acknowledge` limpa `pending` ANTES de a UI
//! drenar o canal. Um `notify` que chegue durante a drenagem faz
//! `pending.swap(true)` devolver `false` (estava limpo) e pede um novo
//! `request_repaint`, garantindo um quadro seguinte para o evento que acabou
//! de entrar. Um `notify` cujo `swap` devolve `true` foi coalescido em um
//! wakeup já pendente, que esse mesmo quadro drena junto. Em ambos os casos
//! nenhum evento fica sem quadro correspondente.
use std::sync::{
    Arc, Mutex, MutexGuard,
    atomic::{AtomicBool, Ordering},
};

#[derive(Clone, Default)]
pub struct UiWake {
    inner: Arc<Inner>,
}

#[derive(Default)]
struct Inner {
    context: Mutex<Option<egui::Context>>,
    /// Verdadeiro desde o primeiro evento ainda não drenado. Além de coalescer
    /// uma rajada, preserva eventos que chegam antes do `egui::Context`.
    pending: AtomicBool,
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    // O valor guardado continua válido se outra thread panicou com o lock.
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

impl UiWake {
    /// Registra o contexto real depois da criação do eframe. Se um evento já
    /// estava pendente, a primeira notificação não se perde.
    pub fn set_context(&self, context: egui::Context) {
        *lock(&self.inner.context) = Some(context.clone());
        if self.inner.pending.load(Ordering::Acquire) {
            context.request_repaint();
        }
    }

    /// Coalesce notificações ainda não drenadas. O primeiro evento pede um
    /// repaint; os seguintes ficam representados pelo mesmo estado pendente.
    pub fn notify(&self) {
        if self.inner.pending.swap(true, Ordering::AcqRel) {
            return;
        }
        let context = lock(&self.inner.context).clone();
        if let Some(context) = context {
            context.request_repaint();
        }
    }

    /// Deve ser chamado ANTES de drenar o canal. Um evento que chegar durante
    /// ou depois da drenagem volta a marcar `pending` (o `swap` em `notify`
    /// vê `false`) e pede outro quadro; um evento coalescido em wakeup já
    /// pendente é drenado neste mesmo quadro.
    pub fn acknowledge(&self) {
        self.inner.pending.store(false, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use super::UiWake;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    fn repaint_count(context: &egui::Context) -> Arc<AtomicUsize> {
        let count = Arc::new(AtomicUsize::new(0));
        let observed = count.clone();
        context.set_repaint_observer(move |_info| {
            observed.fetch_add(1, Ordering::SeqCst);
        });
        count
    }

    #[test]
    fn wake_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<UiWake>();
    }

    #[test]
    fn events_before_context_are_not_lost() {
        let wake = UiWake::default();
        wake.notify();
        wake.notify();
        let context = egui::Context::default();
        let count = repaint_count(&context);

        wake.set_context(context.clone());

        assert_eq!(count.load(Ordering::SeqCst), 1);
        assert!(context.has_requested_repaint());
    }

    #[test]
    fn bursts_coalesce_until_the_ui_acknowledges() {
        let wake = UiWake::default();
        let context = egui::Context::default();
        let count = repaint_count(&context);
        wake.set_context(context);

        for _ in 0..50 {
            wake.notify();
        }
        assert_eq!(count.load(Ordering::SeqCst), 1);

        wake.acknowledge();
        wake.notify();
        assert_eq!(count.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn replacing_context_preserves_a_pending_wakeup() {
        let wake = UiWake::default();
        let first = egui::Context::default();
        let first_count = repaint_count(&first);
        wake.set_context(first);
        wake.notify();
        assert_eq!(first_count.load(Ordering::SeqCst), 1);

        let second = egui::Context::default();
        let second_count = repaint_count(&second);
        wake.set_context(second);
        assert_eq!(second_count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn acknowledge_without_context_keeps_the_next_event() {
        let wake = UiWake::default();
        wake.notify();
        wake.acknowledge();
        let context = egui::Context::default();
        let count = repaint_count(&context);
        wake.set_context(context);
        assert_eq!(count.load(Ordering::SeqCst), 0);

        wake.notify();
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }
}
