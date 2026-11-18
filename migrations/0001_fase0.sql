PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS users (
  id TEXT PRIMARY KEY,
  username TEXT NOT NULL UNIQUE,
  password_hash TEXT NOT NULL,
  role TEXT NOT NULL CHECK(role IN ('ADMIN','ANALYST','VIEWER','AUDITOR')),
  active INTEGER NOT NULL DEFAULT 1 CHECK(active IN (0,1)),
  failed_login_count INTEGER NOT NULL DEFAULT 0,
  locked_until TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS sessions (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  token_hash TEXT NOT NULL UNIQUE,
  csrf_token TEXT NOT NULL,
  expires_at TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_sessions_expiry ON sessions(expires_at);

CREATE TABLE IF NOT EXISTS incidents (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  state TEXT NOT NULL DEFAULT 'OPEN' CHECK(state IN ('OPEN','CLOSED')),
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  created_by TEXT REFERENCES users(id)
);

CREATE TABLE IF NOT EXISTS incident_access (
  user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  incident_id TEXT NOT NULL REFERENCES incidents(id) ON DELETE CASCADE,
  permission TEXT NOT NULL,
  granted_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  granted_by TEXT REFERENCES users(id),
  PRIMARY KEY(user_id, incident_id, permission)
);

CREATE TABLE IF NOT EXISTS audit_events (
  sequence INTEGER PRIMARY KEY AUTOINCREMENT,
  id TEXT NOT NULL UNIQUE,
  timestamp TEXT NOT NULL,
  actor_id TEXT REFERENCES users(id),
  action TEXT NOT NULL,
  object_type TEXT NOT NULL,
  object_id TEXT NOT NULL,
  incident_id TEXT REFERENCES incidents(id),
  metadata TEXT NOT NULL,
  previous_hash TEXT,
  event_hash TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_audit_incident ON audit_events(incident_id);

CREATE TABLE IF NOT EXISTS audit_state (
  id INTEGER PRIMARY KEY CHECK(id=1),
  event_count INTEGER NOT NULL DEFAULT 0,
  head_hash TEXT
);
INSERT OR IGNORE INTO audit_state(id,event_count,head_hash) VALUES(1,0,NULL);

CREATE TRIGGER IF NOT EXISTS audit_events_no_update
BEFORE UPDATE ON audit_events
BEGIN
  SELECT RAISE(ABORT, 'audit_events are append-only');
END;

CREATE TRIGGER IF NOT EXISTS audit_events_no_delete
BEFORE DELETE ON audit_events
BEGIN
  SELECT RAISE(ABORT, 'audit_events are append-only');
END;

CREATE TABLE IF NOT EXISTS schema_meta (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
);
INSERT OR REPLACE INTO schema_meta(key,value) VALUES ('fase0','2');
