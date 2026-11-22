#!/usr/bin/env python3
"""Adversarial standard-library controls for coordination_playbook_gate.py.

These tests call ``validate()`` in memory. They do not create, delete, or mutate
checkout documents, do not touch ``$JCODE_SCRATCH_DIR``, and do not require
third-party packages. Every negative mutation proves it altered the input and
fails by the expected rule.

Scope covered:
- CP / role headings inside fenced code blocks (``` and ~~~): structural heading
  detection ignores literal code while preserving offsets and line numbers.
- Placeholders in fenced code versus prose; inline code protection.
- CRLF line endings.
- Unicode content and headings.
- Multiple duplicates and the ``excessive_duplicate`` threshold (limit 2).
- Empty CP sections whose only content is a fence.
- TODO / TBD word boundaries.
- Normalized duplicate detection across whitespace/case differences.

The linter is never edited. Observed behaviour that looks questionable is kept
separate from any expectation not yet authorised by the project.
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

# Synthetic base containing every required CP section and every required role
# heading, with enough non-code prose to avoid triggering rules incidentally.
BASE = """# Coordination Playbook adversarial base

## CP-00 — Purpose
Synthetic base for adversarial deterministic controls of the coordination playbook gate.

## CP-01 — Authority
This base references AGENTS.md, CONVENTIONS.md, docs/DOCUMENT_AUTHORITY.md, docs/SESSION_IMPROVEMENT.md, docs/SESSION_LEARNINGS.md, .jcode/swarm-prompt.md, .jcode/skills/dfir-improve/SKILL.md, .jcode/skills/dfir-change/SKILL.md, .jcode/skills/dfir-verify/SKILL.md, .jcode/skills/dfir-detect/SKILL.md, docs/RELEASE_GATE.md, docs/EVIDENCE_REGISTER.md.

## CP-02 — Roles
### Cricket — root coordinator
Cricket retains authority, conflict resolution, risk decisions, and final acceptance.
### Fox — editorial coordinator
Fox coordinates independent lanes and integrates drafts without granting release acceptance.
### Lobster — visual specialist
Lobster grounds visual and documentary findings with controlled synthetic evidence.
### Ox — tooling specialist
Ox performs narrow edits, deterministic harnesses, and structural tooling checks.
### Rat — artifact specialist
Rat normalizes artifact metadata and preserves links to the original validation logs.

## CP-03 — States
Work advances through planned, implemented, integrated, verified, and accepted states with explicit evidence.

## CP-04 — Task selection
Only tasks with an owner, observable postconditions, an explicit budget, and independent scope are started.

## CP-05 — Context budget
Setup, delegation, execution, and integration each have numeric time and request limits for the root coordinator.

## CP-06 — Tool decision tree
Direct reads, code search, independent batches, stateful orchestration, and real commands each have distinct routing rules.

## CP-07 — Skills
Local skills are selected by trigger and re-read from disk after any local edit.

## CP-08 — Mission contract
The mission contract records authority, scope, exclusions, postconditions, budget, gates, and external authorization state.

## CP-09 — Lane contract
Each lane names one owner, exclusive write boundary, dependency, budget, observable acceptance, and one start mechanism.

## CP-10 — Cricket workflow
The root workflow separates difficult decisions, delegation, integration, evidence review, and final acceptance.

## CP-11 — Fox workflow
The editorial workflow distributes work, tracks complete identifiers, reviews final artifacts, and escalates conflicts.

## CP-12 — Lobster workflow
The visual workflow uses one image at a time, grounded textual references, and explicit uncertainty states.

## CP-13 — Ox workflow
The narrow-edit workflow freezes a baseline hash, applies the smallest edit, and compares hashes and diffs at every iteration.

## CP-14 — Rat workflow
The artifact workflow separates normalization from execution and links every summary to an absolute source log.

## CP-15 — State machine
Each handoff names its producer, consumer, artifact, and observable acceptance condition.

## CP-16 — Risk classes
Unsafe code, concurrency, lifetimes, macros, public contracts, behavior, evidence data, and external actions have distinct controls.

## CP-17 — Visual workflow
Visual review records one controlled image per step and never turns a rejected provider response into a negative finding.

