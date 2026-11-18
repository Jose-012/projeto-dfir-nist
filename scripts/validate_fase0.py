#!/usr/bin/env python3
"""Static conformance checks for the local forensic application baseline."""
from __future__ import annotations

import json
import re
import shutil
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
errors: list[str] = []
warnings: list[str] = []

required = [
    "toolchains.toml",
    "AGENTS.md",
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain.toml",
    "migrations/0001_fase0.sql",
    "migrations/0002_access_visibility.sql",
    "migrations/0003_local_forensic_scope.sql",
    "src/main.rs",
    "src/application/engine.rs",
    "src/application/process.rs",
    "src/persistence/db.rs",
    "src/persistence/audit.rs",
    "src/infrastructure/storage.rs",
    "src/service/mod.rs",
    "src/gui/mod.rs",
    "src/gui/backend.rs",
    "engine/src/main.rs",
    "protocol/src/lib.rs",
    "scripts/setup.sh",
    "scripts/setup.ps1",
    "docs/rtm.md",
    "docs/FASE0_STATUS.md",
    "docs/RELEASE_GATE.md",
    "docs/adrs/ADR-0001-fase0.md",
    "docs/adrs/ADR-0018-realinhamento-demanda-original.md",
    "docs/DEMAND_TRACE.md",
    "docs/FASE1_BACKLOG.md",
    "docs/source/DEMANDA_ORIGINAL_SENAI_12412.md",
    "README.md",
    "tests/security/fase0_config.py",
]
for item in required:
    if not (ROOT / item).is_file():
        errors.append(f"missing required file: {item}")

cargo = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
toolchain = (ROOT / "rust-toolchain.toml").read_text(encoding="utf-8")
_toolchains_manifest = (ROOT / "toolchains.toml").read_text(encoding="utf-8")
_file_pin = re.search(r'channel = "(\d+\.\d+\.\d+)"', toolchain)
_manifest_pin = re.search(r'\[rust\]\s*\nchannel = "(\d+\.\d+\.\d+)"', _toolchains_manifest)
if not _file_pin or not _manifest_pin:
    errors.append("Rust toolchain is not pinned to an exact version (X.Y.Z) in rust-toolchain.toml and toolchains.toml")
elif _file_pin.group(1) != _manifest_pin.group(1):
    errors.append(f"Rust pin differs: rust-toolchain.toml {_file_pin.group(1)} vs toolchains.toml {_manifest_pin.group(1)}")
if 'members = [".", "engine", "protocol"]' not in cargo:
    errors.append("workspace must contain dfir-app, dfir-engine and dfir-protocol")
for forbidden_dep in ["argon2 =", "axum-extra =", "getrandom =", "axum =", "askama =", "tower-http ="]:
    if forbidden_dep in cargo:
        errors.append(f"obsolete authentication dependency remains: {forbidden_dep}")
for required_dep in ["eframe", "egui_extras", "rfd", "blake3", "chrono", "dfir-protocol", "serde_json_canonicalizer", "serde_json", "sqlx", "tokio", "tokio-util", "futures-util", "tracing", "thiserror", "uuid"]:
    if required_dep not in cargo:
        errors.append(f"missing adopted direct dependency: {required_dep}")

engine_manifest = (ROOT / "engine/Cargo.toml").read_text(encoding="utf-8")
evtx_dependency = re.search(
    r'evtx = \{ version = "([0-9]+(?:\.[0-9]+){2})", default-features = false \}',
    engine_manifest,
)
if not evtx_dependency or tuple(int(part) for part in evtx_dependency.group(1).split(".")) < (
    0,
    15,
    1,
):
    errors.append(
        "engine must own direct evtx >= 0.15.1 dependency (RUSTSEC-2026-0431; default-features = false)"
    )
if 'serde = "1.0.229"' not in engine_manifest:
    errors.append("engine must declare direct serde dependency")

# Removed authentication/server concerns must not leak into active source.
active_paths = [ROOT / "src", ROOT / "migrations"]
active_text = "\n".join(
    path.read_text(encoding="utf-8")
    for root in active_paths
    for path in root.rglob("*")
    if path.is_file() and path.suffix in {".rs", ".sql", ".html", ".yml", ".yaml"}
)
for forbidden in [
    "SqliteUserRepository",
    "SqliteSessionRepository",
    "SqliteIncidentAccessRepository",
    "role_allows",
    "Permission::",
    "DFIR_BOOTSTRAP_PASSWORD",
    "DFIR_HTTPS",
    "DFIR_ALLOW_INSECURE_HTTP",
    'dfir_session',
    "argon2",
]:
    if forbidden in active_text:
        errors.append(f"obsolete application concern remains in active tree: {forbidden}")

migration = (ROOT / "migrations/0001_fase0.sql").read_text(encoding="utf-8")
cleanup_migration = (ROOT / "migrations/0003_local_forensic_scope.sql").read_text(encoding="utf-8")
for required_sql in [
    "PRAGMA foreign_keys = ON",
    "CREATE TABLE IF NOT EXISTS incidents",
    "CREATE TABLE IF NOT EXISTS audit_events",
    "CREATE TABLE IF NOT EXISTS audit_state",
    "CREATE TRIGGER IF NOT EXISTS audit_events_no_update",
    "CREATE TRIGGER IF NOT EXISTS audit_events_no_delete",
]:
    if required_sql not in migration:
        errors.append(f"baseline migration missing required construct: {required_sql}")
