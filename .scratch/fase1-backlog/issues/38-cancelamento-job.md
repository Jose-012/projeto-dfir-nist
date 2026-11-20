# 38 — Cancelamento explícito de job de análise

**What to build:** Sinal interno por job que acorda esperas, encerra o engine supervisionado e persiste estado `CANCELLED` com erro específico e auditoria/custódia.

**Blocked by:** None.

**Status:** VERIFIED em Linux x86_64 (2026-11-20, ADR-0030 D-1/D-4, `EVD-056`). Verificação externa e fluxo nativo X11 permanecem nos estados próprios.

**Origem:** RF-037, RF-041, RF-042; melhoria de engenharia, não requisito novo da demanda.

- [x] GUI e CLI permitem cancelar a análise vigente sem shell e sem confundir seletor de arquivo cancelado com job.
- [x] Espera por semáforo, escrita, leitura JSONL e espera do processo respondem ao cancelamento.
- [x] Resultados parciais não são indexados nem apresentados como análise concluída.
- [x] `CANCELLED` é persistido e auditado; job que já concluiu permanece `COMPLETED`.
- [x] Testes provam término do processo, ausência de escrita analítica, resposta tardia ignorada e recovery preservado.

**Evidência:** `EVD-056`; 12 testes filtrados, um teste entre instâncias com PID encerrado/zero índice/cadeias íntegras, `just verify` 573 passed/1 skipped e E2E 123/123. O teste de GUI é headless; `EVD-050` continua `BLOCKED` para o fluxo nativo X11.
