//! Outbound-message-kind allowlist.
//!
//! This file is one of three source files exempt from the scope-boundary
//! scanner — it exists to state the *complete* permitted informational
//! surface for outbound relay. Adding a variant here requires an explicit
//! boundary review recorded in `docs/architecture/SCOPE_BOUNDARY.md`.
//!
//! The four permitted kinds are the exact variants of
//! [`aeon_contracts::relay::RelayMessageKind`]. Nothing else may be
//! wire-serialised by the gateway.

use aeon_contracts::relay::RelayMessageKind;

/// The set of message kinds the gateway will consider for outbound
/// transmission.
pub const ALLOWED_KINDS: &[RelayMessageKind] = &[
    RelayMessageKind::TrackState,
    RelayMessageKind::ObservationSummary,
    RelayMessageKind::SystemHealth,
    RelayMessageKind::Alert,
];

pub fn is_allowed(kind: RelayMessageKind) -> bool {
    ALLOWED_KINDS.contains(&kind)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_four_permitted_kinds_are_allowed() {
        for k in ALLOWED_KINDS {
            assert!(is_allowed(*k));
        }
    }

    #[test]
    fn allowlist_size_is_exactly_four() {
        // Guardrail: any addition to the informational surface must be a
        // deliberate boundary-review-recorded change. Growing the list
        // silently would slip past code review.
        assert_eq!(ALLOWED_KINDS.len(), 4);
    }
}
