//! Prova de contenção do engine (F1-22, ADR-0024), de ponta a ponta: sobe o
//! binário real, de verdade, como o app o sobe, e confere o que ele consegue
//! e o que não consegue fazer depois de contido.
//!
//! Em Linux sem Landlock (kernel < 5.13 ou LSM desligado) os testes de
//! enforcement avisam e saem; o resto continua valendo.

#![cfg(target_os = "linux")]

use dfir_protocol::{PROTOCOL_VERSION, SCHEMA_VERSION};
use serde_json::{Value, json};
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

const EACCES: i64 = 13;
const EPERM: i64 = 1;

fn engine() -> Command {
    // O nextest informa o binário real em runtime e remapeia esse caminho ao
    // reutilizar builds. O `env!` é fallback para o `cargo test`.
    let executable = std::env::var_os("NEXTEST_BIN_EXE_dfir_engine")
        .or_else(|| std::env::var_os("CARGO_BIN_EXE_dfir-engine"))
        .unwrap_or_else(|| env!("CARGO_BIN_EXE_dfir-engine").into());
    let mut command = Command::new(executable);
    // Ambiente limpo: o resultado não depende do shell de quem roda o teste.
    command.env_clear();
    command
}

/// O kernel tem Landlock? Mesma pergunta que o engine faz.
fn kernel_has_landlock() -> bool {
    // SAFETY: syscall sem ponteiros (attr nulo, tamanho 0, flag de versão).
    let abi = unsafe { libc::syscall(444, std::ptr::null::<u8>(), 0_usize, 1_u32) };
    abi >= 1
}

