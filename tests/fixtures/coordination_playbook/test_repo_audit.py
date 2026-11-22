#!/usr/bin/env python3
"""Positive and isolated negative controls for coordination_playbook_repo_audit.py.

These tests call validate() in memory. They do not create, delete, or mutate
checkout documents, do not require JCODE_SCRATCH_DIR or third-party packages,
and do not write to disk.

Each rule has a positive control and one or more negative controls isolating
exactly one violated condition.
"""

from __future__ import annotations

import importlib.util
import re
import sys
import unittest
from pathlib import Path

FIXTURE_DIR = Path(__file__).resolve().parent
ROOT = FIXTURE_DIR.parents[2]
AUDIT_PATH = ROOT / "scripts" / "coordination_playbook_repo_audit.py"
SPEC = importlib.util.spec_from_file_location("coordination_playbook_repo_audit", AUDIT_PATH)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError(f"cannot import {AUDIT_PATH}")
AUDIT = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = AUDIT
SPEC.loader.exec_module(AUDIT)


def findings_of(diagnostics) -> set[str]:
    return {d.rule for d in diagnostics}


# ---------------------------------------------------------------------------
# Fixtures
# ---------------------------------------------------------------------------

VALID_PLAYBOOK = """# Coordination Playbook audit fixture

Um texto narrativo com palavras que nao sao proibidas: a rodada de validacao
terminou, o arquivo esta em scratch e houve um pequeno dado corrompido na
anotacao, mas nada disso e um achado porque sao apenas narrativas.

```json
{"ok": true, "n": 1}
```

Outro bloco valido:

```json
{"a": 1, "b": 2}
```

Referencias versionadas existentes:

`docs/PROJECT_SPEC_v6_5_1.md:10-20`
`AGENTS.md:5`
`.jcode/skills/dfir-improve/SKILL.md:1`
`src/gui/theme.rs:1`
`scripts/coordination_playbook_repo_audit.py:1`
`tests/fixtures/coordination_playbook/test_repo_audit.py:1`
`CONVENTIONS.md:3`
`justfile:1`
"""

VALID_LESSONS = """# Lições reutilizáveis entre sessões

### 1. Primeira lição
Conteúdo da primeira lição.

### 2. Segunda lição
Conteúdo da segunda lição.

### 3. Terceira lição
Conteúdo da terceira lição.
"""

VALID_JUSTFILE = """verify:
    just validate-fase0
    just fmt
    just check
    just nextest
    just doctest
    just clippy

other:
    echo nao e parte da receita verify
"""

ALL_VALID = (VALID_PLAYBOOK, VALID_LESSONS, VALID_JUSTFILE)


# ---------------------------------------------------------------------------
# Negative mutations
# ---------------------------------------------------------------------------

def _remove_lesson(text: str, n: int) -> str:
    pattern = re.compile(
        rf"(?ms)^### {n}\..*?(?=^### {n + 1}\.|\Z)"
    )
    return pattern.sub("", text, count=1)


def _add_lesson(text: str, n: int) -> str:
    return text.rstrip() + f"\n\n### {n}. Lição extra\nConteudo da lição extra.\n"


def _swap_adjacent(text: str, n: int) -> str:
    m_a = re.search(rf"(?ms)^### {n}\..*?(?=^### {n + 1}\.)", text)
    m_b = re.search(rf"(?ms)^### {n + 1}\..*?(?=^### {n + 2}\.|\Z)", text)
    assert m_a and m_b
    a, b = m_a.group(0), m_b.group(0)
    return text.replace(a, "__A__").replace(b, a).replace("__A__", b)


