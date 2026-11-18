-- F1-01: modelo analítico comum e versionamento de resultados.
--
-- Base compartilhada por YARA-X (F1-02), NIST gap (F1-03), ATT&CK/caminho
-- crítico/impacto (F1-04) e IOCs/recomendações (F1-05): identificadores,
-- evidência/incidente de origem, confiança, provenance e versão de
-- regra/modelo, com histórico preservado em reprocessamento.
--
-- Append-only, no mesmo espírito de audit_events/chain_of_custody: nunca há
-- UPDATE. Reprocessamento insere uma nova linha com `supersedes` apontando
-- para a anterior; a linha anterior nunca é alterada ou removida. O
-- "resultado atual" para uma correlation_key é a linha mais recente que
-- nenhuma linha posterior referencia via `supersedes`.
CREATE TABLE IF NOT EXISTS analytical_results (
    sequence INTEGER PRIMARY KEY AUTOINCREMENT,
    id TEXT NOT NULL UNIQUE,
    schema_version TEXT NOT NULL,
    result_type TEXT NOT NULL CHECK (result_type IN (
        'YARA_MATCH',
        'NIST_GAP',
        'ATTACK_MAPPING',
        'CRITICAL_PATH_LINK',
        'IMPACT_ASSESSMENT',
        'IOC',
        'RECOMMENDATION'
    )),
    -- Chave estável que agrupa versões sucessivas do "mesmo" achado lógico.
    -- Cada produtor (F1-02..F1-05) decide como derivá-la (ex.: YARA usa
    -- rule_id+evidence_id+match_offset; NIST usa control_id+incident_id).
    correlation_key TEXT NOT NULL,
    -- Evidência/incidente de origem. Ambas nullable no schema; a exigência de
    -- qual (ou ambas) é obrigatória por result_type é validada em Rust
    -- (AnalyticalResultType::requires_evidence/requires_incident), não aqui,
    -- para não exigir migration a cada novo tipo/regra de negócio.
    evidence_id TEXT REFERENCES evidence(id),
    incident_id TEXT REFERENCES incidents(id),
    -- Referência lógica (não FK): analysis_jobs é apagada quando a evidência
    -- de origem é destruída (ver DELETE /api/evidence/{id} em src/web/mod.rs);
    -- um resultado analítico é forense/append-only e não pode ficar
    -- condicionado à sobrevivência do job que o produziu.
    analysis_job_id TEXT,
    rule_or_model_id TEXT,
    rule_or_model_version TEXT,
    confidence TEXT CHECK (confidence IS NULL OR confidence IN ('LOW', 'MEDIUM', 'HIGH', 'CONFIRMED')),
    -- Payload e provenance são JSON canonicalizado (mesma disciplina JCS de
    -- audit_events): chave-ordenada, sem inteiros fora do intervalo seguro de
    -- f64, para que reprodução/hash futuros sejam determinísticos.
    payload TEXT NOT NULL,
    provenance TEXT NOT NULL,
    supersedes TEXT REFERENCES analytical_results(id),
    created_at TEXT NOT NULL,
    CHECK (evidence_id IS NOT NULL OR incident_id IS NOT NULL)
);

CREATE INDEX IF NOT EXISTS idx_analytical_results_correlation
    ON analytical_results(correlation_key, sequence);
CREATE INDEX IF NOT EXISTS idx_analytical_results_evidence
    ON analytical_results(evidence_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_analytical_results_incident
    ON analytical_results(incident_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_analytical_results_type
    ON analytical_results(result_type, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_analytical_results_supersedes
    ON analytical_results(supersedes);

CREATE TRIGGER IF NOT EXISTS analytical_results_no_update
BEFORE UPDATE ON analytical_results
BEGIN
    SELECT RAISE(ABORT, 'analytical_results are append-only; insert a new row referencing supersedes instead');
END;

CREATE TRIGGER IF NOT EXISTS analytical_results_no_delete
BEFORE DELETE ON analytical_results
BEGIN
    SELECT RAISE(ABORT, 'analytical_results are append-only; insert a new row referencing supersedes instead');
END;

-- Relações entre resultados analíticos: caminho crítico (F1-04, sequência de
-- eventos/achados relacionados) e recomendações que referenciam os achados
-- que as sustentam (F1-05). Tabela própria em vez de array JSON para manter
-- integridade referencial e permitir consultas "quem referencia X" sem
-- varrer payloads.
CREATE TABLE IF NOT EXISTS analytical_result_relations (
    id TEXT PRIMARY KEY,
    result_id TEXT NOT NULL REFERENCES analytical_results(id),
    related_result_id TEXT NOT NULL REFERENCES analytical_results(id),
    relation_type TEXT NOT NULL CHECK (relation_type IN ('DERIVED_FROM', 'SEQUENCE_NEXT')),
    created_at TEXT NOT NULL,
    UNIQUE (result_id, related_result_id, relation_type)
);

CREATE INDEX IF NOT EXISTS idx_analytical_relations_result
    ON analytical_result_relations(result_id);
CREATE INDEX IF NOT EXISTS idx_analytical_relations_related
    ON analytical_result_relations(related_result_id);

CREATE TRIGGER IF NOT EXISTS analytical_result_relations_no_update
BEFORE UPDATE ON analytical_result_relations
BEGIN
    SELECT RAISE(ABORT, 'analytical_result_relations are append-only');
END;

CREATE TRIGGER IF NOT EXISTS analytical_result_relations_no_delete
BEFORE DELETE ON analytical_result_relations
BEGIN
    SELECT RAISE(ABORT, 'analytical_result_relations are append-only');
END;
