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
    cargo run --locked --bin dfir-app

test:
    cargo test --workspace --locked

nextest:
    cargo nextest run --workspace --locked

clippy:
    cargo clippy --workspace --all-targets --all-features --locked -- -D warnings

audit:
    # `.cargo/audit.toml` espelha a exceção já justificada no deny.toml; sem ele
    # o cargo audit falharia em RUSTSEC-2026-0459 sem análise de alcance.
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
    cargo run --locked --bin dfir-app -- engine-smoke

# Runtime tests exercise the native local engine process on each supported host platform.
# Teste ponta a ponta headless com engine real (ADR-0019). Uso: just e2e caminho/arquivo.evtx
# `-p` é obrigatório: a raiz é package e workspace, então sem ele o cargo procura
# o binário `dfir-engine` no pacote `dfir-app` e falha (BUILD-01).
e2e evtx:
    cargo build --locked -p dfir-app -p dfir-engine
    python3 scripts/e2e_headless.py --evtx {{evtx}}

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
    just test
    just nextest
    just clippy

release-tools:
    cargo install --locked cargo-audit@0.22.2
    cargo install --locked cargo-deny@0.20.2
    cargo install --locked cargo-nextest@0.9.151

# Documentation checks intentionally remain lightweight; runtime/build gates remain authoritative.
doc-gate:
    {{python}} scripts/validate_fase0.py
    {{python}} scripts/doc_gate.py
