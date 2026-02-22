#!/usr/bin/env bash
set -euo pipefail

# Fails when core SCULPT files changed but Cargo.toml package version did not change.
# Usage:
#   scripts/ensure_version_bump.sh <base-ref> [head-ref]
# Example:
#   scripts/ensure_version_bump.sh origin/main HEAD

BASE_REF="${1:-}"
HEAD_REF="${2:-HEAD}"

if [[ -z "${BASE_REF}" ]]; then
  echo "Usage: scripts/ensure_version_bump.sh <base-ref> [head-ref]" >&2
  exit 2
fi

if ! git rev-parse --verify "${BASE_REF}" >/dev/null 2>&1; then
  echo "Base ref not found: ${BASE_REF}" >&2
  exit 2
fi

if ! git rev-parse --verify "${HEAD_REF}" >/dev/null 2>&1; then
  echo "Head ref not found: ${HEAD_REF}" >&2
  exit 2
fi

CHANGED_FILES="$(git diff --name-only "${BASE_REF}".."${HEAD_REF}")"
if [[ -z "${CHANGED_FILES}" ]]; then
  echo "No changed files between ${BASE_REF} and ${HEAD_REF}."
  exit 0
fi

NEEDS_BUMP=0
while IFS= read -r file; do
  [[ -z "${file}" ]] && continue
  case "${file}" in
    src/*|tests/*|examples/*|ir-schemas/*|Cargo.lock)
      NEEDS_BUMP=1
      break
      ;;
  esac
done <<< "${CHANGED_FILES}"

if [[ "${NEEDS_BUMP}" -eq 0 ]]; then
  echo "Version bump not required (no core code/test/example/schema changes)."
  exit 0
fi

if ! git diff --name-only "${BASE_REF}".."${HEAD_REF}" | grep -q '^Cargo.toml$'; then
  echo "ERROR: core files changed but Cargo.toml was not changed." >&2
  echo "Please bump [package].version in Cargo.toml before push." >&2
  exit 1
fi

BASE_VERSION="$(git show "${BASE_REF}:Cargo.toml" 2>/dev/null | sed -n 's/^version = \"\\([^\"]*\\)\"/\\1/p' | head -n1)"
HEAD_VERSION="$(sed -n 's/^version = \"\\([^\"]*\\)\"/\\1/p' Cargo.toml | head -n1)"

if [[ -z "${BASE_VERSION}" || -z "${HEAD_VERSION}" ]]; then
  echo "ERROR: could not parse version from Cargo.toml." >&2
  exit 1
fi

if [[ "${BASE_VERSION}" == "${HEAD_VERSION}" ]]; then
  echo "ERROR: core files changed but version stayed ${HEAD_VERSION}." >&2
  echo "Please bump [package].version in Cargo.toml before push." >&2
  exit 1
fi

echo "Version bump check passed: ${BASE_VERSION} -> ${HEAD_VERSION}"
