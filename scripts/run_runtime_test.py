from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
script = ROOT / "tests" / "integration" / "fase0_runtime.sh"
if os.name == "nt":
    # The runtime gate is QA tooling. Force a process-scoped Bypass so
    # PowerShell can execute this fixed local test script in non-interactive
    # Windows hosts without changing the machine or user execution policy.
    subprocess.run(
        [
            "powershell.exe",
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            str(ROOT / "tests" / "integration" / "fase0_runtime.ps1"),
        ],
        cwd=ROOT,
        check=True,
    )
else:
    subprocess.run(["bash", str(script)], cwd=ROOT, check=True)