for required_sql in [
    "CREATE TABLE incidents_local",
    "CREATE TABLE audit_events_local",
    "DROP TABLE incident_access",
    "DROP TABLE sessions",
    "DROP TABLE users",
    "ALTER TABLE incidents_local RENAME TO incidents",
    "ALTER TABLE audit_events_local RENAME TO audit_events",
    "audit_events_no_update",
    "audit_events_no_delete",
]:
    if required_sql not in cleanup_migration:
        errors.append(f"local-scope migration missing required construct: {required_sql}")

persistence = (ROOT / "src/persistence/db.rs").read_text(encoding="utf-8")
for marker in ["journal_mode(SqliteJournalMode::Wal)", "synchronous(SqliteSynchronous::Full)", "foreign_keys(true)", "busy_timeout"]:
    if marker not in persistence:
        errors.append(f"SQLite policy missing: {marker}")

audit = (ROOT / "src/persistence/audit.rs").read_text(encoding="utf-8")
engine_src = (ROOT / "engine/src/main.rs").read_text(encoding="utf-8")
if "run_process_smoke_probe" not in engine_src:
    errors.append("engine smoke entrypoint is missing run_process_smoke_probe")

for marker in ["BEGIN IMMEDIATE", "event_count", "head_hash", "BLAKE3-256-v2-JCS", "BLAKE3-256-v1", "serde_json_canonicalizer"]:
    if marker not in audit:
        errors.append(f"audit integrity policy missing: {marker}")

process = (ROOT / "src/application/process.rs").read_text(encoding="utf-8")
for marker in ["launch_process", "ProcessHandle", "ProcessInput", "current_dir", "DFIR_ENGINE_INPUT_ROOT", "DFIR_ENGINE_OUTPUT_ROOT"]:
    if marker not in process:
        errors.append(f"engine process runner missing: {marker}")

engine = (ROOT / "src/application/engine.rs").read_text(encoding="utf-8")
for marker in ["max_output_bytes", "max_line_bytes", "FramedRead", "LinesCodec", "new_with_max_length"]:
    if marker not in engine:
        errors.append(f"engine protocol requirement missing: {marker}")
for marker in ["--stdio", "trusted_data_root", "canonicalize"]:
    if marker not in process:
        errors.append(f"engine process runner requirement missing: {marker}")

if "child.kill().await" not in engine and "child.kill().await" not in process:
    errors.append("engine timeout/error path must terminate the engine process")
if "child.wait()" not in engine and "child.wait()" not in process:
    errors.append("engine path must reap/observe subprocess exit")

config = (ROOT / "src/config.rs").read_text(encoding="utf-8")
for marker in [
    "DFIR_ENGINE_MAX_LINE_BYTES", "DFIR_ENGINE_TIMEOUT_SECONDS", "DFIR_DATA_ROOT",
    "DFIR_ENGINE_PATH", "DFIR_ENGINE_MAX_CONCURRENT_JOBS",
]:
    if marker not in config:
        errors.append(f"typed configuration missing: {marker}")

service = (ROOT / "src/service/mod.rs").read_text(encoding="utf-8")
for marker in ["pub async fn import_evidence", "pub async fn analyze", "BEGIN IMMEDIATE"]:
    if marker not in service:
        errors.append(f"service layer missing required construct: {marker}")

gui_mod = (ROOT / "src/gui/mod.rs").read_text(encoding="utf-8")
gui_backend = (ROOT / "src/gui/backend.rs").read_text(encoding="utf-8")
if "eframe::run_native" not in gui_mod:
    errors.append("native UI entrypoint must call eframe::run_native")
if "tokio::runtime::Builder" not in gui_backend:
    errors.append("GUI backend must run its own Tokio runtime off the UI thread")
for forbidden_route in ['"/login"', '"/logout"', '"/admin/']:
    if forbidden_route in gui_mod or forbidden_route in service:
        errors.append(f"obsolete authenticated route remains: {forbidden_route}")

engine_main = (ROOT / "engine/src/main.rs").read_text(encoding="utf-8")
if "stdout" not in engine_main:
    errors.append("engine must emit protocol messages to stdout")
protocol_text = (ROOT / "protocol/src/lib.rs").read_text(encoding="utf-8")
for required_type in ["pub struct Request", "pub struct Progress", "pub struct ResultMessage", "pub struct ErrorMessage"]:
    if required_type not in protocol_text:
        errors.append(f"shared protocol missing concrete type: {required_type}")

# No rejected external DFIR tools in active runtime/build artifacts.
for candidate in [ROOT / "Cargo.toml", ROOT / "README.md"]:
    text = candidate.read_text(encoding="utf-8").lower()
    for forbidden in ["hayabusa", "chainsaw"]:
        if forbidden in text:
            errors.append(f"rejected external DFIR tool referenced by active artifact: {forbidden} in {candidate.relative_to(ROOT)}")

