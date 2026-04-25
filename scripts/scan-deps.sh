#!/usr/bin/env bash
# Run the full dependency-vulnerability scan suite locally.
#
# Mirrors .github/workflows/dependency-scan.yml so contributors can validate
# changes before pushing. Each scanner is run independently — a missing tool
# is reported and skipped rather than aborting the whole run.
#
# Usage:
#   ./scripts/scan-deps.sh                # run all scanners
#   ./scripts/scan-deps.sh rust           # run only the rust scanners
#   ./scripts/scan-deps.sh npm python     # run multiple targeted scanners
#   STRICT=1 ./scripts/scan-deps.sh       # exit non-zero if any scanner fails
#                                         # (default is to report and continue)

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

STRICT="${STRICT:-0}"
TARGETS=("$@")
if [ ${#TARGETS[@]} -eq 0 ]; then
  TARGETS=(rust npm python docker osv)
fi

color() {
  local c="$1"; shift
  if [ -t 1 ]; then
    case "$c" in
      red)    printf '\033[31m%s\033[0m' "$*" ;;
      green)  printf '\033[32m%s\033[0m' "$*" ;;
      yellow) printf '\033[33m%s\033[0m' "$*" ;;
      blue)   printf '\033[34m%s\033[0m' "$*" ;;
      *) printf '%s' "$*" ;;
    esac
  else
    printf '%s' "$*"
  fi
}

declare -a RESULTS

record() {
  RESULTS+=("$1|$2|$3")
}

want() {
  for t in "${TARGETS[@]}"; do
    if [ "$t" = "$1" ]; then return 0; fi
  done
  return 1
}

run_scanner() {
  local label="$1"; shift
  local cmd_check="$1"; shift
  echo
  color blue "==> $label"; echo
  if ! command -v "$cmd_check" >/dev/null 2>&1; then
    color yellow "    skipped: '$cmd_check' not installed"; echo
    record "$label" "skip" "$cmd_check not installed"
    return
  fi
  if "$@"; then
    color green "    OK"; echo
    record "$label" "pass" ""
  else
    color red "    FAILED"; echo
    record "$label" "fail" ""
  fi
}

# ---------------------------------------------------------------------------
# Rust workspaces — cargo-audit + cargo-deny
# ---------------------------------------------------------------------------
if want rust; then
  for ws in backend smart-contract sdk/rust; do
    if [ -f "$ws/Cargo.toml" ]; then
      run_scanner "cargo-audit ($ws)" cargo-audit \
        bash -c "cd '$ws' && cargo-audit audit --deny warnings"
      run_scanner "cargo-deny ($ws)" cargo-deny \
        cargo-deny --manifest-path "$ws/Cargo.toml" --config "$REPO_ROOT/deny.toml" check
    fi
  done
fi

# ---------------------------------------------------------------------------
# Node packages — npm audit
# ---------------------------------------------------------------------------
if want npm; then
  for pkg in frontend .; do
    if [ -f "$pkg/package-lock.json" ]; then
      run_scanner "npm audit ($pkg)" npm \
        bash -c "cd '$pkg' && npm audit --audit-level=high"
    fi
  done
fi

# ---------------------------------------------------------------------------
# Python SDK — pip-audit
# ---------------------------------------------------------------------------
if want python; then
  if [ -f "sdk/python/pyproject.toml" ]; then
    run_scanner "pip-audit (sdk/python)" pip-audit \
      bash -c "cd sdk/python && pip-audit --strict --vulnerability-service osv ."
  fi
fi

# ---------------------------------------------------------------------------
# Container scan — Trivy on the backend image config + filesystem
# ---------------------------------------------------------------------------
if want docker; then
  if [ -f "backend/Dockerfile" ]; then
    run_scanner "trivy config (backend/Dockerfile)" trivy \
      trivy config --severity CRITICAL,HIGH --exit-code 1 backend/Dockerfile
    run_scanner "trivy fs (backend)" trivy \
      trivy fs --severity CRITICAL,HIGH --ignore-unfixed --exit-code 1 backend
  fi
fi

# ---------------------------------------------------------------------------
# OSV — multi-ecosystem scan
# ---------------------------------------------------------------------------
if want osv; then
  run_scanner "osv-scanner (recursive)" osv-scanner \
    osv-scanner scan source --recursive ./
fi

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
echo
color blue "==> Summary"; echo
fail_count=0
skip_count=0
for r in "${RESULTS[@]}"; do
  IFS='|' read -r label status note <<<"$r"
  case "$status" in
    pass) color green "  PASS"; printf ' %s\n' "$label" ;;
    skip) color yellow "  SKIP"; printf ' %s (%s)\n' "$label" "$note"; skip_count=$((skip_count+1)) ;;
    fail) color red   "  FAIL"; printf ' %s\n' "$label"; fail_count=$((fail_count+1)) ;;
  esac
done

echo
if [ "$fail_count" -gt 0 ]; then
  color red "$fail_count scan(s) failed"; echo
  if [ "$STRICT" = "1" ]; then exit 1; fi
fi
if [ "$skip_count" -gt 0 ]; then
  color yellow "$skip_count scan(s) skipped (missing tools — see above)"; echo
fi
exit 0
