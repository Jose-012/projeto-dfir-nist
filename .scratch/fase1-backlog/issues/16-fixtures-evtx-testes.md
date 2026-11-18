# 16 — Fixtures EVTX e testes de integração do pipeline

**What to build:** Fixtures EVTX sanitizados (pequeno, médio, corrompido, com SystemTime ausente) com proveniência documentada em `tests/fixtures/README.md`. Teste positivo: parse → eventos → timeline ordenada. Testes de `service` com SQLite temporário: import, analyze (engine real), integridade adulterada e destroy. Medir o tamanho real de saída em EVTX grande para ajustar `MAX_OUTPUT_BYTES`/`MAX_ITEMS` (ver F1-18).

**Blocked by:** F1-13.

**Status:** VERIFIED (2026-11-15, EVD-011: `tests/fixtures/senai-demo.evtx` versionada e sanitizada, e2e 110/110 sem arquivo externo)

**Origem:** AUD-31, AUD-05, DEM-FONTE-02, DEM-FONTE-06

- [ ] Teste positivo de parsing de EVTX válido passa.
- [ ] Camada `service` possui testes de import/analyze/destroy.
- [ ] Caso de EVTX grande documenta o limite real de saída.
