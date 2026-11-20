# 36 — Âncora de cabeça e contagem da cadeia de custódia

**What to build:** Estado transacional por evidência com contagem e hash da cabeça; verificação deve detectar remoção da cauda, além das adulterações já cobertas.

**Blocked by:** None.

**Status:** IMPLEMENTED (2026-11-20, ADR-0030 D-1/D-3; gates comuns e E2E ainda PENDING).

**Origem:** RF-004, RF-029; melhoria de engenharia, não requisito novo da demanda.

- [x] Migration aditiva preenche âncoras para entradas existentes sem reescrever histórico.
- [x] Append de custódia e atualização da âncora são atômicos, inclusive em appends concorrentes.
- [x] Remoção da cauda, contagem/cabeça divergentes e âncora ausente falham fechado.
- [x] Limitação de alteração conjunta de banco+âncora e de remoção pré-migration está documentada.
- [ ] Testes Before/After, `just verify`, E2E sanitizado e `just doc-gate` executados antes de qualquer promoção. Before/After focado e `just doc-gate` concluídos; `just verify` e E2E sanitizado PENDING.
