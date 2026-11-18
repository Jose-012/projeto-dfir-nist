<!-- doc-type: status | status: current -->
# Fase 0 - Status

> Estado de ticket e de requisito: `docs/rtm.md` (autoridade). Este arquivo guarda a matriz de capacidades da Fase 0; o log cronológico da Fase 1 está em `docs/FASE1_STATUS.md`. Gap origem: INC-03 (2026-11-04).

## Escopo atual

Ferramenta forense local/offline. Autenticação, RBAC, sessões e server-mode foram retirados do runtime por não serem requisitos da demanda original. Ver ADR-0007.

## Matriz

| Capacidade | Estado | Evidência | Validação externa |
|---|---|---|---|
| Monólito modular Rust | IMPLEMENTED | `src/`, `engine/`, `protocol/` | Cargo |
| SQLite + WAL + FK + FULL sync | IMPLEMENTED | `src/persistence/db.rs` | Cargo + testes |
| Migrations locais | IMPLEMENTED | `migrations/0001_fase0.sql` até `0012_structured_events_and_detection_candidates.sql` (12 arquivos em `migrations/`) | migração do zero + upgrade legado |
| Audit chain append-only | IMPLEMENTED | `src/persistence/audit.rs` | testes concorrentes |
| BLAKE3-256-v2-JCS + BLAKE3-v1 | IMPLEMENTED | `src/persistence/audit.rs` | testes de hash |
| Storage/staging | IMPLEMENTED | `src/infrastructure/storage.rs` | testes adversariais |
| Engine separado | IMPLEMENTED | `src/application/engine.rs`, `engine/` | subprocesso real |
| JSONL streaming | IMPLEMENTED | `protocol/`, `src/application/engine.rs` | testes de protocolo |
| Podman/containerização | RETIRADO | adendo à ADR-0007 | não aplicável |
| UI local server-rendered (`templates/`, `src/web`) | RETIRADO | ADR-0017 | substituída por interface nativa, ver linha abaixo |
| Frontend TypeScript/Node.js/npm (`frontend/`) | RETIRADO | ADR-0017 | removido por inteiro junto com `src/web`/`templates/`; ADR-0009 (Bun→Node) fica só de valor histórico |
| Interface nativa (eframe/egui/egui_extras) | IMPLEMENTED / PENDING-HUMAN | `src/gui/`, `src/service/`, ADR-0017, `docs/NATIVE_UI_ARCHITECTURE.md` | Histórico (2026-09-22, toolchain não pinado): `cargo check`/`test`/`fmt`/`clippy` verdes; UI não renderizada visualmente nessa sessão. Estado vigente: `docs/rtm.md`; log cronológico: `docs/FASE1_STATUS.md`. Verificação no toolchain pinado: [PENDING-HUMAN → `docs/EVIDENCE_REGISTER.md`] |
| EVTX via `evtx` (requisito 0.15.1, lock 0.15.1) | IMPLEMENTED / PENDING-HUMAN | `engine/` | sem fixture EVTX sanitizada versionada (F1-16); e2e com `security.evtx` real passou (`EVD-002`) |
| Auth/RBAC/server-mode | RETIRADO | ADR-0007 | não aplicável |

## Gates pendentes

Sem `EVD-*` em [`docs/EVIDENCE_REGISTER.md`](EVIDENCE_REGISTER.md), nenhum gate abaixo é declarado `PASS`. **Atualização 2026-11-14:** `cargo test/clippy/nextest` (`EVD-001`), e2e (`EVD-002`), `cargo audit`/`cargo deny` (`EVD-003`) e `doc-gate` (`EVD-004`) foram executados em Linux x86_64 e estão registrados; o que permanece sem execução está indicado abaixo.

- `cargo test --workspace --locked`;
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`;
- `cargo nextest run --workspace --locked`;
- testes adversariais Linux;
- revisão de instalação Windows sem privilégios administrativos;
- `cargo audit` e `cargo deny --workspace check` (`cargo vet` está dispensado no MVP — ADR-0018 D-2, confirmado pelo operador em 2026-11-04);
- SBOM (`cyclonedx`): fora do escopo, decisão do operador (ADR-0028); não é gate.

> Gap origem: INC-07 e INC-04 (2026-11-04): migrations atualizadas para 0001–0012, verificação de 2026-09-22 marcada como histórica e gates reescritos com ponteiro para o registro de evidências.

## Histórico

A narrativa de correções de 2026-08-20 a 2026-08-25 (frontend, rotas HTTP, Bun→Node, descarte via `DELETE`) foi movida para `docs/archive/FASE0_STATUS_historico_2026-08.md`: descreve código removido pela ADR-0017.
