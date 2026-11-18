-- F1-26 (evento normalizado com campos estruturais) e F1-28 (candidatos de
-- detecção). Ver ADR-0025.

-- `analysis_jobs.kind` foi criado na 0007 com `CHECK (kind IN ('EVTX_TIMELINE',
-- 'YARA_SCAN'))`. A análise de detecção (`EVTX_DETECTION`) precisa de um job, e
-- o SQLite não altera um CHECK de coluna existente: a tabela é recriada com a
-- lista ampliada, como a 0010 fez com `analytical_results`. Nenhuma outra
-- tabela referencia `analysis_jobs` por chave estrangeira e ela não tem
-- triggers; `defer_foreign_keys` cobre a janela entre o DROP e a recarga (a FK
-- dela mesma, para `evidence`, é satisfeita ao reinserir).
PRAGMA defer_foreign_keys = ON;

CREATE TABLE analysis_jobs_prev AS SELECT * FROM analysis_jobs;
DROP TABLE analysis_jobs;

CREATE TABLE analysis_jobs (
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
    finished_at TEXT,
    kind TEXT NOT NULL DEFAULT 'EVTX_TIMELINE'
        CHECK (kind IN ('EVTX_TIMELINE', 'YARA_SCAN', 'EVTX_DETECTION')),
    warning_count INTEGER NOT NULL DEFAULT 0,
    warnings TEXT
);

INSERT INTO analysis_jobs (
    id, request_id, evidence_id, status, max_output_bytes, max_items,
    message_count, item_count, output_reference, error_code, error_message,
    created_at, started_at, finished_at, kind, warning_count, warnings
)
SELECT
    id, request_id, evidence_id, status, max_output_bytes, max_items,
    message_count, item_count, output_reference, error_code, error_message,
    created_at, started_at, finished_at, kind, warning_count, warnings
FROM analysis_jobs_prev;

DROP TABLE analysis_jobs_prev;

CREATE INDEX idx_analysis_jobs_evidence ON analysis_jobs(evidence_id, created_at DESC);
CREATE INDEX idx_analysis_jobs_status ON analysis_jobs(status, created_at);
CREATE INDEX idx_analysis_jobs_kind ON analysis_jobs(kind);

-- Campos estruturais do evento, extraídos pelo engine (ou, em resultados
-- antigos, pelo app a partir do próprio registro). Todos opcionais.
ALTER TABLE timeline_events ADD COLUMN host TEXT;
ALTER TABLE timeline_events ADD COLUMN user_name TEXT;
ALTER TABLE timeline_events ADD COLUMN src_ip TEXT;
ALTER TABLE timeline_events ADD COLUMN dst_ip TEXT;
ALTER TABLE timeline_events ADD COLUMN process TEXT;
ALTER TABLE timeline_events ADD COLUMN action TEXT;
ALTER TABLE timeline_events ADD COLUMN outcome TEXT;
ALTER TABLE timeline_events ADD COLUMN source_type TEXT;

-- Os índices de timeline existentes foram montados sem esses campos. Apagar a
-- marca de "indexado" faz cada job ser reindexado, a partir do results.jsonl,
-- na primeira consulta (mecanismo que já existe desde a 0009).
DELETE FROM timeline_index;

-- Hipóteses produzidas pelas regras internas. NADA aqui é achado: o analista
-- confirma (e o app cria mapeamento ATT&CK, achado e IOCs) ou rejeita.
CREATE TABLE detection_candidates (
    id TEXT PRIMARY KEY,
    evidence_id TEXT NOT NULL,
    job_id TEXT NOT NULL,
    -- Chave estável (regra + versão + agrupamento): rodar de novo não duplica.
    candidate_key TEXT NOT NULL,
    rule_id TEXT NOT NULL,
    rule_version TEXT NOT NULL,
    rule_set_version TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    tactic TEXT NOT NULL,
    technique_id TEXT NOT NULL,
    sub_technique_id TEXT,
    severity TEXT NOT NULL CHECK (severity IN ('LOW','MEDIUM','HIGH','CRITICAL')),
    confidence TEXT NOT NULL CHECK (confidence IN ('LOW','MEDIUM','HIGH')),
    aggregation TEXT NOT NULL,
    event_count INTEGER NOT NULL CHECK (event_count >= 1),
    first_seen TEXT NOT NULL,
    last_seen TEXT NOT NULL,
    -- JSON: ["record:12", ...] (as referências são capadas pelo engine).
    event_refs TEXT NOT NULL,
    refs_truncated INTEGER NOT NULL DEFAULT 0,
    host TEXT,
    user_name TEXT,
    src_ip TEXT,
    -- JSON: [{"ioc_type","value","event_ref"}, ...]
    iocs TEXT NOT NULL,
    limitations TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'PENDING' CHECK (status IN ('PENDING','CONFIRMED','REJECTED')),
    created_at TEXT NOT NULL,
    decided_at TEXT,
    decision_note TEXT,
    -- Preenchidos na confirmação, um a um (a confirmação pode ser retomada).
    incident_id TEXT,
    attack_mapping_id TEXT,
    finding_id TEXT,
    UNIQUE (evidence_id, candidate_key)
);

CREATE INDEX idx_detection_candidates_evidence
    ON detection_candidates(evidence_id, status, first_seen);

-- O que a regra afirmou não muda depois de gravado: só a decisão e os vínculos.
CREATE TRIGGER detection_candidates_facts_are_immutable
BEFORE UPDATE ON detection_candidates
WHEN OLD.evidence_id IS NOT NEW.evidence_id
  OR OLD.job_id IS NOT NEW.job_id
  OR OLD.candidate_key IS NOT NEW.candidate_key
  OR OLD.rule_id IS NOT NEW.rule_id
  OR OLD.rule_version IS NOT NEW.rule_version
  OR OLD.rule_set_version IS NOT NEW.rule_set_version
  OR OLD.title IS NOT NEW.title
  OR OLD.description IS NOT NEW.description
  OR OLD.tactic IS NOT NEW.tactic
  OR OLD.technique_id IS NOT NEW.technique_id
  OR OLD.sub_technique_id IS NOT NEW.sub_technique_id
  OR OLD.severity IS NOT NEW.severity
  OR OLD.confidence IS NOT NEW.confidence
  OR OLD.aggregation IS NOT NEW.aggregation
  OR OLD.event_count IS NOT NEW.event_count
  OR OLD.first_seen IS NOT NEW.first_seen
  OR OLD.last_seen IS NOT NEW.last_seen
  OR OLD.event_refs IS NOT NEW.event_refs
  OR OLD.refs_truncated IS NOT NEW.refs_truncated
  OR OLD.host IS NOT NEW.host
  OR OLD.user_name IS NOT NEW.user_name
  OR OLD.src_ip IS NOT NEW.src_ip
  OR OLD.iocs IS NOT NEW.iocs
  OR OLD.limitations IS NOT NEW.limitations
  OR OLD.created_at IS NOT NEW.created_at
BEGIN
    SELECT RAISE(ABORT, 'detection candidate facts are immutable');
END;

-- Decidido, decidido: confirmado ou rejeitado não volta a pendente nem muda.
CREATE TRIGGER detection_candidates_decision_is_final
BEFORE UPDATE ON detection_candidates
WHEN OLD.status <> 'PENDING'
BEGIN
    SELECT RAISE(ABORT, 'detection candidate already decided');
END;
