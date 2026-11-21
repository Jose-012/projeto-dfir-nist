use super::{FilePicker, Picked};

#[test]
fn an_idle_picker_has_nothing_to_drain() {
    let mut picker = FilePicker::default();
    assert!(!picker.is_busy());
    assert_eq!(picker.poll(), None);
}

#[test]
fn a_finished_request_is_drained_exactly_once() {
    let (mut picker, sender) = FilePicker::for_test();
    sender
        .send(Picked::Path("/tmp/a.evtx".into()))
        .expect("send");
    assert!(picker.is_busy());
    assert_eq!(picker.poll(), Some(Picked::Path("/tmp/a.evtx".into())));
    assert!(!picker.is_busy());
    assert_eq!(picker.poll(), None);
}

#[test]
fn cancellation_unavailability_and_failure_remain_distinct() {
    for expected in [
        Picked::Cancelled,
        Picked::Unavailable("seletor ausente".into()),
        Picked::Failed("falha específica".into()),
    ] {
        let (mut picker, sender) = FilePicker::for_test();
        sender.send(expected.clone()).expect("send");
        assert_eq!(picker.poll(), Some(expected));
        assert_eq!(picker.poll(), None);
    }
}

#[test]
fn a_dead_worker_is_failure_not_cancellation() {
    let (mut picker, sender) = FilePicker::for_test();
    drop(sender);
    assert!(
        matches!(picker.poll(), Some(Picked::Failed(message)) if message.contains("sem responder"))
    );
    assert!(!picker.is_busy());
}
