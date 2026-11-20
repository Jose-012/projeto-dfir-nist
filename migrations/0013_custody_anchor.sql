-- F1-36 / ADR-0030: cabeça e contagem independentes da cadeia de custódia.
-- A migration não reescreve entradas nem tenta inferir remoções anteriores.
CREATE TABLE custody_state (
    evidence_id TEXT PRIMARY KEY REFERENCES evidence(id),
    entry_count INTEGER NOT NULL CHECK(entry_count >= 0),
    head_hash TEXT
);

INSERT INTO custody_state (evidence_id, entry_count, head_hash)
SELECT
    entries.evidence_id,
    COUNT(*),
    (
        SELECT newest.entry_hash
        FROM chain_of_custody AS newest
        WHERE newest.evidence_id = entries.evidence_id
        ORDER BY newest.sequence DESC
        LIMIT 1
    )
FROM chain_of_custody AS entries
GROUP BY entries.evidence_id;

INSERT OR REPLACE INTO schema_meta(key,value) VALUES ('custody_anchor','1');
