#!/usr/bin/env python3
"""Integration QA for the project RTK filters using only sanitized fake commands."""

from __future__ import annotations

import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]
PASS_LINE = "        PASS [   0.001s] sanitized::alpha"
SUMMARY = "    Summary [   0.003s] 2 tests run: 2 passed, 0 failed"
STDERR_SENTINEL = "warning: sanitized stderr FIXTURE_TOKEN_LIKE_VALUE"

FAKE_COMMAND = r"""#!/bin/sh
if [ "${FAKE_RTK_MODE:-success}" = failure ]; then
    printf '%s\n' '        PASS [   0.001s] sanitized::alpha'
    printf '%s\n' '        FAIL [   0.002s] sanitized::injected_failure'
    printf '%s\n' '    Summary [   0.003s] 2 tests run: 1 passed, 1 failed'
    printf '%s\n' 'error: sanitized stdout FIXTURE_TOKEN_LIKE_VALUE'
    printf '%s\n' 'error: sanitized stderr failure FIXTURE_TOKEN_LIKE_VALUE' >&2
    exit 23
fi
printf '%s\n' '        PASS [   0.001s] sanitized::alpha'
printf '%s\n' '        PASS [   0.002s] sanitized::beta'
printf '%s\n' '    Summary [   0.003s] 2 tests run: 2 passed, 0 failed'
printf '%s\n' 'RUSTSEC-0000-0001: sanitized advisory'
printf '%s\n' '[full output: rtk recall 0123456789abcdef]'
printf '%s\n' 'warning: sanitized stderr FIXTURE_TOKEN_LIKE_VALUE' >&2
exit 0
"""


def run(command: list[str], env: dict[str, str], cwd: Path = ROOT) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(command, cwd=cwd, env=env, text=True, capture_output=True, check=False)
    print(f"command={command!r} exit={result.returncode}")
    return result


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def main() -> int:
    scratch_value = os.environ.get("JCODE_SCRATCH_DIR")
    rtk = shutil.which("rtk")
    require(scratch_value is not None, "JCODE_SCRATCH_DIR is required; /tmp is not used")
    require(rtk is not None, "rtk is required")
    scratch = Path(scratch_value).resolve()
    scratch.mkdir(parents=True, exist_ok=True)

    with tempfile.TemporaryDirectory(prefix="qa-rtk-filters-", dir=scratch) as temporary:
        work = Path(temporary)
        home = work / "home"
        config = work / "config"
        data = work / "data"
        fake_bin = work / "bin"
        for directory in (home, config, data, fake_bin):
            directory.mkdir()
        for name in ("just",):
            executable = fake_bin / name
            executable.write_text(FAKE_COMMAND, encoding="utf-8")
            executable.chmod(0o700)

        env = os.environ.copy()
        env.update(
            HOME=str(home),
            XDG_CONFIG_HOME=str(config),
            XDG_DATA_HOME=str(data),
            RTK_RECALL="0",
            RTK_TELEMETRY_DISABLED="1",
            PATH=f"{fake_bin}{os.pathsep}{env['PATH']}",
        )
        env.pop("FAKE_RTK_MODE", None)

        trusted = run([rtk, "trust", "--yes"], env)
        require(trusted.returncode == 0, f"isolated trust failed: {trusted.stderr}")
        verified = run([rtk, "verify", "--require-all"], env)
        require(verified.returncode == 0, f"inline verification failed: {verified.stdout}{verified.stderr}")

        for tool, arguments in (("just", ["verify"]),):
            direct = run([str(fake_bin / tool), *arguments], env)
            filtered = run([rtk, tool, *arguments], env)
            require(direct.returncode == filtered.returncode == 0, f"{tool}: success exit changed")
            require(PASS_LINE not in filtered.stdout, f"{tool}: successful nextest line was not filtered")
            require(SUMMARY in filtered.stdout, f"{tool}: summary lost; stdout={filtered.stdout!r}; stderr={filtered.stderr!r}")
            require("RUSTSEC-0000-0001" in filtered.stdout, f"{tool}: advisory lost")
            require("rtk recall 0123456789abcdef" in filtered.stdout, f"{tool}: recovery hint lost")
            require(filtered.stderr == direct.stderr == STDERR_SENTINEL + "\n", f"{tool}: stderr changed")

            recovered = run([rtk, "proxy", tool, *arguments], env)
            require(recovered.returncode == 0, f"{tool}: proxy exit changed")
            require(recovered.stdout == direct.stdout, f"{tool}: proxy did not recover raw stdout")
            require(recovered.stderr == direct.stderr, f"{tool}: proxy changed stderr")
            print(f"{tool}: sanitized bytes {len(direct.stdout)} -> {len(filtered.stdout)}")

            failure_env = env.copy()
            failure_env["FAKE_RTK_MODE"] = "failure"
            direct_failure = run([str(fake_bin / tool), *arguments], failure_env)
            filtered_failure = run([rtk, tool, *arguments], failure_env)
            require(direct_failure.returncode == filtered_failure.returncode == 23, f"{tool}: failure exit changed")
            require("FAIL [" in filtered_failure.stdout, f"{tool}: injected failure lost")
            require("1 passed, 1 failed" in filtered_failure.stdout, f"{tool}: failure count lost")
            require("stdout FIXTURE_TOKEN_LIKE_VALUE" in filtered_failure.stdout, f"{tool}: token-like diagnostic lost")
            require(filtered_failure.stderr == direct_failure.stderr, f"{tool}: failure stderr changed")

        # A near command must not activate the nextest-specific local filter.
        non_match = run([rtk, "just", "supply-chain"], env)
        require(non_match.returncode == 0, "non-matching command failed")
        require(PASS_LINE in non_match.stdout, "non-matching command was filtered")

        recalled = run([rtk, "recall", "--list"], env)
        require(recalled.returncode == 0, f"recall listing failed: {recalled.stderr}")
        recalled_text = (recalled.stdout + recalled.stderr).lower()
        require("no" in recalled_text and ("entr" in recalled_text or "record" in recalled_text), "recall disabled still retained entries")
        require("fixture_token_like_value" not in recalled_text, "sanitized command output entered recall")

    print("RTK filter QA: PASS (inline, positive, negative, stderr, exits, proxy and recall-disabled)")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (AssertionError, OSError) as error:
        print(f"RTK filter QA: FAIL: {error}", file=sys.stderr)
        sys.exit(1)