fn selftest(pinned: Option<&Path>, policy: &str) -> Value {
    let mut command = engine();
    command
        .arg("--sandbox-selftest")
        .env("DFIR_ENGINE_SANDBOX", policy);
    if let Some(pinned) = pinned {
        command.env("DFIR_ENGINE_INPUT_FILE", pinned);
    }
    let output = command.output().expect("engine runs");
    let text = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(text.trim()).unwrap_or_else(|error| {
        panic!(
            "selftest did not print JSON ({error}): stdout={text:?} stderr={:?}",
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

fn request(input_reference: &str, kind: &str) -> String {
    json!({
        "protocol_version": PROTOCOL_VERSION,
        "schema_version": SCHEMA_VERSION,
        "message_type": "REQUEST",
        "request_id": "req-1",
        "job_id": "job-1",
        "input_reference": input_reference,
        "output_reference": null,
        "max_output_bytes": 1_000_000,
        "max_items": 1_000,
        "analysis_kind": kind,
    })
    .to_string()
}

/// Roda o engine em modo stdio com uma requisição e devolve as linhas JSON.
fn run_stdio(
    envs: &[(&str, &Path)],
    policy: &str,
    input_reference: &str,
    kind: &str,
) -> Vec<Value> {
    let mut command = engine();
    command
        .env("DFIR_ENGINE_SANDBOX", policy)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for (key, value) in envs {
        command.env(key, value);
    }
    let mut child = command.spawn().expect("spawn engine");
    {
        let mut stdin = child.stdin.take().expect("stdin");
        writeln!(stdin, "{}", request(input_reference, kind)).expect("write request");
    }
    let output = child.wait_with_output().expect("engine exits at EOF");
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("each stdout line is JSON"))
        .collect()
}

#[test]
fn a_contained_engine_reads_only_its_pinned_file_and_nothing_else() {
    if !kernel_has_landlock() {
        eprintln!("SKIPPED: this kernel has no Landlock; enforcement cannot be proven here");
        return;
    }
    let dir = tempfile::tempdir().expect("dir");
    let pinned = dir.path().join("job.evtx");
    std::fs::write(&pinned, b"conteudo da evidencia").expect("write");

    let report = selftest(Some(&pinned), "required");
    assert_eq!(report["fully_applied"], true, "{report}");
    let probes = &report["probes"];

    // Permitido: o arquivo fixado, threads e memória.
    assert_eq!(probes["read_pinned_file"], 0, "{report}");
    assert_eq!(probes["thread_ok"], true, "{report}");
    assert_eq!(probes["allocation_ok"], true, "{report}");
    // Negado pelo Landlock: qualquer outro arquivo ou pasta, e criar arquivo.
    assert_eq!(probes["read_other_file"], EACCES, "{report}");
    assert_eq!(probes["list_other_dir"], EACCES, "{report}");
    assert_eq!(probes["create_file"], EACCES, "{report}");
    // Negado pelo seccomp: rede, sockets Unix, exec e ptrace.
    assert_eq!(probes["tcp_socket"], EPERM, "{report}");
    assert_eq!(probes["unix_socket"], EPERM, "{report}");
    // Exceção deliberada: o par anônimo do runtime (ver `sandbox::denied`).
    assert_eq!(probes["socketpair"], 0, "{report}");
    assert_eq!(probes["execve"], EPERM, "{report}");
    assert_eq!(probes["ptrace"], EPERM, "{report}");
}

#[test]
fn with_the_sandbox_off_the_same_probes_succeed_so_the_test_is_not_vacuous() {
    let dir = tempfile::tempdir().expect("dir");
    let pinned = dir.path().join("job.evtx");
    std::fs::write(&pinned, b"x").expect("write");
    let report = selftest(Some(&pinned), "off");
    assert_eq!(report["fully_applied"], false, "{report}");
    let probes = &report["probes"];
    assert_eq!(
        probes["read_other_file"], 0,
        "controle: sem sandbox, lê /etc/passwd: {report}"
    );
    assert_eq!(
        probes["tcp_socket"], 0,
        "controle: sem sandbox, cria socket: {report}"
    );
}

#[test]
fn the_environment_of_the_test_does_not_change_the_outcome() {
    // `env_clear` no helper: nada de HOME, PATH ou LANG, e a contenção vale.
    if !kernel_has_landlock() {
        return;
    }
    let dir = tempfile::tempdir().expect("dir");
    let pinned = dir.path().join("a.evtx");
    std::fs::write(&pinned, b"x").expect("write");
    let report = selftest(Some(&pinned), "required");
    assert!(
        report["sandbox"]
            .as_str()
            .unwrap()
            .starts_with("policy=required;nnp=on;landlock=abi")
    );
}

#[test]
fn required_policy_fails_closed_and_analyses_nothing_when_a_layer_cannot_be_applied() {
    // Arquivo fixado inexistente: a regra do Landlock não pode ser criada, a
    // camada fica indisponível e a política `required` recusa tudo.
    let dir = tempfile::tempdir().expect("dir");
    let missing = dir.path().join("nao-existe.evtx");
    let messages = run_stdio(
        &[
            ("DFIR_ENGINE_INPUT_ROOT", dir.path()),
            ("DFIR_ENGINE_INPUT_FILE", &missing),
        ],
        "required",
        "nao-existe.evtx",
        "EVTX_TIMELINE",
    );
    let error = messages
        .iter()
        .find(|message| message["message_type"] == "ERROR")
        .unwrap_or_else(|| panic!("expected an ERROR, got {messages:?}"));
    assert_eq!(error["code"], "SANDBOX_UNAVAILABLE", "{messages:?}");
    assert!(
        !messages.iter().any(|m| m["phase"] == "parsing_evtx"),
        "nenhuma análise pode começar sem o sandbox exigido: {messages:?}"
    );
    assert!(
        !messages.iter().any(|m| m["message_type"] == "RESULT"),
        "{messages:?}"
    );
}

#[test]
fn an_invalid_policy_value_is_treated_as_required_not_as_off() {
    let dir = tempfile::tempdir().expect("dir");
    let missing = dir.path().join("nao-existe.evtx");
    let messages = run_stdio(
        &[
            ("DFIR_ENGINE_INPUT_ROOT", dir.path()),
            ("DFIR_ENGINE_INPUT_FILE", &missing),
        ],
        "talvez",
        "nao-existe.evtx",
        "EVTX_TIMELINE",
    );
    assert!(
        messages.iter().any(|m| m["code"] == "SANDBOX_UNAVAILABLE"),
        "um valor inválido não pode desligar a contenção: {messages:?}"
    );
}

#[test]
fn the_engine_refuses_any_file_other_than_the_pinned_one() {
    // Política `off` isola a regra do arquivo único da contenção do kernel.
    let dir = tempfile::tempdir().expect("dir");
    let pinned = dir.path().join("a.evtx");
    let other = dir.path().join("b.evtx");
    std::fs::write(&pinned, b"nao e um evtx").expect("write");
    std::fs::write(&other, b"nao e um evtx").expect("write");
    let envs: [(&str, &Path); 2] = [
        ("DFIR_ENGINE_INPUT_ROOT", dir.path()),
        ("DFIR_ENGINE_INPUT_FILE", &pinned),
    ];

    let rejected = run_stdio(&envs, "off", "b.evtx", "EVTX_TIMELINE");
    let error = rejected
        .iter()
        .find(|m| m["message_type"] == "ERROR")
        .unwrap_or_else(|| panic!("{rejected:?}"));
    assert_eq!(error["code"], "INPUT_REJECTED", "{rejected:?}");
    assert!(
        error["message"].as_str().unwrap().contains("pinned"),
        "{rejected:?}"
    );

    // O arquivo fixado passa da checagem e só falha por não ser um EVTX.
    let accepted = run_stdio(&envs, "off", "a.evtx", "EVTX_TIMELINE");
    let error = accepted
        .iter()
        .find(|m| m["message_type"] == "ERROR")
        .unwrap_or_else(|| panic!("{accepted:?}"));
    assert_eq!(error["code"], "PARSER_ERROR", "{accepted:?}");
}

#[test]
fn a_contained_engine_still_runs_the_full_protocol_and_reports_its_state() {
    if !kernel_has_landlock() {
        eprintln!("SKIPPED: this kernel has no Landlock");
        return;
    }
    let dir = tempfile::tempdir().expect("dir");
    let pinned = dir.path().join("a.evtx");
    std::fs::write(&pinned, b"nao e um evtx").expect("write");
    let messages = run_stdio(
        &[
            ("DFIR_ENGINE_INPUT_ROOT", dir.path()),
            ("DFIR_ENGINE_INPUT_FILE", &pinned),
        ],
        "required",
        "a.evtx",
        "EVTX_TIMELINE",
    );
    let phases: Vec<&str> = messages
        .iter()
        .filter_map(|m| m["phase"].as_str())
        .collect();
    assert_eq!(phases.first().copied(), Some("accepted"), "{messages:?}");
    let sandbox = phases
        .iter()
        .find(|phase| phase.starts_with("sandbox:"))
        .unwrap_or_else(|| panic!("o estado do sandbox deve ser informado: {phases:?}"));
    assert!(sandbox.contains("landlock=abi"), "{sandbox}");
    assert!(sandbox.contains("seccomp=on"), "{sandbox}");
    assert!(
        !sandbox.contains(dir.path().to_str().unwrap()),
        "sem caminhos: {sandbox}"
    );
    // O arquivo é lido sob contenção: o erro é do parser (não é EVTX), não de acesso.
    let error = messages
        .iter()
        .find(|m| m["message_type"] == "ERROR")
        .unwrap_or_else(|| panic!("{messages:?}"));
    assert_eq!(error["code"], "PARSER_ERROR", "{messages:?}");
}
