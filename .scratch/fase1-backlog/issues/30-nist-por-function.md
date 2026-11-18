# 30 — Visão NIST CSF por Function

**What to build:** Visão agregada GV/ID/PR/DE/RS/RC com contagem por status e Tier descritivo, derivada do Gap vigente, sem score oficial (spec §3 veta).

**Blocked by:** F1-19.

**Status:** IMPLEMENTED / PENDING-HUMAN (ADR-0023, 2026-10-15). Verificado em rustc 1.91.1, não no 1.97.1 pinado.

**Nota (2026-11-04) — Gap origem: EVI-05.** Verificado em rustc 1.91.1 (execução registrada no texto acima); **não** verificado no 1.97.1 pinado — `[PENDING-HUMAN]` (sem `EVD-*` em `docs/EVIDENCE_REGISTER.md`; a rodada manual de 2026-10-27 está narrada em `docs/rtm.md`, sem registro de evidência).

**Nota (2026-11-14) — DOC-01.** O pin atual é **1.99.1**; `just verify` de 2026-11-14 reexecutou os testes deste ticket no 1.99.1 (`EVD-001`). A menção a "1.97.1 pinado" acima é histórica.

**Origem:** RF-053, DEM-NIST-01..06

- [x] Cada Function mostra contagens por status (Current), subcategorias do Target e gaps por prioridade. O Tier é descritivo e do profile inteiro: aparece uma vez por profile, com a justificativa, e não por Function (o dado não existe nessa granularidade).
- [x] Nenhum número é apresentado como score oficial: só contagens, com nota explícita na tela.
- [ ] Visão alimenta o Entregável 3. A função de serviço (`function_view`) está pronta; o documento depende de F1-07.
