from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
cargo = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
config = (ROOT / "src/config.rs").read_text(encoding="utf-8")
# ADR-0017: src/web/mod.rs (Axum handlers) was removed; the equivalent
# "no obsolete route/mutating-endpoint string survives" check now reads the
# service layer + native UI that replaced it.
service = (ROOT / "src/service/mod.rs").read_text(encoding="utf-8")
gui_mod = (ROOT / "src/gui/mod.rs").read_text(encoding="utf-8")
migration = (ROOT / "migrations/0003_local_forensic_scope.sql").read_text(encoding="utf-8")

for obsolete in ["argon2 =", "axum-extra =", "getrandom =", "axum =", "askama =", "tower-http ="]:
    assert obsolete not in cargo, f"obsolete dependency remains: {obsolete}"
for obsolete in ["DFIR_HTTPS", "DFIR_ALLOW_INSECURE_HTTP", "DFIR_BOOTSTRAP_PASSWORD", "DFIR_BIND"]:
    assert obsolete not in config, f"obsolete configuration remains: {obsolete}"
for obsolete in ["/login", "/logout", "/admin/", "/audit/verify"]:
    assert obsolete not in service and obsolete not in gui_mod, f"obsolete/mutating local route remains: {obsolete}"
for obsolete in ["CREATE TABLE IF NOT EXISTS users", "CREATE TABLE IF NOT EXISTS sessions", "CREATE TABLE IF NOT EXISTS incident_access"]:
    assert obsolete not in migration, f"obsolete access schema remains: {obsolete}"
# ADR-0017: there is no HTTP bind left to default to loopback — the native UI
# opens no network port at all, which is the stronger property.
assert "bind" not in config.lower(), "no HTTP bind configuration should remain in a binary with no server"
assert 'runtime' in config
print("local forensic security config: PASS")
