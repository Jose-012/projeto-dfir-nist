#!/usr/bin/env python3
"""Fast controls for the coordination playbook benchmark harness."""
from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

FIXTURE_DIR = Path(__file__).resolve().parent
ROOT = FIXTURE_DIR.parents[2]
MODULE_PATH = ROOT / "scripts" / "coordination_playbook_benchmark.py"
SPEC = importlib.util.spec_from_file_location("coordination_playbook_benchmark", MODULE_PATH)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError(f"cannot import {MODULE_PATH}")
BENCHMARK = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = BENCHMARK
SPEC.loader.exec_module(BENCHMARK)


def comparison_pair() -> tuple[dict, dict]:
    before = {
        "gate_sha256": "before",
        "base_sha256": "same-corpus",
        "cases": [{"target_bytes": 1, "actual_bytes": 1, "timing": {"median_ms": 2.0}, "memory": {"peak_bytes": 10}}],
    }
    after = {
        "gate_sha256": "after",
        "base_sha256": "same-corpus",
        "cases": [{"target_bytes": 1, "actual_bytes": 1, "timing": {"median_ms": 4.0}, "memory": {"peak_bytes": 20}}],
    }
    return before, after


class CoordinationPlaybookBenchmarkTests(unittest.TestCase):
    def test_corpus_is_valid_and_reaches_target(self) -> None:
        gate = BENCHMARK.load_gate(BENCHMARK.DEFAULT_GATE)
        base = BENCHMARK.DEFAULT_BASE.read_text(encoding="utf-8")
        target = len(base.encode("utf-8")) + 32768
        corpus = BENCHMARK.make_corpus(base, target)
        self.assertGreaterEqual(len(corpus.encode("utf-8")), target)
        self.assertEqual(gate.validate(corpus), [])

    def test_corpus_mutation_is_detected(self) -> None:
        gate = BENCHMARK.load_gate(BENCHMARK.DEFAULT_GATE)
        base = BENCHMARK.DEFAULT_BASE.read_text(encoding="utf-8")
        corpus = BENCHMARK.make_corpus(base, len(base.encode("utf-8")) + 1024)
        mutated = corpus.replace("## CP-00", "", 1)
        self.assertNotEqual(mutated, corpus)
        self.assertIn("missing_cp_id", {item.rule for item in gate.validate(mutated)})

    def test_timing_summary_has_variability_and_throughput(self) -> None:
        summary = BENCHMARK.summarize_timings([0.001, 0.002, 0.003, 0.004, 0.005], 1024 * 1024)
        for key in ("median_ms", "p95_ms", "iqr_ms", "stdev_ms", "median_mib_per_second"):
            self.assertIn(key, summary)
            self.assertGreaterEqual(summary[key], 0)
        self.assertEqual(summary["rounds"], 5)

    def test_comparison_preserves_before_and_after_hash(self) -> None:
        before, after = comparison_pair()
        comparison = BENCHMARK.compare_results(before, after)
        self.assertEqual(comparison["before_sha256"], "before")
        self.assertEqual(comparison["after_sha256"], "after")
        self.assertEqual(comparison["cases"][0]["median_ratio"], 2.0)
        self.assertEqual(comparison["cases"][0]["peak_ratio"], 2.0)

    def test_comparison_rejects_different_corpus_or_cases(self) -> None:
        for mutation in ("base_hash", "actual_bytes", "target_sizes"):
            with self.subTest(mutation=mutation):
                before, after = comparison_pair()
                if mutation == "base_hash":
                    after["base_sha256"] = "different-corpus"
                elif mutation == "actual_bytes":
                    after["cases"][0]["actual_bytes"] = 2
                else:
                    after["cases"][0]["target_bytes"] = 2
                with self.assertRaises(ValueError):
                    BENCHMARK.compare_results(before, after)


if __name__ == "__main__":
    unittest.main()
