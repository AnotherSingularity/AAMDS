# Security Validation Checklist

Sponsor-owned checks that Aeon cannot perform for the target.

## Identity + credentials
- [ ] Sponsor IdP integration validated (login for viewer, operator,
      maintainer). Role → API-endpoint mapping matches
      `docs/security/ACCESS_CONTROL_MATRIX.md`.
- [ ] Credential rotation exercised for at least one role.
- [ ] No default credentials remain.

## Signing + key custody
- [ ] Relay signature method upgraded from `dev-hmac-sha256` to
      `kms-hsm` in `package.manifest.json.signature`.
- [ ] KMS/HSM key custodians named; access is quorum-based per sponsor
      policy.
- [ ] Signing operation exercised end-to-end from an outbound
      `RelayEnvelope` to a peer that verifies the signature.

## Encryption
- [ ] Encryption-at-rest for `data/aeon.sqlite` matches sponsor
      policy (LUKS / dm-crypt / vendor DAR).
- [ ] Encryption-in-transit for the operator API and outbound relay is
      terminated in the sponsor's approved TLS stack.

## Audit + logging
- [ ] Audit chain `verify_integrity` passes at cutover.
- [ ] Logs land in the sponsor's SIEM without secret leakage
      (verified by sample inspection).
- [ ] Retention policy activated per sponsor governance.

## Scope-boundary re-check
- [ ] Sponsor security review confirms no weapon, firing, launch,
      guidance, aimpoint, or engagement interface exists in the
      installed package.
- [ ] Prohibited-token scan (via `tools/scope_boundary_scan.sh` on the
      exact deployed commit) PASSES.

## Sign-off
- Security-approver:
- Date:
- Result (PASS / FAIL):
