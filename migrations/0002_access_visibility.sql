PRAGMA foreign_keys = ON;

ALTER TABLE incident_access ADD COLUMN max_visibility TEXT NOT NULL DEFAULT 'SUMMARY'
  CHECK(max_visibility IN ('SUMMARY','SANITIZED','DETAILED','RAW'));

INSERT OR REPLACE INTO schema_meta(key,value) VALUES ('fase0_access_visibility','1');
