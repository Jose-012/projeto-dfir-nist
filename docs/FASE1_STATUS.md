<!-- doc-type: status | status: current -->
# Status da Fase 1

> **Autoridade de estado de ticket: `docs/rtm.md`.** Este arquivo espelha a matriz e guarda um resumo por rodada; o log cronológico completo (até 2026-10-29) está em [`docs/archive/FASE1_STATUS_historico_2026-10.md`](archive/FASE1_STATUS_historico_2026-10.md).
> Gap origem: INC-03 (2026-11-04).

## Tickets

| Ticket | RFs | Estado |
|---|---|---|
| F1-01 | Enabler (sem RF próprio) | IMPLEMENTED / PENDING-HUMAN |
| F1-02 | RF-008 | IMPLEMENTED / PENDING-HUMAN |
| F1-03 | RF-009, RF-010 | IMPLEMENTED / PENDING-HUMAN (E0505 corrigido em 2026-10-02) |
| F1-04 | RF-011, RF-018, RF-020 | IMPLEMENTED / PENDING-HUMAN |
| F1-05 | RF-019, RF-022 | IMPLEMENTED / PENDING-HUMAN |
| F1-06 | RF-052 (RF-012 → Fase 4) | IMPLEMENTED / PENDING-HUMAN (absorvido por F1-29, domínio `OS_HARDENING`; a saída no Entregável 4 é gerada por F1-07) |
| F1-07 | RF-013, RF-040, RF-046 | IMPLEMENTED / PENDING-HUMAN (ADR-0023; testado em rustc 1.97.1 em 2026-10-27) |
| F1-08 | RF-025 | DEFERRED — Fase 4 |
| F1-09 | RF-026 | DEFERRED — Fase 4 |
| F1-10 | RF-032, RF-042, RF-043 (RF-041 substituído) | IMPLEMENTED / PENDING-HUMAN (ADR-0026: cópia verificada do banco antes de migration, automática e testada; AUD-03a e recovery na partida já existiam) |
| F1-11 | RF-016, RF-017, RF-046 (RF-045 → Fase 4) | IMPLEMENTED / PENDING-HUMAN (reduzido: ADR-0023, Etapa 7; só EVTX, sem CSV/JSONL; testado em rustc 1.97.1 em 2026-10-27) |
| F1-12 | RF-014, RF-030, RF-032, RF-047 + RFs MVP restantes | PARCIAL (documentos reconciliados em 2026-10-27; RF-032 automático; F1-26 e F1-28 implementados; RF-049 adiado por decisão do operador; **pendentes:** CI no toolchain pinado (F1-14), ~~`cargo audit` (F1-15)~~ [executado em 2026-10-29, ver a linha F1-15; nota de 2026-11-04, Gap origem: INC-03], execução em Windows e em incidente real) |
| F1-13 | RF-047 | IMPLEMENTED / PENDING-HUMAN |
| F1-14 | (gates) | PARCIAL (2026-11-14: `just verify` — `fmt`, `check --workspace`, `test`, `nextest` e `clippy -D warnings` — e `e2e_headless` rodados **no 1.99.1**, de forma manual; `EVD-001`/`EVD-002`. CI automático segue PLANNED) |
| F1-15 | (supply chain) | PARCIAL (2026-11-14: `cargo audit` e `cargo deny --workspace check` **executados** e verdes com **uma** exceção de vulnerabilidade justificada, `RUSTSEC-2026-0459`, formalizada como `WVR-004` (válida até 2026-12-31); `EVD-003`/`EVD-007`; manual, sem CI (`WVR-003`); `cargo vet` dispensado no MVP; as exceções informativas da UI (zbus/ttf-parser) foram **removidas do grafo** com o eframe 0.37.0 sem accesskit) |
| F1-16 | RF-005, RF-006 | VERIFIED (2026-11-15, `EVD-011`): fixture EVTX sanitizada versionada `tests/fixtures/senai-demo.evtx` (determinística) e e2e 110/110 sem arquivo externo |
| F1-17 | RF-006 | IMPLEMENTED / PENDING-HUMAN (ADR-0021) |
| F1-18 | RF-007, RF-046 | IMPLEMENTED / PENDING-HUMAN (ADR-0020/0021/0025; filtros estruturados `campo:valor` implementados) |
| F1-19 | RF-039 | VERIFIED (2026-11-15, `EVD-010`): vínculo evidência↔incidente idempotente com auditoria `EVIDENCE_LINKED_TO_INCIDENT`, vigente sem janela, YARA idempotente; GUI + CLI (`link-evidence`, `incident-evidence`) com testes próprios |
| F1-20 | RF-008 | VERIFIED (2026-11-15, `EVD-010`): `wide ascii` provado em UTF-16LE, `scan_timeout_for` por tamanho com teto, `MAX_MATCHES_PER_PATTERN` sem descarte silencioso, sem truncamento de varredura; testes em `engine/src/yara_scan.rs` |
| F1-21 | RF-004 | VERIFIED (2026-11-15, `EVD-010`): ordem registro→arquivo→jobs com retomada em reexecução; testes cobrindo falha de banco (nada apagado), falha de I/O (registro mantido) e purga completa incl. candidatos de detecção, custódia preservada |
| F1-22 | RF-033, RF-037 | IMPLEMENTED / PENDING-HUMAN (ADR-0024: ambiente reduzido, arquivo único, `kill_on_drop`, Landlock + seccomp + `no_new_privs` no Linux, **provados com o engine real**; Job Object no Windows **só compilado, nunca executado**; confinamento de arquivo/rede no Windows não feito) |
| F1-23 | RF-006 | VERIFIED (2026-11-15, `EVD-010`): nenhum `let _ =` em código de produção fora de testes; `rollback_loudly`, `discard_temp_file/dir`, `mark_job_failed_loudly` com `tracing::warn!` e testes próprios; sends de canal logados |
| F1-24 | RF-004, RF-029 | VERIFIED (2026-11-15, `EVD-010`): painel de custódia no detalhe da evidência e abas Evidências/Auditoria em Incidentes na GUI; CLI `custody` e `audit`; testes `egui_kittest` 62/62 na suíte `gui::` |
| F1-25 | RF-051 | IMPLEMENTED / PENDING-HUMAN (ADR-0023; testado em rustc 1.97.1 em 2026-10-27) |
| F1-26 | RF-048 | IMPLEMENTED / PENDING-HUMAN (ADR-0025; esquema 4, migration 0012, campos estruturais e filtro; testado em rustc 1.97.1 em 2026-10-27) |
| F1-27 | RF-049 | DEFERRED — decisão do operador (ADR-0025): escopo do protótipo é EVTX |
| F1-28 | RF-050 | IMPLEMENTED / PENDING-HUMAN (ADR-0025; 19 regras, candidatos com confirmação/rejeição motivada, entregável 1 §4.3; verificado no `security.evtx` real e em eventos sintéticos, **não em incidente real**) |
| F1-29 | RF-052 | IMPLEMENTED / PENDING-HUMAN (ADR-0023; testado em rustc 1.97.1 em 2026-10-27) |
| F1-30 | RF-053 | IMPLEMENTED / PENDING-HUMAN (ADR-0023; testado em rustc 1.97.1 em 2026-10-27) |
| F1-31 | RF-054 | IMPLEMENTED / PENDING-HUMAN (ADR-0023; testado em rustc 1.97.1 em 2026-10-27) |
| F1-32 | RF-055 | IMPLEMENTED / PENDING-HUMAN (ADR-0023; testado em rustc 1.97.1 em 2026-10-27) |
| F1-33 | RF-046 | VERIFIED (2026-11-15, `EVD-010`): AUD-32 corrigido — YARA não sobrescreve a timeline (só análise EVTX recarrega eventos); teste próprio em `src/gui/evidence.rs` |
| F1-34 | Enabler (sem RF próprio; ADR-0019) | IMPLEMENTED / PENDING-HUMAN |
| F1-35 | Enabler (sem RF próprio; ADR-0022) | IMPLEMENTED / PENDING-HUMAN |

