# 04 — ATT&CK, caminho crítico e matriz de impacto

**What to build:** Correlacionar eventos/resultados relevantes a ATT&CK, ligar etapas de um caminho crítico e calcular impacto com confiança e referências à evidência.

**Blocked by:** 01 — Modelo analítico comum e versionamento de resultados; 03 — NIST Current/Target e Gap reproduzível

**Status:** IMPLEMENTED / PENDING-HUMAN (ver `docs/FASE1_STATUS.md` — código escrito e revisado manualmente; `cargo`/`rustc` indisponíveis neste ambiente, gates Rust não executados)

**Nota (2026-11-04) — Gap origem: EVI-05.** Verificado por leitura, sem `cargo`/`rustc` no ambiente daquele registro; **não** verificado no 1.97.1 pinado — `[PENDING-HUMAN]` (sem `EVD-*` em `docs/EVIDENCE_REGISTER.md`; a rodada manual de 2026-10-27 está narrada em `docs/rtm.md`, sem registro de evidência).

- [x] Mapping ATT&CK possui técnica, evidência, origem e confiança. (`NewAttackMapping`: `technique_id`/`evidence_id`/`basis`/`confidence`; `rule_or_model_id` guarda a técnica como origem rastreável)
- [x] Eventos relacionados podem formar uma sequência/caminho crítico explicitamente rastreável. (`insert_critical_path_stage` + `link_critical_path_stages` via `RelationType::SequenceNext`, suporta ramificação)
- [x] Matriz de impacto contém classificação e confiança. (`ImpactLevel` por disponibilidade/confidencialidade/integridade + `confidence` obrigatório, política já definida em F1-01)
- [x] Relações não suportadas por evidência são marcadas como incerteza, não como fato. (`require_evidence_for_high_confidence` rejeita HIGH/CONFIRMED sem `basis`; payload grava `evidence_status: GAP|DOCUMENTED`)
- [x] Correlação funciona offline e preserva provenance. (nenhuma chamada de rede; `provenance` gravado em cada resultado via F1-01, mesma disciplina de auditoria)
