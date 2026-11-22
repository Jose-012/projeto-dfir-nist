#!/usr/bin/env python3
"""Positive and isolated negative controls for coordination_playbook_gate.py."""

from __future__ import annotations

import os
import re
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


FIXTURE_DIR = Path(__file__).resolve().parent
ROOT = FIXTURE_DIR.parents[2]
LINTER = ROOT / "scripts" / "coordination_playbook_gate.py"
VALID = FIXTURE_DIR / "valid.md"


class CoordinationPlaybookGateTests(unittest.TestCase):
    def run_gate(self, target: Path) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [sys.executable, str(LINTER), str(target)],
            check=False,
            capture_output=True,
            text=True,
        )

    def test_positive_fixture_passes(self) -> None:
        result = self.run_gate(VALID)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("structural checks only", result.stdout)

    def test_negative_mutations_fail_by_expected_rule(self) -> None:
        source = VALID.read_text(encoding="utf-8")

        def remove_section(text: str, cp_id: str) -> str:
            pattern = re.compile(
                rf"(?ms)^## {re.escape(cp_id)}\b.*?(?=^## CP-\d{{2}}\b|\Z)"
            )
            mutated, count = pattern.subn("", text)
            self.assertEqual(count, 1, f"fixture section {cp_id} must exist exactly once")
            return mutated

        def remove_role_section(text: str, role: str) -> str:
            pattern = re.compile(
                rf"(?ms)^### {re.escape(role)}\b[^\n]*\n.*?(?=^### [A-Za-z]+\b|^## CP-\d{{2}}\b|\Z)"
            )
            mutated, count = pattern.subn("", text)
            self.assertEqual(count, 1, f"fixture role {role} must exist exactly once")
            return mutated

        duplicate = (
            "Repeated operational paragraph for an isolated negative control. "
            "It records the same decision, owner, artifact, and verification condition "
            "so the deterministic duplicate detector must reject the mutation."
        )
        cases = (
            (
                "missing_cp_id",
                lambda text: remove_section(text, "CP-15"),
            ),
            ("placeholder_field", lambda text: text + "\n{{UNRESOLVED_FIELD}}\n"),
            ("placeholder_token", lambda text: text + "\nTBD\n"),
            (
                "missing_reference",
                lambda text: text.replace(".jcode/skills/dfir-verify/SKILL.md", ""),
            ),
            (
                "missing_role",
                lambda text: remove_role_section(text, "Fox"),
            ),
            (
                "excessive_duplicate",
                lambda text: text + f"\n{duplicate}\n\n{duplicate}\n\n{duplicate}\n",
            ),
            (
                "duplicate_cp_id",
                lambda text: text.replace(
                    "## CP-15 — State machine\n",
                    "## CP-15 — State machine\n\n## CP-15 — Duplicate state machine\n\nA duplicate section heading must be rejected by the structural gate.\n",
                ),
            ),
            (
                "empty_cp_section",
                lambda text: re.sub(
                    r"(?ms)(^## CP-15\b[^\n]*\n).*?(?=^## CP-16\b)",
                    r"\1",
                    text,
                ),
            ),
        )

        scratch_root = os.environ.get("JCODE_SCRATCH_DIR")
        if not scratch_root:
            self.skipTest("JCODE_SCRATCH_DIR is required for isolated negative controls")

        for expected_rule, mutate in cases:
            with self.subTest(rule=expected_rule):
                mutated = mutate(source)
                self.assertNotEqual(mutated, source)
                with tempfile.TemporaryDirectory(
                    prefix="coordination-playbook-negative-",
                    dir=scratch_root,
                ) as temporary_directory:
                    target = Path(temporary_directory) / "playbook.md"
                    target.write_text(mutated, encoding="utf-8")
                    result = self.run_gate(target)
                self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
                self.assertIn(expected_rule, result.stderr)


if __name__ == "__main__":
    unittest.main()
