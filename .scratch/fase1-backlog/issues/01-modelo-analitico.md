# 01 — Modelo analítico comum e versionamento de resultados

**What to build:** Uma base persistente e versionada para resultados analíticos, permitindo que YARA-X, NIST, ATT&CK, IOCs, impacto e recomendações compartilhem identificadores, evidência, confiança, origem e versão de regra/modelo.

**Blocked by:** None — can start immediately.

**Status:** IMPLEMENTED / PENDING-HUMAN (ver `docs/FASE1_STATUS.md` — código escrito e revisado manualmente; `cargo`/`rustc` indisponíveis neste ambiente, gates Rust não executados)

**Nota (2026-11-04) — Gap origem: EVI-05.** Verificado por leitura, sem `cargo`/`rustc` no ambiente daquele registro; **não** verificado no 1.97.1 pinado — `[PENDING-HUMAN]` (sem `EVD-*` em `docs/EVIDENCE_REGISTER.md`; a rodada manual de 2026-10-27 está narrada em `docs/rtm.md`, sem registro de evidência).

- [x] Resultados analíticos possuem tipo, versão, incidente/evidência de origem, timestamp e provenance. (`migrations/0006_analytical_results.sql`, `src/persistence/analytical_results.rs`)
- [x] Evidência e confiança são campos obrigatórios quando exigidos pelos requisitos analíticos. (`AnalyticalResultType::requires_evidence`/`requires_incident`/`requires_confidence`, validado em `validate_new_result`)
- [x] Reprocessamento preserva histórico e não sobrescreve silenciosamente resultados anteriores. (append-only + `supersedes`; teste `reprocessing_preserves_history_via_supersedes`)
- [x] Modelo é compatível com persistência local/offline e audit trail. (SQLite via `sqlx`, evento `ANALYTICAL_RESULT_RECORDED` na mesma transação; teste `insert_writes_audit_event`)
- [x] RTM rastreia os requisitos analíticos para o novo contrato. (`docs/rtm.md`, atualização 2026-08-25)
