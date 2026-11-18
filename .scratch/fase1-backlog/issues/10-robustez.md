# 10 — Robustez operacional (reduzida): recovery, shutdown limpo, espaço em disco, backup de migration

**What to build:** Somente o que protege a evidência e o estado: (1) na partida, jobs `PROCESSING` órfãos são marcados `FAILED` com código específico e auditados; (2) fechar a janela/SIGINT/SIGTERM durante análise cancela o engine e marca o job, nunca como `COMPLETED`; (3) `analyze` marca o job em **todos** os caminhos de erro (inclui falha de `begin_immediate`); (4) checagem de espaço livre antes de importar (alerta, bloqueio, auditado; nunca apaga evidência); (5) backup verificado do banco antes de migration. **Retry/backoff/`PERMANENTLY_FAILED`/DLQ removidos** (RF-041 substituído, ADR-0018): parse determinístico local não melhora ao repetir.

**Blocked by:** F1-16 (testes de serviço).

**Status:** IMPLEMENTED / PENDING-HUMAN (RF-032 automático: ADR-0026, 2026-10-27; demais itens como antes).

**Origem:** RF-032 (reduzido), RF-042, RF-043; AUD-03, AUD-11, AUD-24

- [ ] Job órfão é recuperado na partida e aparece na UI como falho com código de erro.
- [ ] Nenhum caminho de `analyze` deixa o job em `PROCESSING` (teste com falha injetada).
- [ ] Fechamento da janela/sinal cancela o engine e registra auditoria.
- [ ] Import com espaço insuficiente é bloqueado com mensagem acionável.
- [ ] Banco copiado e a cópia verificada antes de aplicar migration.
