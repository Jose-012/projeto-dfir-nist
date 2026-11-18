<!-- doc-type: reference | status: current -->
# Registro de evidências e waivers — DFIR/NIST / v6.5.1

Este arquivo guarda as evidências de execução (`EVD-*`) e as exceções (`WVR-*`) dos gates de [`RELEASE_GATE.md`](RELEASE_GATE.md). Gap origem: AUS-03.

## Contrato

Cada evidência segue o Apêndice F.9 da especificação (`docs/PROJECT_SPEC_v6_5_1.md`). IDs `EVD-*` e `WVR-*` nunca são reutilizados (`AGENTS.md` §10).

| Campo | Conteúdo |
|---|---|
| `id` | `EVD-*`, único |
| `gate` | referência ao gate. O F.9 prevê `GATE-*`; enquanto o projeto não definir IDs `GATE-*`, cite a seção ou o item de `docs/RELEASE_GATE.md` (por exemplo, "Gate Rust") |
| `status` | `PASS`, `FAIL`, `PENDING`, `BLOCKED`, `WAIVED` ou `STALE` |
| `executed-at` | timestamp da execução |
| `commit-tree` | commit ou hash da árvore testada |
| `toolchain` | `rustc`/`cargo` e versão relevante |
| `environment` | host, container ou configuração, quando relevante |
| `command` | comando ou procedimento executado |
| `artifact` | arquivo, log, hash ou fixture, quando aplicável |
| `result` | resultado resumido e objetivo |

Notas do Apêndice F:

- `PENDING-HUMAN` é um qualificador de `PENDING` (F.8) para gates cuja execução depende do operador.
- `WAIVED` é exceção de governança, não prova de verificação. `PENDING`, `BLOCKED` e `STALE` impedem apresentar o gate como verificado (F.8).
- Texto como "deve passar", "está correto" ou "foi validado" não substitui evidência de execução (F.9).

## Registro

Criado vazio em 2026-11-04. As linhas `EVD-001` a `EVD-004` são execuções reais de 2026-11-14, em Linux x86_64; o resumo de cada uma está em `docs/evidence/`. A linha `EVD-XXXX` continua sendo só ilustração de formato e não sustenta nenhum `PASS`.

