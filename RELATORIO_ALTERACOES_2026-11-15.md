# Relatório de alterações — 2026-11-15 (pistas C e D do plano remanescente)

**Pedido do operador:** executar as pistas C e D do plano remanescente (`docs/FASE1_BACKLOG.md` §7): C = F1-16 (fixture EVTX sanitizada versionada + e2e sem arquivo externo), D = lacunas parciais F1-19, F1-20, F1-21, F1-23, F1-24, F1-33, cada ticket com teste próprio e `EVD-*` registrado.

**Ambiente de execução:** Linux x86_64 (Debian 13), `rustc/cargo 1.99.1`, sandbox `required` (Landlock + seccomp) no e2e. Sem repositório Git neste diretório; o "commit-tree" das evidências é a árvore de trabalho de 2026-11-15.

## Resultado por pista/ticket

| Pista/Ticket | Antes | Depois | Evidência |
|---|---|---|---|
| C / F1-16 | e2e dependia de arquivo externo (`security.evtx` do operador) | `tests/fixtures/senai-demo.evtx` determinística versionada (SHA-256 `256a655a…d448be`); `just e2e tests/fixtures/senai-demo.evtx` **110/110** sem arquivo externo; `docs/OPERATION.md` §5 atualizado | `EVD-011` |
| D / F1-19 | vínculo evidência↔incidente pendente | `incident_id` na evidência, `link_evidence_to_incident` idempotente com auditoria `EVIDENCE_LINKED_TO_INCIDENT`, vigentes sem janela (`evidence_for_incident`), YARA idempotente; GUI (painel no detalhe da evidência) + CLI (`link-evidence`, `incident-evidence`); `confirm_candidate` vincula ao confirmar; testes de serviço e `egui_kittest` | `EVD-010` |
| D / F1-20 | limites YARA parciais | `wide ascii` provado em payload UTF-16LE (teste novo), `scan_timeout_for` por tamanho com teto, `MAX_MATCHES_PER_PATTERN`/`MAX_REPORTED_OCCURRENCES_PER_PATTERN` invariantes em `const {}`, varredura sem truncamento silencioso | `EVD-010` |
| D / F1-21 | ordem/retomada parcialmente cobertas | testes novos: falha de banco na transação não apaga nada; purga completa inclui candidatos de detecção e preserva custódia/auditoria; retomada em reexecução já coberta pelos 4 testes existentes | `EVD-010` |
| D / F1-23 | `let _ =` em vários pontos de produção | nenhum `let _ =` fora de testes em `src/`, `engine/`, `protocol/`; helpers `rollback_loudly` (persistence), `discard_temp_file/dir`, `mark_job_failed_loudly` (service), sends de canal e abort de stderr logados (`tracing::warn!`/`debug!`/`trace!` conforme criticidade), `NotFound` tratado como esperado; testes próprios para cada helper | `EVD-010` |
| D / F1-24 | custódia/auditoria não visíveis na GUI | painel "Vínculo com incidente e custódia" no detalhe da evidência; abas **Evidências** e **Auditoria** na aba Incidentes; CLI `custody` e `audit`; 4 testes `egui_kittest` novos (suíte `gui::` 62/62) | `EVD-010` |
| D / F1-33 (AUD-32) | YARA sobrescrevia a timeline com resultado vazio | só análise EVTX recarrega eventos de timeline; teste próprio em `src/gui/evidence.rs` | `EVD-010` |

## Gates executados nesta rodada (todos em rustc/cargo 1.99.1, Linux x86_64)

- `cargo fmt --all -- --check` — ok
- `cargo check --workspace --locked --all-targets` — limpo
- `cargo clippy --workspace --locked --all-targets --all-features -- -D warnings` — limpo
- `cargo nextest run --workspace --locked` — **424/424** (1 ignorado) — `EVD-010`
- `just e2e tests/fixtures/senai-demo.evtx` — **110/110** — `EVD-011`
- `just doc-gate` — **PASS** (0 erros, 0 avisos): README.md documenta os 20 subcomandos CLI (`link-evidence`, `incident-evidence`, `custody`, `audit` incluídos), `docs/API.md` documenta `link_evidence_to_incident`/`evidence_for_incident`, e os arquivos de ticket de F1-16/19/24/33 em `.scratch/fase1-backlog/issues/` estão alinhados ao `VERIFIED` do `docs/rtm.md`

Artefatos: `docs/evidence/2026-11-15-nextest-pistaD.txt`, `docs/evidence/2026-11-15-e2e-pistaD.txt`, `docs/evidence/2026-11-15-clippy-fmt-pistaD.txt`.

## Documentação atualizada

- `docs/EVIDENCE_REGISTER.md`: `EVD-010` (Gate Rust, pista D) e `EVD-011` (e2e sem arquivo externo, pista C) registrados.
- `docs/rtm.md` e `docs/FASE1_STATUS.md`: F1-16, F1-19, F1-20, F1-21, F1-23, F1-24, F1-33 promovidos a `VERIFIED` citando `EVD-010`/`EVD-011`; nova seção "Atualização 2026-11-15" no rtm.md e linha no histórico do FASE1_STATUS.
- `docs/FASE1_BACKLOG.md` §7: pistas C e D marcadas CONCLUÍDAS com `EVD-010`/`EVD-011`.
- `docs/RELEASE_GATE.md`: atualização 2026-11-15 no topo.
- `docs/OPERATION.md`: §3E (vínculo/custódia/auditoria na GUI e CLI) e §5 (fixture versionada, F1-16 concluído).
- `CHANGELOG.md`: entrada "Unreleased — 2026-11-15 — pistas C e D".
- `README.md` e `docs/API.md`: lacunas do doc-gate fechadas (20 subcomandos CLI; funções novas de serviço F1-19).

## Pendências inalteradas

`WVR-001` (Windows), `WVR-002` (aarch64), `WVR-003` (CI automático) e `WVR-004` (SEC-02) continuam válidos até 2026-12-31; as pistas A, B, E e F do §7 seguem abertas.
