-- Avisos não fatais do parser por job (AUD-04, AUD-36) e índice persistido da
-- timeline (F1-18, ADR-0021): permite paginar e filtrar no banco em vez de
-- carregar todos os eventos para a memória.

ALTER TABLE analysis_jobs ADD COLUMN warning_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE analysis_jobs ADD COLUMN warnings TEXT;

-- Um evento de timeline por linha, ordenado fisicamente por (job, instante UTC,
-- ordem no arquivo): a chave primária É a ordem cronológica de exibição.
CREATE TABLE timeline_events (
    job_id TEXT NOT NULL,
    sort_ts INTEGER NOT NULL,
    ord INTEGER NOT NULL,
    id TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    category TEXT NOT NULL,
    severity TEXT NOT NULL,
    event_id TEXT,
    provider TEXT,
    channel TEXT,
    evidence_id TEXT,
    raw_reference TEXT,
    record_id TEXT,
    PRIMARY KEY (job_id, sort_ts, ord)
) WITHOUT ROWID;

-- Marca que o job já foi indexado (e quantos itens não puderam ser exibidos).
CREATE TABLE timeline_index (
    job_id TEXT PRIMARY KEY,
    event_count INTEGER NOT NULL,
    skipped_count INTEGER NOT NULL DEFAULT 0,
    indexed_at TEXT NOT NULL
);
