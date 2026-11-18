# 02 — YARA-X e resultados estruturados

**What to build:** Detectar conteúdo de evidência com um ruleset YARA-X versionado, persistir matches estruturados e apresentar origem, regra, versão e contexto sem expor dados arbitrários.

**Blocked by:** 01 — Modelo analítico comum e versionamento de resultados

**Status:** IMPLEMENTED / PENDING-HUMAN (ver `docs/FASE1_STATUS.md` — código escrito e revisado manualmente; `cargo`/`rustc` indisponíveis neste ambiente, gates Rust não executados; `Cargo.lock` ainda não regenerado para `yara-x`/`sha2`)

**Nota (2026-11-04) — Gap origem: REM-02.** `src/web/mod.rs` foi removido pela ADR-0017 (não há servidor HTTP). Os critérios marcados `[x]` cobrem a lógica migrada: a persistência dos resultados `YARA_MATCH` na conclusão do job está em `src/service/mod.rs` (`analyze`, bloco que monta `yara_items`) e em `src/persistence/analytical_results.rs` (`AnalyticalResultType::YaraMatch`). Os critérios não foram reescritos.

**Nota (2026-11-04) — Gap origem: EVI-05.** Verificado por leitura, sem `cargo`/`rustc` no ambiente daquele registro; **não** verificado no 1.97.1 pinado — `[PENDING-HUMAN]` (sem `EVD-*` em `docs/EVIDENCE_REGISTER.md`; a rodada manual de 2026-10-27 está narrada em `docs/rtm.md`, sem registro de evidência).

- [x] Ruleset local/offline é versionado e identificado em cada resultado. (`engine/rules/dfir_mvp_ruleset.yar`, `RULESET_ID`/`RULESET_VERSION`/`ruleset_sha256()` em `engine/src/yara_scan.rs`)
- [x] Match contém regra, evidência, referência de origem e confiança/contexto. (item `YARA_MATCH`: `rule_id`, `evidence_id`=`input_reference`, `confidence`/`description`/`reference` da metadata da regra)
- [x] Falha de parsing/execução é registrada e não descartada silenciosamente. (`YARA_RULESET_ERROR`/`YARA_SCAN_ERROR` propagados como `ErrorMessage`, nunca engolidos; teste `yara_scan_reports_a_controlled_error_for_an_unreadable_path`)
- [x] Resultados são persistidos como artefatos analíticos reproduzíveis. (`AnalyticalResultType::YaraMatch` via F1-01, na mesma transação de conclusão do job em `src/web/mod.rs`)
- [x] Teste de regressão cobre match positivo, ausência de match e falha controlada. (`engine/src/yara_scan.rs` mod tests: `yara_scan_detects_eicar_test_string`, `yara_scan_reports_no_match_for_benign_content`, `yara_scan_reports_a_controlled_error_for_an_unreadable_path`, `invalid_ruleset_source_is_reported_as_a_controlled_error_not_a_panic`)
