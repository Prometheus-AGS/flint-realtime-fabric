#!/usr/bin/env bash
# KBD per-change QA gate for flint-realtime-fabric.
#
# Evaluates the BLOCKING subset of .kbd-orchestrator/constraints.md against a change and
# writes .refiner/artifacts/<change-id>/refinement_log.md (the artifact-refiner log path).
# Exit 0 = ALL PASS (proceed to verify/archive); exit 1 = a BLOCKING constraint failed
# (mark the change BLOCKED in progress.json, then refine).
#
# Usage: qa-gate.sh <change-id> [--kind rust|config|docs|frontend|tooling]
#   --kind selects the applicable BLOCKING subset (see constraints.md). Default: rust.
set -uo pipefail

CHANGE="${1:?usage: qa-gate.sh <change-id> [--kind ...]}"
KIND="rust"
shift || true
while [ $# -gt 0 ]; do
  case "$1" in
    --kind) KIND="${2:-rust}"; shift 2 ;;
    *) shift ;;
  esac
done

ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$ROOT" || exit 2
LOG_DIR=".refiner/artifacts/$CHANGE"
mkdir -p "$LOG_DIR"
LOG="$LOG_DIR/refinement_log.md"
FAILED=0
declare -a RESULTS

record() { # status message
  RESULTS+=("$1|$2")
  [ "$1" = "FAIL" ] && FAILED=1
  return 0
}

run() { # label cmd...  → PASS/FAIL by exit code
  local label="$1"; shift
  if "$@" >/dev/null 2>&1; then record PASS "$label"; else record FAIL "$label"; fi
}

# ── R5 file-size (hand-authored source only) ─────────────────────────────────
# The 500-line limit targets hand-written files; generated bindings (SDK codegen,
# uniffi/protoc output) are exempt — exclude known generated paths.
oversize="$(git ls-files -- '*.rs' '*.ts' '*.tsx' '*.proto' 2>/dev/null \
  | grep -vE '/gen/|/src/rust/|_pb\.(ts|go)$|\.connect\.(ts|go)$|_connect\.ts$' \
  | while read -r f; do [ -f "$f" ] && n=$(wc -l < "$f") && [ "$n" -gt 500 ] && echo "$f:$n"; done)"
if [ -n "$oversize" ]; then record FAIL "R5 file-size >500 lines: $oversize"; else record PASS "R5 file-size ≤500 lines"; fi

# ── S1 no hardcoded secret in production source ──────────────────────────────
# Scope to source we ship; exclude test fixtures (throwaway keys are expected there),
# this constraints/gate tooling (it names the forbidden literal by definition), and
# archived spec text describing past removals. A real private-key block or the known dev
# signing-secret literal in production source is a FAIL.
s1_hits="$(git grep -inE 'signing-secret-not-for-production|-----BEGIN [A-Z ]*PRIVATE KEY-----' -- \
  'crates/**/src/**' 'compose*.yml' 'deploy/**' 'admin-ui/src/**' 'sdks/**/src/**' 2>/dev/null \
  | grep -viE 'tests?/fixtures?/' || true)"
if [ -n "$s1_hits" ]; then
  record FAIL "S1 hardcoded secret in production source: $s1_hits"
else
  record PASS "S1 no hardcoded secret in production source"
fi

# ── P1 valid OpenSpec delta (skip if already archived) ───────────────────────
if [ -d "openspec/changes/$CHANGE" ]; then
  run "P1 openspec validate $CHANGE" openspec validate "$CHANGE"
else
  record PASS "P1 (change already archived — validated at archive time)"
fi

# ── Rust-only BLOCKING subset (R1–R4, S-checks via clippy) ───────────────────
if [ "$KIND" = "rust" ]; then
  run "R4 fmt --check"       cargo fmt --check --all
  run "R1 check --workspace" cargo check --workspace
  run "R2/R3 clippy pedantic + unwrap_used" \
      cargo clippy --workspace --lib --bins -- -D warnings -W clippy::pedantic -W clippy::unwrap_used
fi

# ── Frontend BLOCKING subset (F1 no-any heuristic) ───────────────────────────
if [ "$KIND" = "frontend" ]; then
  if git grep -nE ':\s*any\b|<any>|as any' -- 'admin-ui/**/*.ts' 'admin-ui/**/*.tsx' 'sdks/ts/**/*.ts' 2>/dev/null | grep -q .; then
    record FAIL "F1 explicit 'any' type present"
  else
    record PASS "F1 no explicit 'any'"
  fi
fi

# ── write the refinement log ─────────────────────────────────────────────────
{
  echo "# Refinement Log — $CHANGE"
  echo
  echo "_QA gate run · kind: $KIND · constraints: .kbd-orchestrator/constraints.md_"
  echo
  echo "| Result | Constraint |"
  echo "|--------|------------|"
  for r in "${RESULTS[@]}"; do
    s="${r%%|*}"; m="${r#*|}"
    icon="✅"; [ "$s" = "FAIL" ] && icon="❌"
    echo "| $icon $s | $m |"
  done
  echo
  if [ "$FAILED" -eq 0 ]; then
    echo "**Verdict: ALL PASS** — proceed to verify + archive."
  else
    echo "**Verdict: BLOCKED** — mark BLOCKED in progress.json and refine before archive."
  fi
} > "$LOG"

echo "QA gate ($KIND) for $CHANGE → $([ "$FAILED" -eq 0 ] && echo PASS || echo BLOCKED)"
echo "  log: $LOG"
exit "$FAILED"
