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
for path in (
    ROOT / "docs" / "DOCUMENT_AUTHORITY.md",
    ROOT / "docs" / "DEFINITION_OF_READY.md",
    ROOT / "docs" / "SESSION_IMPROVEMENT.md",
    ROOT / "docs" / "SESSION_LEARNINGS.md",
    ROOT / ".jcode" / "swarm-prompt.md",
    ROOT / "toolchains.toml",
):
    if not path.is_file():
        print(f"required documentation/build artifact missing: {path}", file=sys.stderr)
        raise SystemExit(1)

# Skills locais: existência e metadados mínimos, sem tratá-las como prova de conduta.
for skill_name in ("dfir-improve", "dfir-change", "dfir-verify", "dfir-detect"):
    skill_path = ROOT / ".jcode" / "skills" / skill_name / "SKILL.md"
    if not skill_path.is_file():
        print(f"required local skill missing: {skill_path.relative_to(ROOT)}", file=sys.stderr)
        raise SystemExit(1)
    skill_text = skill_path.read_text(encoding="utf-8")
    if not skill_text.startswith("---\n"):
        print(f"local skill has no frontmatter: {skill_path.relative_to(ROOT)}", file=sys.stderr)
        raise SystemExit(1)
    for needle in (
        f"name: {skill_name}",
        "allowed-tools:",
    ):
        if needle not in skill_text:
            print(f"local skill metadata missing: {skill_path.relative_to(ROOT)}: {needle}", file=sys.stderr)
            raise SystemExit(1)
    if not re.search(r"(?m)^description:\s*\S+", skill_text):
        print(f"local skill description missing: {skill_path.relative_to(ROOT)}", file=sys.stderr)
        raise SystemExit(1)
    if skill_name == "dfir-improve":
        for needle in (
            "Após uma mutação de todo",
            "nunca a `bg wait`",
            "escrita atrasada",
            "Em `skill_manage`, declare `action` explicitamente",
            "overlay local é relido do disco",
            ".jcode/swarm-prompt.md",
        ):
            if needle not in skill_text:
                print(f"dfir-improve safeguard missing: {needle}", file=sys.stderr)
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

# AGENTS §16/§17: o loop entre sessões e a política de coordenação precisam
# permanecer instalados nos pontos de entrada dos agentes. O gate prova
# presença e ligação, não a conduta futura.
session_instruction_needles = {
    ROOT / "AGENTS.md": (
        "## 16. Autoaprimoramento obrigatório entre sessões",
        "## 17. Swarm, skills e MCPs",
        "nunca a `bg wait`",
        ".jcode/skills/dfir-improve/SKILL.md",
        ".jcode/skills/dfir-detect/SKILL.md",
        ".jcode/swarm-prompt.md",
        "docs/SESSION_IMPROVEMENT.md",
        "docs/SESSION_LEARNINGS.md",
        "get_call_hierarchy",
        "tools.mcp",
        "rtk recall",
        "agent-models.meta",
    ),
    ROOT / "CLAUDE.md": (
        "# Loop obrigatório de sessão",
        "`AGENTS.md` §17",
        ".jcode/skills/dfir-improve/SKILL.md",
        ".jcode/swarm-prompt.md",
        "docs/SESSION_IMPROVEMENT.md",
        "docs/SESSION_LEARNINGS.md",
    ),
    ROOT / "CONVENTIONS.md": (
        "docs/SESSION_IMPROVEMENT.md",
        "docs/SESSION_LEARNINGS.md",
        ".jcode/skills/dfir-improve/SKILL.md",
        ".jcode/skills/dfir-detect/SKILL.md",
        ".jcode/swarm-prompt.md",
    ),
    ROOT / "docs" / "DOCUMENT_AUTHORITY.md": (
        ".jcode/skills/dfir-improve/SKILL.md",
        ".jcode/swarm-prompt.md",
        "docs/SESSION_IMPROVEMENT.md",
        "docs/SESSION_LEARNINGS.md",
    ),
    ROOT / "docs" / "SESSION_IMPROVEMENT.md": (
        ".jcode/skills/dfir-improve/SKILL.md",
    ),
}
for path, needles in session_instruction_needles.items():
    instruction_text = path.read_text(encoding="utf-8")
    for needle in needles:
        if needle not in instruction_text:
            print(f"missing session-improvement instruction: {path.relative_to(ROOT)}: {needle}", file=sys.stderr)
            raise SystemExit(1)

# O template precisa conservar autoridade, loop, prevenção de worker duplicado e
# publicação explícita via gh. O gate não valida a qualidade da missão preenchida.
swarm_prompt_text = (ROOT / ".jcode" / "swarm-prompt.md").read_text(encoding="utf-8")
for needle in (
    "AGENTS.md",
    ".jcode/skills/dfir-improve/SKILL.md",
    "swarm spawn",
    "await_members",
    "escrita atrasada",
    "agent-models.meta",
    "swarm_model_selection",
    "gh",
):
    if needle not in swarm_prompt_text:
        print(f"missing swarm-prompt requirement: {needle}", file=sys.stderr)
        raise SystemExit(1)

print("documentation gate: PASS")
