# 34 — CLI headless fina e teste ponta a ponta com engine real

**What to build:** Subcomandos headless (`import`, `evidence-list`, `analyze`, `results`, `findings`, `destroy`) como adaptador fino sobre `service::*`, saída JSON em stdout, logs em stderr, e `scripts/e2e_headless.py` para exercitar o pipeline real com um EVTX fornecido. Sem TUI (ADR-0019).

**Blocked by:** None.

**Status:** IMPLEMENTED / PENDING-HUMAN — compilado e executado em rustc 1.91.1 (não o 1.97.1 pinado) com engine real e EVTX real de 2,1 MB (2261 eventos): **38 de 38 verificações** em 2026-10-11 (timeline por páginas, filtro no banco, EVTX truncado com avisos, guarda de disco, YARA ASCII/UTF-16 sem duplicar, hash/custódia/auditoria, descarte). Subcomandos atuais: `import`, `evidence-list`, `analyze`, `results --offset --limit --filter`, `findings`, `destroy`, `diagnostics`. Fixtures EVTX versionadas continuam em F1-16.

**Nota (2026-11-04) — Gap origem: EVI-05.** Verificado em rustc 1.91.1 (execução registrada no texto acima); **não** verificado no 1.97.1 pinado — `[PENDING-HUMAN]` (sem `EVD-*` em `docs/EVIDENCE_REGISTER.md`; a rodada manual de 2026-10-27 está narrada em `docs/rtm.md`, sem registro de evidência).

**Nota (2026-11-14) — DOC-01/BUILD-01.** O pin atual é **1.99.1**. `just e2e` foi executado no 1.99.1 em Linux x86_64, com engine real e o `security.evtx` de referência: **110/110** (`EVD-002`). O script aceita qualquer EVTX, não estoura mais sem candidatos e usa os Event IDs presentes nos filtros. Windows e Linux aarch64 continuam pendentes.

**Origem:** ADR-0019; achados do primeiro teste real (AUD-35, confirmação de AUD-05).

- [x] Subcomandos headless com saída JSON e `destroy` com confirmação do nome exato.
- [x] Logs em stderr (stdout só carrega o JSON).
- [x] `scripts/e2e_headless.py` e alvo `just e2e`.
- [x] Teste ponta a ponta executado com engine real e EVTX real.
- [x] Executar `just e2e` no toolchain pinado (agora 1.99.1) em Linux x86_64 (`EVD-002`); **Windows e aarch64 continuam abertos**.
- [ ] Usar o mesmo roteiro com fixtures versionadas (F1-16) e com o dataset BEC (F1-11).
