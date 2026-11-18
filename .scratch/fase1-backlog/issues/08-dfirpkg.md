# 08 — `.dfirpkg` portátil (ADIADO para a Fase 4)

**What to build:** Pacote portátil de caso com manifest, checksums e validação completa **antes** de persistir. Adiado: a demanda não pede pacote; o resultado é entregue pelo F1-07. Importar pacote cria uma nova entrada não confiável (zip-slip, bomba de compressão, manifest forjado).

**Blocked by:** Fase 4.

**Status:** DEFERRED — Fase 4 (ADR-0018). Não bloqueia o MVP.

**Origem:** RF-025 movido para Fase 4

- [ ] (Fase 4) Importação rejeita pacote truncado, alterado, com zip-slip ou bomba de compressão (spec §20.3).
- [ ] (Fase 4) Round-trip preserva proveniência e custódia.
