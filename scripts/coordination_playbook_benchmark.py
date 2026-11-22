#!/usr/bin/env python3
"""Reproducible scaling benchmark for the coordination playbook gate.

The benchmark is descriptive Engineering/QA tooling. It is not a release gate and
never measures prose quality. Full JSON results are optional; default output is a
compact human summary.
"""
from __future__ import annotations

import argparse
import cProfile
import gc
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import pstats
import statistics
import sys
import time
import tracemalloc

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_GATE = ROOT / "scripts" / "coordination_playbook_gate.py"
DEFAULT_BASE = ROOT / "docs" / "COORDINATION_PLAYBOOK.md"
DEFAULT_SIZES = (131072, 524288, 2097152, 8388608)


def load_gate(path: Path):
    name = f"coordination_playbook_benchmark_gate_{hashlib.sha256(str(path).encode()).hexdigest()[:12]}"
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot import {path}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


def make_corpus(base: str, target_bytes: int) -> str:
    """Extend the final CP section with unique prose, preserving a valid document."""
    if target_bytes <= len(base.encode("utf-8")):
        return base
    paragraphs = []
    current_bytes = len(base.encode("utf-8"))
    index = 0
    while current_bytes < target_bytes:
        paragraph = (
            f"Benchmark paragraph {index:08d} records a distinct owner, artifact, "
            "command, diagnostic and independently observed postcondition for "
            "scaling analysis without repeating normalized prose."
        )
        paragraphs.append(paragraph)
        current_bytes += len(paragraph.encode("utf-8")) + 2
        index += 1
    return base.rstrip() + "\n\n" + "\n\n".join(paragraphs) + "\n"


def percentile(values: list[float], fraction: float) -> float:
    if len(values) == 1:
        return values[0]
    ordered = sorted(values)
    position = (len(ordered) - 1) * fraction
    lower = int(position)
    upper = min(lower + 1, len(ordered) - 1)
    weight = position - lower
    return ordered[lower] * (1 - weight) + ordered[upper] * weight


def summarize_timings(seconds: list[float], size_bytes: int) -> dict[str, float | int]:
    milliseconds = [value * 1000 for value in seconds]
    median_ms = statistics.median(milliseconds)
    return {
        "rounds": len(seconds),
        "min_ms": min(milliseconds),
        "max_ms": max(milliseconds),
        "mean_ms": statistics.mean(milliseconds),
        "median_ms": median_ms,
        "stdev_ms": statistics.stdev(milliseconds) if len(milliseconds) > 1 else 0.0,
        "iqr_ms": percentile(milliseconds, 0.75) - percentile(milliseconds, 0.25),
        "p95_ms": percentile(milliseconds, 0.95),
        "median_mib_per_second": (size_bytes / (1024 * 1024)) / (median_ms / 1000) if median_ms else 0.0,
    }


def profile_validate(gate, text: str) -> list[dict[str, float | int | str]]:
    profiler = cProfile.Profile()
    profiler.enable()
    gate.validate(text)
    profiler.disable()
    stats = pstats.Stats(profiler).sort_stats("cumulative")
    rows = []
    ordered = sorted(stats.stats.items(), key=lambda item: item[1][3], reverse=True)
    for (filename, line, function), (cc, nc, tt, ct, _callers) in ordered[:12]:
        rows.append({
            "function": f"{Path(filename).name}:{line}:{function}",
            "primitive_calls": cc,
            "total_calls": nc,
            "internal_seconds": tt,
            "cumulative_seconds": ct,
        })
    return rows


def compare_results(before: dict, after: dict) -> dict:
    if before.get("base_sha256") != after.get("base_sha256"):
        raise ValueError("benchmark corpus hashes differ")
    before_cases = {case["target_bytes"]: case for case in before["cases"]}
    after_targets = {case["target_bytes"] for case in after["cases"]}
    if set(before_cases) != after_targets:
        raise ValueError("benchmark target sizes differ")
    comparisons = []
    for case in after["cases"]:
        old = before_cases[case["target_bytes"]]
        if old["actual_bytes"] != case["actual_bytes"]:
            raise ValueError(f"benchmark corpus sizes differ at target {case['target_bytes']}")
        old_median = old["timing"]["median_ms"]
        new_median = case["timing"]["median_ms"]
        old_peak = old["memory"]["peak_bytes"]
        new_peak = case["memory"]["peak_bytes"]
        comparisons.append({
            "target_bytes": case["target_bytes"],
            "before_median_ms": old_median,
            "after_median_ms": new_median,
            "median_ratio": new_median / old_median if old_median else None,
            "before_peak_bytes": old_peak,
            "after_peak_bytes": new_peak,
            "peak_ratio": new_peak / old_peak if old_peak else None,
        })
    return {"before_sha256": before.get("gate_sha256"), "after_sha256": after.get("gate_sha256"), "cases": comparisons}


