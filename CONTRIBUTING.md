# Contributing

Every change must:

1. Pass `tools/verify.sh all` locally.
2. Preserve the scope boundary (`docs/architecture/SCOPE_BOUNDARY.md`).
   Any addition of a prohibited token outside `contracts/src/prohibited.rs`,
   `secure-relay/src/allowlist.rs`, `verification/scope-boundary/`, or the
   `docs/` tree fails CI.
3. Add or update a versioned contract in `contracts/` if it changes any
   cross-boundary data.
4. Add tests that would fail without the change.
5. Update `docs/verification/KNOWN_LIMITATIONS.md` if new gaps are
   introduced.

No commit that removes tests or lowers coverage without an explicit and
recorded justification will be accepted.
