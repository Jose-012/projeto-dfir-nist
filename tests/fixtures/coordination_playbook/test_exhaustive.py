#!/usr/bin/env python3
"""Exhaustive deterministic controls for coordination_playbook_gate.py.

These tests call validate() in memory. They do not create, delete, or mutate
checkout documents and do not require third-party packages.
"""

from __future__ import annotations

import importlib.util
import re
import sys
import unittest
from pathlib import Path


FIXTURE_DIR = Path(__file__).resolve().parent
ROOT = FIXTURE_DIR.parents[2]
LINTER_PATH = ROOT / "scripts" / "coordination_playbook_gate.py"
SPEC = importlib.util.spec_from_file_location("coordination_playbook_gate", LINTER_PATH)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError(f"cannot import {LINTER_PATH}")
GATE = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = GATE
SPEC.loader.exec_module(GATE)
VALID = (FIXTURE_DIR / "valid.md").read_text(encoding="utf-8")


def section_span(text: str, cp_id: str) -> tuple[int, int]:
    match = re.search(
        rf"(?ms)^## {re.escape(cp_id)}\b.*?(?=^## CP-\d{{2}}\b|\Z)",
        text,
    )
    if match is None:
        raise AssertionError(f"section not found: {cp_id}")
    return match.start(), match.end()


def rules(text: str) -> set[str]:
    return {diagnostic.rule for diagnostic in GATE.validate(text)}


class ExhaustiveCoordinationPlaybookGateTests(unittest.TestCase):
    def test_positive_fixture_has_no_diagnostics(self) -> None:
        self.assertEqual(GATE.validate(VALID), [])

    def test_every_missing_cp_is_detected(self) -> None:
        for cp_id in GATE.REQUIRED_CP_IDS:
            with self.subTest(cp_id=cp_id):
                start, end = section_span(VALID, cp_id)
                mutated = VALID[:start] + VALID[end:]
                self.assertNotEqual(mutated, VALID)
                diagnostics = GATE.validate(mutated)
                self.assertIn("missing_cp_id", {item.rule for item in diagnostics})
                self.assertTrue(
                    any(cp_id in item.message for item in diagnostics if item.rule == "missing_cp_id")
                )

    def test_every_empty_cp_is_detected(self) -> None:
        for cp_id in GATE.REQUIRED_CP_IDS:
            with self.subTest(cp_id=cp_id):
                start, end = section_span(VALID, cp_id)
                heading = VALID[start:].splitlines()[0]
                mutated = VALID[:start] + heading + "\n\n" + VALID[end:]
                self.assertNotEqual(mutated, VALID)
                diagnostics = GATE.validate(mutated)
                self.assertTrue(
                    any(
                        item.rule == "empty_cp_section" and cp_id in item.message
                        for item in diagnostics
                    )
                )

    def test_every_duplicate_cp_is_detected(self) -> None:
        for cp_id in GATE.REQUIRED_CP_IDS:
            with self.subTest(cp_id=cp_id):
                start, end = section_span(VALID, cp_id)
                section = VALID[start:end].strip()
                mutated = VALID.rstrip() + "\n\n" + section + "\n"
                diagnostics = GATE.validate(mutated)
                self.assertTrue(
                    any(
                        item.rule == "duplicate_cp_id" and cp_id in item.message
                        for item in diagnostics
                    )
                )

    def test_every_required_role_is_detected(self) -> None:
        for role in GATE.REQUIRED_ROLES:
            with self.subTest(role=role):
                heading = f"### {role}"
                self.assertIn(heading, VALID)
                mutated = VALID.replace(heading, "", 1)
                diagnostics = GATE.validate(mutated)
                self.assertTrue(
                    any(
                        item.rule == "missing_role" and role in item.message
                        for item in diagnostics
                    )
                )

    def test_every_required_reference_is_detected(self) -> None:
        for reference in GATE.REQUIRED_REFERENCES:
            with self.subTest(reference=reference):
                self.assertIn(reference, VALID)
                mutated = VALID.replace(reference, "")
                self.assertNotEqual(mutated, VALID)
                diagnostics = GATE.validate(mutated)
                self.assertTrue(
                    any(
                        item.rule == "missing_reference" and reference in item.message
                        for item in diagnostics
                    )
                )

    def test_placeholder_boundaries(self) -> None:
        cases = (
            ("plain_field", "{{UNRESOLVED}}", "placeholder_field"),
            ("fenced_field", "```markdown\n{{UNRESOLVED}}\n```", "placeholder_field"),
            ("todo", "TODO", "placeholder_token"),
            ("tbd", "TBD", "placeholder_token"),
        )
        for label, fragment, expected_rule in cases:
            with self.subTest(case=label):
                self.assertIn(expected_rule, rules(VALID + "\n" + fragment + "\n"))
        for label, fragment in (
            ("inline_code", "`{{UNRESOLVED}}`"),
            ("copyable_label", "<PREENCHER: CAMPO>"),
        ):
            with self.subTest(case=label):
                found = rules(VALID + "\n" + fragment + "\n")
                self.assertNotIn("placeholder_field", found)
                self.assertNotIn("placeholder_token", found)

    def test_excessive_duplicate_requires_three_normalized_paragraphs(self) -> None:
        paragraph = (
            "A deterministic operational paragraph records an owner, an artifact, "
            "a verification command, and an independently checked postcondition."
        )
        self.assertNotIn("excessive_duplicate", rules(VALID + f"\n{paragraph}\n\n{paragraph}\n"))
        self.assertIn(
            "excessive_duplicate",
            rules(VALID + f"\n{paragraph}\n\n{paragraph}\n\n{paragraph}\n"),
        )


if __name__ == "__main__":
    unittest.main()