| id | gate | status | executed-at | commit-tree | toolchain | environment | command | artifact | result |
|---|---|---|---|---|---|---|---|---|---|
| `EVD-001` | Gate Rust | `PASS` | `2026-11-14T19:35:15Z..19:37:44Z` | árvore de trabalho 2026-11-14 (sem VCS neste diretório) | `rustc 1.99.1 / cargo 1.99.1` | Linux x86_64, Debian 13 | `just verify`; `cargo metadata --locked`; `cargo tree --locked -e features` | `docs/evidence/2026-11-14-verify.txt` | app 294 (1 ignorado), engine 83 + 7 do sandbox, protocolo 26; nextest 410/410 (1 ignorado); clippy limpo; Fase 0 0 erros/0 avisos |
| `EVD-002` | Ponta a ponta com engine real | `PASS` | `2026-11-14T19:37:59Z..19:41:47Z` | árvore de trabalho 2026-11-14 (sem VCS neste diretório) | `rustc/cargo 1.99.1` | Linux x86_64; sandbox `required` (Landlock + seccomp) | `just e2e <security.evtx>` | `docs/evidence/2026-11-14-e2e.txt` | 110/110 etapas; 2 261 eventos; 6 candidatos; sem avisos do parser; demo, documentos, máscara, determinismo e descarte conferidos |
| `EVD-003` | Supply chain | `PASS` | `2026-11-14T19:41:54Z..19:42:14Z` | árvore de trabalho 2026-11-14 (sem VCS neste diretório) | `cargo 1.99.1`, `cargo-audit 0.22.2`, `cargo-deny 0.20.2` | Linux x86_64 | `cargo audit`; `cargo audit --no-yanked`; `cargo deny --workspace check`; `just supply-chain` | `docs/evidence/2026-11-14-supply-chain.txt` | 0 vulnerabilidades no audit; 5 avisos informativos; deny `advisories/bans/licenses/sources ok`; exceção documentada `RUSTSEC-2026-0459` |
| `EVD-004` | Gate documental | `PASS` | `2026-11-14T19:43:47Z` | árvore de trabalho 2026-11-14 (sem VCS neste diretório) | Python 3.13.5 (QA) | Linux x86_64 | `just doc-gate` | `docs/evidence/2026-11-14-doc-gate.txt` | Fase 0 0 erros/0 avisos; documentation gate PASS |
| `EVD-005` | Gate Rust (`just verify`) após upgrade eframe/egui 0.37.0 + `cargo update` geral | `PASS` | `2026-11-14T21:21:01Z..21:25:39Z` | árvore de trabalho 2026-11-14 (sem VCS neste diretório) | `rustc/cargo 1.99.1` | Linux x86_64, Debian 13 | `just verify` | `docs/evidence/2026-11-14-verify-egui037.txt` | nextest 410/410; clippy `-D warnings` limpo; fmt ok; 58 testes `gui::` com `egui_kittest 0.37` (harness com `allow_missing_glyphs`, campos com nome acessível via `labelled_by`/`from_label`) |
| `EVD-006` | Ponta a ponta com engine real após upgrade eframe/egui 0.37.0 | `PASS` | `2026-11-14T21:22:25Z..21:27:32Z` | árvore de trabalho 2026-11-14 (sem VCS neste diretório) | `rustc/cargo 1.99.1` | Linux x86_64; sandbox `required` (Landlock + seccomp) | `just e2e <security.evtx>` | `docs/evidence/2026-11-14-e2e-egui037.txt` | 110/110 etapas passaram (exit 0) |
| `EVD-007` | Supply chain após upgrade eframe/egui 0.37.0 (cadeia zbus/AT-SPI fora do grafo de build) | `PASS` | `2026-11-14T21:21:30Z..21:33:17Z` | árvore de trabalho 2026-11-14 (sem VCS neste diretório) | `cargo 1.99.1`, `cargo-audit 0.22.2`, `cargo-deny 0.20.2` | Linux x86_64 | `cargo audit`; `cargo audit --no-yanked`; `cargo deny --workspace check`; `just supply-chain`; `cargo tree --locked -e normal,build` | `docs/evidence/2026-11-14-supply-chain-egui037.txt` | audit 0 vulnerabilidades (1 aviso permitido: RUSTSEC-2025-0141 bincode); deny `advisories/bans/licenses/sources ok`; zbus/zvariant/atspi/ashpd/ttf-parser ausentes do grafo normal+build; exceções restantes: RUSTSEC-2026-0459 (SEC-02) e RUSTSEC-2025-0141 |
| `EVD-008` | Gate documental após upgrade eframe/egui 0.37.0 e atualização de docs | `PASS` | `2026-11-14T21:56:05Z` | árvore de trabalho 2026-11-14 (sem VCS neste diretório) | Python 3.13.5 (QA) | Linux x86_64 | `just doc-gate` | `docs/evidence/2026-11-14-doc-gate-egui037.txt` | Fase 0 0 erros/0 avisos; documentation gate PASS; LICENSES_THIRD_PARTY.md regenerado (612 pacotes, SHA-256 do lock `e2187520…`) |
| `EVD-009` | Janela real da GUI no display X11 do host (GUI-01/GFX-01) | `PASS` | `2026-11-14T22:14:58Z..22:15:23Z` | árvore de trabalho 2026-11-14 (sem VCS neste diretório) | `rustc/cargo 1.99.1`, build debug | Linux x86_64, Xorg `:0` (Debian 13) | `timeout 25 ./target/debug/dfir-app` com `DISPLAY=:0` | `docs/evidence/2026-11-14-gui-real-window.txt` | janela abriu e ficou aberta até o timeout (exit 124); log registrou `native UI GPU adapter`: Mesa Intel HD Graphics 2000 (SNB GT1), backend Gl, IntegratedGpu, driver_info `3.3 (Core Profile) Mesa 25.0.7`, 2 candidatos (llvmpipe/Vulkan era o outro); erros EGL do wgpu_hal visíveis e interpretáveis pela linha do adaptador |
| `EVD-010` | Gate Rust — pista D do backlog (F1-19 vínculo, F1-20 limites YARA, F1-21 ordem da destruição, F1-23 auditoria sem `let _ =`, F1-24 custódia/auditoria visíveis, F1-33/AUD-32) | `PASS` | `2026-11-15T02:29Z..02:47Z` | árvore de trabalho 2026-11-15 (sem VCS neste diretório) | `rustc/cargo 1.99.1` | Linux x86_64, Debian 13 | `cargo fmt --all -- --check`; `cargo clippy --workspace --locked --all-targets --all-features -- -D warnings`; `cargo nextest run --workspace --locked` | `docs/evidence/2026-11-15-clippy-fmt-pistaD.txt`; `docs/evidence/2026-11-15-nextest-pistaD.txt` | fmt ok; clippy `-D warnings` limpo; nextest 424/424 (1 ignorado), incluindo os testes novos de cada ticket da pista D (F1-19 vínculo idempotente, F1-20 wide/timeout, F1-21 ordem+retomada, F1-23 rollback_loudly/discard/mark_job_failed_loudly, F1-24 GUI custódia/auditoria, F1-33 AUD-32) |
| `EVD-011` | Ponta a ponta com engine real e fixture versionada — pista D | `PASS` | `2026-11-15T02:41:55Z..02:46:49Z` | árvore de trabalho 2026-11-15 (sem VCS neste diretório) | `rustc/cargo 1.99.1` | Linux x86_64; sandbox `required` (Landlock + seccomp) | `just e2e tests/fixtures/senai-demo.evtx` | `docs/evidence/2026-11-15-e2e-pistaD.txt` | 110/110 etapas passaram (exit 0), sem arquivo externo: fixture determinística `tests/fixtures/senai-demo.evtx` |
| `EVD-012` | Upgrade do pin para Rust 1.100.0 (stable de 2026-11-12) + lacunas de teste RF-003/RF-027 fechadas | `PASS` | `2026-11-18T02:14Z..03:11Z` | árvore de trabalho 2026-11-18 (sem VCS neste diretório) | `rustc/cargo 1.100.0` | Linux x86_64, Debian 13; sandbox `required` no e2e (Landlock + seccomp) | `cargo metadata --locked`; `cargo fmt --all -- --check`; `cargo check --workspace --locked --all-targets`; `cargo clippy --workspace --locked --all-targets --all-features -- -D warnings`; `cargo nextest run --workspace --locked`; `just e2e tests/fixtures/senai-demo.evtx`; `cargo deny --workspace check`; `cargo audit`; `python3 scripts/validate_fase0.py`; `python3 scripts/doc_gate.py` | `docs/evidence/2026-11-18-nextest-1_100.txt`; `docs/evidence/2026-11-18-e2e-1_100.txt`; `docs/evidence/2026-11-18-clippy-1_100.txt`; `docs/evidence/2026-11-18-deny-1_100.txt` | nextest **426/426** (1 ignorado), incluindo os testes novos `import_leaves_the_source_file_byte_for_byte_untouched` (RF-003) e `no_network_listener_exists_in_app_or_engine_source` (RF-027); clippy `-D warnings` limpo (1 fix `needless_bool` em `src/persistence/audit.rs`); e2e 110/110 com engine real; deny/audit verdes com 2 exceções justificadas (WVR-004/RUSTSEC-2026-0459 + RUSTSEC-2026-0543 chrono informativo, revisão 2026-12-31); doc-gate PASS; Fase 0 0 erros/0 avisos. Windows/aarch64/incidente real continuam pendentes (WVR-001/WVR-002) |
| `EVD-013` | Fechamento da rodada de engenharia: doctor, gate Rust, supply chain, doc-gate e e2e | `PASS` | `2026-11-18T04:53Z..04:55Z` | `f8dd8c6a0a2b75e54d129055c9d3107f6927d06b` | `rustc/cargo 1.100.0`; Python 3.13.5; nextest 0.9.151; audit 0.22.2; deny 0.20.2 | Linux x86_64, Debian 13; sandbox `required` no e2e | `just doctor`; `just verify`; `just supply-chain`; `just doc-gate`; `just e2e tests/fixtures/senai-demo.evtx` | `docs/evidence/2026-11-18-evd-013.txt` | doctor 0 erros/1 aviso (Python 3.13.5 versus referência 3.14.6); nextest 432 passed/1 skipped; doc-tests, fmt/check/clippy verdes; supply chain verde com exceções documentadas; doc-gate PASS; e2e 110/110. Windows/aarch64/incidente real/CI não executados e não promovidos |
| `EVD-014` | Verificação integrada do ajuste visual da GUI | `PASS` | `2026-11-18T05:44Z..05:52Z` | `26cc8b38a10f5d4dadbc72f81ba418cf10458d7b` | `rustc/cargo 1.100.0`; Python 3.13.5; nextest 0.9.151; audit 0.22.2; deny 0.20.2 | Linux x86_64, Debian 13; sandbox `required` no e2e | `cargo fmt --all -- --check`; `cargo check --workspace --locked --all-targets`; `cargo nextest run --workspace --locked`; `cargo test --workspace --locked --doc`; `cargo clippy --workspace --locked --all-targets --all-features -- -D warnings`; `just supply-chain`; `just doc-gate`; `just e2e tests/fixtures/senai-demo.evtx` | `docs/evidence/2026-11-18-evd-014.txt` | nextest 435 passed/1 skipped, incluindo os três novos testes de glifos/layout; demais gates locais verdes; e2e 110/110. Sem mudança de código nesta verificação. Janela real nesta rodada, Windows, aarch64, incidente real e CI não executados e não promovidos |
| `EVD-015` | Cobertura estrita de glifos na navegação | `PASS` | `2026-11-18T05:54Z..06:00Z` | `3ff341e3ae4655f576b7061e4c218d41c4b7cba4` | `rustc/cargo 1.100.0`; Python 3.13.5; nextest 0.9.151 | Linux x86_64, Debian 13 | mutação temporária do emoji 📁 seguida de restauração; `just verify`; `just doc-gate` | `docs/evidence/2026-11-18-evd-015.txt` | teste falha com `No glyph for 📁` na mutação e passa após restauração; nextest 436 passed/1 skipped; demais gates Rust e doc-gate verdes. Supply chain/e2e não repetidos e mantidos em `EVD-014`; nenhuma janela real ou verificação externa nova |
| `EVD-016` | Abas de incidentes na largura mínima real | `PASS` | `2026-11-18T06:01Z..06:05Z` | `536f009a9b8daba4015f6b1aa68719bd24d6fe25` | `rustc/cargo 1.100.0`; Python 3.13.5; nextest 0.9.151 | Linux x86_64, Debian 13 | teste a 780 px; mutação temporária `horizontal_wrapped` para `horizontal` e restauração; `just verify`; `just doc-gate` | `docs/evidence/2026-11-18-evd-016.txt` | teste passa com wrapping e falha na mutação com `Plano` até x=819,3; nextest 436 passed/1 skipped; demais gates Rust e doc-gate verdes. Sem mudança de layout. Supply chain/e2e mantidos em `EVD-014`; nenhuma janela real ou verificação externa nova |
| `EVD-XXXX` *(exemplo ilustrativo, não é evidência real)* | Gate Rust | `PASS` | `AAAA-MM-DDThh:mm:ssZ` | `hash-da-árvore` | `rustc X.Y.Z / cargo X.Y.Z` | `host de exemplo` | `cargo check --workspace --locked --all-targets` | `caminho-do-log` | resultado de exemplo *(exemplo ilustrativo, não é evidência real)* |

