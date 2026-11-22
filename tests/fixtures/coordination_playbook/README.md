# Coordination Playbook gate fixtures

These files are synthetic structural fixtures for `scripts/coordination_playbook_gate.py`.

- `valid.md` is the positive fixture.
- `test_gate.py` copies the positive fixture into `$JCODE_SCRATCH_DIR`, mutates exactly one condition per case, and confirms the expected non-zero exit and rule name.
- `test_exhaustive.py` runs in memory with no temporary files. It covers every CP section missing, empty, and duplicated (87 cases), every required role and reference, placeholder boundaries, and duplicate thresholds.
- The checkout document is never mutated to induce a negative result.

Run both suites with caches and temporary controls outside the checkout:

```bash
JCODE_SCRATCH_DIR=/absolute/scratch \
PYTHONPYCACHEPREFIX=/absolute/pycache \
python3 -m unittest \
  tests/fixtures/coordination_playbook/test_gate.py \
  tests/fixtures/coordination_playbook/test_exhaustive.py -v
```

`just test-coordination-playbook` runs only the exhaustive in-memory suite and needs no scratch directory. `just doc-gate` runs that suite after the canonical structural check.

The gate checks deterministic structure only. It does not measure prose quality or grant semantic or release acceptance.