def run_benchmark(gate_path: Path, base_path: Path, sizes: tuple[int, ...], rounds: int, warmup: int) -> dict:
    gate = load_gate(gate_path)
    base = base_path.read_text(encoding="utf-8")
    result = {
        "schema_version": 1,
        "gate": str(gate_path),
        "gate_sha256": hashlib.sha256(gate_path.read_bytes()).hexdigest(),
        "base": str(base_path),
        "base_sha256": hashlib.sha256(base_path.read_bytes()).hexdigest(),
        "python": sys.version,
        "load_average": os.getloadavg(),
        "rounds": rounds,
        "warmup": warmup,
        "cases": [],
    }
    for target_bytes in sizes:
        text = make_corpus(base, target_bytes)
        actual_bytes = len(text.encode("utf-8"))
        for _ in range(warmup):
            diagnostics = gate.validate(text)
        tracemalloc.start()
        diagnostics = gate.validate(text)
        _current, peak = tracemalloc.get_traced_memory()
        tracemalloc.stop()
        timings = []
        gc.disable()
        try:
            for _ in range(rounds):
                started = time.perf_counter_ns()
                diagnostics = gate.validate(text)
                timings.append((time.perf_counter_ns() - started) / 1_000_000_000)
        finally:
            gc.enable()
        case = {
            "target_bytes": target_bytes,
            "actual_bytes": actual_bytes,
            "lines": text.count("\n") + 1,
            "diagnostics": len(diagnostics),
            "timing": summarize_timings(timings, actual_bytes),
            "memory": {"peak_bytes": peak, "measurement": "tracemalloc, one separate validation"},
        }
        result["cases"].append(case)
        print(
            f"{actual_bytes} bytes: median={case['timing']['median_ms']:.3f} ms, "
            f"p95={case['timing']['p95_ms']:.3f} ms, "
            f"throughput={case['timing']['median_mib_per_second']:.3f} MiB/s, "
            f"peak={peak / (1024 * 1024):.3f} MiB, diagnostics={len(diagnostics)}"
        )
    if result["cases"]:
        result["profile_largest"] = profile_validate(gate, make_corpus(base, sizes[-1]))
    return result


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--gate", type=Path, default=DEFAULT_GATE)
    parser.add_argument("--base", type=Path, default=DEFAULT_BASE)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--compare-to", type=Path)
    parser.add_argument("--rounds", type=int, default=7)
    parser.add_argument("--warmup", type=int, default=2)
    parser.add_argument("--sizes", default=",".join(str(value) for value in DEFAULT_SIZES))
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    if args.rounds < 1 or args.warmup < 0:
        print("rounds must be >= 1 and warmup >= 0", file=sys.stderr)
        return 2
    sizes = tuple(int(value) for value in args.sizes.split(",") if value.strip())
    if not sizes or any(value < 1 for value in sizes) or tuple(sorted(sizes)) != sizes:
        print("sizes must be positive and sorted", file=sys.stderr)
        return 2
    gate_path = args.gate.resolve()
    base_path = args.base.resolve()
    result = run_benchmark(gate_path, base_path, sizes, args.rounds, args.warmup)
    if any(case["diagnostics"] for case in result["cases"]):
        print("benchmark corpus produced structural diagnostics", file=sys.stderr)
        return 1
    if args.compare_to:
        before = json.loads(args.compare_to.read_text(encoding="utf-8"))
        try:
            result["comparison"] = compare_results(before, result)
        except ValueError as error:
            print(f"invalid benchmark comparison: {error}", file=sys.stderr)
            return 2
        for case in result["comparison"]["cases"]:
            print(
                f"compare {case['target_bytes']}: median ratio={case['median_ratio']:.3f}, "
                f"peak ratio={case['peak_ratio']:.3f}"
            )
    if args.output:
        output = args.output.expanduser().resolve()
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
        print(f"json_output={output}")
    else:
        print("json_output=disabled (use --output for full results)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
