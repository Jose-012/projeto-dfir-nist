-- F1-03: Current Profile / Target Profile persistidos e versionados, base
-- para a Gap Analysis reproduzível (RF-009/RF-010).
--
-- Um "profile" (CURRENT ou TARGET) é uma coleção nomeada/versionada/sourced
-- de avaliações por subcategoria (seção 13 da spec canônica). Igual a
-- analytical_results (F1-01), é append-only: uma nova versão do profile é
-- uma NOVA linha em nist_profiles (com supersedes apontando para a
-- anterior) com seu próprio conjunto de entradas em nist_profile_entries —
-- a versão anterior nunca é alterada ou apagada ("Alterações de profile não
-- apagam versões anteriores").
--
-- O Tier (seção 13.2) é registrado como CONTEXTO do profile inteiro, nunca
-- por subcategoria — a spec é explícita: "Não representar Tier como score
-- oficial por Function, Category ou Subcategory".
--
-- subcategory_id NÃO é validado contra um catálogo oficial embutido do CSF
-- 2.0 (só a forma estrutural FUNCTION.CATEGORY-NN é validada em Rust) — ver
-- docs/adrs/ADR-0011-nist-profile-gap.md para a justificativa.
CREATE TABLE IF NOT EXISTS nist_profiles (
    sequence INTEGER PRIMARY KEY AUTOINCREMENT,
    id TEXT NOT NULL UNIQUE,
    incident_id TEXT NOT NULL REFERENCES incidents(id),
    profile_type TEXT NOT NULL CHECK (profile_type IN ('CURRENT', 'TARGET')),
    version_label TEXT NOT NULL,
    source TEXT NOT NULL,
    tier TEXT CHECK (tier IS NULL OR tier IN (
        'TIER_1_PARTIAL', 'TIER_2_RISK_INFORMED', 'TIER_3_REPEATABLE', 'TIER_4_ADAPTIVE'
    )),
    tier_justification TEXT,
    supersedes TEXT REFERENCES nist_profiles(id),
    created_at TEXT NOT NULL,
    actor_id TEXT
);

CREATE INDEX IF NOT EXISTS idx_nist_profiles_incident
    ON nist_profiles(incident_id, profile_type, sequence);
CREATE INDEX IF NOT EXISTS idx_nist_profiles_supersedes
    ON nist_profiles(supersedes);

CREATE TRIGGER IF NOT EXISTS nist_profiles_no_update
BEFORE UPDATE ON nist_profiles
BEGIN
    SELECT RAISE(ABORT, 'nist_profiles are append-only; insert a new row referencing supersedes instead');
END;

CREATE TRIGGER IF NOT EXISTS nist_profiles_no_delete
BEFORE DELETE ON nist_profiles
BEGIN
    SELECT RAISE(ABORT, 'nist_profiles are append-only; insert a new row referencing supersedes instead');
END;

CREATE TABLE IF NOT EXISTS nist_profile_entries (
    sequence INTEGER PRIMARY KEY AUTOINCREMENT,
    id TEXT NOT NULL UNIQUE,
    profile_id TEXT NOT NULL REFERENCES nist_profiles(id),
    subcategory_id TEXT NOT NULL,
    subcategory_label TEXT NOT NULL,
    implementation_status TEXT NOT NULL CHECK (implementation_status IN (
        'IMPLEMENTED', 'PARTIALLY_IMPLEMENTED', 'NOT_IMPLEMENTED', 'NOT_ASSESSED', 'NOT_APPLICABLE'
    )),
    justification TEXT,
    evidence_reference TEXT,
    confidence TEXT CHECK (confidence IS NULL OR confidence IN ('LOW', 'MEDIUM', 'HIGH', 'CONFIRMED')),
    created_at TEXT NOT NULL,
    UNIQUE (profile_id, subcategory_id)
);

CREATE INDEX IF NOT EXISTS idx_nist_profile_entries_profile
    ON nist_profile_entries(profile_id);
CREATE INDEX IF NOT EXISTS idx_nist_profile_entries_subcategory
    ON nist_profile_entries(subcategory_id);

CREATE TRIGGER IF NOT EXISTS nist_profile_entries_no_update
BEFORE UPDATE ON nist_profile_entries
BEGIN
    SELECT RAISE(ABORT, 'nist_profile_entries are append-only; insert a new profile version instead');
END;

CREATE TRIGGER IF NOT EXISTS nist_profile_entries_no_delete
BEFORE DELETE ON nist_profile_entries
BEGIN
    SELECT RAISE(ABORT, 'nist_profile_entries are append-only; insert a new profile version instead');
END;
