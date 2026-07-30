# RC1 Relay Boundary Confirmation

Mechanical re-confirmation on the RC1 commit.

## `RelayMessageKind` variants (the *entire* informational surface)

| # | Variant | Meaning |
|---|---|---|
| 1 | `TrackState`         | published track state / update summary |
| 2 | `ObservationSummary` | summary of one or more observations |
| 3 | `SystemHealth`       | system-health snapshot |
| 4 | `Alert`              | operator alert intended for authorized peers |

`allowlist_size_is_exactly_four` unit test PASSES — any silent
growth of this enum will fail CI.

## Schema digests (from `RC1_CONTRACT_FREEZE.md`)

| Component | SHA-256 |
|---|---|
| relay_envelope schema (`contracts/src/relay.rs`)      | `8df11068c15b60dd731ce8e5007bf2fce493201d27b425ea58f1dd6cdd7eab2e` |
| relay allowlist        (`secure-relay/src/allowlist.rs`) | `ce39910977934543c940a3b0ce3c41725126b190f1c42a041abf02fc5908fd17` |
| prohibited registry    (`contracts/src/prohibited.rs`)   | `dcb89f13cebfaf9cda104206ee7385e4753cc0b4394bf71362367147b0c3d675` |
| outbound policy        (`secure-relay/src/policy.rs`)    | `ef23c7a3fdaeb2333096b0ae8f628c9126fde45e19a57a8ae14e5d02748f0ccd` |

## Static scan

`tools/scope_boundary_scan.sh` — PASS on the RC1 commit. No prohibited
token appears outside the four exempt paths
(`contracts/src/prohibited.rs`, `secure-relay/src/allowlist.rs`,
the boundary scanner itself, and the docs / ICD tree).

## Runtime rejection tests

| Test | Location | Result |
|---|---|---|
| `allowlist_size_is_exactly_four`           | `secure-relay/src/allowlist.rs` | PASS |
| `all_four_permitted_kinds_are_allowed`     | `secure-relay/src/allowlist.rs` | PASS |
| `prohibited_content_is_rejected`           | `secure-relay/src/gateway.rs`   | PASS (builds the offending key from `PROHIBITED_TOKENS` at runtime so this test source itself contains no forbidden literal) |
| `invalid_signature_is_rejected`            | `secure-relay/src/gateway.rs`   | PASS |
| `replay_attempt_is_rejected`               | `secure-relay/src/gateway.rs`   | PASS |
| `expired_envelope_is_rejected`             | `secure-relay/src/gateway.rs`   | PASS |
| `unknown_destination_is_rejected`          | `secure-relay/src/gateway.rs`   | PASS |
| `kind_not_authorized_for_destination_rejected` | `secure-relay/src/gateway.rs` | PASS |
| `oversized_payload_is_rejected`            | `secure-relay/src/gateway.rs`   | PASS |
| `queue_full_dead_letters`                  | `secure-relay/src/gateway.rs`   | PASS |

## Written statement

- No weapon-control interface exists.
- No engagement-computation implementation exists.
- No firing-solution implementation exists.
- No launch, guidance, or aimpoint command exists.
- Relay output is restricted to the four informational schemas above.
- The scanner uses documented, narrowly scoped logic (see the exempt-path
  list in `tools/scope_boundary_scan.sh`); tests assert behavior
  directly rather than by textual match.