# Dependency metadata should not gain obsolete direct components.
try:
    import tomllib
    manifest = tomllib.loads(cargo)
    lock = tomllib.loads((ROOT / "Cargo.lock").read_text(encoding="utf-8"))
except Exception as exc:
    errors.append(f"dependency metadata could not be parsed: {exc}")
else:
    direct = set(manifest.get("dependencies", {}))
    lock_names = {pkg.get("name") for pkg in lock.get("package", [])}
    for obsolete in {"argon2", "axum-extra"} & direct:
        errors.append(f"obsolete direct dependency remains: {obsolete}")
    for name in direct:
        if name not in lock_names:
            warnings.append(f"Cargo.lock has not yet been regenerated for direct dependency {name}")

spec_candidates = sorted((ROOT / "docs").glob("PROJECT_SPEC_v*.md"))  # EVI-04: glob unificado com o doc_gate (v*, não v6_*)
root_spec_candidates = sorted(ROOT.glob("nist-senai-proj_integrador_v*.md"))
if not spec_candidates or not root_spec_candidates:
    errors.append("canonical PROJECT_SPEC_v* specification copies are missing")
else:
    docs_spec = spec_candidates[-1]
    root_spec = root_spec_candidates[-1]
    if docs_spec.stem.removeprefix("PROJECT_SPEC_v") != root_spec.stem.removeprefix("nist-senai-proj_integrador_v"):
        errors.append(f"specification copies are for different versions: {docs_spec.name} vs {root_spec.name}")
    if docs_spec.read_bytes() != root_spec.read_bytes():
        errors.append(f"canonical specification copies differ: {docs_spec.name} vs {root_spec.name}")
    spec_text = docs_spec.read_text(encoding="utf-8")
    if "plataforma local de dfir" not in spec_text.lower():
        errors.append("canonical specification must state the local-forensic scope")


# ---------------------------------------------------------------------------
# Coerência manifesto x lock (AUD-01): todo requisito direto de versão precisa
# ser satisfeito por alguma versão presente no Cargo.lock. Sem isso, todo alvo
# `--locked` falha e o gate estático antes dizia PASS.
# ---------------------------------------------------------------------------
def _parse_version(text: str):
    core = text.split("+", 1)[0].split("-", 1)[0]
    parts = core.split(".")
    nums = []
    for part in parts:
        if not part.isdigit():
            return None
        nums.append(int(part))
    return nums


def _caret_satisfied(requirement: str, version: str) -> bool | None:
    """Semântica caret do Cargo. Devolve None quando a forma não é suportada."""
    requirement = requirement.strip()
    if requirement.startswith("^"):
        requirement = requirement[1:]
    if not requirement or requirement[0] in "<>=~*," or "," in requirement or " " in requirement:
        return None
    req = _parse_version(requirement)
    ver = _parse_version(version)
    if req is None or ver is None or len(ver) < 3:
        return None
    lower = req + [0] * (3 - len(req))
    if ver[:3] < lower:
        return False
    if req[0] > 0 or len(req) == 1:
        upper = [req[0] + 1, 0, 0]
    elif len(req) == 2 or req[1] > 0:
        upper = [0, req[1] + 1, 0]
    else:
        upper = [0, 0, req[2] + 1]
    return ver[:3] < upper


def _direct_requirements(manifest_path: Path):
    try:
        data = tomllib.loads(manifest_path.read_text(encoding="utf-8"))
    except Exception as exc:  # noqa: BLE001
        errors.append(f"could not parse {manifest_path.relative_to(ROOT)}: {exc}")
        return
    tables = [data.get("dependencies", {}), data.get("dev-dependencies", {}), data.get("build-dependencies", {})]
    for target in data.get("target", {}).values():
        tables.extend([target.get("dependencies", {}), target.get("dev-dependencies", {})])
    for table in tables:
        for name, spec in table.items():
            requirement = spec if isinstance(spec, str) else spec.get("version") if isinstance(spec, dict) else None
            if isinstance(spec, dict) and "path" in spec:
                continue
            if requirement:
                yield name, requirement


try:
    lock_versions: dict[str, list[str]] = {}
    for package in lock.get("package", []):
        lock_versions.setdefault(package["name"], []).append(package["version"])
    for manifest_file in ["Cargo.toml", "engine/Cargo.toml", "protocol/Cargo.toml"]:
        for name, requirement in _direct_requirements(ROOT / manifest_file) or []:
            versions = lock_versions.get(name)
            if not versions:
                continue  # ausência já é avisada acima
            verdicts = [_caret_satisfied(requirement, version) for version in versions]
            if None in verdicts:
                warnings.append(f"{manifest_file}: could not check requirement {name} = {requirement!r}")
            elif not any(verdicts):
                errors.append(
                    f"Cargo.lock out of sync: {manifest_file} requires {name} {requirement} but the lock has {', '.join(versions)}"
                )
except NameError:
    pass  # manifest/lock não foram lidos acima; o erro já foi registrado

