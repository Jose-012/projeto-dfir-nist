#!/usr/bin/env python3
"""Deterministic repo audit for the coordination playbook.

Structural/deterministic checks only, separated by context:

- docs/COORDINATION_PLAYBOOK.md: JSON block validity, versioned backtick
  reference existence, prohibited path/token patterns.
- docs/SESSION_LEARNINGS.md: lesson heading sequence (contiguous from 1, no
  duplicates/absences/out-of-order, maximum derived from content).
- justfile: the ``verify:`` recipe only, exact six steps in canonical order.

Does not measure prose quality, infer semantic correctness, or grant
acceptance. Deterministic audit only.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from collections.abc import Callable
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_PLAYBOOK = ROOT / "docs" / "COORDINATION_PLAYBOOK.md"
DEFAULT_LESSONS = ROOT / "docs" / "SESSION_LEARNINGS.md"
DEFAULT_JUSTFILE = ROOT / "justfile"

# Lesson headings: "### N. Title"
LESSON_HEADING = re.compile(r"(?m)^###[ \t]+(\d+)\.\s+(.+?)\s*$")

# The six exact `just verify` steps in canonical order.
JUST_VERIFY_STEPS = (
    "just validate-fase0",
    "just fmt",
    "just check",
    "just nextest",
    "just doctest",
    "just clippy",
)

# Prohibited path/token patterns (root-authorized). Scanned over the whole
# text (including inside {{...}}); the placeholder linter handles {{...}}
# itself, so prohibited patterns are never skipped just because they sit in a
# placeholder.
PROHIBITED_PATTERNS = [
    ("absolute_host_path", re.compile(r"/home/jose|/mnt/hd")),
    ("temporary_round_reference", re.compile(r"validation/task-")),
    ("hash_40", re.compile(r"(?<![0-9a-fA-F])[0-9a-fA-F]{40}(?![0-9a-fA-F])")),
    ("hash_64", re.compile(r"(?<![0-9a-fA-F])[0-9a-fA-F]{64}(?![0-9a-fA-F])")),
    ("truncation_sentinel", re.compile(r"\\ No newline|\.\.\. \d+ more lines")),
    (
        "known_token",
        re.compile(
            r"Invalid recipient|命令|Entrã|Costo|implementasi|Never use|\blinked\b|\binsertar\b"
        ),
    ),
    ("codemode_payloads", re.compile(r"codemode-payloads")),
    ("bytes_zero", re.compile(r'"bytes": 0')),
    (
        "reserved_phrase",
        re.compile(r"Fox executa, Cricket valida|Proibir exceto com ADR|cargo expand"),
    ),
]

# Fenced ```json blocks.
JSON_BLOCK_RE = re.compile(r"(?m)^```json\s*$")

# Repository references in backticks, with optional :line or :line-line suffix.
BACKTICK_RE = re.compile(r"`([^`\n]+)`")
REFERENCE_LINE_SUFFIX_RE = re.compile(r":\d+(?:-\d+)?$")

# Tracked reference base prefixes / root names whose existence is checked.
TRACKED_PREFIXES = ("docs/", ".jcode/", "src/", "scripts/", "tests/")
TRACKED_ROOT_FILES = ("AGENTS.md", "CONVENTIONS.md", "justfile")


@dataclass(frozen=True)
class Diagnostic:
    line: int
    rule: str
    message: str
    is_finding: bool = True

    def render(self, path: Path) -> str:
        return f"{path}:{self.line}: {self.rule}: {self.message}"


def _line_number(text: str, offset: int) -> int:
    return text.count("\n", 0, offset) + 1


def _is_tracked_ref(base: str) -> bool:
    if any(base.startswith(prefix) for prefix in TRACKED_PREFIXES):
        return True
    return base in TRACKED_ROOT_FILES


def _is_ignored_ref(raw: str) -> bool:
    return any(token in raw for token in ("{{", "<", "*", "?"))


# ---------------------------------------------------------------------------
# Rule checks
# ---------------------------------------------------------------------------

def _check_lessons(text: str) -> list[Diagnostic]:
    diagnostics: list[Diagnostic] = []
    matches = list(LESSON_HEADING.finditer(text))
    found = [
        (int(m.group(1)), m.group(2).strip(), _line_number(text, m.start()))
        for m in matches
    ]
    found_numbers = [n for n, _, _ in found]
    if not found_numbers:
        return [Diagnostic(1, "missing_lessons", "no numbered lesson headings found")]

    max_lesson = max(found_numbers)
    expected = list(range(1, max_lesson + 1))

    # Duplicates.
    seen: set[int] = set()
    for n, _, line in found:
        if n in seen:
            diagnostics.append(Diagnostic(line, "duplicate_lesson", f"duplicate lesson {n:02d}"))
        seen.add(n)

    # Missing relative to contiguous 1..max (max derived from content).
    missing = set(expected) - set(found_numbers)
    for n in sorted(missing):
        diagnostics.append(
            Diagnostic(1, "missing_lesson", f"lesson {n:02d} is missing")
        )

    # Out of order.
    for i in range(len(found) - 1):
        if found[i][0] > found[i + 1][0]:
            diagnostics.append(
                Diagnostic(
                    found[i + 1][2],
                    "lesson_order",
                    f"lesson {found[i + 1][0]:02d} appears before lesson {found[i][0]:02d}",
                )
            )

    return diagnostics


def _check_verify_recipe(text: str) -> list[Diagnostic]:
    diagnostics: list[Diagnostic] = []
    lines = text.split("\n")
    in_recipe = False
    body: list[tuple[int, str]] = []
    for i, line in enumerate(lines, start=1):
        if not in_recipe:
            if re.match(r"^verify\s*:", line):
                in_recipe = True
            continue
        # Stop at the first blank line or non-indented line.
        if line == "" or not line.startswith((" ", "\t")):
            break
        body.append((i, line))

    step_re = re.compile(r"^\s*just\s+(.+?)\s*$")
    found_steps: list[tuple[str, int]] = []
    for actual_line, line in body:
        m = step_re.match(line)
        if m:
            found_steps.append((f"just {m.group(1)}", actual_line))

    found_step_names = [step for step, _ in found_steps]

    for step in JUST_VERIFY_STEPS:
        if step not in found_step_names:
            diagnostics.append(
                Diagnostic(1, "missing_just_verify_step", f"missing step: {step}")
            )

    prev_idx: int | None = None
    prev_step: str | None = None
    for step, actual_line in found_steps:
        if step not in JUST_VERIFY_STEPS:
            diagnostics.append(
                Diagnostic(
                    actual_line,
                    "unexpected_just_verify_step",
                    f"unexpected step in exact six-step recipe: {step}",
                )
            )
            continue
        idx = JUST_VERIFY_STEPS.index(step)
        if prev_idx is not None and idx <= prev_idx:
            diagnostics.append(
                Diagnostic(
                    actual_line,
                    "just_verify_order",
                    f"{step} out of order (after {prev_step})",
                )
            )
        prev_idx = idx
        prev_step = step

    return diagnostics


def _check_json_blocks(text: str) -> list[Diagnostic]:
    diagnostics: list[Diagnostic] = []
    for match in JSON_BLOCK_RE.finditer(text):
        line = _line_number(text, match.start())
        start = match.end()
        end = text.find("\n```", start)
        block = text[start:] if end == -1 else text[start:end]
        try:
            json.loads(block)
        except json.JSONDecodeError as error:
            diagnostics.append(
                Diagnostic(
                    line,
                    "invalid_json",
                    f"invalid JSON in ```json block: {error}",
                )
            )
        else:
            diagnostics.append(
                Diagnostic(line, "json_block", "valid ```json block", is_finding=False)
            )
    return diagnostics


def _check_versioned_refs(
    text: str, exists: Callable[[str], bool]
) -> list[Diagnostic]:
    diagnostics: list[Diagnostic] = []
    seen_references: set[str] = set()
    for match in BACKTICK_RE.finditer(text):
        raw = match.group(1)
        base = REFERENCE_LINE_SUFFIX_RE.sub("", raw)
        if _is_ignored_ref(raw):
            continue
        if any(character.isspace() for character in base):
            continue
        if not _is_tracked_ref(base):
            continue
        line = _line_number(text, match.start())
        if not exists(base):
            diagnostics.append(
                Diagnostic(line, "missing_reference", f"missing reference: {raw}")
            )
        else:
            if base not in seen_references:
                diagnostics.append(
                    Diagnostic(
                        line,
                        "versioned_reference",
                        f"reference exists: `{base}`",
                        is_finding=False,
                    )
                )
                seen_references.add(base)
    return diagnostics


def _check_prohibited(text: str) -> list[Diagnostic]:
    diagnostics: list[Diagnostic] = []
    for rule, pattern in PROHIBITED_PATTERNS:
        for match in pattern.finditer(text):
            line = _line_number(text, match.start())
            diagnostics.append(
                Diagnostic(
                    line,
                    rule,
                    f"prohibited pattern: {match.group(0)!r}",
                )
            )
    return diagnostics


def validate(
    playbook_text: str,
    lessons_text: str,
    justfile_text: str,
    exists: Callable[[str], bool] | None = None,
) -> tuple[list[Diagnostic], list[Diagnostic]]:
    """Run every rule over the three contexts and return (findings, informational)."""
    if exists is None:
        exists = lambda p: (ROOT / p).exists()

    findings: list[Diagnostic] = []
    informational: list[Diagnostic] = []

    # Playbook context only: JSON validity, versioned refs, prohibited patterns.
    for diagnostic in (
        _check_json_blocks(playbook_text)
        + _check_versioned_refs(playbook_text, exists)
    ):
        if diagnostic.is_finding:
            findings.append(diagnostic)
        else:
            informational.append(diagnostic)
    findings.extend(_check_prohibited(playbook_text))

    # Lessons context only: SESSION_LEARNINGS.md.
    findings.extend(_check_lessons(lessons_text))

    # just verify context only: justfile verify recipe.
    findings.extend(_check_verify_recipe(justfile_text))

    findings.sort(key=lambda d: (d.line, d.rule, d.message))
    informational.sort(key=lambda d: (d.line, d.rule, d.message))
    return findings, informational


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------

def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Deterministic repo audit for the coordination playbook."
    )
    parser.add_argument(
        "playbook",
        nargs="?",
        type=Path,
        default=DEFAULT_PLAYBOOK,
        help=f"playbook file (default: {DEFAULT_PLAYBOOK})",
    )
    parser.add_argument(
        "lessons",
        nargs="?",
        type=Path,
        default=DEFAULT_LESSONS,
        help=f"lessons file (default: {DEFAULT_LESSONS})",
    )
    parser.add_argument(
        "justfile",
        nargs="?",
        type=Path,
        default=DEFAULT_JUSTFILE,
        help=f"justfile (default: {DEFAULT_JUSTFILE})",
    )
    parser.add_argument(
        "--verbose",
        action="store_true",
        help="print each valid JSON block and unique reference",
    )
    return parser.parse_args(argv)


def _read(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except (OSError, UnicodeError) as error:
        print(f"{path}:1: unreadable_target: {error}", file=sys.stderr)
        raise SystemExit(2)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    playbook = args.playbook.expanduser().resolve()
    lessons = args.lessons.expanduser().resolve()
    justfile_path = args.justfile.expanduser().resolve()

    for path in (playbook, lessons, justfile_path):
        if not path.is_file():
            print(f"{path}:1: missing_target: target file does not exist", file=sys.stderr)
            return 2

    playbook_text = _read(playbook)
    lessons_text = _read(lessons)
    justfile_text = _read(justfile_path)

    findings, informational = validate(playbook_text, lessons_text, justfile_text)

    if args.verbose:
        for diagnostic in informational:
            print(diagnostic.render(playbook), file=sys.stdout)

    if findings:
        for diagnostic in findings:
            print(diagnostic.render(playbook), file=sys.stderr)
        print(
            f"repo audit: FAIL ({len(findings)} deterministic finding(s), "
            f"{len(informational)} informational item(s))",
            file=sys.stderr,
        )
        return 1

    json_blocks = sum(item.rule == "json_block" for item in informational)
    references = sum(item.rule == "versioned_reference" for item in informational)
    print(
        f"repo audit: PASS ({json_blocks} valid JSON block(s), {references} reference(s), "
        "deterministic checks only, no semantic acceptance)"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
