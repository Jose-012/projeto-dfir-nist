#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
STORAGE_ROOT="${DFIR_DATA_ROOT:-${ROOT}/storage}"

for dir in db runtime evidence staging sanitized exports quarantine; do
  install -d -m 0700 "${STORAGE_ROOT}/${dir}"
done

printf 'Fase 0 storage initialized at %s\n' "${STORAGE_ROOT}"
printf 'Engine: resolved next to dfir-app (target/{debug,release}/dfir-engine), falling back to /usr/local/bin/dfir-engine; override with DFIR_ENGINE_PATH.\n'
