# Cross-platform command runner.
# Unix/WSL2 uses bash; native Windows uses PowerShell so Cargo/Node/Python
# are resolved from the user's Windows PATH instead of a separate /bin/bash PATH.
set shell := ["bash", "-eu", "-o", "pipefail", "-c"]
set windows-shell := ["powershell.exe", "-NoLogo", "-NoProfile", "-NonInteractive", "-Command"]

python := if os_family() == "windows" { "python" } else { "python3" }

# Runtime/production commands deliberately remain small and explicit.

default:
    @just --list

doctor:
    {{python}} scripts/doctor.py

validate-fase0:
    {{python}} scripts/validate_fase0.py

fmt:
    cargo fmt --all -- --check

check:
    cargo check --workspace --locked --all-targets

build:
    cargo build --workspace --locked --release

run:
    cargo build --locked -p dfir-app -p dfir-engine
    cargo run --locked --bin dfir-app

test:
    cargo test --workspace --locked

doctest:
    cargo test --workspace --locked --doc

nextest:
    cargo nextest run --workspace --locked

clippy:
    cargo clippy --workspace --all-targets --all-features --locked -- -D warnings

audit:
    # `.cargo/audit.toml` espelha as exceções informativas justificadas no deny.toml.
    # Uma exceção removida do grafo deve sair dos dois arquivos na mesma mudança.
    cargo audit --no-yanked

# --workspace is required: the root manifest is a package, so without it
# cargo-deny only checks dfir-app and silently skips the engine (evtx, yara-x).
licenses:
    cargo deny --workspace check

supply-chain:
    # As duas etapas rodam sempre e o relatório sai completo mesmo quando a
    # primeira falha; o código de saída final é o da última etapa que falhou
    # (TOOL-01). O `|| status=$?` evita que o `set -eu` da recipe aborte antes.
    status=0; just audit || status=$?; just licenses || status=$?; exit $status

engine-protocol-smoke:
    {{python}} scripts/engine_protocol_smoke.py

audit-verify:
    cargo run --locked --bin dfir-app -- audit-verify

engine-smoke:
    cargo build --locked -p dfir-app -p dfir-engine
    cargo run --locked --bin dfir-app -- engine-smoke

# Runtime tests exercise the native local engine process on each supported host platform.
# Teste ponta a ponta headless com engine real (ADR-0019). Uso: just e2e caminho/arquivo.evtx
# `-p` é obrigatório: a raiz é package e workspace, então sem ele o cargo procura
# o binário `dfir-engine` no pacote `dfir-app` e falha (BUILD-01).
e2e evtx:
    cargo build --locked -p dfir-app -p dfir-engine
    {{python}} scripts/e2e_headless.py --evtx {{evtx}}

# Incidente de demonstração (F1-11): importa o .evtx, roda as análises reais e registra o cenário BEC sintético. Ver docs/DEMO.md.
demo evtx:
    cargo build --locked -p dfir-app -p dfir-engine
    cargo run --locked --bin dfir-app -- demo {{evtx}}

runtime-test:
    {{python}} scripts/run_runtime_test.py

runtime-security:
    {{python}} tests/security/fase0_config.py
    {{python}} tests/security/rejected_external_tools.py

verify:
    just validate-fase0
    just fmt
    just check
    just nextest
    just doctest
    just clippy

release-tools:
    cargo install --locked cargo-audit@0.22.2
    cargo install --locked cargo-deny@0.20.2
    cargo install --locked cargo-nextest@0.9.151

# Documentation checks intentionally remain lightweight; runtime/build gates remain authoritative.
test-coordination-playbook:
    {{python}} -m unittest tests/fixtures/coordination_playbook/test_exhaustive.py tests/fixtures/coordination_playbook/test_adversarial.py tests/fixtures/coordination_playbook/test_repo_audit.py tests/fixtures/coordination_playbook/test_benchmark.py -v

test-coordination-playbook-all:
    {{python}} -m unittest tests/fixtures/coordination_playbook/test_gate.py tests/fixtures/coordination_playbook/test_exhaustive.py tests/fixtures/coordination_playbook/test_adversarial.py tests/fixtures/coordination_playbook/test_repo_audit.py tests/fixtures/coordination_playbook/test_benchmark.py -v

benchmark-coordination-playbook:
    {{python}} scripts/coordination_playbook_benchmark.py

doc-gate:
    {{python}} scripts/validate_fase0.py
    {{python}} scripts/doc_gate.py
    {{python}} scripts/coordination_playbook_gate.py
    {{python}} scripts/coordination_playbook_repo_audit.py
    just test-coordination-playbook
