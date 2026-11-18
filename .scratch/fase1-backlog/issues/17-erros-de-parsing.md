# 17 — Erros de parsing persistidos e visíveis (RF-006)

**What to build:** O engine emite `PARSER_RECORD_ERROR`, mas o app só grava mensagens `RESULT`. Gravar os erros (contagem e amostra) em `results.jsonl`/banco, exibir na tela de resultados e terminar o job como concluído **com erros parciais** (nunca como `COMPLETED` limpo).

**Blocked by:** F1-16.

**Status:** IMPLEMENTED / PENDING-HUMAN (ADR-0021, 2026-10-11) — avisos não fatais do engine (`EVTX_TRUNCATED`, `EVTX_RECORDS_MISSING`, `PARSER_RECORD_ERROR`) agregados no job (`warning_count`, `warnings`), exibidos na GUI e na CLI; verificado com EVTX real truncado (2 avisos) e íntegro (nenhum). Heurística: truncamento exato na fronteira de chunk com cabeçalho defasado pode passar. **Evidência original (AUD-36):** um EVTX truncado em 1,2 MB de 2,1 MB termina `COMPLETED` com 1587 de 2261 eventos e nenhuma mensagem de erro (o engine não emite `PARSER_RECORD_ERROR` para chunks ausentes, e o app descarta erros não fatais). Este ticket deve cobrir também a comparação entre o número de chunks do cabeçalho EVTX e os realmente lidos, marcando o job como parcial.

**Origem:** AUD-04, RF-006

- [ ] Teste com EVTX parcialmente corrompido exibe contagem e amostra de erros.
- [ ] Estado do job distingue conclusão limpa de conclusão com erros.
