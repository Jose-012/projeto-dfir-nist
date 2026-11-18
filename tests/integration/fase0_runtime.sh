#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
APP="${ROOT}/target/release/dfir-app"

test -x "${APP}" || { echo "build target missing: ${APP}" >&2; exit 3; }

TMP="$(mktemp -d)"
trap 'rm -rf "${TMP}"' EXIT

export DFIR_DATA_ROOT="${TMP}/storage"
export DFIR_ENGINE_PATH="${APP/dfir-app/dfir-engine}"

mkdir -p "${DFIR_DATA_ROOT}/db" "${DFIR_DATA_ROOT}/evidence" "${DFIR_DATA_ROOT}/staging" "${DFIR_DATA_ROOT}/sanitized" "${DFIR_DATA_ROOT}/exports" "${DFIR_DATA_ROOT}/quarantine"

"${APP}" engine-smoke >/dev/null

