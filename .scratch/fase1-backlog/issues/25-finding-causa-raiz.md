# 25 — Finding/vulnerabilidade, causa-raiz e lacunas de conclusão

**What to build:** Novos tipos analíticos: `FINDING` (condição explorada, evidência, impacto, severidade, confiança) e `ROOT_CAUSE` (nível: causa-raiz / causa provável / fatores contribuintes, com lacunas da conclusão). Matriz de risco derivada de severidade × confiança, sem inventar valores. Migration nova (o CHECK de `result_type` precisa ser reconstruído).

**Blocked by:** F1-19.

**Status:** IMPLEMENTED / PENDING-HUMAN (ADR-0023, 2026-10-14). Migration `0010` e tipos em 2026-10-13; modelo, serviço e aba "Achados" em 2026-10-14. Verificado em rustc 1.91.1, não no 1.97.1 pinado.

**Nota (2026-11-04) — Gap origem: EVI-05.** Verificado em rustc 1.91.1 (execução registrada no texto acima); **não** verificado no 1.97.1 pinado — `[PENDING-HUMAN]` (sem `EVD-*` em `docs/EVIDENCE_REGISTER.md`; a rodada manual de 2026-10-27 está narrada em `docs/rtm.md`, sem registro de evidência).

**Nota (2026-11-14) — DOC-01.** O pin atual é **1.99.1** (`rust-toolchain.toml`/`toolchains.toml`; `rust-version = "1.99"` nos três manifests). A rodada de 2026-11-14 executou `just verify` no 1.99.1, incluindo os testes deste ticket (`EVD-001`). A menção a "1.97.1 pinado" acima é histórica e foi preservada.

**Origem:** AUD-12, RF-051, DEM-ID-01, DEM-ID-02, DEM-CR-01

- [x] Finding exige evidência e confiança; liga a ATT&CK e a subcategoria NIST.
- [x] Conclusão de causa registra o nível e as lacunas.
- [x] Migration testada com banco existente.
