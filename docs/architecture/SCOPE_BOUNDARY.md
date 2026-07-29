# Scope Boundary

Aeon is an **information, tracking, fusion, operator-awareness, and secure-relay
system**. This document defines the boundary mechanically enforced by the
codebase and by CI (`verify-scope-boundary`).

## In scope

- Ingest of authorized sensor observations through published adapters.
- Normalization of time, coordinates, units, uncertainty, integrity, provenance.
- Correlation of observations into persistent tracks with explicit uncertainty.
- Recognized operational picture for authorized operators (read-oriented).
- Bounded, auditable machine-learning correction of measurement bias,
  clock drift, and confidence calibration.
- Secure, policy-enforced relay of informational messages
  (`track_state`, `observation_summary`, `system_health`, `alert`) to
  approved external systems, subject to allowlist, signing, anti-replay,
  classification, and releasability policy.
- Deterministic replay of recorded scenarios for verification and analysis.
- Immutable audit of every state change, correction, relay, alert, and
  configuration activation.

## Out of scope — mechanically prohibited

The following concepts must not appear in Aeon's public APIs, message-type
registry, relay outbound allowlist, contracts, or runtime call graph:

- Weapon-to-target assignment
- Weapon recommendation
- Engagement ranking for destructive action
- Intercept-point calculation
- Firing-solution calculation
- Launch authorization
- Launch recommendation
- Aimpoint selection
- Probability-of-kill (Pk) optimization
- Missile / interceptor guidance
- Terminal-course correction
- Fire-control bus output
- Autonomous engagement authorization
- Commands that actuate, launch, guide, aim, arm, or fire a weapon

## Mechanical enforcement

Enforcement does not rely on comments or policy documents alone. It is
implemented at four layers:

1. **Prohibited-message-type registry**
   [`contracts/src/prohibited.rs`](../../contracts/src/prohibited.rs) — a
   canonical list of forbidden concept tokens, exported for compile-time and
   runtime reuse.

2. **Compile-time surface exclusion** — the `secure-relay` outbound
   allowlist accepts only variants of `RelayMessageKind` corresponding to
   `TrackState`, `ObservationSummary`, `SystemHealth`, `Alert`. All other
   variants are rejected as an unreachable branch (see
   [`secure-relay/src/allowlist.rs`](../../secure-relay/src/allowlist.rs)).

3. **Runtime outbound validation** — every `RelayEnvelope` is passed through
   `contracts::prohibited::scan_envelope` before signing. Any string field
   matching a prohibited token causes the envelope to be rejected and an
   audit event `RelayRejected{reason=ProhibitedContent}` recorded.

4. **Static source scan** — the CI job `verify-scope-boundary` runs
   `verification/scope-boundary`, which walks the workspace, greps public
   API declarations for prohibited tokens (with clearly-marked test/doc
   exemptions), and fails the build if any match survives.

## Exemptions

The three files below are the only source files that may reference
prohibited tokens as string literals — they exist to enforce the
boundary and must not contribute the tokens to any exported API:

- `contracts/src/prohibited.rs`
- `secure-relay/src/allowlist.rs`
- `verification/scope-boundary/src/lib.rs`
- `docs/**` and any file under `interface-control-documents/`
  (documentation is allowed to describe the boundary)

The scope-boundary scanner recognizes these paths explicitly.
