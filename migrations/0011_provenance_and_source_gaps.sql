-- ADR-0023 (Etapa 6): proveniência da evidência e registro de lacunas de
-- fonte (F1-31, RF-054) e dado pessoal (F1-32, RF-055).
--
-- 1. `evidence.provided_by`: quem ou qual área forneceu a evidência (RB-002).
--    As demais colunas de proveniência já existem desde a 0005.
-- 2. `evidence.contains_personal_data`: o operador marca a evidência que
--    contém dado pessoal; a exportação avisa e oferece mascarar (RB-006).
-- 3. `analytical_results` aceita o tipo `SOURCE_GAP`: o registro de uma
--    lacuna (fonte esperada, período sem dados, motivo) entra nos relatórios
--    (RB-001). O `CHECK` de `result_type` não se altera no SQLite, então a
--    tabela é recriada com a mesma receita da 0010: dentro da transação do
--    sqlx, com `defer_foreign_keys`, preservando linhas, `sequence`,
--    índices, triggers append-only e a tabela de relações.
--
-- Evidência é preservada e imutável por regra de negócio; estas colunas são
-- gravadas só na importação.
ALTER TABLE evidence ADD COLUMN provided_by TEXT;
ALTER TABLE evidence ADD COLUMN contains_personal_data INTEGER NOT NULL DEFAULT 0
    CHECK (contains_personal_data IN (0, 1));

PRAGMA defer_foreign_keys = ON;

CREATE TABLE analytical_results_prev AS SELECT * FROM analytical_results;

-- O DROP remove também índices e triggers da tabela. O DELETE implícito do
-- DROP não dispara triggers, então o append-only da 0006 não interfere.
DROP TABLE analytical_results;

CREATE TABLE analytical_results (
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
        'RECOMMENDATION',
        'FINDING',
        'ROOT_CAUSE',
        'PLAN_ITEM',
        'SOURCE_GAP'
    )),
    correlation_key TEXT NOT NULL,
    evidence_id TEXT REFERENCES evidence(id),
    incident_id TEXT REFERENCES incidents(id),
    analysis_job_id TEXT,
    rule_or_model_id TEXT,
    rule_or_model_version TEXT,
    confidence TEXT CHECK (confidence IS NULL OR confidence IN ('LOW', 'MEDIUM', 'HIGH', 'CONFIRMED')),
    payload TEXT NOT NULL,
    provenance TEXT NOT NULL,
    supersedes TEXT REFERENCES analytical_results(id),
    created_at TEXT NOT NULL,
    CHECK (evidence_id IS NOT NULL OR incident_id IS NOT NULL)
);

INSERT INTO analytical_results (
    sequence, id, schema_version, result_type, correlation_key,
    evidence_id, incident_id, analysis_job_id,
    rule_or_model_id, rule_or_model_version, confidence,
    payload, provenance, supersedes, created_at
)
SELECT
    sequence, id, schema_version, result_type, correlation_key,
    evidence_id, incident_id, analysis_job_id,
    rule_or_model_id, rule_or_model_version, confidence,
    payload, provenance, supersedes, created_at
FROM analytical_results_prev
ORDER BY sequence;

DROP TABLE analytical_results_prev;

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
