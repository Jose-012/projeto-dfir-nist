<!-- doc-type: gate | status: current -->
# Definition of Ready - Fase 0 → Fase 1

A próxima fase só pode começar quando a base local estiver verificável.

- [x] especificação e cópia raiz sincronizadas — `just doc-gate` (`scripts/doc_gate.py` confirma a igualdade byte a byte): `EVD-004`, `EVD-008`;
- [x] interface nativa verificada por `cargo check`/`cargo test` no toolchain pinado (frontend removido por ADR-0017): `EVD-001`, `EVD-005` (58 testes `gui::` com `egui_kittest 0.37`); janela real aberta no display do host: `EVD-009`;
- [x] runtime final sem Node/Bun/Deno/Python — checagem estática de `scripts/validate_fase0.py` (Fase 0 0 erros) + `cargo tree --locked -e normal,build` sem nenhum desses runtimes no grafo de build: `EVD-005`, `EVD-007`, `EVD-008`;
- [x] audit chain verificada — `dfir-app audit-verify` sobre o banco real (e2e conferiu nas etapas 344/356/589/601) + testes de `src/persistence/audit.rs`: `EVD-002`, `EVD-006`;
- [x] processo separado Linux validado — `cargo test --workspace` (inclui `engine/tests/sandbox.rs`, Landlock+seccomp `required`) e `just e2e <security.evtx>` em Linux x86_64: `EVD-001`/`EVD-002`, reexecutados após o egui 0.37.0 como `EVD-005`/`EVD-006`;
- [x] processo separado Windows — **não suportado nesta rodada**: waiver formal `WVR-001` (Job Object só compilado; saída: `engine-smoke` + e2e em Windows, válido até 2026-12-31);
- [x] protocolo JSONL e limites validados — testes de `protocol/` e `engine/` no workspace: `EVD-005`;
- [x] storage e migrations validados — testes de `src/persistence/backup.rs` (migrations 0001–0012, cópia verificada antes de publicar, recusa por adulteração/espaço): `EVD-005`;
- [x] todos os gates obrigatórios em `PASS` ou waiver formal — gates locais `PASS` com `EVD-001`..`EVD-009`; exceções formais `WVR-001` (Windows), `WVR-002` (aarch64), `WVR-003` (CI automático), `WVR-004` (SEC-02/RUSTSEC-2026-0459, revisão até 2026-12-31), todos aprovados pelo operador em 2026-11-14T22:00Z e registrados em `docs/EVIDENCE_REGISTER.md`;

> Gap origem: EVI-01 (2026-11-04). Em 2026-11-14 o operador registrou `EVD-001` a `EVD-009` e aprovou `WVR-001` a `WVR-004` em [`docs/EVIDENCE_REGISTER.md`](EVIDENCE_REGISTER.md); os itens acima passaram a `[x]` com citação de evidência/waiver existente (regra EVI-01).

`PASS` = executado e evidenciado.
`WAIVED` = exceção formal.
`PENDING` = sem evidência suficiente.


## Estado de transição — 2026-08-24

Os gates e testes da Fase 0 foram declarados concluídos pelo responsável do projeto. Este documento, portanto, registra a base como **Ready** para iniciar a Fase 1; a execução da Fase 1 passa a ser acompanhada em `docs/FASE1_BACKLOG.md`.

> **Nota (2026-11-04) — Gap origem: EVI-01.** A declaração acima é do responsável do projeto, sem `EVD-*`. O estado "Ready" de 2026-08-24 fica como histórico, não como estado atual: os itens do checklist voltaram a `[ ]` `[PENDING-HUMAN]` até o operador registrar evidência ou aprovar `WVR-*`.

## Atualização — 2026-08-25

O item "frontend buildado/testado com Bun" reflete o estado real no momento da transição (2026-08-24) e não é reescrito retroativamente. Após a transição, Bun foi removido da toolchain de build/test do frontend por não ser instalável em todos os ambientes de execução do projeto; a toolchain atual é Node.js/npm (`esbuild`, `vitest`). Ver ADR-0009, `CHANGELOG.md` e `docs/FASE1_STATUS.md`. O critério "runtime final sem Node/Bun/Deno/Python" continua satisfeito sem alteração: nenhum dos dois entra no runtime distribuído.

> **Nota (2026-11-04) — Gap origem: VAZ-06.** Superseded por ADR-0017 em 2026-09-22: a toolchain Node.js/npm (`esbuild`, `vitest`) citada acima deixou de existir quando o frontend foi removido (ver ADR-0017).