# ---------------------------------------------------------------------------
# Rastreabilidade da demanda (ADR-0018): todo ticket citado precisa existir e
# todo RF citado em DEMAND_TRACE precisa existir na seção 39 da spec.
# ---------------------------------------------------------------------------
ticket_dir = ROOT / ".scratch/fase1-backlog/issues"
ticket_numbers = {path.name.split("-", 1)[0] for path in ticket_dir.glob("*.md")} if ticket_dir.is_dir() else set()
spec_for_trace = (spec_candidates[-1].read_text(encoding="utf-8") if spec_candidates else "")
for doc in ["docs/DEMAND_TRACE.md", "docs/FASE1_BACKLOG.md", "docs/adrs/ADR-0018-realinhamento-demanda-original.md", "docs/rtm.md"]:
    doc_path = ROOT / doc
    if not doc_path.is_file():
        continue
    doc_text = doc_path.read_text(encoding="utf-8")
    for number in sorted(set(re.findall(r"F1-(\d{2})", doc_text))):
        if number not in ticket_numbers:
            errors.append(f"{doc} cites ticket F1-{number} but .scratch/fase1-backlog/issues/{number}-*.md does not exist")
trace_path = ROOT / "docs/DEMAND_TRACE.md"
if trace_path.is_file():
    trace_text = trace_path.read_text(encoding="utf-8")
    for rf in sorted(set(re.findall(r"RF-(\d{3})", trace_text))):
        if f"| RF-{rf} |" not in spec_for_trace:
            errors.append(f"docs/DEMAND_TRACE.md cites RF-{rf} which is not a row in the specification section 39")
    for marker in [f"RB-00{n}" for n in range(1, 7)] + [f"DEM-ENT-0{n}" for n in range(1, 7)] + [f"DEM-PLAN-0{n}" for n in range(1, 8)]:
        if marker not in trace_text:
            errors.append(f"docs/DEMAND_TRACE.md is missing the demand item {marker}")
agents_text = (ROOT / "AGENTS.md").read_text(encoding="utf-8")
if spec_candidates and f"docs/{spec_candidates[-1].name}" not in agents_text:
    errors.append(f"AGENTS.md does not reference the canonical specification docs/{spec_candidates[-1].name}")

# ---------------------------------------------------------------------------
# Documentação em dia com as decisões e com o código (ADR-0022): (1) toda ADR
# a partir da 0018 precisa ser citada na especificação canônica; (2) a
# documentação viva não pode citar identificadores que o código removeu.
# ---------------------------------------------------------------------------
# ADRs de processo (harness de desenvolvimento), sem efeito sobre o produto: não exigem citação na
# especificação canônica. ADR-0027 (decisão do operador de 2026-11-04); a spec não foi alterada nessa etapa.
# Remover a isenção quando a especificação passar a citar a ADR-0027.
SPEC_CITATION_EXEMPT_ADRS = {"0027"}

if spec_candidates:
    canonical_spec_text = spec_candidates[-1].read_text(encoding="utf-8")
    for adr_file in sorted((ROOT / "docs/adrs").glob("ADR-*.md")):
        match = re.match(r"ADR-(\d{4})-", adr_file.name)
        if match and match.group(1) in SPEC_CITATION_EXEMPT_ADRS:
            continue
        if match and int(match.group(1)) >= 18 and f"ADR-{match.group(1)}" not in canonical_spec_text:
            errors.append(f"{adr_file.name} is not cited in the canonical specification {spec_candidates[-1].name}")

STALE_IDENTIFIERS = [
    "job_results",
    "ResultsResponse",
    "JobResultsLoaded",
    "MAX_TIMELINE_EVENTS",
    "MAX_RESULT_LINES",
    "MAX_LISTED_RESULTS",
    "MAX_PERSISTED_YARA_MATCHES",
]
HISTORICAL_DOCS = {
    "docs/FASE1_STATUS.md",
    "docs/FASE0_STATUS.md",
    "docs/rtm.md",
    "CHANGELOG.md",
}
live_docs = [ROOT / "README.md", ROOT / "AGENTS.md", ROOT / "PROMPT_NOVA_CONVERSA.md"]
live_docs += [path for path in (ROOT / "docs").glob("*.md") if not path.name.startswith("PROJECT_SPEC")]
for doc_path in live_docs:
    relative = doc_path.relative_to(ROOT).as_posix()
    if relative in HISTORICAL_DOCS or not doc_path.is_file():
        continue
    doc_text = doc_path.read_text(encoding="utf-8")
    for identifier in STALE_IDENTIFIERS:
        if identifier in doc_text:
            errors.append(f"{relative} cites the removed identifier {identifier}")

# ---------------------------------------------------------------------------
# Coerência entre documentos (gap EVI-04, 2026-11-04). Cada verificação cita o
# gap que ela impede de voltar. Lacunas conhecidas e ainda abertas saem como
# WARNING (nunca em silêncio); o resto é ERROR.
# ---------------------------------------------------------------------------


def _read(rel: str) -> str:
    path = ROOT / rel
    return path.read_text(encoding="utf-8") if path.is_file() else ""


def _code_blocks(text: str):
    """Linhas dentro de blocos de código cercados por ``` (número 1-based, texto sem recuo)."""
    inside = False
    for number, line in enumerate(text.split("\n"), 1):
        if line.startswith("```"):
            inside = not inside
            continue
        if inside:
            yield number, line.strip()