## CP-18 — Rust workflow
Rust gates run sequentially under one build owner and preserve warnings observed before a correction.

## CP-19 — Documentation workflow
Document ownership and authority prevent duplicate claims and preserve the canonical source for each domain.

## CP-20 — Artifact contract
Before and after records, logs, reports, task cards, and paths use absolute and reproducible identifiers.

## CP-21 — Gates
Each changed artifact selects applicable gates, proves negative controls, invalidates stale evidence, and binds results to a tree.

## CP-22 — Recovery
Recovery runbooks cover wrong directory, invalid schema, empty search, silent build, divergent report, delayed write, and cancellation.

## CP-23 — Integration
Integration reviews semantic diffs, checks whitespace, confirms processes are closed, and separates evidence from acceptance.

## CP-24 — Metrics
Metrics describe elapsed time, request count, first-pass outcome, rework, and observed lessons without scoring prose quality by lines.

## CP-25 — Checklists
Each role has an actionable checklist with observable criteria for setup, execution, integration, verification, and closure.

## CP-26 — Templates
Copied mission, lane, report, decision, task-card, and handoff templates contain every required field without unresolved fields.

## CP-27 — Error matrix
Each observed error maps to a prevention, an executable control, and a localizable evidence reference.

## CP-28 — Definition of done
Completion states distinguish lane delivery, integration, and release acceptance and keep unresolved work explicit.
"""

REQUIRED_CP_IDS = tuple(f"CP-{index:02d}" for index in range(29))
REQUIRED_ROLES = ("Cricket", "Fox", "Lobster", "Ox", "Rat")


def validate(text: str) -> list:
    return GATE.validate(text)


def rules(text: str) -> set[str]:
    return {diagnostic.rule for diagnostic in validate(text)}


def assert_no_diagnostics(text: str) -> None:
    diagnostics = validate(text)
    if diagnostics:
        raise AssertionError(
            "expected no diagnostics, got: "
            + ", ".join(f"{d.line}:{d.rule}:{d.message}" for d in diagnostics)
        )


class AdversarialCoordinationPlaybookGateTests(unittest.TestCase):
    """Deterministic, in-memory adversarial controls."""

    def test_module_registered_in_sys_modules(self) -> None:
        """The linter module is importable and registered in sys.modules.

        Note: test_exhaustive.py registers the same module name in the same
        process, so identity (assertIs) is not stable across suites. We assert
        registration and the presence of validate(), which is the meaningful
        contract.
        """
        self.assertIn("coordination_playbook_gate", sys.modules)
        registered = sys.modules["coordination_playbook_gate"]
        self.assertTrue(hasattr(registered, "validate"))

    def test_positive_base_has_no_diagnostics(self) -> None:
        assert_no_diagnostics(BASE)

    def test_every_required_cp_present_in_base(self) -> None:
        for cp_id in REQUIRED_CP_IDS:
            with self.subTest(cp_id=cp_id):
                self.assertIn(cp_id, BASE, f"base must contain {cp_id}")

    def test_every_required_role_present_in_base(self) -> None:
        for role in REQUIRED_ROLES:
            with self.subTest(role=role):
                self.assertIn(f"### {role}", BASE, f"base must contain role heading {role}")

    # ------------------------------------------------------------------
    # CP / role headings inside fenced code blocks
    # ------------------------------------------------------------------
    def test_cp_heading_inside_triple_backtick_fence_is_ignored(self) -> None:
        mutated = (
            BASE
            + "\n\n```\n## CP-15 — hidden inside backtick fence\nthis should not count\n```\n"
        )
        self.assertNotEqual(mutated, BASE)
        assert_no_diagnostics(mutated)

    def test_cp_heading_inside_tilde_fence_is_ignored(self) -> None:
        mutated = BASE + "\n\n~~~\n## CP-15 — hidden inside tilde fence\n~~~\n"
        self.assertNotEqual(mutated, BASE)
        assert_no_diagnostics(mutated)

    def test_long_fence_requires_an_equally_long_closing_fence(self) -> None:
        mutated = BASE + "\n\n````\n## CP-15 hidden\n```\n## CP-16 hidden\n````\n"
        self.assertNotEqual(mutated, BASE)
        assert_no_diagnostics(mutated)

    def test_unclosed_fence_is_masked_to_end_of_document(self) -> None:
        mutated = BASE + "\n\n~~~\n## CP-15 hidden at EOF\n"
        self.assertNotEqual(mutated, BASE)
        assert_no_diagnostics(mutated)

    def test_role_heading_inside_triple_backtick_fence_does_not_satisfy_role(self) -> None:
        mutated = BASE.replace("### Fox — editorial coordinator\n", "", 1)
        mutated += "\n\n```\n### Fox — hidden inside backtick fence\n```\n"
        self.assertNotEqual(mutated, BASE)
        self.assertTrue(
            any(
                item.rule == "missing_role" and "Fox" in item.message
                for item in validate(mutated)
            )
        )

    # ------------------------------------------------------------------
    # Placeholders: fenced code versus prose
    # ------------------------------------------------------------------
    def test_placeholder_inline_code_is_protected(self) -> None:
        mutated = BASE + "\nThe field `{{UNRESOLVED_FIELD}}` is intentionally inline.\n"
        self.assertNotEqual(mutated, BASE)
        self.assertNotIn("placeholder_field", rules(mutated))

    def test_placeholder_in_prose_is_detected(self) -> None:
        mutated = BASE + "\nThe field {{UNRESOLVED_FIELD}} remains unresolved.\n"
        self.assertNotEqual(mutated, BASE)
        self.assertIn("placeholder_field", rules(mutated))

    def test_placeholder_inside_triple_backtick_fence_is_detected(self) -> None:
        """Placeholder scanning is intentionally global, including code fences."""
        mutated = BASE + "\n```\n{{UNRESOLVED_FIELD}}\n```\n"
        self.assertNotEqual(mutated, BASE)
        self.assertIn("placeholder_field", rules(mutated))

    def test_placeholder_inside_tilde_fence_is_detected(self) -> None:
        mutated = BASE + "\n~~~\n{{UNRESOLVED_FIELD}}\n~~~\n"
        self.assertNotEqual(mutated, BASE)
        self.assertIn("placeholder_field", rules(mutated))

    # ------------------------------------------------------------------
    # CRLF line endings
    # ------------------------------------------------------------------
    def test_crlf_base_passes_without_diagnostics(self) -> None:
        mutated = BASE.replace("\n", "\r\n")
        self.assertNotEqual(mutated, BASE)
        assert_no_diagnostics(mutated)

    def test_crlf_cp_heading_is_still_detected(self) -> None:
        mutated = BASE.replace("\n", "\r\n")
        mutated = mutated.replace("## CP-15 — State machine\r\n", "", 1)
        self.assertNotEqual(mutated, BASE)
        diagnostics = validate(mutated)
        self.assertTrue(
            any(item.rule == "missing_cp_id" and "CP-15" in item.message for item in diagnostics),
            f"expected missing_cp_id for CP-15 under CRLF, got: {diagnostics}",
        )

    def test_fence_mask_preserves_offsets_and_crlf(self) -> None:
        text = (
            "prefix\r\n````json\r\n## CP-99 hidden\r\n```\r\n"
            "### Fox hidden\r\n````\r\nsuffix\n"
        )
        masked = GATE._mask_fenced_blocks(text)
        self.assertEqual(len(masked), len(text))
        self.assertEqual(masked.count("\n"), text.count("\n"))
        self.assertEqual(masked.count("\r"), text.count("\r"))
        self.assertNotIn("CP-99", masked)
        self.assertNotIn("Fox", masked)
        self.assertIn("prefix", masked)
        self.assertIn("suffix", masked)

    # ------------------------------------------------------------------
    # Unicode
    # ------------------------------------------------------------------
    def test_unicode_placeholder_is_detected(self) -> None:
        mutated = BASE + "\nO campo {{CAMPO_PENDENTE_ÇÃO}} não foi resolvido.\n"
        self.assertNotEqual(mutated, BASE)
        self.assertIn("placeholder_field", rules(mutated))

    def test_unicode_cp_heading_is_counted(self) -> None:
        mutated = BASE.replace(
            "## CP-28 — Definition of done",
            "## CP-28 — Definição de conclusão",
            1,
        )
        self.assertNotEqual(mutated, BASE)
        # The CP id is unchanged, so no missing_cp_id for CP-28; but the original
        # English heading text is gone, which is not a rule on its own.
        self.assertNotIn("missing_cp_id", rules(mutated))

    def test_unicode_prose_does_not_crash(self) -> None:
        mutated = BASE + "\nExemplos: café, naïve, 東京, 🚀, αβγ.\n"
        self.assertNotEqual(mutated, BASE)
        assert_no_diagnostics(mutated)

    # ------------------------------------------------------------------
    # Multiple duplicates and the excessive_duplicate threshold
    # ------------------------------------------------------------------
    def test_two_identical_paragraphs_are_allowed(self) -> None:
        paragraph = (
            "A deterministic operational paragraph records an owner, an artifact, "
            "a verification command, and an independently checked postcondition."
        )
        mutated = BASE + f"\n{paragraph}\n\n{paragraph}\n"
        self.assertNotEqual(mutated, BASE)
        self.assertNotIn("excessive_duplicate", rules(mutated))

    def test_three_identical_paragraphs_are_rejected(self) -> None:
        paragraph = (
            "A deterministic operational paragraph records an owner, an artifact, "
            "a verification command, and an independently checked postcondition."
        )
        mutated = BASE + f"\n{paragraph}\n\n{paragraph}\n\n{paragraph}\n"
        self.assertNotEqual(mutated, BASE)
        self.assertIn("excessive_duplicate", rules(mutated))

    def test_four_identical_paragraphs_are_rejected(self) -> None:
        paragraph = (
            "A deterministic operational paragraph records an owner, an artifact, "
            "a verification command, and an independently checked postcondition."
        )
        mutated = BASE + f"\n{paragraph}\n\n{paragraph}\n\n{paragraph}\n\n{paragraph}\n"
        self.assertNotEqual(mutated, BASE)
        diagnostics = [d for d in validate(mutated) if d.rule == "excessive_duplicate"]
        self.assertTrue(diagnostics, "expected at least one excessive_duplicate diagnostic")

    def test_normalized_duplicate_across_whitespace_and_case_is_rejected(self) -> None:
        """Normalisation collapses whitespace and casefolds, so these are one
        normalized paragraph occurring three times. Each variant is long enough
        (>= 80 chars) to pass the paragraph-length gate."""
        variants = (
            "A deterministic operational paragraph records an owner, an artifact, a verification command.",
            "A Deterministic   operational   paragraph records an owner, an artifact, a verification command.",
            "A  deterministic  operational  paragraph  records  an  owner,  an  artifact,  a  verification  command.",
        )
        mutated = BASE + "\n" + "\n\n".join(variants) + "\n"
        self.assertNotEqual(mutated, BASE)
        self.assertIn("excessive_duplicate", rules(mutated))

    # ------------------------------------------------------------------
    # Empty CP sections whose only content is a fence
    # ------------------------------------------------------------------
    def test_cp_section_with_only_a_backtick_fence_is_not_flagged_empty(self) -> None:
        """The linter treats the fence delimiter line itself as meaningful
        content (it does not start with #, -, *, +, | or a numbered list), so
        a CP section containing only a ``` block is not reported as empty.

        This is observed linter behaviour; whether an empty-fence section
        should be rejected is a separate governance decision, not a test
        expectation introduced here.
        """
        mutated = re.sub(
            r"(?ms)(^## CP-15\b[^\n]*\n).*?(?=^## CP-16\b)",
            r"\1\n\n```\ncode only\n```\n",
            BASE,
            count=1,
        )
        self.assertNotEqual(mutated, BASE)
        diagnostics = [d for d in validate(mutated) if d.rule == "empty_cp_section" and "CP-15" in d.message]
        self.assertEqual(
            diagnostics,
            [],
            f"expected CP-15 with a fence to NOT be flagged empty, got: {diagnostics}",
        )

    # ------------------------------------------------------------------
    # TODO / TBD word boundaries
    # ------------------------------------------------------------------
    def test_todo_standalone_is_detected(self) -> None:
        mutated = BASE + "\nThis is a TODO item.\n"
        self.assertNotEqual(mutated, BASE)
        self.assertIn("placeholder_token", rules(mutated))

    def test_tbd_standalone_is_detected(self) -> None:
        mutated = BASE + "\nThis is a TBD item.\n"
        self.assertNotEqual(mutated, BASE)
        self.assertIn("placeholder_token", rules(mutated))

    def test_todo_inside_identifier_is_not_detected(self) -> None:
        mutated = BASE + "\nThe TODOlist file tracks pending work.\n"
        self.assertNotEqual(mutated, BASE)
        self.assertNotIn("placeholder_token", rules(mutated))

    def test_tbd_inside_identifier_is_not_detected(self) -> None:
        mutated = BASE + "\nThe TBDx flag is reserved.\n"
        self.assertNotEqual(mutated, BASE)
        self.assertNotIn("placeholder_token", rules(mutated))

    def test_prefixed_todo_is_not_detected(self) -> None:
        mutated = BASE + "\nThe XTODO marker is reserved.\n"
        self.assertNotEqual(mutated, BASE)
        self.assertNotIn("placeholder_token", rules(mutated))

    # ------------------------------------------------------------------
    # Every CP removal produces missing_cp_id (negative controls)
    # ------------------------------------------------------------------
    def test_every_missing_cp_is_detected(self) -> None:
        for cp_id in REQUIRED_CP_IDS:
            with self.subTest(cp_id=cp_id):
                pattern = re.compile(
                    rf"(?ms)^## {re.escape(cp_id)}\b[^\n]*\n.*?(?=^## CP-\d{{2}}\b|\Z)"
                )
                mutated, count = pattern.subn("", BASE)
                self.assertEqual(count, 1, f"section {cp_id} must exist exactly once in base")
                self.assertNotEqual(mutated, BASE)
                diagnostics = validate(mutated)
                self.assertTrue(
                    any(item.rule == "missing_cp_id" and cp_id in item.message for item in diagnostics),
                    f"expected missing_cp_id for {cp_id}, got: {diagnostics}",
                )

    def test_every_duplicate_cp_is_detected(self) -> None:
        for cp_id in REQUIRED_CP_IDS:
            with self.subTest(cp_id=cp_id):
                pattern = re.compile(
                    rf"(?ms)^## {re.escape(cp_id)}\b[^\n]*\n.*?(?=^## CP-\d{{2}}\b|\Z)"
                )
                match = pattern.search(BASE)
                self.assertIsNotNone(match, f"section {cp_id} must exist in base")
                section = match.group(0)
                mutated = BASE.rstrip() + "\n\n" + section + "\n"
                self.assertNotEqual(mutated, BASE)
                diagnostics = validate(mutated)
                self.assertTrue(
                    any(item.rule == "duplicate_cp_id" and cp_id in item.message for item in diagnostics),
                    f"expected duplicate_cp_id for {cp_id}, got: {diagnostics}",
                )

    def test_every_empty_cp_is_detected(self) -> None:
        for cp_id in REQUIRED_CP_IDS:
            with self.subTest(cp_id=cp_id):
                pattern = re.compile(
                    rf"(?ms)^## {re.escape(cp_id)}\b[^\n]*\n.*?(?=^## CP-\d{{2}}\b|\Z)"
                )
                match = pattern.search(BASE)
                self.assertIsNotNone(match, f"section {cp_id} must exist in base")
                heading = match.group(0).splitlines()[0]
                mutated = BASE[: match.start()] + heading + "\n\n" + BASE[match.end():]
                self.assertNotEqual(mutated, BASE)
                diagnostics = validate(mutated)
                self.assertTrue(
                    any(
                        item.rule == "empty_cp_section" and cp_id in item.message
                        for item in diagnostics
                    ),
                    f"expected empty_cp_section for {cp_id}, got: {diagnostics}",
                )

    def test_placeholder_token_at_line_start_and_end(self) -> None:
        for token in ("TODO", "TBD"):
            with self.subTest(token=token):
                mutated = BASE + f"\n{token}\n"
                self.assertNotEqual(mutated, BASE)
                self.assertIn("placeholder_token", rules(mutated))


if __name__ == "__main__":
    unittest.main()
