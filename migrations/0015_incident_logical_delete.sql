-- Exclusão lógica de incidentes solicitada pelo operador local.
-- O tombstone remove o incidente do trabalho ativo sem apagar evidências,
-- resultados analíticos, vínculos históricos ou a cadeia de auditoria.
ALTER TABLE incidents ADD COLUMN deleted_at TEXT;

INSERT OR REPLACE INTO schema_meta(key,value) VALUES ('incident_logical_delete','1');