def _strip_struck(text: str) -> str:
    return re.sub(r"~~.*?~~", "", text, flags=re.S)


def _status_token(text: str) -> str:
    text = text.replace("`", "").replace("*", "").strip()
    match = re.match(r"(IMPLEMENTED\s*/\s*PENDING-HUMAN|IMPLEMENTED|PARCIAL|PLANNED|DEFERRED|ADIADO|PENDING-HUMAN|INTEGRATED|VERIFIED|ACCEPTED)", text)
    return re.sub(r"\s+", "", match.group(1)) if match else text[:20]


def _rtm_ticket_rows() -> dict[str, tuple[str, str]]:
    rows: dict[str, tuple[str, str]] = {}
    for line in _read("docs/rtm.md").split("\n"):
        match = re.match(r"^\|\s*(F1-\d{2})\s*\|", line)
        if match:
            cells = [cell.strip() for cell in line.strip().strip("|").split("|")]
            rows[match.group(1)] = (cells[1], cells[-1])
    return rows


rtm_rows = _rtm_ticket_rows()
agents_text_all = _read("AGENTS.md")
spec_all = spec_candidates[-1].read_text(encoding="utf-8") if spec_candidates else ""
adr_numbers = sorted({m.group(1) for p in (ROOT / "docs/adrs").glob("ADR-*.md") if (m := re.match(r"ADR-(\d{4})-", p.name))})
ticket_files = sorted(ticket_dir.glob("*.md")) if ticket_dir.is_dir() else []

# 1) Documentos exigidos por AGENTS.md §11 (ampliado em AUS-01) e o registro de evidências (AUS-03).
sec11 = re.search(r"^## 11\..*?(?=^## 12\.)", agents_text_all, re.S | re.M)
if not sec11:
    errors.append("AGENTS.md section 11 (required documentation) was not found")
else:
    for token in re.findall(r"`([^`]+)`", sec11.group(0)):
        if "/" not in token and "*" not in token and not token.endswith(".md"):
            continue
        if token.endswith("/"):
            present = (ROOT / token).is_dir()
        elif "*" in token:
            present = any(ROOT.glob(token))
        else:
            present = (ROOT / token).exists()
        if not present:
            errors.append(f"AGENTS.md section 11 requires {token}, which does not exist (AUS-01)")
for required in ("docs/EVIDENCE_REGISTER.md", "docs/archive/FASE1_STATUS_historico_2026-10.md"):
    if not (ROOT / required).is_file():
        errors.append(f"required document is missing: {required} (AUS-03 / INC-13)")

# 2) IDs citados precisam resolver: ADR-NNNN, RF-NNN, AUD-NN.
# Só a tabela principal da seção 39 (antes de 39.1): as tabelas de mudanças repetem IDs e não contam como definição.
_sec39_start = spec_all.find("# 39. ")
_sec39_end = spec_all.find("### 39.1", _sec39_start)
rf_ids = set(re.findall(r"^\|\s*(RF-\d{3})\s*\|", spec_all[_sec39_start:_sec39_end], re.M)) if _sec39_start != -1 and _sec39_end != -1 else set()
_rf_claim = re.search(r"RF-001 a RF-(\d{3})", spec_all)  # INC-16: o índice declara a faixa; todos precisam existir na tabela
if _rf_claim:
    for _n in range(1, int(_rf_claim.group(1)) + 1):
        if f"RF-{_n:03d}" not in rf_ids:
            errors.append(f"specification requirement table is missing RF-{_n:03d}, although the XML index declares RF-001 to RF-{_rf_claim.group(1)} (INC-16)")
aud_ids = set(re.findall(r"\bAUD-\d{2}[a-z]?\b", _read("docs/archive/REVIEW_2026-10-02-auditoria-demanda.md") + _read("docs/DOCUMENT_AUTHORITY.md")))
KNOWN_OPEN_RF: set[str] = set()  # INC-16 fechado na spec 6.5.1 (RF-024 e RF-031 de volta à tabela); sem exceções abertas
id_sources = ["docs/DEMAND_TRACE.md", "docs/FASE1_BACKLOG.md", "docs/rtm.md"] + [p.relative_to(ROOT).as_posix() for p in ticket_files]
for rel in id_sources:
    text = _read(rel)
    for number in sorted(set(re.findall(r"\bADR-(\d{4})\b", text))):
        if number not in adr_numbers:
            errors.append(f"{rel} cites ADR-{number}, which does not exist")
    for rf in sorted(set(re.findall(r"\bRF-\d{3}\b", text))):
        if rf not in rf_ids:
            (warnings if rf in KNOWN_OPEN_RF else errors).append(f"{rel} cites {rf}, which is not in the specification requirement table" + (" (INC-16 open)" if rf in KNOWN_OPEN_RF else ""))
    for aud in sorted(set(re.findall(r"\bAUD-\d{2}[a-z]?\b", text))):
        if aud not in aud_ids:
            errors.append(f"{rel} cites {aud}, which is neither in the archived review nor registered in DOCUMENT_AUTHORITY")

