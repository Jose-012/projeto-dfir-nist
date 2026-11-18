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
| Interface nativa (eframe/egui/egui_extras) | VERIFIED (Linux x86_64) | `src/gui/`, `src/service/`, ADR-0017, `docs/NATIVE_UI_ARCHITECTURE.md` | `EVD-009` (janela real X11) e `EVD-012` (426/426 no Rust 1.100.0); Windows/aarch64 seguem `WVR-001`/`WVR-002` |
| EVTX via `evtx` (requisito e lock 0.15.1) | VERIFIED (Linux x86_64) | `engine/`, `tests/fixtures/senai-demo.evtx` | `EVD-011`/`EVD-012`: fixture sanitizada versionada e e2e 110/110 sem arquivo externo |
| Auth/RBAC/server-mode | RETIRADO | ADR-0007 | não aplicável |

## Gates e limites de verificação

- `PASS` em Linux x86_64 com Rust/Cargo 1.100.0: validação estática, fmt, check, test, nextest 426/426, clippy e e2e 110/110 (`EVD-012`); supply chain verde (`EVD-012`).
- `WAIVED`: Windows (`WVR-001`), Linux aarch64 (`WVR-002`) e CI automático (`WVR-003`), todos com validade e escopo em `docs/EVIDENCE_REGISTER.md`.
- `WVR-004`: exceção temporária para `RUSTSEC-2026-0459`; risco aceito não equivale a verificação de inexploração.
- `PENDING-HUMAN`: execução sobre incidente real. A fixture sanitizada e o cenário DEMO não substituem essa verificação.
- SBOM (`cyclonedx`): fora do escopo por ADR-0028, não é gate.

> Gap origem: INC-07 e INC-04 (2026-11-04): migrations atualizadas para 0001–0012, verificação de 2026-09-22 marcada como histórica e gates reescritos com ponteiro para o registro de evidências.

## Histórico

A narrativa de correções de 2026-08-20 a 2026-08-25 (frontend, rotas HTTP, Bun→Node, descarte via `DELETE`) foi movida para `docs/archive/FASE0_STATUS_historico_2026-08.md`: descreve código removido pela ADR-0017.
