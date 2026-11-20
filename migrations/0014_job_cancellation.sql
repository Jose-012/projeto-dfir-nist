-- F1-38 / ADR-0030: cancelamento explícito e persistido de jobs de análise.
-- O SQLite não altera o CHECK de uma coluna existente; recria a tabela sem
-- reeditar a migration 0012 e preserva todos os jobs e índices anteriores.
PRAGMA defer_foreign_keys = ON;

CREATE TABLE analysis_jobs_prev AS SELECT * FROM analysis_jobs;
DROP TABLE analysis_jobs;

CREATE TABLE analysis_jobs (
    id TEXT PRIMARY KEY,
    request_id TEXT NOT NULL UNIQUE,
    evidence_id TEXT NOT NULL REFERENCES evidence(id),
    status TEXT NOT NULL CHECK(status IN ('PENDING','PROCESSING','COMPLETED','FAILED','CANCELLED')),
    max_output_bytes INTEGER NOT NULL,
    max_items INTEGER NOT NULL,
    message_count INTEGER NOT NULL DEFAULT 0,
    item_count INTEGER NOT NULL DEFAULT 0,
    output_reference TEXT,
    error_code TEXT,
    error_message TEXT,
    created_at TEXT NOT NULL,
    started_at TEXT,
    finished_at TEXT,
    kind TEXT NOT NULL DEFAULT 'EVTX_TIMELINE'
        CHECK (kind IN ('EVTX_TIMELINE', 'YARA_SCAN', 'EVTX_DETECTION')),
    warning_count INTEGER NOT NULL DEFAULT 0,
    warnings TEXT,
    cancellation_requested_at TEXT,
    CHECK (status <> 'CANCELLED' OR cancellation_requested_at IS NOT NULL)
);

INSERT INTO analysis_jobs (
    id, request_id, evidence_id, status, max_output_bytes, max_items,
    message_count, item_count, output_reference, error_code, error_message,
    created_at, started_at, finished_at, kind, warning_count, warnings,
    cancellation_requested_at
)
SELECT
    id, request_id, evidence_id, status, max_output_bytes, max_items,
    message_count, item_count, output_reference, error_code, error_message,
    created_at, started_at, finished_at, kind, warning_count, warnings,
    NULL
FROM analysis_jobs_prev;

DROP TABLE analysis_jobs_prev;

CREATE INDEX idx_analysis_jobs_evidence ON analysis_jobs(evidence_id, created_at DESC);
CREATE INDEX idx_analysis_jobs_status ON analysis_jobs(status, created_at);
CREATE INDEX idx_analysis_jobs_kind ON analysis_jobs(kind);

INSERT OR REPLACE INTO schema_meta(key,value) VALUES ('job_cancellation','1');
