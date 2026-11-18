from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

# EVI-04 (2026-11-04): pareamento da especificação por versão, para qualquer versão
# (`v6_5_0`, `v6_5_1`, ...): docs/PROJECT_SPEC_v<V>.md + nist-senai-proj_integrador_v<V>.md.
specs = sorted((ROOT / "docs").glob("PROJECT_SPEC_v*.md"))
if len(specs) != 1:
    print(f"expected exactly one active project spec, found {len(specs)}", file=sys.stderr)
    raise SystemExit(1)

spec = specs[0]
version = spec.stem.removeprefix("PROJECT_SPEC_v")
if not re.fullmatch(r"\d+(?:_\d+)+", version):
    print(f"unexpected specification version token: {version!r} (expected e.g. 6_5_1)", file=sys.stderr)
    raise SystemExit(1)
root_spec = ROOT / f"nist-senai-proj_integrador_v{version}.md"
if not root_spec.is_file():
    print(f"root specification copy missing: {root_spec.name}", file=sys.stderr)
    raise SystemExit(1)
orphans = [path.name for path in sorted(ROOT.glob("nist-senai-proj_integrador_v*.md")) if path != root_spec]
if orphans:
    print(f"root specification copy for another version: {', '.join(orphans)}", file=sys.stderr)
    raise SystemExit(1)
if spec.read_bytes() != root_spec.read_bytes():
    print("specification copies differ", file=sys.stderr)
    raise SystemExit(1)

# ADR-0017: no more frontend/dist artifact to check — the native UI (src/gui/)
# is part of the dfir-app binary itself, verified by cargo check/test, not by
# a build artifact on disk.
for path in (ROOT / "docs" / "DOCUMENT_AUTHORITY.md", ROOT / "docs" / "DEFINITION_OF_READY.md", ROOT / "toolchains.toml"):
    if not path.is_file():
        print(f"required documentation/build artifact missing: {path}", file=sys.stderr)
        raise SystemExit(1)

text = (ROOT / "toolchains.toml").read_text(encoding="utf-8")
# Before ADR-0017 this also checked 'initial_reference = "1.3.14"' (the Bun
# pin). That needle was already stale by the time Node replaced Bun
# (ADR-0009) and was never updated to Node's own pin — this check was
# failing before this session touched anything. Removing the frontend
# toolchain entirely (ADR-0017) removes the needle along with its bug,
# rather than trading it for a new one.
# 2026-11: o operador decidiu usar a stable mais recente, não uma versão fixa neste script.
# O pin vem de toolchains.toml e precisa ser uma versão exata (X.Y.Z) igual à de rust-toolchain.toml.
manifest_pin = re.search(r'\[rust\]\s*\nchannel = "(\d+\.\d+\.\d+)"', text)
file_pin = re.search(r'channel = "(\d+\.\d+\.\d+)"', (ROOT / "rust-toolchain.toml").read_text(encoding="utf-8"))
if not manifest_pin or not file_pin:
    print("missing exact Rust pin (X.Y.Z) in toolchains.toml or rust-toolchain.toml", file=sys.stderr)
    raise SystemExit(1)
if manifest_pin.group(1) != file_pin.group(1):
    print(f"Rust pin differs: toolchains.toml {manifest_pin.group(1)} vs rust-toolchain.toml {file_pin.group(1)}", file=sys.stderr)
    raise SystemExit(1)
if 'version = "3.14.6"' not in text:
    print('missing toolchain pin: version = "3.14.6"', file=sys.stderr)
    raise SystemExit(1)

print("documentation gate: PASS")
