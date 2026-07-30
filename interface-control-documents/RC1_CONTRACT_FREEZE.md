# RC1 Contract Freeze

The following canonical contract types are frozen at RC1. Any later
incompatible change requires a new **major** contract version — a minor
bump is not sufficient. See `interface-control-documents/VERSIONING_POLICY.md`.

## Frozen versions

| Contract | Schema version | Rust source | SHA-256 |
|---|---|---|---|
| observation      | 1.0 | `contracts/src/observation.rs` | `7d82636ead4d28d8e64322ae7d6e0618f48758ace6ee9132b98d40648ff4bd58` |
| normalized       | 1.0 | `contracts/src/observation.rs` | `7d82636ead4d28d8e64322ae7d6e0618f48758ace6ee9132b98d40648ff4bd58` |
| track            | 1.0 | `contracts/src/track.rs`       | `2d008ffca4d645c43cfb979605bad61e34a04db4b20e959dbd38b4fbaceabf06` |
| track_update     | 1.0 | `contracts/src/track.rs`       | `2d008ffca4d645c43cfb979605bad61e34a04db4b20e959dbd38b4fbaceabf06` |
| health           | 1.0 | `contracts/src/health.rs`      | `5a56978bafa4d0da10cef2b1234e8f77593c3c3a53e1c521bc7b6058a5b3031f` |
| alert            | 1.0 | `contracts/src/alert.rs`       | `5b380cb2aca5fc45ca7ce050c8bf6cfad95eae4fcfa0b8fab74d19789a0378e0` |
| relay_envelope   | 1.0 | `contracts/src/relay.rs`       | `8df11068c15b60dd731ce8e5007bf2fce493201d27b425ea58f1dd6cdd7eab2e` |
| audit            | 1.0 | `contracts/src/audit.rs`       | `9034f071464cb742619821189128205ebaac71f6c624441c8d7bc58af0e176a0` |
| adapter          | 1.0 | `sensor-adapter-sdk/src/adapter.rs` | `7850f3aa4fea65aae6187925ec4b95d65c83106aaf0a61516d09bc3dc7666a83` |
| configuration    | 1.0 | `core-runtime/src/config.rs`   | `3b003575b3373d013632fdb1c7da06f129a4ed2268621b31f19663baa5f176bc` |
| prohibited-registry | 1.0 | `contracts/src/prohibited.rs` | `dcb89f13cebfaf9cda104206ee7385e4753cc0b4394bf71362367147b0c3d675` |
| relay-allowlist  | 1.0 | `secure-relay/src/allowlist.rs` | `ce39910977934543c940a3b0ce3c41725126b190f1c42a041abf02fc5908fd17` |

Digests are SHA-256 over the exact Rust source file at the RC1 commit.
The commit and full release manifest are at
`release/AEON_AIR_DEFENSE_RC1_MANIFEST.json`.

## Change policy after RC1

1. **Additive-only minor bump**: adding an optional field via
   `Known::Unavailable`, or a new enum variant that a peer can safely
   ignore. Digests change; the schema `major` does not.
2. **Breaking change**: any field removal, field-type change,
   semantics change of an existing field, or removal / re-meaning of
   an enum variant. Requires a `major` bump and a new
   `interface-control-documents/RC2_CONTRACT_FREEZE.md` (or later).
3. The **prohibited-registry** may only *grow*. Removing a token is a
   scope-boundary widening and requires an explicit ADR-style
   decision recorded in `docs/architecture/SCOPE_BOUNDARY.md`.
4. The **relay-allowlist** may only *shrink*. Adding a variant is a
   scope-boundary widening and follows the same rule.

## Verification

The RC1 tag pins the exact commit whose contract digests are recorded
above. Any consumer that needs to prove compatibility can:

```
git checkout aeon-air-defense-rc1
python3 -c "import hashlib; print(hashlib.sha256(open('contracts/src/track.rs','rb').read()).hexdigest())"
```

Anything but the recorded digest means the checkout has drifted or a
change slipped in without a new release identity.