# 3) Comandos de gate: receitas `just` existentes e `--workspace` onde aplicável (INC-02, INC-05).
just_text = _read("justfile")
just_recipes = set(re.findall(r"^([a-z][\w-]*)\b[^\n:=]*:(?!=)", just_text, re.M))
WORKSPACE_COMMAND = re.compile(r"^(?:[@-]\s*)?cargo (check|clippy|nextest run|deny)\b")
command_files = ["docs/RELEASE_GATE.md", "AGENTS.md", "PROMPT_NOVA_CONVERSA.md", "CONVENTIONS.md"]
for rel in command_files:
    text = _read(rel)
    for number, line in _code_blocks(text):
        match = re.match(r"just\s+([a-z][\w-]*)", line)
        if match and match.group(1) not in just_recipes:
            errors.append(f"{rel}:{number} runs `just {match.group(1)}`, which is not a recipe in the justfile")
        script = re.match(r"python3?\s+(scripts/\S+\.py)", line)
        if script and not (ROOT / script.group(1)).is_file():
            errors.append(f"{rel}:{number} runs {script.group(1)}, which does not exist")
        if WORKSPACE_COMMAND.match(line) and "--workspace" not in line:
            errors.append(f"{rel}:{number} runs a cargo gate without --workspace: only dfir-app would be covered (INC-02)")
        if re.match(r"cargo[ -]vet\b", line):
            errors.append(f"{rel}:{number} requires cargo vet, which is dispensed in the MVP (INC-04)")
    for text_line in text.split("\n"):
        if re.search(r"(?i)fase 4|fora do mvp|n[ãa]o existe|ainda n[ãa]o", text_line):
            continue  # menção explícita a receita que não existe (adiada ou removida)
        for match in re.finditer(r"`just\s+([a-z][\w-]*)", text_line):
            if match.group(1) not in just_recipes:
                errors.append(f"{rel} mentions `just {match.group(1)}`, which is not a recipe in the justfile")
for number, line in enumerate(just_text.split("\n"), 1):
    if WORKSPACE_COMMAND.match(line.strip()) and "--workspace" not in line:
        errors.append(f"justfile:{number} runs a cargo gate without --workspace (INC-02)")

# 4) README: subcomandos da CLI iguais aos de src/main.rs, nos dois sentidos (INC-08).
readme_text = _read("README.md")
command_enum = re.search(r"enum Command \{(.*?)\n\}\n", _read("src/main.rs"), re.S)
if command_enum:
    code_commands = {re.sub(r"(?<!^)(?=[A-Z])", "-", name).lower() for name in re.findall(r"^    ([A-Z][A-Za-z]+)\b", command_enum.group(1), re.M)}
    cli_section = re.search(r"^## Linha de comando \(headless\)\n(.*?)(?=^## )", readme_text, re.S | re.M)
    readme_commands = set(re.findall(r"^\| `([a-z][a-z-]*)` \|", cli_section.group(1), re.M)) if cli_section else set()
    readme_commands.discard("subcomando")
    for name in sorted(code_commands - readme_commands):
        errors.append(f"README.md does not document the CLI subcommand `{name}` that exists in src/main.rs (INC-08)")
    for name in sorted(readme_commands - code_commands):
        errors.append(f"README.md documents the CLI subcommand `{name}`, which does not exist in src/main.rs (INC-08)")
else:
    errors.append("could not find `enum Command` in src/main.rs")

# 5) README: toda DFIR_* existe em src/config.rs (as internas, em outro arquivo do código).
config_text = _read("src/config.rs")
code_text = "\n".join(p.read_text(encoding="utf-8", errors="replace") for root in ("src", "engine/src") for p in (ROOT / root).rglob("*.rs"))
for variable in sorted(set(re.findall(r"\bDFIR_[A-Z0-9_]+\b", readme_text))):
    if variable in config_text:
        continue
    if variable in code_text:
        continue  # variável interna passada ao engine; existe no código, não em config.rs
    errors.append(f"README.md documents {variable}, which does not exist in the code")

# 6) Estado de ticket: a última linha **Status…** do ticket coincide com a RTM (INC-03).
for ticket in ticket_files:
    ticket_id = f"F1-{ticket.name.split('-', 1)[0]}"
    status_lines = [m.group(1) for line in ticket.read_text(encoding="utf-8").split("\n") if (m := re.match(r"^\*\*Status\b.*?:\*\*\s*(\S.*)$", line))]
    if ticket_id not in rtm_rows:
        errors.append(f"{ticket_id} has no row in docs/rtm.md")
    elif not status_lines:
        errors.append(f"{ticket.name} has no **Status** line")
    elif _status_token(status_lines[-1]) != _status_token(rtm_rows[ticket_id][1]):
        errors.append(f"{ticket.name} status {_status_token(status_lines[-1])} differs from docs/rtm.md {_status_token(rtm_rows[ticket_id][1])} (INC-03)")

