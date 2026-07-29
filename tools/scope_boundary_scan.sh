#!/usr/bin/env bash
# Shell fallback for the scope-boundary scanner. The canonical scanner is
# `verification/scope-boundary` (Rust); this script exists so the boundary
# check works even on a fresh checkout without a compiled scanner.

set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

# Canonical prohibited token list — kept in sync with
# contracts/src/prohibited.rs PROHIBITED_TOKENS.
tokens=(
  "weapon_assignment" "weapon_recommendation" "engagement_ranking"
  "intercept_point" "intercept_calculation" "firing_solution" "fire_solution"
  "fire_control_bus" "launch_authorization" "launch_recommendation"
  "launch_command" "aimpoint_selection" "aimpoint"
  "probability_of_kill" "pk_optimization"
  "missile_guidance" "interceptor_guidance" "terminal_guidance"
  "terminal_course_correction"
  "autonomous_engagement" "engage_target" "engagement_authorization"
  "target_engagement"
  "actuate_weapon" "arm_weapon" "fire_weapon"
)

# Files/dirs explicitly exempted (they exist to enforce the boundary).
exempt=(
  "contracts/src/prohibited.rs"
  "secure-relay/src/allowlist.rs"
  "verification/scope-boundary/"
  "tools/scope_boundary_scan.sh"
  "docs/"
  "interface-control-documents/"
  "cybersecurity/gitleaks.toml"
  ".git/"
  "target/"
  "node_modules/"
)

exempt_grep=$(printf -- '--exclude-dir=%s\n' "${exempt[@]}" | tr '\n' ' ')
# convert dir-style exempts to grep-friendly flags
exclude_flags=(
  --exclude-dir=.git --exclude-dir=target --exclude-dir=node_modules
  --exclude-dir=scope-boundary
)

fail=0
for tok in "${tokens[@]}"; do
  # Search source only, exempt docs / prohibited registry / allowlist / scanner
  matches=$(grep -RIinE --include='*.rs' --include='*.ts' --include='*.tsx' \
    --include='*.py' --include='*.js' --include='*.json' \
    "${exclude_flags[@]}" -- "$tok" . 2>/dev/null || true)
  while IFS= read -r line; do
    [ -z "$line" ] && continue
    path="${line%%:*}"
    path="${path#./}"
    # doc-comment mentions are permitted anywhere in .md files
    if [[ "$path" == *.md ]]; then continue; fi
    for e in "${exempt[@]}"; do
      # exact-file or dir-prefix match
      if [[ "$path" == "$e" || "$path" == "$e"* ]]; then
        line=""
        break
      fi
    done
    if [ -n "$line" ]; then
      echo "scope-boundary VIOLATION: $line" >&2
      fail=1
    fi
  done <<< "$matches"
done

if [ "$fail" -ne 0 ]; then
  echo "scope-boundary scan FAILED — prohibited tokens detected outside exempt paths." >&2
  exit 1
fi

echo "scope-boundary scan: PASS"