class RepoAuditTests(unittest.TestCase):
    """Positive and negative controls in memory, no disk writes."""

    # ------------------------------------------------------------------
    # Positive: all valid
    # ------------------------------------------------------------------

    def test_positive_all_valid_no_findings(self) -> None:
        findings, informational = AUDIT.validate(*ALL_VALID)
        self.assertEqual(findings, [])
        self.assertEqual(sum(item.rule == "json_block" for item in informational), 2)
        self.assertEqual(sum(item.rule == "versioned_reference" for item in informational), 8)

    def test_positive_real_files_no_findings(self) -> None:
        playbook = AUDIT.DEFAULT_PLAYBOOK.read_text(encoding="utf-8")
        lessons = AUDIT.DEFAULT_LESSONS.read_text(encoding="utf-8")
        justfile = AUDIT.DEFAULT_JUSTFILE.read_text(encoding="utf-8")
        findings, informational = AUDIT.validate(playbook, lessons, justfile)
        self.assertEqual(findings, [])
        self.assertGreaterEqual(sum(item.rule == "json_block" for item in informational), 9)
        self.assertGreaterEqual(sum(item.rule == "versioned_reference" for item in informational), 27)

    def test_negative_empty_lessons_detected(self) -> None:
        findings, _ = AUDIT.validate(VALID_PLAYBOOK, "# empty lessons\n", VALID_JUSTFILE)
        self.assertIn("missing_lessons", findings_of(findings))

    # ------------------------------------------------------------------
    # Negative: lessons (contiguous from 1, no dup/missing/out-of-order)
    # ------------------------------------------------------------------

    def test_negative_missing_lesson_detected(self) -> None:
        text = _remove_lesson(VALID_LESSONS, 2)
        findings, _ = AUDIT.validate(VALID_PLAYBOOK, text, VALID_JUSTFILE)
        self.assertIn("missing_lesson", findings_of(findings))

    def test_negative_extra_lesson_detected(self) -> None:
        """Add an unexpected lesson heading beyond the contiguous range."""
        text = _add_lesson(VALID_LESSONS, 100)
        findings, _ = AUDIT.validate(VALID_PLAYBOOK, text, VALID_JUSTFILE)
        # Lesson 100 is expected (max derived), but lessons 4..99 are missing.
        self.assertIn("missing_lesson", findings_of(findings))

    def test_negative_duplicate_lesson_detected(self) -> None:
        """Duplicate lesson 1 heading."""
        heading_line = VALID_LESSONS.splitlines()[2]  # "### 1. Primeira lição"
        text = VALID_LESSONS.rstrip() + "\n" + heading_line + "\nConteudo duplicado.\n"
        findings, _ = AUDIT.validate(VALID_PLAYBOOK, text, VALID_JUSTFILE)
        self.assertIn("duplicate_lesson", findings_of(findings))

    def test_negative_lesson_order_detected(self) -> None:
        text = _swap_adjacent(VALID_LESSONS, 1)
        findings, _ = AUDIT.validate(VALID_PLAYBOOK, text, VALID_JUSTFILE)
        self.assertIn("lesson_order", findings_of(findings))

    # ------------------------------------------------------------------
    # Negative: just verify recipe (stops at blank / non-indented line)
    # ------------------------------------------------------------------

    def test_negative_missing_just_verify_step_detected(self) -> None:
        text = VALID_JUSTFILE.replace("\n    just doctest\n", "\n")
        findings, _ = AUDIT.validate(VALID_PLAYBOOK, VALID_LESSONS, text)
        self.assertIn("missing_just_verify_step", findings_of(findings))

    def test_negative_just_verify_order_detected(self) -> None:
        text = (
            VALID_JUSTFILE.replace("    just fmt\n", "    __FMT__\n")
            .replace("    just check\n", "    just fmt\n")
            .replace("__FMT__\n", "    just check\n")
        )
        findings, _ = AUDIT.validate(VALID_PLAYBOOK, VALID_LESSONS, text)
        self.assertIn("just_verify_order", findings_of(findings))

    def test_negative_unexpected_just_verify_step_detected(self) -> None:
        text = VALID_JUSTFILE.replace(
            "    just clippy\n",
            "    just clippy\n    just test-coordination-playbook\n",
        )
        findings, _ = AUDIT.validate(VALID_PLAYBOOK, VALID_LESSONS, text)
        self.assertIn("unexpected_just_verify_step", findings_of(findings))

    def test_verify_parser_stops_at_blank_line(self) -> None:
        """Steps after a blank line inside the recipe must not be parsed."""
        text = (
            "verify:\n"
            "    just validate-fase0\n"
            "    just fmt\n"
            "\n"
            "    just check\n"
            "    just nextest\n"
            "    just doctest\n"
            "    just clippy\n"
        )
        findings, _ = AUDIT.validate(VALID_PLAYBOOK, VALID_LESSONS, text)
        self.assertNotIn("just_verify_order", findings_of(findings))
        missing = {d.message for d in findings if d.rule == "missing_just_verify_step"}
        self.assertTrue(any("check" in m for m in missing))

    def test_verify_parser_stops_at_non_indented_line(self) -> None:
        """A non-indented line ends the verify recipe body."""
        text = (
            "verify:\n"
            "    just validate-fase0\n"
            "    just fmt\n"
            "release-tools:\n"
            "    just check\n"
            "    just nextest\n"
            "    just doctest\n"
            "    just clippy\n"
        )
        findings, _ = AUDIT.validate(VALID_PLAYBOOK, VALID_LESSONS, text)
        missing = {d.message for d in findings if d.rule == "missing_just_verify_step"}
        # Body stops at the non-indented line: check is missing. nextest (and the
        # rest) are also missing because parsing stopped before them.
        self.assertTrue(any("check" in m for m in missing))

    # ------------------------------------------------------------------
    # Negative: JSON blocks (invalid JSON is a finding)
    # ------------------------------------------------------------------

    def test_negative_invalid_json_detected(self) -> None:
        text = VALID_PLAYBOOK.replace('{"ok": true, "n": 1}', "{not valid json")
        findings, _ = AUDIT.validate(text, VALID_LESSONS, VALID_JUSTFILE)
        self.assertIn("invalid_json", findings_of(findings))

    def test_negative_missing_reference_detected(self) -> None:
        text = VALID_PLAYBOOK.replace(
            "`docs/PROJECT_SPEC_v6_5_1.md:10-20`", "`docs/arquivo-inexistente.md:5`"
        )
        findings, _ = AUDIT.validate(text, VALID_LESSONS, VALID_JUSTFILE)
        self.assertIn("missing_reference", findings_of(findings))

    def test_negative_missing_reference_without_line_suffix_detected(self) -> None:
        text = VALID_PLAYBOOK + "\n`docs/arquivo-inexistente.md`\n"
        findings, _ = AUDIT.validate(text, VALID_LESSONS, VALID_JUSTFILE)
        self.assertIn("missing_reference", findings_of(findings))

    def test_directory_references_are_allowed(self) -> None:
        text = VALID_PLAYBOOK + "\n`docs/adrs/` `.jcode/skills/` `src/gui`\n"
        findings, _ = AUDIT.validate(text, VALID_LESSONS, VALID_JUSTFILE)
        self.assertNotIn("missing_reference", findings_of(findings))

    def test_placeholder_and_wildcard_references_ignored(self) -> None:
        text = (
            VALID_PLAYBOOK
            + "\n`docs/{{PLACEHOLDER}}.md:5` e `docs/arquivo-*.md:1` nao sao checados.\n"
        )
        findings, _ = AUDIT.validate(text, VALID_LESSONS, VALID_JUSTFILE)
        self.assertNotIn("missing_reference", findings_of(findings))

    # ------------------------------------------------------------------
    # Negative: prohibited patterns (root-authorized, scanned on playbook only)
    # ------------------------------------------------------------------

    def test_negative_absolute_host_path_detected(self) -> None:
        text = VALID_PLAYBOOK.replace(
            "Um texto narrativo", "/home/jose/.jcode/scratch/x.md Um texto narrativo"
        )
        findings, _ = AUDIT.validate(text, VALID_LESSONS, VALID_JUSTFILE)
        self.assertIn("absolute_host_path", findings_of(findings))

    def test_negative_validation_task_ref_detected(self) -> None:
        text = VALID_PLAYBOOK.replace(
            "Um texto narrativo", "validation/task-0123 relato de verificacao"
        )
        findings, _ = AUDIT.validate(text, VALID_LESSONS, VALID_JUSTFILE)
        self.assertIn("temporary_round_reference", findings_of(findings))

    def test_negative_hash_40_detected(self) -> None:
        text = VALID_PLAYBOOK.replace(
            "Um texto narrativo",
            "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2 relato",
        )
        findings, _ = AUDIT.validate(text, VALID_LESSONS, VALID_JUSTFILE)
        self.assertIn("hash_40", findings_of(findings))

    def test_negative_hash_64_detected(self) -> None:
        text = VALID_PLAYBOOK.replace(
            "Um texto narrativo",
            "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2 relato",
        )
        findings, _ = AUDIT.validate(text, VALID_LESSONS, VALID_JUSTFILE)
        self.assertIn("hash_64", findings_of(findings))

    def test_negative_truncation_sentinel_detected(self) -> None:
        text = VALID_PLAYBOOK.replace(
            "Um texto narrativo", "resultado ... 42 more lines relato"
        )
        findings, _ = AUDIT.validate(text, VALID_LESSONS, VALID_JUSTFILE)
        self.assertIn("truncation_sentinel", findings_of(findings))

    def test_negative_known_token_detected(self) -> None:
        text = VALID_PLAYBOOK.replace(
            "Um texto narrativo", "O linked ponto foi processado."
        )
        findings, _ = AUDIT.validate(text, VALID_LESSONS, VALID_JUSTFILE)
        self.assertIn("known_token", findings_of(findings))

    def test_negative_codemode_payloads_detected(self) -> None:
        text = VALID_PLAYBOOK.replace(
            "Um texto narrativo", "Leia codemode-payloads.md antes de iniciar."
        )
        findings, _ = AUDIT.validate(text, VALID_LESSONS, VALID_JUSTFILE)
        self.assertIn("codemode_payloads", findings_of(findings))

    def test_negative_bytes_zero_detected(self) -> None:
        text = VALID_PLAYBOOK.replace(
            "Um texto narrativo", '"bytes": 0 no relatorio.'
        )
        findings, _ = AUDIT.validate(text, VALID_LESSONS, VALID_JUSTFILE)
        self.assertIn("bytes_zero", findings_of(findings))

    def test_negative_reserved_phrase_detected(self) -> None:
        text = VALID_PLAYBOOK.replace(
            "Um texto narrativo", "Fox executa, Cricket valida tudo."
        )
        findings, _ = AUDIT.validate(text, VALID_LESSONS, VALID_JUSTFILE)
        self.assertIn("reserved_phrase", findings_of(findings))

    # ------------------------------------------------------------------
    # Non-false-positive: narrative words must not fire prohibited rules
    # ------------------------------------------------------------------

    def test_narrative_rodada_scratch_corrompido_no_finding(self) -> None:
        """Narrative uses rodada/scratch/corrompido; none must fire findings."""
        text = (
            "A rodada de validacao terminou em scratch com um registro "
            "corrompido, mas isso e apenas narrativa do playbook.\n"
        )
        findings, _ = AUDIT.validate(text, VALID_LESSONS, VALID_JUSTFILE)
        self.assertEqual(findings, [])


if __name__ == "__main__":
    unittest.main()
