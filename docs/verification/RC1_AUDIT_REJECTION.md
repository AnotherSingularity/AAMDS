# RC1 Audit Rejection

> **Verdict: RC1 REJECTED by independent audit.**
> RC1 must be labelled **ENGINEERING PROTOTYPE / LAB BASELINE** and
> **MUST NOT** be advanced to sponsor Gate 11B testing as an
> integration-ready release. Corrections proceed on a new RC2 branch;
> RC1 history and evidence are preserved intact.

## Result

```
AUDIT RESULT: FAIL
RC1 RELEASE CLAIM: REJECTED
```

## Reassessed gates

```
Tag preservation:  PASS
Source freeze:     PASS
Gate 10:           FAIL
Gate 11A:          FAIL
Gate 11B:          NOT EVALUATED
Scope boundary:    NOT VERIFIED
RC1 rating:        REJECTED
```

## Rating supersession

The prior RC1 rating recorded in
`docs/verification/RELEASE_CANDIDATE.md`,
`release/AEON_AIR_DEFENSE_RC1_MANIFEST.json`, and the
`aeon-air-defense-rc1` tag annotation —

> `MILITARY INTEGRATION-READY RELEASE CANDIDATE: REPOSITORY VERIFIED`
> `SPONSOR PLATFORM VALIDATION: REQUIRED`

— is **withdrawn**. The correct rating for `d1b8414` is:

```
ENGINEERING PROTOTYPE / LAB BASELINE
REJECTED BY INDEPENDENT AUDIT
```

The prior rating is preserved verbatim inside the frozen artifacts
above so the audit history is transparent; the frozen artifacts
are **not edited**. This rejection record supersedes any conflicting
statement elsewhere in the repository.

## Findings (independently reproduced)

Each finding was reproduced from the RC1 source before this record
was written.

### 1. Relay signature does not authenticate payload or envelope fields

`secure-relay/src/gateway.rs:150` calls
`verify(&self.private_key_material, &envelope.payload_digest_hex, &envelope.signature_hex)`.
The signature is checked against the **caller-supplied**
`payload_digest_hex` string. The gateway never recomputes the digest
from `payload_json`, and the `canonical_envelope_digest` helper
(which does bind destination, kind, timestamps, sender, nonce) is
defined in `secure-relay/src/signing.rs:47` but is only referenced
from the tests — the production `submit` path does not use it.
Consequence: destination, message type, timestamps, sender, nonce,
classification, and releasability can all be modified without
invalidating the signature.

### 2. `verify:all` is fail-open

`tools/verify.sh` still contains:
- `step_lint()` … `|| true` (line 32)
- `step_integration()` … `|| true` (line 36)
- `step_migrations()` prints `"no migrations yet"` (line 43) even
  though `persistence/src/migrations.rs` defines migration 0001.
- `step_packages()` logs `"would run … (skipped in verify)"`
  (line 93) instead of actually building packages.
- `run_all()` invokes `step_property || true`, `step_integration || true`,
  `step_e2e || true` (lines 121–123).

Failures in these steps are converted into silent skips.

### 3. Package identity is outside checksum protection

`tools/deployment/build-package.sh` writes `manifest.sha256` using
`find … -not -name manifest.sha256 -not -name package.manifest.json`
(line 81–82) **before** the Python step that writes `PROFILE`,
`COMMIT`, `VERSION`, and `package.manifest.json` (lines 152–156).
`deployment/_common/scripts/install.sh` runs only
`sha256sum -c manifest.sha256`; it does not verify the manifest
signature. `PROFILE`, `COMMIT`, `VERSION`, and `package.manifest.json`
are therefore not authenticated by `install.sh`, `upgrade.sh`,
`rollback.sh`, or `healthcheck.sh`.

### 4. Operator API has no authentication or authorization

- `operator-api/src/routes.rs::ack_alert` accepts an actor name in the
  request body and mutates in-memory state without any authenticated
  identity or persistent audit event.
- `operator-api/src/main.rs` constructs `ApiState` with an empty
  `TrackEngine`, a hard-coded `RuntimeState::Ready`, and empty
  alerts — it does not initialise or validate adapters, persistence,
  relay, or the runtime supervisor. Readiness is asserted, not
  measured.

### 5. Frozen Gate 10 evidence contradicts itself

`docs/evidence/releases/aeon-air-defense-rc1/gate-10/TOOLCHAIN_VERSIONS.json`
records `cargo-audit expected_version: 9.99.9, installed_version: 0.22.2,
status: version_mismatch` (produced when the deployment negative-path
regression test temporarily rewrote `security/toolchain.lock` and I
failed to regenerate the evidence afterward).
`docs/evidence/releases/aeon-air-defense-rc1/gate-10/GATE_10_REPORT.md`
claims all 5 tools PASS. The report and the machine-readable evidence
contradict each other.

### 6. Scope-boundary scan fails on the release tree

Running `tools/scope_boundary_scan.sh` on the RC1 tree returns:

```
scope-boundary VIOLATION: ./release/AEON_AIR_DEFENSE_RC1_MANIFEST.json
    "no_firing_solution": true
    "no_aimpoint_or_engagement": true
scope-boundary VIOLATION: ./release/AEON_AIR_DEFENSE_RC1_MANIFEST.schema.json
    "no_weapon_control","no_firing_solution","no_launch_or_guidance",
    "no_aimpoint_or_engagement"
scope-boundary scan FAILED
```

The RC1 manifest records `static_scan: PASS`. The scanner run
against the same commit disagrees. The static-scan PASS claim in the
manifest is therefore not true on the release-frozen tree.

