# 27 — Importador estruturado CSV/JSONL

**What to build:** Mapeamento declarativo de colunas para o evento normalizado, para exportações de SIEM, autenticação, firewall/IDS, Zeek/Wireshark em CSV e logs de e-mail/identidade (M365/Entra, cenário BEC), sem parser por produto. Linhas inválidas são contadas e exibidas, nunca descartadas em silêncio; análise roda no engine isolado.

**Blocked by:** F1-26.

**Status:** DEFERRED — decisão do operador (ADR-0025, 2026-10-27): o escopo do protótipo é EVTX. Independente: entra por um importador que produza o mesmo `NormalizedEvent`; as regras de F1-28 passam a valer sobre ele sem alteração.

**Origem:** RF-049, DEM-FONTE-03, DEM-FONTE-04, DEM-FONTE-05, DEM-FONTE-07

- [ ] Exportação CSV de autenticação e JSONL de identidade viram eventos na timeline.
- [ ] Linhas inválidas aparecem com contagem e amostra.
- [ ] Arquivo hostil (campos gigantes, linhas sem fim, encoding inválido) é rejeitado com limite explícito.
