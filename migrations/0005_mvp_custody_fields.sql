PRAGMA foreign_keys = ON;

ALTER TABLE evidence ADD COLUMN incident_id TEXT REFERENCES incidents(id);
ALTER TABLE evidence ADD COLUMN mime_type TEXT NOT NULL DEFAULT 'application/octet-stream';
ALTER TABLE evidence ADD COLUMN source TEXT NOT NULL DEFAULT 'LOCAL_BROWSER';
ALTER TABLE evidence ADD COLUMN collection_method TEXT NOT NULL DEFAULT 'MANUAL_UPLOAD';
ALTER TABLE evidence ADD COLUMN collection_tool TEXT;
ALTER TABLE evidence ADD COLUMN collection_tool_version TEXT;
ALTER TABLE evidence ADD COLUMN authorization_reference TEXT;
ALTER TABLE evidence ADD COLUMN collected_at TEXT;
ALTER TABLE evidence ADD COLUMN ingested_by TEXT NOT NULL DEFAULT 'local-analyst';
ALTER TABLE evidence ADD COLUMN preservation_status TEXT NOT NULL DEFAULT 'ACTIVE'
  CHECK(preservation_status IN ('ACTIVE','ARCHIVED','QUARANTINED','DESTROYED'));

CREATE TABLE IF NOT EXISTS chain_of_custody (
  sequence INTEGER PRIMARY KEY AUTOINCREMENT,
  entry_id TEXT NOT NULL UNIQUE,
  evidence_id TEXT NOT NULL REFERENCES evidence(id),
  action TEXT NOT NULL,
  actor TEXT NOT NULL,
  timestamp TEXT NOT NULL,
  reason TEXT NOT NULL,
  metadata TEXT NOT NULL,
  previous_entry_hash TEXT,
  entry_hash TEXT NOT NULL,
  signature_reference TEXT
);
CREATE INDEX IF NOT EXISTS idx_custody_evidence ON chain_of_custody(evidence_id, sequence);
CREATE TRIGGER IF NOT EXISTS custody_no_update
BEFORE UPDATE ON chain_of_custody
BEGIN
  SELECT RAISE(ABORT, 'chain_of_custody is append-only');
END;
CREATE TRIGGER IF NOT EXISTS custody_no_delete
BEFORE DELETE ON chain_of_custody
BEGIN
  SELECT RAISE(ABORT, 'chain_of_custody is append-only');
END;

INSERT OR REPLACE INTO schema_meta(key,value) VALUES ('mvp_custody','1');
