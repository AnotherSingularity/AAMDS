# Pre-installation Checklist

Confirm every item **before** running any acceptance procedure.

## Personnel

- [ ] Named installer (authorized).
- [ ] Named observer (independent of installer).
- [ ] Named security-approver (authorized to hold decryption keys).

## Target environment

- [ ] Operating system + patch level recorded.
- [ ] Filesystem layout meets minimum-resource requirements from
      `<package>/package.manifest.json` field `minimum_resources`.
- [ ] Time source is disciplined (NTP or PTP against a trusted
      reference) OR a documented degraded-time-source posture is
      approved by the security-approver.
- [ ] No prior Aeon installation exists under the target `AEON_HOME`
      (or an uninstall has been executed and evidence captured).
- [ ] Network policy for outbound relay destinations is defined.
- [ ] KMS/HSM (or approved dev-signing substitute) is reachable from
      the target and its key IDs are recorded.
- [ ] Identity provider integration is documented and reachable.

## Package integrity

- [ ] `sha256sum -c manifest.sha256` inside the package passes.
- [ ] `python3 tools/deployment/validate-manifest.py --manifest
      <package>/package.manifest.json --schema deployment/schemas/
      package-manifest.schema.json` returns clean.
- [ ] Package signature (`signature.method`) matches sponsor policy.
      For non-`kms-hsm` methods, the security-approver has explicitly
      accepted the deployment as non-production.

## Scope-boundary re-check

- [ ] `./tools/verify.sh scope-boundary` PASSES on the exact commit
      being deployed.
- [ ] Sponsor personnel confirm the package does not contain any
      firing / launch / guidance / aimpoint / engagement interface.