### 7. Scope boundary is structurally bypassable

`RelayEnvelope.payload_json` is a `serde_json::Value`. There is no
per-`RelayMessageKind` payload schema; the four informational labels
constrain neither structure nor semantics. The runtime prohibited
scan (`aeon_contracts::prohibited::scan_json`) is a substring match
against a fixed word list — it does not enforce that a
`TrackState` payload is actually a track summary. An informational
label with an unrelated payload is not rejected by any structural
check.

### 8. Gate 11A never proves the installed service runs

`docs/evidence/gate-11/developer/fresh-install-health.json` contains
`"api_checked": false`. `tools/deployment/test-profile.sh::op_fresh_install`
runs `install.sh`, verifies file layout, and calls `healthcheck.sh` —
which only issues an HTTP request when `AEON_API_BASE` is set (it
is not). No test starts the service, loads the config through the
Rust runtime, ingests a synthetic observation, produces a track,
exercises the API, tests relay behaviour, or persists an audit
event.

### 9. Determinism is not established

- `contracts/src/ids.rs::TrackId::new` uses `Uuid::new_v4`.
- `track-management/src/lib.rs::TrackEngine::tracks` is a
  `HashMap<TrackId, Track>` (line 77); `find_association` iterates it
  unordered.
- `simulation/src/determinism.rs::trace_digest` deliberately excludes
  `TrackId` and compares only a selected projection of each update.

Under inputs that place two candidate tracks equally close to a new
observation, the winner depends on `HashMap` iteration order — a
deterministic property is claimed that the code does not enforce.

### 10. The packaged executable is not the described integrated system

The shipped package contains only `bin/aeon-operator-api`. Its
`main.rs` (see finding 4) creates an in-memory mock. It does not
compose the adapters, normalisation, persistence, ML correction,
relay, or audit subsystems into an operational service. The
description of the release candidate as an integrated information
layer is not supported by what is actually installed.

## Findings that passed

- Preserved git bundle is valid and carries the tag.
- Tag object `8bc2edac76afbb75ded176c2f39c717e39784297` resolves to
  RC1 commit `d1b8414181bc164b426ef23e4f591ec5c3c5eeb7`.
- Modular crate layout, uncertainty modelling, provenance contracts,
  and `#![forbid(unsafe_code)]` policy are sound foundations to
  correct forward.

## Sponsor guidance

**Do not** advance RC1 through `deployment/sponsor-validation/`. The
Gate 11B acceptance procedures in that directory reference the
release-candidate rating that is now withdrawn. If a sponsor already
started an acceptance run against `d1b8414`, they should record the
result as `REJECTED — audit findings apply` and file the finished
`ACCEPTANCE_RESULTS_TEMPLATE.md` under
`docs/evidence/gate-11b/rejected/` in their own evidence store.

## What is *not* changed by this record

- Commit `d1b8414181bc164b426ef23e4f591ec5c3c5eeb7` and the annotated
  tag object `8bc2edac76afbb75ded176c2f39c717e39784297` remain as they
  were tagged. They now identify the **rejected** RC1, not an
  integration-ready release.
- Every file under `docs/evidence/releases/aeon-air-defense-rc1/`,
  `release/AEON_AIR_DEFENSE_RC1_MANIFEST.json`,
  `release/rc1-tag-preservation/`, and
  `interface-control-documents/RC1_CONTRACT_FREEZE.md` is preserved
  intact. Do **not** edit them; the audit history depends on their
  being immutable.
- `release/rc1-tag-preservation/aeon-air-defense-rc1-tag.bundle`
  still transfers the exact tag object.

## Next stage

Corrections are made on a new branch that produces **RC2**. Every
finding above must be closed with a mechanically verifiable check
before RC2 is proposed for freeze. Suggested minimum RC2 scope:

1. Sign the canonical envelope digest (not `payload_digest_hex`) and
   have the gateway recompute `payload_digest_hex` from `payload_json`
   before signing. Add a fuzz test that mutates each envelope field
   and asserts signature verification fails.
2. Rewrite `verify:all` so every step fails closed; expand
   `step_packages` to actually build and integrity-check each profile.
3. Include `PROFILE`, `COMMIT`, `VERSION`, and `package.manifest.json`
   in `manifest.sha256`; have `install.sh` verify the manifest
   signature before touching `$AEON_HOME`.
4. Front the operator API with an authenticating middleware; persist
   every write to the append-only audit log; wire readiness to actual
   dependency probes.
5. Bind the release manifest to the *regenerated* Gate 10 evidence and
   fail-close the manifest builder if the toolchain report does not
   read all-`ok`.
6. Exempt `release/*.json` from the boundary scanner via a documented
   allowlist, or rewrite the manifest keys to avoid the tokens; either
   way, run the scanner over the release tree in CI.
7. Introduce per-`RelayMessageKind` payload schemas; reject envelopes
   whose payload does not deserialise into the schema for the declared
   kind.
8. Extend the deployment harness to actually run the service, drive a
   synthetic ingest, verify a track through the API, and audit-verify
   the persistence file — a truly `api_checked: true` result.
9. Replace `HashMap` with `BTreeMap` in the track engine (or otherwise
   impose a total order over correlation candidates); remove
   `TrackId::new()` from any code path whose determinism the replay
   harness relies on.
10. Assemble a single supervisor binary that actually composes
    adapters → normalisation → tracking → correction → persistence →
    API → relay, and ship *that* in the deployment package.
