# RC2 Verification-Path Inventory

Complete inventory of every verification path at RC2 base commit
`a770f1a`. Each entry is classified as `mandatory` (must fail-close
in RC2) or `advisory` (kept but never gating).

## `tools/verify.sh` (RC1 runner — will be replaced)

| Line | Snippet | Kind | RC2 disposition |
|---|---|---|---|
| 32 | `cargo clippy … \|\| true` in `step_lint` | mandatory, currently suppressed | rewrite: fail-close in RC2 runner |
| 36 | `cargo test --workspace --test '*' \|\| true` in `step_integration` | mandatory, currently suppressed | rewrite: fail-close |
| 37 | `cargo test … end_to_end 2>/dev/null \|\| echo "verify: e2e skipped"` | mandatory, currently silent-skips on discovery | rewrite: fail-close if targets exist; otherwise `FAIL: MISSING_REQUIRED_TARGET` |
| 43 | `log "no migrations yet"` — **wrong**, `persistence/src/migrations.rs::ALL` contains migration 0001 | mandatory, currently misreports | rewrite: real migration-integrity check |
| 46 | replay `2>/dev/null … \|\| echo "skipped"` | mandatory, currently silent-skips | rewrite: fail-close |
| 54 | scope-boundary `\|\| { … shell fallback }` | mandatory, silently falls back | rewrite: single canonical scanner, fail-close |
| 93 | `log "would run $f (skipped in verify; run manually per profile)"` | mandatory, currently doesn't build | rewrite: real package build in verify:all |
| 121 | `step_property \|\| true` in `run_all` | mandatory, currently suppressed | rewrite: fail-close |
| 122 | `step_integration \|\| true` | mandatory, currently suppressed | rewrite: fail-close |
| 123 | `step_e2e \|\| true` | mandatory, currently suppressed | rewrite: fail-close |

## `tools/security/run-security-checks.sh`

| Line | Snippet | Kind | RC2 disposition |
|---|---|---|---|
| 21–24 | `--advisory` flag path | advisory (opt-in) | keep for local pre-CI use; **not** invoked by RC2 verify:all |
| 36–37 | advisory downgrade when tool missing | mandatory, currently soft-fails | RC2 mandates version-checked tools; verify:all fails-close on any tool missing |
| 57 | `[ "$advisory" -eq 1 ] \|\| die "unresolved / expired vulnerabilities"` | mandatory | correct behaviour; keep |
| 67, 89 | `set +e` around real invocations | mandatory | rewrite: capture exit code, fail-close |
| 100 | `[ "$advisory" -eq 1 ] \|\| die "clippy failed"` | mandatory | correct behaviour; keep |
| 106 | grep pipeline `\|\| true` | advisory (unsafe-code inventory is best-effort) | keep — inventory even when grep exits 1 on no-match |

## `.github/workflows/ci.yml`

| Line | Snippet | Kind | RC2 disposition |
|---|---|---|---|
| 56 | `cargo audit --json > … \|\| true` | mandatory | rewrite: fail-close (evidence file must reflect actual exit code) |

## `tools/scope_boundary_scan.sh`

- Single-pass regex-based scanner. RC2-F replaces it with a layered
  scanner in `tools/scope/run.py` that scopes each check
  (production source / public contracts / schemas / deployment /
  architecture / documentation claims) and honours a versioned
  exclusions manifest.

## `tools/deployment/test-profile.sh`

- Mandatory. Not currently in `verify.sh` except through
  `step_deployment`. RC2-B integrates it as a mandatory step; RC2-C
  reworks the underlying build order.

## `tools/deployment/build-package.sh`

- Mandatory. Order of operations is wrong: identity files are
  written after `manifest.sha256`. RC2-C fixes.

## `2>/dev/null` / `>/dev/null` occurrences

Every occurrence in `tools/verify.sh` was inspected. All ten either
(a) discard error output that must instead be surfaced, or
(b) hide "tool missing" behind an advisory pretense. RC2 runner
routes stdout+stderr to per-step log files and evaluates exit code
independently.

## RC2 disposition summary

- The RC1 `tools/verify.sh` is **retained** so its historical behaviour
  is auditable, but is **removed from any RC2 verification path**.
- The RC2 runner lives in `tools/verify/run.py` and consumes the
  versioned requirements manifest at
  `verification/verification-requirements.json`.
- Every mandatory step fails-close.
- `NOT_APPLICABLE` may only be declared in the requirements manifest,
  never inferred at runtime.
- `BLOCKED_EXTERNAL` is forbidden in RC2 `verify:all`.