## Histórico recente

| Rodada | Resumo | Detalhe |
|---|---|---|
| 2026-10-23 | Etapa 7 (ADR-0023): incidente de demonstração (`dfir-app demo`), e2e dos cinco documentos (79/79) com engine e `security.evtx` reais em rustc 1.91.1; F1-11 e F1-12 reduzidos; dois defeitos reais corrigidos (`analyze` com `ColumnNotFound("source")`; evidência descartada omitida dos documentos). | arquivo histórico, "Atualização 2026-10-23" |
| 2026-10-27 | Contenção do engine (ADR-0024), evento normalizado e detecção por regras (ADR-0025: F1-26 e F1-28 implementados, F1-27 adiado) e cópia verificada antes de migration (ADR-0026). Rustc 1.97.1 (conda-forge), `--locked`: app 288 (1 ignorado), engine 83 + 7 do sandbox, protocolo 26, e2e 110/110; cinco defeitos reais corrigidos. Pendente: Windows, aarch64, incidente real, CI. | arquivo histórico, "Atualização 2026-10-27" |
| 2026-10-29 | Dependências (F1-15): `cargo audit` achou 9 vulnerabilidades no lock de 2026-10-27; `cargo update` (111 pacotes: `yara-x 1.22.1`, `wasmtime 48.0.8`, `tokio 1.54.1`, `ctutils 0.4.7`); depois, `cargo audit` com 0 vulnerabilidades e `cargo deny --workspace check` ok. Pendente: 3 exceções informativas do `deny.toml` (UI). | arquivo histórico, "Atualização 2026-10-29" |
| 2026-11-04 | Auditoria documental (41 gaps) e decisões do operador: RTM como autoridade de estado de ticket, `cargo vet` dispensado, Aider removido (ADR-0027). Execução dos gaps P0 e P1: ver `docs/rtm.md`, "Atualização 2026-11-04" e seguintes. | `docs/rtm.md`; `docs/EVIDENCE_REGISTER.md` (vazio: nenhuma execução registrada como `EVD-*`) |
| 2026-11-14 | Registro de problemas de execução local: `evtx 0.15.1` (SEC-01), exceção documentada de `libsqlite3-sys` (SEC-02), `just supply-chain` verde (TOOL-01), diálogo de arquivos fora da thread da UI + caminhos alternativos (GUI-01), adaptador de GPU no log (GFX-01), `just e2e`/`just demo` corrigidos e script robusto (BUILD-01), toolchain/docs em 1.99.1 (DOC-01). Em rustc 1.99.1, Linux x86_64: `just verify` e e2e **110/110** com `security.evtx` (2 261 eventos, 6 candidatos). Windows/aarch64/incidente real continuam pendentes. | `docs/rtm.md` "Atualização 2026-11-14"; `docs/EVIDENCE_REGISTER.md` (`EVD-001` a `EVD-003`) |
| 2026-11-14 (tarde) | Item 1 do plano do operador: upgrade `eframe`/`egui`/`egui_extras`/`egui_kittest` 0.36.2 → 0.37.0 + `rfd` 0.18.0, com `eframe` sem `accesskit` (cadeia zbus/AT-SPI fora do grafo; exceções RUSTSEC-2026-0370/0371/0372 e -0192 removidas do `deny.toml`), mais `cargo update --workspace` geral (minors `jiff`/`toml`/`uuid`; 52 pacotes restantes bloqueados por `rust-version > 1.99`). Testes de GUI adaptados ao `egui_kittest 0.37` (`allow_missing_glyphs`, nomes acessíveis). `just verify` 410/410, e2e **110/110**, audit 0 vulnerabilidades, deny ok, doc-gate PASS. Windows/aarch64/janela real da GUI continuam pendentes. | `docs/rtm.md` "Atualização 2026-11-14 (tarde)"; `docs/EVIDENCE_REGISTER.md` (`EVD-005` a `EVD-008`); `RELATORIO_ALTERACOES_2026-11-14.md` "Rodada da tarde" |
| 2026-11-14 (noite) | Itens p2–p6 do plano do operador: waivers formais `WVR-001` (Windows), `WVR-002` (aarch64), `WVR-003` (CI) e `WVR-004` (SEC-02/`RUSTSEC-2026-0459`), aprovados até 2026-12-31; `DEFINITION_OF_READY` todo `[x]`; `EVD-009` (janela real da GUI no Xorg `:0`, adaptador wgpu no log, cobre GUI-01/GFX-01); políticas de `backups/` e de distribuição de `.evtx` em `docs/OPERATION.md`. Gates finais: doc-gate PASS, deny ok, audit 0 vulnerabilidades, fmt ok, nextest 410/410. | `docs/EVIDENCE_REGISTER.md` (`EVD-009`, `WVR-001`..`WVR-004`); `RELATORIO_ALTERACOES_2026-11-14.md` "Rodada da noite" |
| 2026-11-15 | Planejamento remanescente consolidado em `docs/FASE1_BACKLOG.md` §7 (pistas A–F: verificação externa, CI, fixtures EVTX sanitizadas, lacunas parciais, supply chain recorrente, fechamento do RC). `AGENTS.md` §15 e `CLAUDE.md` passam a fixar as ferramentas preferidas do agente (agentgrep, ast-grep, `rtk`, `fd`/`eza`, zoxide, `bat`). | `docs/FASE1_BACKLOG.md` §7; `AGENTS.md` §15; `CLAUDE.md` |
| 2026-11-15 (pistas C e D) | Pista C (F1-16): fixture `tests/fixtures/senai-demo.evtx` determinística e versionada; e2e **110/110** sem arquivo externo (`EVD-011`). Pista D: F1-19 vínculo evidência↔incidente idempotente + GUI/CLI; F1-20 limites YARA (`wide` UTF-16LE, timeout por tamanho); F1-21 ordem/retomada da destruição com testes de falha; F1-23 sem `let _ =` em produção (`rollback_loudly`/`discard_temp_*`/`mark_job_failed_loudly`); F1-24 custódia/auditoria visíveis (GUI + `custody`/`audit`); F1-33/AUD-32 corrigido. fmt ok, clippy `-D warnings` limpo, nextest **424/424** (`EVD-010`). Tickets promovidos a `VERIFIED` em `docs/rtm.md`. | `docs/EVIDENCE_REGISTER.md` (`EVD-010`, `EVD-011`); `RELATORIO_ALTERACOES_2026-11-15.md`; `docs/rtm.md` "Atualização 2026-11-15" |
| 2026-11-18 | Upgrade do pin para **Rust 1.100.0** (1.99.1 → 1.100.0 em `rust-toolchain.toml`/`toolchains.toml`/`rust-version`); 1 fix de clippy novo (`needless_bool` em `audit.rs`). Lacunas de teste fechadas: RF-003 (`import_leaves_the_source_file_byte_for_byte_untouched`) e RF-027 (`no_network_listener_exists_in_app_or_engine_source`). Em rustc 1.100.0, Linux x86_64: fmt ok, check ok, clippy `-D warnings` limpo, nextest **426/426** (1 ignorado), e2e **110/110** com engine real e sandbox `required`, deny+audit verdes (nova exceção informativa RUSTSEC-2026-0543 chrono, revisão 2026-12-31), doc-gate PASS (`EVD-012`). Windows/aarch64/incidente real continuam pendentes (WVR-001/WVR-002). | `docs/EVIDENCE_REGISTER.md` (`EVD-012`); `docs/rtm.md` "Atualização 2026-11-18"; `docs/evidence/2026-11-18-*.txt` |
| 2026-11-18 | Micro-consolidações: `validation_error` (7→1 em `persistence/mod.rs`), helpers de UI (`non_empty`, `str_field`, `string_list`, `toggle`, `split_ids` 9→1 em `gui/mod.rs`), helper de teste `drain` (11→1 em `gui/test_harness`). Sem mudança de comportamento; gates verdes (fmt, check, clippy, nextest 426/426). | `CHANGELOG.md` (entrada 2026-11-18); esta linha |