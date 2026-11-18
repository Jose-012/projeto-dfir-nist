from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

request = {
    "protocol_version": "1",
    "schema_version": "2",
    "message_type": "REQUEST",
    "request_id": "req-smoke",
    "job_id": "job-smoke",
    "input_reference": "evidence/__missing_protocol_smoke__.evtx",
    "output_reference": None,
    "max_output_bytes": 65536,
    "max_items": 100,
}

# The engine defaults DFIR_ENGINE_INPUT_ROOT to the fixed path "/input" (see
# engine/src/main.rs::input_root), which only exists inside the production
# authorized input root. Outside the authorized input root (bare `cargo run`, this smoke test, any
# dev machine or CI runner, and *especially* Windows where "/input" resolves
# under the current drive) that root does not exist, so
# std::fs::canonicalize(root) fails before the engine ever looks at
# input_reference. That failure is reported under the SAME "INPUT_UNAVAILABLE"
# code as a genuinely-missing evidence file, so a naive code-only assertion
# below would report PASS even though it never exercised the file-not-found
# path this test claims to cover.
#
# Fix: give the engine a real, empty root via DFIR_ENGINE_INPUT_ROOT so only
# the referenced *file* is missing, and assert on the specific message text
# so a regression back to "root unavailable" fails loudly instead of passing
# for the wrong reason.
with tempfile.TemporaryDirectory(prefix="dfir-engine-protocol-smoke-") as input_root:
    env = {**os.environ, "DFIR_ENGINE_INPUT_ROOT": input_root}

    proc = subprocess.run(
        ["cargo", "run", "--locked", "--quiet", "-p", "dfir-engine"],
        input=json.dumps(request) + "\n",
        # Explicit encoding: the protocol is UTF-8 JSONL regardless of host
        # locale. Relying on subprocess's implicit locale-preferred encoding
        # (what plain text=True does) mangles any non-ASCII text in engine
        # messages -- e.g. OS-localized error strings -- on hosts where the
        # default locale encoding isn't UTF-8 (notably Windows/PowerShell).
        encoding="utf-8",
        capture_output=True,
        timeout=30,
        cwd=ROOT,
        env=env,
    )
if proc.returncode != 0:
    print(proc.stderr, file=sys.stderr, end="")
    raise SystemExit(proc.returncode or 1)
lines = [json.loads(line) for line in proc.stdout.splitlines() if line.strip()]
if len(lines) < 3:
    print(proc.stdout, file=sys.stderr, end="")
    raise SystemExit("engine protocol smoke: expected at least 3 JSONL messages")

if lines[0].get("message_type") != "PROGRESS" or lines[0].get("phase") != "accepted":
    raise SystemExit("engine protocol smoke: missing accepted PROGRESS message")
if not any(m.get("message_type") == "PROGRESS" and m.get("phase") == "parsing_evtx" for m in lines):
    raise SystemExit("engine protocol smoke: missing parsing_evtx PROGRESS message")

errors = [m for m in lines if m.get("message_type") == "ERROR"]
if not errors or errors[-1].get("code") != "INPUT_UNAVAILABLE":
    print(proc.stdout, file=sys.stderr, end="")
    raise SystemExit("engine protocol smoke: expected INPUT_UNAVAILABLE for the intentionally missing EVTX fixture")

last_message = errors[-1].get("message", "")
if not last_message.startswith("input file unavailable"):
    print(proc.stdout, file=sys.stderr, end="")
    raise SystemExit(
        "engine protocol smoke: got INPUT_UNAVAILABLE but for the wrong reason "
        f"(message: {last_message!r}); expected the missing-file path, which means "
        "DFIR_ENGINE_INPUT_ROOT is not resolving to a real directory -- check that "
        "the temp input root is actually being passed to the engine subprocess"
    )

print(proc.stdout, end="")
print("engine protocol smoke: PASS")
