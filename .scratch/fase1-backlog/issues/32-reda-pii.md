# 32 — Redação de dados pessoais na exportação

**What to build:** Flag "contém dado pessoal" por evidência; exportação com opção de mascarar IP/usuário/e-mail e aviso antes de exportar sem máscara; registro em auditoria.

**Blocked by:** F1-07.

**Status:** IMPLEMENTED / PENDING-HUMAN (ADR-0023, 2026-10-18). Verificado em rustc 1.91.1, não no 1.97.1 pinado.

**Nota (2026-11-04) — Gap origem: EVI-05.** Verificado em rustc 1.91.1 (execução registrada no texto acima); **não** verificado no 1.97.1 pinado — `[PENDING-HUMAN]` (sem `EVD-*` em `docs/EVIDENCE_REGISTER.md`; a rodada manual de 2026-10-27 está narrada em `docs/rtm.md`, sem registro de evidência).

**Nota (2026-11-14) — DOC-01.** O pin atual é **1.99.1**; `just verify` de 2026-11-14 reexecutou os testes deste ticket no 1.99.1 (`EVD-001`). A menção a "1.97.1 pinado" acima é histórica.

**Origem:** RF-055, RB-006

- [x] Exportação mascarada não contém os valores originais (testado nos cinco documentos e no manifest).
- [x] Aviso e escolha explícita antes de exportar sem máscara (confirmação na interface; `--no-redaction` na CLI; registro em auditoria).
