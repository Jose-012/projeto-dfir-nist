-- F1-02: distingue o tipo de análise de um job. Antes desta migration, todo
-- job era implicitamente EVTX_TIMELINE (era o único analysis_kind que o
-- protocolo suportava); o DEFAULT preserva esse significado histórico para
-- linhas já existentes sem reescrevê-las.
ALTER TABLE analysis_jobs
    ADD COLUMN kind TEXT NOT NULL DEFAULT 'EVTX_TIMELINE'
    CHECK (kind IN ('EVTX_TIMELINE', 'YARA_SCAN'));

CREATE INDEX IF NOT EXISTS idx_analysis_jobs_kind ON analysis_jobs(kind);
