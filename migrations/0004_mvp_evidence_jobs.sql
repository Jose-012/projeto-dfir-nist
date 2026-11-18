PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS evidence (
  id TEXT PRIMARY KEY,
  original_filename TEXT NOT NULL,
  storage_reference TEXT NOT NULL UNIQUE,
  format TEXT NOT NULL,
  size_bytes INTEGER NOT NULL CHECK(size_bytes >= 0),
  blake3 TEXT NOT NULL,
  sha256 TEXT NOT NULL,
  imported_at TEXT NOT NULL,
  status TEXT NOT NULL CHECK(status IN ('FORENSIC','SANITIZED','DEMO'))
);
CREATE INDEX IF NOT EXISTS idx_evidence_imported_at ON evidence(imported_at DESC);

CREATE TABLE IF NOT EXISTS analysis_jobs (
  id TEXT PRIMARY KEY,
  request_id TEXT NOT NULL UNIQUE,
  evidence_id TEXT NOT NULL REFERENCES evidence(id),
  status TEXT NOT NULL CHECK(status IN ('PENDING','PROCESSING','COMPLETED','FAILED')),
  max_output_bytes INTEGER NOT NULL,
  max_items INTEGER NOT NULL,
  message_count INTEGER NOT NULL DEFAULT 0,
  item_count INTEGER NOT NULL DEFAULT 0,
  output_reference TEXT,
  error_code TEXT,
  error_message TEXT,
  created_at TEXT NOT NULL,
  started_at TEXT,
  finished_at TEXT
);
CREATE INDEX IF NOT EXISTS idx_analysis_jobs_evidence ON analysis_jobs(evidence_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_analysis_jobs_status ON analysis_jobs(status, created_at);

INSERT OR REPLACE INTO schema_meta(key,value) VALUES ('mvp_evidence_jobs','1');
