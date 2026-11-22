# Coordination Playbook gate fixtures

These synthetic fixtures cover the coordination playbook linter, repository audit and benchmark harness.

- `valid.md` is the positive structural fixture.
- `test_gate.py` copies that fixture into `$JCODE_SCRATCH_DIR`, mutates exactly one condition per case and confirms the expected non-zero exit and rule name.
- `test_exhaustive.py` runs in memory. It covers every CP section missing, empty and duplicated (87 cases), every required role and reference, placeholder boundaries and duplicate thresholds.
- `test_adversarial.py` runs deterministic in-memory controls for CommonMark fences, CRLF, Unicode, module registration, placeholders and duplicate normalization.
- `test_repo_audit.py` validates JSON blocks, repository references, lesson sequencing and the exact `just verify` recipe, including rejection of additional `just` commands, with positive, negative and false-positive controls.
- `test_benchmark.py` checks valid scaling corpus generation, a negative mutation, timing statistics and before/after comparisons that require identical corpus hashes, actual sizes and target cases.
- The checkout document is never mutated to induce a negative result.

Run the complete suite with temporary controls and caches outside the checkout:

```bash
JCODE_SCRATCH_DIR=/absolute/scratch \
PYTHONPYCACHEPREFIX=/absolute/pycache \
python3 -m unittest \
  tests/fixtures/coordination_playbook/test_gate.py \
  tests/fixtures/coordination_playbook/test_exhaustive.py \
  tests/fixtures/coordination_playbook/test_adversarial.py \
  tests/fixtures/coordination_playbook/test_repo_audit.py \
  tests/fixtures/coordination_playbook/test_benchmark.py -v
```

`just test-coordination-playbook` runs all in-memory suites and needs no scratch directory. `just doc-gate` runs those suites after the structural and repository audits. `just test-coordination-playbook-all` additionally runs the two subprocess controls and requires `JCODE_SCRATCH_DIR` to name a dedicated directory.

Run the descriptive scaling benchmark separately:

```bash
PYTHONPYCACHEPREFIX=/absolute/pycache \
python3 scripts/coordination_playbook_benchmark.py \
  --rounds 7 \
  --output /absolute/benchmarks/current.json
```

For before/after analysis, pass a baseline linter with `--gate /absolute/baseline_gate.py` and compare the later run with `--compare-to /absolute/benchmarks/baseline.json`. Benchmarks are not part of `doc-gate` and are not universal performance or prose-quality targets.

The gates check deterministic structure only. They do not measure prose quality or grant semantic or release acceptance.
