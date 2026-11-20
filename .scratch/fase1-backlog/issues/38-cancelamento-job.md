# 38 — Cancelamento explícito de job de análise

**What to build:** Sinal interno por job que acorda esperas, encerra o engine supervisionado e persiste estado `CANCELLED` com erro específico e auditoria/custódia.

**Blocked by:** None.

**Status:** PLANNED (ativado pelo operador em 2026-11-20, ADR-0030 D-1/D-4).

**Origem:** RF-037, RF-041, RF-042; melhoria de engenharia, não requisito novo da demanda.

- [ ] GUI e CLI permitem cancelar a análise vigente sem shell e sem confundir seletor de arquivo cancelado com job.
- [ ] Espera por semáforo, escrita, leitura JSONL e espera do processo respondem ao cancelamento.
- [ ] Resultados parciais não são indexados nem apresentados como análise concluída.
- [ ] `CANCELLED` é persistido e auditado; job que já concluiu permanece `COMPLETED`.
- [ ] Testes provam término do processo, ausência de escrita analítica, resposta tardia ignorada e recovery preservado.
