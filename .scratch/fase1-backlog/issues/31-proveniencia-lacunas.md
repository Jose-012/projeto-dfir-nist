# 31 — Proveniência, cobertura declarada e lacunas

**What to build:** Preencher na importação: origem (sistema/área), método e ferramenta de coleta, data de coleta e referência de autorização (colunas existem e nunca são usadas); corrigir o default stale `LOCAL_BROWSER`. Tela "formatos analisados × apenas preservados" por evidência. Registro de **lacuna**: fonte esperada, período sem dados, motivo (retenção, não fornecido); entra nos relatórios.

**Blocked by:** F1-19.

**Status:** IMPLEMENTED / PENDING-HUMAN (ADR-0023, 2026-10-18). Verificado em rustc 1.91.1, não no 1.97.1 pinado.

**Nota (2026-11-04) — Gap origem: EVI-05.** Verificado em rustc 1.91.1 (execução registrada no texto acima); **não** verificado no 1.97.1 pinado — `[PENDING-HUMAN]` (sem `EVD-*` em `docs/EVIDENCE_REGISTER.md`; a rodada manual de 2026-10-27 está narrada em `docs/rtm.md`, sem registro de evidência).

**Origem:** RF-054, RB-001, RB-002, DEM-FONTE-01, AUD-19

- [x] Importação grava origem, método e autorização (e quem forneceu, ferramenta, versão e data da coleta); o que não é informado fica como não informado, sem suposição.
- [x] A UI declara o que é analisado e o que é só preservado (coluna "Tratamento" e resumo da evidência selecionada).
- [x] Lacuna registrada aparece no Entregável 1 (seção 2.1), junto com a proveniência não informada de cada evidência.
