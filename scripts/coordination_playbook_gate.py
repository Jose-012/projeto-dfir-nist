#!/usr/bin/env python3
"""Structural gate for the coordination playbook.

This gate checks deterministic document structure only. It does not measure
writing quality, infer semantic correctness, or grant release acceptance.
"""

from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_TARGET = ROOT / "docs" / "COORDINATION_PLAYBOOK.md"

REQUIRED_CP_IDS = tuple(f"CP-{index:02d}" for index in range(29))
REQUIRED_ROLES = ("Cricket", "Fox", "Lobster", "Ox", "Rat")
REQUIRED_REFERENCES = (
    "AGENTS.md",
    "CONVENTIONS.md",
    "docs/DOCUMENT_AUTHORITY.md",
    "docs/SESSION_IMPROVEMENT.md",
    "docs/SESSION_LEARNINGS.md",
    ".jcode/swarm-prompt.md",
    ".jcode/skills/dfir-improve/SKILL.md",
    ".jcode/skills/dfir-change/SKILL.md",
    ".jcode/skills/dfir-verify/SKILL.md",
    ".jcode/skills/dfir-detect/SKILL.md",
    "docs/RELEASE_GATE.md",
    "docs/EVIDENCE_REGISTER.md",
)

CP_HEADING = re.compile(r"(?m)^#{2,6}[ \t]+(CP-\d{2})\b[^\n]*$")
ROLE_HEADING = re.compile(
    r"(?m)^#{3,6}[ \t]+(" + "|".join(re.escape(role) for role in REQUIRED_ROLES) + r")\b[^\n]*$"
)
PLACEHOLDER_FIELD = re.compile(r"\{\{[^{}\n]+\}\}")
PLACEHOLDER_TOKEN = re.compile(r"(?<![A-Za-z0-9_])(?:TODO|TBD)(?![A-Za-z0-9_])")
INLINE_CODE = re.compile(r"`[^`\n]*`")


@dataclass(frozen=True)
class Diagnostic:
    line: int
    rule: str
    message: str

    def render(self, path: Path) -> str:
        return f"{path}:{self.line}: {self.rule}: {self.message}"


def _line_number(text: str, offset: int) -> int:
    return text.count("\n", 0, offset) + 1


def _without_inline_code(text: str) -> str:
    """Blank inline code while preserving offsets and line structure."""
    return INLINE_CODE.sub(lambda match: " " * len(match.group(0)), text)


def _non_code_paragraphs(text: str) -> list[tuple[int, str]]:
    paragraphs: list[tuple[int, str]] = []
    in_fence = False
    block: list[str] = []
    block_start = 1

    def flush() -> None:
        nonlocal block
        if not block:
            return
        candidate = "\n".join(block).strip()
        first = block[0].lstrip()
        excluded = (
            first.startswith(("#", "-", "*", "+", "|"))
            or re.match(r"^\d+[.)]\s", first) is not None
        )
        if not excluded and len(candidate) >= 80:
            normalized = re.sub(r"\s+", " ", candidate).casefold()
            paragraphs.append((block_start, normalized))
        block = []

    for line_number, line in enumerate(text.splitlines(), start=1):
        if line.lstrip().startswith("```"):
            flush()
            in_fence = not in_fence
            continue
        if in_fence:
            continue
        if not line.strip():
            flush()
            continue
        if not block:
            block_start = line_number
        block.append(line)
    flush()
    return paragraphs


def validate(text: str) -> list[Diagnostic]:
    diagnostics: list[Diagnostic] = []
    headings = list(CP_HEADING.finditer(text))
    heading_ids = [match.group(1) for match in headings]
    heading_lines = {match.group(1): _line_number(text, match.start()) for match in headings}

    for cp_id in REQUIRED_CP_IDS:
        if cp_id not in heading_lines:
            diagnostics.append(
                Diagnostic(1, "missing_cp_id", f"required section {cp_id} is missing")
            )

    for cp_id in sorted(set(heading_ids)):
        count = heading_ids.count(cp_id)
        if count > 1:
            diagnostics.append(
                Diagnostic(
                    heading_lines[cp_id],
                    "duplicate_cp_id",
                    f"section {cp_id} has {count} headings",
                )
            )

    for index, match in enumerate(headings):
        section_end = headings[index + 1].start() if index + 1 < len(headings) else len(text)
        section = text[match.end() : section_end]
        meaningful = [
            line
            for line in section.splitlines()
            if line.strip()
            and line.strip() not in {"---", "***", "___"}
            and not line.lstrip().startswith("#")
        ]
        if not meaningful:
            diagnostics.append(
                Diagnostic(
                    _line_number(text, match.start()),
                    "empty_cp_section",
                    f"section {match.group(1)} has no content",
                )
            )

    placeholder_patterns = (
        ("placeholder_field", PLACEHOLDER_FIELD),
        ("placeholder_token", PLACEHOLDER_TOKEN),
    )
    text_without_inline_code = _without_inline_code(text)
    for rule, pattern in placeholder_patterns:
        for match in pattern.finditer(text_without_inline_code):
            diagnostics.append(
                Diagnostic(
                    _line_number(text, match.start()),
                    rule,
                    f"unresolved placeholder {match.group(0)!r}",
                )
            )

    for reference in REQUIRED_REFERENCES:
        if reference not in text:
            diagnostics.append(
                Diagnostic(1, "missing_reference", f"required reference {reference!r} is absent")
            )

    dedicated_role_headings = [match.group(1) for match in ROLE_HEADING.finditer(text)]
    for role in REQUIRED_ROLES:
        if role not in dedicated_role_headings:
            diagnostics.append(
                Diagnostic(
                    1,
                    "missing_role",
                    f"required dedicated role heading {role!r} is absent",
                )
            )

    normalized_paragraphs = _non_code_paragraphs(text)
    grouped: dict[str, list[int]] = {}
    for line_number, normalized in normalized_paragraphs:
        grouped.setdefault(normalized, []).append(line_number)
    for normalized, line_numbers in sorted(grouped.items()):
        if len(line_numbers) >= 3:
            diagnostics.append(
                Diagnostic(
                    line_numbers[0],
                    "excessive_duplicate",
                    "normalized paragraph occurs "
                    f"{len(line_numbers)} times (limit 2): {normalized[:80]!r}",
                )
            )

    return sorted(diagnostics, key=lambda item: (item.line, item.rule, item.message))


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Check deterministic structure of the coordination playbook."
    )
    parser.add_argument(
        "target",
        nargs="?",
        type=Path,
        default=DEFAULT_TARGET,
        help=f"playbook path (default: {DEFAULT_TARGET})",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    target = args.target.expanduser().resolve()
    if not target.is_file():
        print(
            f"{target}:1: missing_target: target file does not exist",
            file=sys.stderr,
        )
        return 2

    try:
        text = target.read_text(encoding="utf-8")
    except (OSError, UnicodeError) as error:
        print(
            f"{target}:1: unreadable_target: {error}",
            file=sys.stderr,
        )
        return 2

    diagnostics = validate(text)
    if diagnostics:
        for diagnostic in diagnostics:
            print(diagnostic.render(target), file=sys.stderr)
        print(
            f"coordination playbook gate: FAIL ({len(diagnostics)} structural diagnostic(s))",
            file=sys.stderr,
        )
        return 1

    print(
        "coordination playbook gate: PASS (structural checks only; no semantic acceptance)"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
