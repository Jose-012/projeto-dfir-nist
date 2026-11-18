# 05 — IOCs e recomendações operacionais

**What to build:** Extrair IOCs com origem e transformar achados em recomendações separadas entre resposta imediata e estratégia de melhoria.

**Blocked by:** 01 — Modelo analítico comum e versionamento de resultados; 04 — ATT&CK, caminho crítico e matriz de impacto

**Status:** IMPLEMENTED / PENDING-HUMAN (ver `docs/FASE1_STATUS.md` — código escrito e revisado manualmente; `cargo`/`rustc` indisponíveis neste ambiente, gates Rust não executados)

**Nota (2026-11-04) — Gap origem: REM-02.** A rota `/api/incidents/{id}/recommendations` foi removida pela ADR-0017 (não há API HTTP). Os critérios marcados `[x]` cobrem a lógica migrada: o filtro por categoria vive em `src/service/findings.rs` (`list_recommendations`, parâmetro `category`) sobre `RecommendationCategory` (`src/persistence/ioc_recommendations.rs`). Os critérios não foram reescritos.

**Nota (2026-11-04) — Gap origem: EVI-05.** Verificado por leitura, sem `cargo`/`rustc` no ambiente daquele registro; **não** verificado no 1.97.1 pinado — `[PENDING-HUMAN]` (sem `EVD-*` em `docs/EVIDENCE_REGISTER.md`; a rodada manual de 2026-10-27 está narrada em `docs/rtm.md`, sem registro de evidência).

- [x] IOCs têm tipo, valor normalizado, origem e evidência associada. (`IocType`, `normalize_ioc_value`, `evidence_id` obrigatório em `NewIocSighting`)
- [x] Duplicatas são normalizadas sem perder provenance. (`record_ioc_sighting` acumula `sightings[]` a cada ocorrência via `supersedes`, nunca descarta uma anterior; teste `recording_the_same_ioc_twice_aggregates_sightings_without_losing_provenance`)
- [x] Recomendações imediatas e estratégicas são distintas. (`RecommendationCategory::Immediate`/`Strategic`, filtro `?category=` no `GET /api/incidents/{id}/recommendations`)
- [x] Cada recomendação referencia achados que a sustentam. (`supporting_result_ids` obrigatório, validado, ligado via `RelationType::DerivedFrom`)
- [x] Exportação preserva a mesma origem usada na UI. (`export_incident_findings` reusa `current_results_for_incident`, a mesma função das rotas de listagem)