## Waivers (`WVR-*`)

Uma exceção a gate obrigatório precisa de ID `WVR-*` e registra, no mínimo, o formato do Apêndice F.14:

| Campo | Conteúdo |
|---|---|
| `id` | `WVR-*`, único |
| `gate` | gate a que a exceção se aplica (mesma convenção do campo `gate` das evidências) |
| `status` | `approved` |
| `reason` | motivo concreto |
| `scope` | escopo da exceção |
| `approved-by` | responsável com autoridade |
| `approved-at` | timestamp |
| `valid-until` | timestamp, ou `omitted` quando não se aplica |

`WAIVED` não deve mascarar ausência de testes por conveniência. A exceção se limita ao gate e à condição descritos, e qualquer alteração relevante no artefato ou no contexto exige reavaliação (F.14).

Waivers aprovados pelo operador na sessão de 2026-11-14 (itens p2/p4 do plano de decisões pendentes; aprovações verbais registrada aqui como `approved-by: operador do projeto, sessão 2026-11-14T22:00Z`):

| id | gate | status | reason | scope | approved-by | approved-at | valid-until |
|---|---|---|---|---|---|---|---|
| `WVR-001` | Runtime — processo separado Windows (Job Object, ADR-0024) | `approved` | o protótipo não declara suporte a Windows nesta rodada: o Job Object foi **só compilado**, nunca executado; não há máquina Windows disponível ao operador agora. A condição de saída é executar `dfir-app engine-smoke` + `just e2e <evtx>` em Windows e conferir o aviso esperado `SANDBOX_NOT_FULLY_APPLIED` | gate "Runtime/Windows" do `docs/RELEASE_GATE.md`; não cobre Linux (provado em `EVD-005`/`EVD-006`) nem aarch64 | operador do projeto, sessão 2026-11-14T22:00Z | 2026-11-14T22:00:30Z | 2026-12-31T23:59:59Z |
| `WVR-002` | Runtime — Linux aarch64 | `approved` | mesma razão: não há host aarch64 disponível; o sandbox (Landlock/seccomp) foi provado só em x86_64 (`EVD-005`/`EVD-006`); RUSTSEC-2026-0379 era específico de aarch64 e não afeta a build x86_64 | gate "Runtime/aarch64"; não cobre x86_64 | operador do projeto, sessão 2026-11-14T22:00Z | 2026-11-14T22:00:30Z | 2026-12-31T23:59:59Z |
| `WVR-003` | Gate Rust/CI — CI automático (F1-14) | `approved` | o baseline de verificação roda manualmente (`just verify`, `EVD-005`); não há runner de CI configurado nesta fase do protótipo | gate "CI automático" (F1-14); não isenta os gates locais, que estão `PASS` com `EVD-005`..`EVD-008` | operador do projeto, sessão 2026-11-14T22:00Z | 2026-11-14T22:00:30Z | 2026-12-31T23:59:59Z |
| `WVR-004` | Supply chain — `RUSTSEC-2026-0459` (`libsqlite3-sys 0.37.0`, SEC-02, CVE-2026-119375) | `approved` | sem correção possível hoje: `sqlx-sqlite 0.9.0` (última publicada) exige `libsqlite3-sys <0.38.0` e nenhuma versão publicada admite o `>=0.39.0` corrigido (`cargo update -p libsqlite3-sys --precise 0.39.0` falha na seleção de versão). Alcance: só o banco SQLite local no volume protegido, WAL, sob operador único (ADR-0007); compensatórios: volume protegido, backup antes de migration (ADR-0026/RF-032), `backup-verify`. **Risco aceito documentado, não prova de inexploração** — mesma análise em `deny.toml` e `.cargo/audit.toml` | exceção de advisory no gate Supply chain (`EVD-007`); não cobre outros advisories | operador do projeto, sessão 2026-11-14T22:00Z | 2026-11-14T22:00:30Z | 2026-12-31T23:59:59Z (reavaliar quando sair `sqlx` com `libsqlite3-sys >= 0.39.0`) |

## Regra

Toda declaração de `PASS` no `docs/RELEASE_GATE.md` cita um `EVD-*` deste registro. Sem `EVD-*`, o gate permanece `PENDING` ou `PENDING-HUMAN`.
