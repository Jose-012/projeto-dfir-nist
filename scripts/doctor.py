#!/usr/bin/env python3
"""Diagnóstico leve do ambiente de desenvolvimento, sem instalar ferramentas."""

from __future__ import annotations

import os
import platform
import re
import shutil
import subprocess
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
VERSION = re.compile(r"\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?")


def command_version(command: list[str]) -> str | None:
    """Return only a version token; never echo command stderr or the environment."""
    if shutil.which(command[0]) is None:
        return None
    try:
        result = subprocess.run(
            command,
            cwd=ROOT,
            env={**os.environ, "LC_ALL": "C", "NO_COLOR": "1"},
            capture_output=True,
            text=True,
            timeout=10,
            check=False,
        )
    except (OSError, subprocess.TimeoutExpired):
        return None
    if result.returncode != 0:
        return None
    match = VERSION.search(result.stdout)
    return match.group(0) if match else None


def recipe_pin(package: str) -> str | None:
    text = (ROOT / "justfile").read_text(encoding="utf-8")
    match = re.search(rf"cargo install --locked {re.escape(package)}@([0-9.]+)", text)
    return match.group(1) if match else None


def main() -> int:
    errors = 0
    warnings = 0
    rust_pin = tomllib.loads((ROOT / "rust-toolchain.toml").read_text(encoding="utf-8"))["toolchain"]["channel"]
    python_reference = tomllib.loads((ROOT / "toolchains.toml").read_text(encoding="utf-8"))["python"]["version"]

    print(f"Host: {platform.system()} {platform.machine()}")

    required = (
        ("rustc", ["rustc", "--version"], rust_pin, True),
        ("cargo", ["cargo", "--version"], None, False),
        ("cargo-nextest", ["cargo", "nextest", "--version"], recipe_pin("cargo-nextest"), False),
    )
    for label, command, reference, strict_version in required:
        actual = command_version(command)
        if actual is None:
            print(f"ERROR required {label}: unavailable")
            errors += 1
        elif reference and actual != reference:
            level = "ERROR" if strict_version else "WARN"
            print(f"{level} required {label}: {actual} (reference {reference})")
            if strict_version:
                errors += 1
            else:
                warnings += 1
        else:
            suffix = f" (reference {reference})" if reference else ""
            print(f"OK required {label}: {actual}{suffix}")

    python_actual = platform.python_version()
    if python_actual == python_reference:
        print(f"OK required Python: {python_actual} (reference {python_reference})")
    else:
        print(f"WARN required Python: {python_actual} (reference {python_reference}); gates apply to the actual interpreter")
        warnings += 1

    for label, command in (
        ("cargo-audit", ["cargo", "audit", "--version"]),
        ("cargo-deny", ["cargo", "deny", "--version"]),
    ):
        actual = command_version(command)
        reference = recipe_pin(label)
        if actual is None:
            print(f"WARN supply-chain {label}: unavailable; just supply-chain cannot run")
            warnings += 1
        elif reference and actual != reference:
            print(f"WARN supply-chain {label}: {actual} (release-tools {reference})")
            warnings += 1
        else:
            print(f"OK supply-chain {label}: {actual}")

    for label, command in (
        ("just", ["just", "--version"]),
        ("git", ["git", "--version"]),
        ("delta", ["delta", "--version"]),
    ):
        actual = command_version(command)
        if actual is None:
            print(f"INFO optional {label}: unavailable")
        else:
            print(f"OK optional {label}: {actual}")

    print(f"Doctor: {errors} error(s), {warnings} warning(s); no tools were installed or changed")
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main())
