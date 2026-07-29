# Security Policy

## Reporting

Security issues in Aeon must be reported privately to the maintainers before
any public disclosure. Do not open a public issue for a suspected
vulnerability.

## Scope-boundary reports

If you believe a code path violates the scope boundary
(`docs/architecture/SCOPE_BOUNDARY.md`) — for example, a public API accepts,
routes, or relays a weapon-control, firing-solution, launch, guidance, or
aimpoint message — treat it as a security-severity report.

## Supported evidence

Security evidence produced by CI (SBOM, dependency audit, secret scan, static
analysis, scope-boundary verification) is preserved under
`docs/evidence/` and referenced from `docs/evidence/RELEASE_EVIDENCE_INDEX.md`.
