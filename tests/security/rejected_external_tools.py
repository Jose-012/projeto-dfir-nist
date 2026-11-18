#!/usr/bin/env python3
"""Fail if rejected full DFIR tools become product artifacts."""
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
artifacts = [
    ROOT / "Cargo.toml", ROOT / "engine" / "Cargo.toml",
]
forbidden = ("hayabusa", "chainsaw")
errors = []
for path in artifacts:
    text = path.read_text(encoding="utf-8").lower()
    for name in forbidden:
        if name in text:
            errors.append(f"{name} referenced by product artifact: {path.relative_to(ROOT)}")

if errors:
    for item in errors:
        print(f"ERROR: {item}")
    raise SystemExit(1)
print("rejected external tool gate: OK")
