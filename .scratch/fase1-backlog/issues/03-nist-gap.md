# 03 — NIST Current/Target e Gap reproduzível

**What to build:** Permitir definir Current Profile e Target Profile persistidos localmente e produzir uma Gap Analysis reproduzível a partir desses dados.

**Blocked by:** 01 — Modelo analítico comum e versionamento de resultados

**Status:** IMPLEMENTED / PENDING-HUMAN (ver `docs/FASE1_STATUS.md` — código escrito e revisado manualmente; `cargo`/`rustc` indisponíveis neste ambiente, gates Rust não executados)

**Nota (2026-11-04) — Gap origem: EVI-05.** Verificado por leitura, sem `cargo`/`rustc` no ambiente daquele registro; **não** verificado no 1.97.1 pinado — `[PENDING-HUMAN]` (sem `EVD-*` em `docs/EVIDENCE_REGISTER.md`; a rodada manual de 2026-10-27 está narrada em `docs/rtm.md`, sem registro de evidência).

- [x] Current Profile e Target Profile são persistidos com versão/fonte. (`nist_profiles.version_label`/`.source`, `migrations/0008_nist_profiles.sql`)
- [x] Categorias/subcategorias relevantes podem ser comparadas. (`nist_profile_entries.subcategory_id`, comparado por `compute_gaps`)
- [x] Gap é derivado de dados persistidos e pode ser reproduzido sem Internet. (`compute_gaps` é função pura, sem I/O; teste `compute_gaps_is_deterministic_across_repeated_calls`)
- [x] Cada gap possui origem e contexto suficientes para auditoria. (persistido como `AnalyticalResultType::NistGap` via F1-01, com `provenance` referenciando os ids/versões dos dois profiles comparados)
- [x] Alterações de profile não apagam versões anteriores. (append-only + `supersedes`; teste `newer_profile_version_supersedes_the_previous_one_and_keeps_history`)
