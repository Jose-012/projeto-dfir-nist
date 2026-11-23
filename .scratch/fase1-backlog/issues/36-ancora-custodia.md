# 36 — Âncora de cabeça e contagem da cadeia de custódia

**What to build:** Estado transacional por evidência com contagem e hash da cabeça; verificação deve detectar remoção da cauda, além das adulterações já cobertas.

**Blocked by:** None.

**Status:** VERIFIED em Linux x86_64 (2026-11-23, `EVD-059`; ADR-0030 D-1/D-3 e commit `63ac821`). Plataformas externas e incidente real não promovidos.

**Origem:** RF-004, RF-029; melhoria de engenharia, não requisito novo da demanda.

- [x] Migration aditiva preenche âncoras para entradas existentes sem reescrever histórico.
- [x] Append de custódia e atualização da âncora são atômicos, inclusive em appends concorrentes.
- [x] Remoção da cauda, contagem/cabeça divergentes e âncora ausente falham fechado.
- [x] Limitação de alteração conjunta de banco+âncora e de remoção pré-migration está documentada.
- [x] Testes Before/After, `just verify`, E2E sanitizado e `just doc-gate` executados antes de qualquer promoção. Before/After focado 7/7, verify 701 passed/1 skipped, E2E 123/123 e doc-gates inicial/final 73/73 concluídos em `EVD-059`.
