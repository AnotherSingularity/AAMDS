# Cryptographic Inventory

| Use | Primitive | Source | Notes |
|---|---|---|---|
| Configuration digest | SHA-256 (`sha2` crate) | RustCrypto | Change-sensitive; not FIPS-validated in this crate |
| Audit-event chained digest | SHA-256 | RustCrypto | Chained per event; tampering fails `verify_integrity` |
| Model artifact digest | SHA-256 | RustCrypto | Bound to name+version+feature+output schema |
| Envelope signature (baseline) | Keyed SHA-256 (HMAC-style) | RustCrypto | **Replace with FIPS/NIAP-validated signature via KMS/HSM in sponsor deployment.** |
| Anti-replay nonce | random UUIDv4 in tests | `uuid` crate | Sponsor deployments should use a monotonic counter + nonce hybrid |

Aeon does not currently ship code that talks to any specific KMS or HSM.
The `sign` / `verify` interface in `secure-relay::signing` is the
integration point.

## FIPS status

This baseline **does not** claim FIPS validation of any primitive. That
is a sponsor obligation and part of the accreditation package they will
build on top.
