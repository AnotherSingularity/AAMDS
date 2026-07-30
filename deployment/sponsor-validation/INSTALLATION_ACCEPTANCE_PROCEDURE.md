# Installation Acceptance Procedure

Run on the sponsor's target environment. Record each result in
`ACCEPTANCE_RESULTS_TEMPLATE.md`. Fail the procedure if any step's
observable result diverges from the expectation.

## 1. Preconditions
- [ ] `PREINSTALLATION_CHECKLIST.md` completed and signed.
- [ ] `SITE_SURVEY_TEMPLATE.md` completed for this site.

## 2. Install
```
AEON_HOME=/opt/aeon <package>/scripts/install.sh
```
Expected:
- Script exits `0`.
- `$AEON_HOME/VERSION`, `PROFILE`, `COMMIT`, `package.manifest.json`
  are present and match the package.

## 3. Verify installed manifest
```
(cd "$AEON_HOME" && sha256sum -c manifest.sha256 --quiet)
python3 <repo>/tools/deployment/validate-manifest.py \
  --manifest "$AEON_HOME/package.manifest.json" \
  --schema   <repo>/deployment/schemas/package-manifest.schema.json
```
Expected: both commands exit `0`.

## 4. Configuration validation
Confirm `$AEON_HOME/config/runtime.json` is a subset of the schema in
`core-runtime/src/config.rs` (deny_unknown_fields) and passes sponsor
policy for `scope_boundary_scan_outbound=true` and
`allow_unsigned_sources=false`.

## 5. Start the runtime
```
AEON_HOME=/opt/aeon $AEON_HOME/bin/aeon-operator-api &
export AEON_API_BASE=http://127.0.0.1:8080
```
Expected: HTTP `GET /api/v1/health` returns 200 within 30s.

## 6. Ingest synthetic observations
Point a synthetic adapter (from `sensor-adapter-sdk`) at the target
runtime. Expected:
- `/api/v1/tracks` returns ≥ 1 track within 60s.
- Each track's `provenance_root.source_observations` is populated.
- Alerts remain empty for the clean scenario.

## 7. Confirm relay simulation
Enable one authorized relay destination, submit a
`TrackState`/`Alert`/`SystemHealth`/`ObservationSummary` envelope, and
confirm delivery + audit event. Expected: no `ProhibitedKind` or
`ProhibitedContent` rejection.

## 8. Confirm audit continuity
```
sqlite3 $AEON_HOME/data/aeon.sqlite \
  "SELECT COUNT(*) FROM events WHERE sequence >= 1;"
```
Expected: monotonic sequence from 1..N with `verify_integrity` clean.

## 9. Version match
- `AEON_HOME/VERSION == package.manifest.json.version`.
- Operator API `/api/v1/version.build_id == package.manifest.json.version`.

## 10. Sign-off
- Installer:
- Observer:
- Security-approver:
- Result (PASS / FAIL):
- Attached evidence bundle path:
