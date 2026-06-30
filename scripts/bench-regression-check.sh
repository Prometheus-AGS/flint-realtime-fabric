#!/usr/bin/env bash
# Criterion regression guard: compare target/criterion/<bench>/new/ against
# .criterion/<bench>/main/ baseline. Exits 1 if any benchmark regresses > 10%.
set -euo pipefail

THRESHOLD=10.0
BASELINE_DIR=".criterion"
CURRENT_DIR="target/criterion"
REGRESSIONS=0

if [[ ! -d "$BASELINE_DIR" ]]; then
  echo "No baseline found at $BASELINE_DIR — skipping regression check"
  exit 0
fi

while IFS= read -r -d '' baseline_file; do
  # Derive the corresponding current-run estimates path.
  # baseline: .criterion/<group>/<bench>/<size>/main/estimates.json
  # current:  target/criterion/<group>/<bench>/<size>/new/estimates.json
  relative="${baseline_file#$BASELINE_DIR/}"
  current_file="$CURRENT_DIR/$(echo "$relative" | sed 's|/main/|/new/|')"

  if [[ ! -f "$current_file" ]]; then
    echo "SKIP: no current run found for $baseline_file"
    continue
  fi

  baseline_mean=$(jq '.mean.point_estimate' "$baseline_file")
  current_mean=$(jq '.mean.point_estimate' "$current_file")

  if [[ -z "$baseline_mean" || "$baseline_mean" == "null" ]]; then
    echo "SKIP: missing mean in $baseline_file"
    continue
  fi

  pct=$(awk "BEGIN { printf \"%.1f\", (($current_mean - $baseline_mean) / $baseline_mean) * 100 }")

  if awk "BEGIN { exit ($pct <= $THRESHOLD) }"; then
    echo "REGRESSION: ${relative%/main/estimates.json} regressed ${pct}% (baseline: ${baseline_mean}ns, current: ${current_mean}ns)"
    REGRESSIONS=$((REGRESSIONS + 1))
  else
    echo "OK: ${relative%/main/estimates.json} ${pct}%"
  fi
done < <(find "$BASELINE_DIR" -name "estimates.json" -path "*/main/estimates.json" -print0)

if [[ "$REGRESSIONS" -gt 0 ]]; then
  echo ""
  echo "FAIL: $REGRESSIONS benchmark(s) exceeded the ${THRESHOLD}% regression threshold."
  exit 1
fi

echo ""
echo "All benchmarks within ${THRESHOLD}% threshold."
