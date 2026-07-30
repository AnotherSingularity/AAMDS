# Sponsor Handoff Index — Aeon Air Defense RC1

> **⛔ HANDOFF WITHDRAWN — RC1 REJECTED BY INDEPENDENT AUDIT.**
> Do **not** execute the procedures below against RC1 as an
> integration-ready release. RC1 is an engineering prototype / lab
> baseline. Ten release-blocking findings are recorded in
> [`../../docs/verification/RC1_AUDIT_REJECTION.md`](../../docs/verification/RC1_AUDIT_REJECTION.md).
> The next legitimate sponsor handoff is RC2 (a future branch).
> This document is preserved verbatim for audit traceability.


An authorized integration team can drive the Gate 11B acceptance from
this single document.

## 1. What the release candidate does

- Ingests observations from authorized sensor sources via versioned
  adapters (`sensor-adapter-sdk/`).
- Normalizes time, coordinates, units, uncertainty, integrity, and
  provenance without silent defaulting (`normalization/`).
- Correlates observations into deterministic, uncertainty-qualified
  tracks with explicit conflict, freshness, and source-contribution
  accounting (`track-management/`).
- Applies **bounded** ML corrections to sensor bias, clock drift,
  measurement noise, and confidence calibration — with a signed
  artifact registry, shadow evaluation, and OOD marking
  (`ml-correction/`).
- Serves the recognized operational picture over a read-oriented HTTP
  API (`operator-api/`) plus a thin single-file client
  (`operator-interface/`).
- Relays four informational message kinds
  (`TrackState`, `ObservationSummary`, `SystemHealth`, `Alert`) to
  authorized peers with allowlisting, signing, anti-replay, expiration,
  rate limits, store-and-forward, and dead-lettering
  (`secure-relay/`).
- Persists every material event append-only with a chained SHA-256
  integrity digest (`persistence/`).
- Ships deterministic simulation + replay
  (`simulation/`) so any track update can be reproduced from the same
  inputs.

## 2. What it explicitly does NOT do

- No weapon-to-target assignment.
- No weapon recommendation, engagement ranking, or firing solution.
- No launch authorization, launch recommendation, or launch command.
- No aimpoint selection, probability-of-kill optimization, missile /
  interceptor guidance, or terminal-course correction.
- No fire-control-bus output.
- No autonomous engagement authorization.
- No command that actuates, launches, guides, aims, arms, or fires a
  weapon.

The prohibition is enforced by `contracts::prohibited`, the four-variant
`RelayMessageKind` allowlist (unit-tested at count exactly 4), the
runtime outbound content scan, and the static source scanner.

## 3. Available packages

| Profile | Path | Status |
|---|---|---|
| developer     | `target/deploy/developer/`     | REPOSITORY_VERIFIED |
| edge          | `target/deploy/edge/`          | REPOSITORY_VERIFIED |
| disconnected  | `target/deploy/disconnected/`  | REPOSITORY_VERIFIED |
| fixed-site    | `target/deploy/fixed-site/`    | PACKAGE_VERIFIED |
| data-center   | `target/deploy/data-center/`   | PACKAGE_VERIFIED |
| private-cloud | `target/deploy/private-cloud/` | PACKAGE_VERIFIED |

Each package carries: signed `package.manifest.json`,
per-artifact `manifest.sha256`, `bin/`, `ui/`, `config/`, `scripts/`,
`docs/`, `sbom/`, and profile metadata files (`PROFILE`, `COMMIT`,
`VERSION`).

## 4. Environments repository-tested

- linux/x86_64 CI container running rustc 1.94.1 stable.
- Full deployment cycle (fresh install + upgrade + rollback +
  backup/restore + offline install + uninstall) for developer, edge,
  disconnected profiles.
- Package build + manifest verification for fixed-site, data-center,
  private-cloud profiles.

## 5. Validations that require sponsor equipment

- Postgres-compatible persistence for the server-class profiles.
- Enterprise IdP integration for the operator API.
- FIPS/NIAP-validated signing via sponsor KMS/HSM
  (replaces `dev-hmac-sha256`).
- HA failover, redundant relay delivery.
- True air-gap installation on sponsor hardware.
- Physical time source (GNSS discipline / PTP with sponsor grandmaster).
- Sponsor SIEM / observability endpoints.
- Sponsor change-control ticketing.

The `deployment/sponsor-validation/` package contains the 12-document
checklist tree that drives each of these.

## 6. Verify package integrity

```
cd <package-root>
sha256sum -c manifest.sha256
python3 <repo>/tools/deployment/validate-manifest.py \
  --manifest ./package.manifest.json \
  --schema   <repo>/deployment/schemas/package-manifest.schema.json
```

Any deviation MUST fail installation. If `signature.method` is
`dev-hmac-sha256` the security-approver must explicitly accept the
deployment as non-production, or the package must be re-issued with a
`kms-hsm` signature.

## 7. Install in a laboratory environment

```
AEON_HOME=/opt/aeon <package>/scripts/install.sh
export AEON_API_BASE=http://127.0.0.1:8080
$AEON_HOME/bin/aeon-operator-api &
curl -sSf "$AEON_API_BASE/api/v1/health"
```

Full lab acceptance procedure:
`deployment/sponsor-validation/INSTALLATION_ACCEPTANCE_PROCEDURE.md`.

## 8. Collect acceptance evidence

Every acceptance run must produce the artifacts in
`deployment/sponsor-validation/EVIDENCE_COLLECTION_GUIDE.md` and be
filed with a completed
`deployment/sponsor-validation/ACCEPTANCE_RESULTS_TEMPLATE.md`.

## 9. Report defects

Open an issue on the sponsor's tracked instance of this repository
(`AnotherSingularity/AAMDS`) referencing the RC1 commit and the
specific acceptance step that failed. Attach:

- the failing evidence artifact from `docs/evidence/gate-11/<profile>/`
  or the site-specific acceptance folder,
- the runtime `AEON_HOME/data/aeon.sqlite` if data-side (or the
  `verify_integrity` output),
- the `SystemHealth` snapshot at time of failure.

## 10. Perform rollback

Roll back the runtime via the shipped
`$AEON_HOME/scripts/rollback.sh --home $AEON_HOME --to <prior-package>`
(with `--restore-backup <backup>` if the schema changed). Sponsor
acceptance procedure:
`deployment/sponsor-validation/ROLLBACK_ACCEPTANCE_PROCEDURE.md`.

## 11. Preserve audit evidence

The audit event log is append-only with a chained SHA-256 digest
(`persistence::EventStore`). Preserve by:

1. Stopping the runtime cleanly.
2. `sqlite3 $AEON_HOME/data/aeon.sqlite ".backup /path/to/snap.sqlite"`.
3. Recording the SHA-256 of the snapshot in the sponsor's evidence store.
4. Confirming `EventStore::verify_integrity` PASSES on the snapshot
   before shipping it anywhere.

Any attempt to delete an event breaks the chain and will fail
`verify_integrity`.

## 12. Prohibited claims until formal authorization

The following claims MUST NOT be made about RC1 until sponsor
authorization is issued:

- Accreditation
- Authority to operate
- Government approval
- Platform certification
- FIPS validation
- Operational deployment
- Combat readiness
- Universal plug-and-play compatibility
- Full military integration

RC1 is a **release candidate** whose repository-controlled evidence is
mechanically verified. Formal authorization is a sponsor decision.