# 7) Índice XML da spec cobre os títulos `# N.` do corpo (AUS-02).
if spec_all:
    in_fence = False
    spec_headings = set()
    for line in spec_all.split("\n"):
        if line.startswith("```"):
            in_fence = not in_fence
        match = None if in_fence else re.match(r"^# (\d+)\. ", line)
        if match:
            spec_headings.add(match.group(1))
    index_ids = set(re.findall(r'<secao id="(\d+)"', spec_all))
    for number in sorted(spec_headings - index_ids, key=int):
        errors.append(f"specification section {number} has no <secao> entry in the XML index")
    for number in sorted(index_ids - spec_headings, key=int):
        errors.append(f"XML index lists section {number}, which has no `# {number}.` heading in the body")

# 8) Identificadores removidos também em CONVENTIONS, ADRs, tickets e testes; menção histórica só com marca de remoção (EVI-04).
HISTORY_MARKER = re.compile(r"(?i)removid|removeu|removida|elimin|superseded|substitu|antes|mostrava|cortavam|gravava|lia no máximo|~~")
extended_scopes = [ROOT / "CONVENTIONS.md"] + sorted((ROOT / "docs/adrs").glob("*.md")) + ticket_files + sorted((ROOT / "tests").glob("*/*.md"))
for scoped in extended_scopes:
    if not scoped.is_file():
        continue
    lines = scoped.read_text(encoding="utf-8").split("\n")
    for number, line in enumerate(lines):
        for identifier in STALE_IDENTIFIERS:
            if identifier in line:
                neighbourhood = " ".join(lines[max(0, number - 1) : number + 2])
                if not HISTORY_MARKER.search(neighbourhood):
                    errors.append(f"{scoped.relative_to(ROOT).as_posix()}:{number + 1} cites the removed identifier {identifier} without marking it as removed")

# 9) Regressões específicas dos gaps já corrigidos.
release_gate = _read("docs/RELEASE_GATE.md")
if re.search(r"cópia automática do banco antes de migration continua pendente", _strip_struck(release_gate)):  # INC-01
    errors.append("docs/RELEASE_GATE.md again presents the automatic pre-migration copy as pending (INC-01)")
conventions_text = _read("CONVENTIONS.md")
for phrase in ("responsável por HTTP local", "caminhos físicos arbitrários do navegador", "Node.js/npm (esbuild", "just verify-supply-chain"):  # INC-05
    if phrase in conventions_text:
        errors.append(f"CONVENTIONS.md again contains the stale text: {phrase} (INC-05)")
for phrase in ("limites de linha, saída, itens", "timeline derivada dos resultados"):  # INC-09
    if phrase in readme_text:
        errors.append(f"README.md again contains the stale text: {phrase} (INC-09)")
migrations = sorted((ROOT / "migrations").glob("*.sql"))
if migrations and migrations[-1].name not in _read("docs/FASE0_STATUS.md"):  # INC-07
    errors.append(f"docs/FASE0_STATUS.md does not cite the latest migration {migrations[-1].name} (INC-07)")
backlog_lines = _read("docs/FASE1_BACKLOG.md").split("\n")
for number, line in enumerate(backlog_lines):  # INC-11
    if "seguem `PLANNED`" in line and "INC-11" not in " ".join(backlog_lines[number : number + 5]):
        errors.append(f"docs/FASE1_BACKLOG.md:{number + 1} keeps a superseded PLANNED statement without the INC-11 note")
fase1_status = _read("docs/FASE1_STATUS.md")  # INC-13
mirror = {m.group(1): m.group(2) for line in fase1_status.split("\n") if (m := re.match(r"^\|\s*(F1-\d{2})\s*\|.*\|\s*([^|]*?)\s*\|\s*$", line))}
for ticket_id, (_, state) in rtm_rows.items():
    if ticket_id not in mirror:
        errors.append(f"docs/FASE1_STATUS.md matrix has no row for {ticket_id} (INC-13)")
    elif _status_token(mirror[ticket_id]) != _status_token(state):
        errors.append(f"docs/FASE1_STATUS.md matrix state for {ticket_id} differs from docs/rtm.md (INC-13)")
for line_number, line in enumerate(agents_text_all.split("\n"), 1):  # INC-14
    if line.startswith("## ") and not re.match(r"^## \d+\. ", line):
        errors.append(f"AGENTS.md:{line_number} has an unnumbered section heading (INC-14)")
if adr_numbers and f"ADR-{adr_numbers[-1]}" not in agents_text_all:
    errors.append(f"AGENTS.md does not cite the latest ADR-{adr_numbers[-1]} (INC-14)")
rtm_text = _read("docs/rtm.md")  # INC-15
for rf in sorted(set(re.findall(r"^\|\s*(RF-\d{3})\s*\|\s*MVP\s*\|", spec_all, re.M))):
    if rf not in rtm_text:
        errors.append(f"docs/rtm.md has no trace for the MVP requirement {rf} (INC-15)")
if re.search(r"(?i)aider", "\n".join(_read(rel) for rel in ("CONVENTIONS.md", "AGENTS.md", "README.md", "PROMPT_NOVA_CONVERSA.md", "CONTRIBUTING.md"))):  # ADR-0027
    errors.append("a live document mentions the removed Aider harness (ADR-0027)")
for path in ROOT.rglob("*.md"):
    relative = path.relative_to(ROOT).as_posix()
    if relative.startswith(("target/", ".git/")) or relative in ("docs/TOOLCHAINS.md",) or relative.startswith("docs/adrs/ADR-0027"):
        continue
    if "Jcode" in path.read_text(encoding="utf-8", errors="replace"):
        errors.append(f"{relative} names the agent harness; only ADR-0027 and docs/TOOLCHAINS.md may (ADR-0027)")
license_text = _read("LICENSE")  # AUS-04
if not license_text.lstrip().startswith("Apache License") or len(license_text) < 10000:
    errors.append("LICENSE does not contain the full Apache License 2.0 text (AUS-04)")
lock_names = {package["name"] for package in tomllib.loads(_read("Cargo.lock")).get("package", [])} if (ROOT / "Cargo.lock").is_file() else set()
third_party = _read("LICENSES_THIRD_PARTY.md")
direct_section = re.search(r"^## Dependências diretas.*?(?=^## )", third_party, re.S | re.M)
for crate in re.findall(r"^- ([A-Za-z0-9_-]+) \d", direct_section.group(0) if direct_section else "", re.M):
    if crate not in lock_names and crate != "cargo-audit":
        errors.append(f"LICENSES_THIRD_PARTY.md lists {crate}, which is not in Cargo.lock (AUS-04)")
register_text = _read("docs/EVIDENCE_REGISTER.md")  # EVI-01
known_evidence = set(re.findall(r"\|\s*`(EVD-(?!XXXX)[\w-]+)`", register_text)) | set(re.findall(r"\|\s*`(WVR-[\w-]+)`", register_text))
for number, line in enumerate(_read("docs/DEFINITION_OF_READY.md").split("\n"), 1):
    if "[x]" in line.lower():
        cited = set(re.findall(r"\b(?:EVD|WVR)-[\w-]+", line))
        if not cited or not cited <= known_evidence:
            errors.append(f"docs/DEFINITION_OF_READY.md:{number} is checked without an EVD-*/WVR-* present in docs/EVIDENCE_REGISTER.md (EVI-01)")
api_text = _read("docs/API.md")  # AUS-05
for service_file in sorted((ROOT / "src/service").rglob("*.rs")) + [ROOT / "src/persistence/backup.rs"]:
    source = service_file.read_text(encoding="utf-8")
    test_module = re.search(r"#\[cfg\(test\)\]\s*mod \w+\s*\{", source)
    source = source[: test_module.start()] if test_module else source
    for match in re.finditer(r"^pub (?:async )?fn (\w+)", source, re.M):
        if service_file.name == "backup.rs" and match.group(1) not in ("migrate_with_backup", "verify_published"):
            continue
        if not re.search(rf"\b{re.escape(match.group(1))}\b", api_text):
            errors.append(f"docs/API.md does not document {service_file.relative_to(ROOT).as_posix()}::{match.group(1)} (AUS-05)")

# 10) Lacunas conhecidas e ainda abertas: aparecem como WARNING até serem corrigidas.
gui_modules = sorted(p.name for p in (ROOT / "src/gui").glob("*.rs") if p.name != "mod.rs")
native_ui = _read("docs/NATIVE_UI_ARCHITECTURE.md")
missing_gui = [name for name in gui_modules if name not in native_ui]
if missing_gui:
    warnings.append(f"INC-10 open: docs/NATIVE_UI_ARCHITECTURE.md does not mention src/gui modules {', '.join(missing_gui)}")
prompt_text = _read("PROMPT_NOVA_CONVERSA.md")
if "F1-15 (cargo audit)" in prompt_text or "F1-26..F1-28 (a spec os classifica como MVP)" in prompt_text:
    warnings.append("INC-12 open: PROMPT_NOVA_CONVERSA.md keeps superseded 'o que resta' statements")
index_range = re.search(r"<requisitos-funcionais[^>]*>\s*RF-001 a RF-055", spec_all)
if index_range and (KNOWN_OPEN_RF - rf_ids):
    warnings.append("INC-16 open: the XML index declares RF-001 to RF-055 but " + ", ".join(sorted(KNOWN_OPEN_RF - rf_ids)) + " are not in the requirement table")
spec_date = re.search(r"<data>(\d{4}-\d{2}-\d{2})</data>", spec_all)
note_dates = re.findall(r"Nota de implementação superseded — (\d{4}-\d{2}-\d{2})", spec_all)
if spec_date and note_dates and max(note_dates) > spec_date.group(1):
    warnings.append(f"INC-06 open: the specification <data> is {spec_date.group(1)} but its implementation notes go up to {max(note_dates)}")

if not (ROOT / "tests/integration/fase0_runtime.ps1").is_file():
    errors.append("native Windows runtime integration script is missing")

for required_tool_file in ["deny.toml", "docs/SUPPLY_CHAIN.md"]:
    if not (ROOT / required_tool_file).is_file():
        errors.append(f"missing supply-chain artifact: {required_tool_file}")

if shutil.which("cargo") is None:
    warnings.append("Cargo is unavailable in this environment; compile/test/lockfile regeneration remain pending")

print(f"Fase 0 static checks: {len(errors)} error(s), {len(warnings)} warning(s)")
for error in errors:
    print(f"ERROR: {error}")
for warning in warnings:
    print(f"WARNING: {warning}")
if errors:
    raise SystemExit(1)
