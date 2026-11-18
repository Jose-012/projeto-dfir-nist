-- ADR-0023 (Etapa 1): novos tipos de resultado analítico FINDING e
-- ROOT_CAUSE (F1-25, RF-051) e PLAN_ITEM (F1-29, RF-052).
--
-- O `CHECK` de `result_type` criado na 0006 lista os tipos permitidos, e o
-- SQLite não altera constraints de uma tabela existente. A única forma de
-- aceitar os dois tipos novos é recriar `analytical_results` com a lista
-- estendida. Não se edita a 0006: o sqlx valida o checksum das migrations
-- já aplicadas.
--
-- Como a migration roda dentro de uma transação do sqlx, `PRAGMA
-- foreign_keys = OFF` não tem efeito aqui. `defer_foreign_keys = ON` adia
-- todas as verificações de FK até o COMMIT; ao reinserir as linhas, as
-- referências de `analytical_result_relations` e de `supersedes` voltam a
-- ser satisfeitas e o contador de violações zera. O pragma é desligado
-- sozinho no COMMIT/ROLLBACK.
--
-- ROOT_CAUSE entra agora, junto com FINDING, porque o ticket F1-25 já prevê
-- os dois tipos: incluí-lo depois exigiria reconstruir a tabela outra vez.
--
-- Preservado sem alteração: todas as colunas e valores (inclusive
-- `sequence`, que mantém a ordem e o contador do AUTOINCREMENT), os cinco
-- índices e os dois triggers append-only. A tabela de relações não é
-- tocada: referencia `analytical_results(id)` pelo nome, que continua o
-- mesmo.
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
        'PLAN_ITEM'
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
