# Security Requirements (Integration)

- Inbound observations from adapters MUST arrive over an authenticated
  transport in production; unauthenticated transports are permitted
  only when `allow_unsigned_sources=true` (rejected in production
  config validation).
- Every relay envelope MUST carry a non-empty classification label,
  non-empty releasability community list, a valid signature over the
  canonical envelope digest, and a non-repeating anti-replay nonce.
- The gateway rejects: unknown message types (compile-time), prohibited
  content (runtime scan), unknown destinations, unauthorized
  kind-for-destination, classification not permitted, missing
  releasability, invalid signatures, replay, expiration, oversize,
  rate-limit exceed, queue-depth exceed. See
  `secure-relay/src/gateway.rs::RelayRejectReason`.
- Prohibited concepts are canonically listed in
  `contracts/src/prohibited.rs` and enforced by
  `verify-scope-boundary`.
- Signature primitives are pluggable — the baseline uses keyed
  SHA-256 for structural correctness; sponsor deployments MUST
  substitute FIPS-validated primitives via KMS/HSM.
- No default credentials, no debug bypass — see `SECURITY.md`.
