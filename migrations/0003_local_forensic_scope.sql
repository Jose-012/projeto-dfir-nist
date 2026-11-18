PRAGMA foreign_keys = ON;

-- Pre-release migration: preserve migration history while removing the now-unused
-- authentication/access schema from databases created by older revisions.
CREATE TABLE incidents_local (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  state TEXT NOT NULL DEFAULT 'OPEN' CHECK(state IN ('OPEN','CLOSED')),
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
INSERT INTO incidents_local(id,name,state,created_at)
SELECT id,name,state,created_at FROM incidents;

CREATE TABLE audit_events_local (
  sequence INTEGER PRIMARY KEY AUTOINCREMENT,
  id TEXT NOT NULL UNIQUE,
  timestamp TEXT NOT NULL,
  actor_id TEXT,
  action TEXT NOT NULL,
  object_type TEXT NOT NULL,
  object_id TEXT NOT NULL,
  incident_id TEXT,
  metadata TEXT NOT NULL,
  previous_hash TEXT,
  event_hash TEXT NOT NULL
);
INSERT INTO audit_events_local(
  sequence,id,timestamp,actor_id,action,object_type,object_id,incident_id,metadata,previous_hash,event_hash
)
SELECT
  sequence,id,timestamp,actor_id,action,object_type,object_id,incident_id,metadata,previous_hash,event_hash
FROM audit_events;

DROP TABLE audit_events;
DROP TABLE incident_access;
DROP TABLE sessions;
DROP TABLE incidents;
DROP TABLE users;

ALTER TABLE incidents_local RENAME TO incidents;
ALTER TABLE audit_events_local RENAME TO audit_events;

CREATE INDEX idx_audit_incident ON audit_events(incident_id);

CREATE TRIGGER audit_events_no_update
BEFORE UPDATE ON audit_events
BEGIN
  SELECT RAISE(ABORT, 'audit_events are append-only');
END;

CREATE TRIGGER audit_events_no_delete
BEFORE DELETE ON audit_events
BEGIN
  SELECT RAISE(ABORT, 'audit_events are append-only');
END;

INSERT OR REPLACE INTO schema_meta(key,value) VALUES ('fase0_local_scope','1');
