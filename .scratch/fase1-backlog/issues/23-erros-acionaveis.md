# 23 — Mensagens de erro acionáveis e auditoria sem `let _ =`

**What to build:** Mensagem com o que aconteceu, impacto, ação sugerida e código técnico (`error_code` do job); sem vazar caminhos. Remover `let _ =` de auditoria em caminho de falha de integridade e reescrever o teste que fixa a mensagem genérica.

**Blocked by:** None — pode começar imediatamente.

**Status:** VERIFIED (2026-11-15, `EVD-010`) — nenhum `let _ =` em código de produção fora de testes (`src/`, `engine/`, `protocol/`): `rollback_loudly` (persistence), `discard_temp_file/dir`, `mark_job_failed_loudly` (service), sends de canal da GUI/backend, abort de stderr do engine, kill/wait do processo e limpeza de temporários/backup logados com `tracing` conforme criticidade; `NotFound` tratado como esperado. Testes próprios para cada helper novo. Erros de engine/storage continuam com mensagem específica via `AppError::user_message()`.

**Origem:** AUD-14, AUD-25

- [ ] Erros de engine, storage e integridade têm mensagem específica e acionável.
- [x] Falha de gravação de auditoria de integridade nunca é ignorada.
