//! Time-quality state.

use serde::{Deserialize, Serialize};

/// Quality of the timestamp attached to an observation.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeQuality {
    /// Timestamp is disciplined to a trusted source (GPS, PTP).
    Disciplined,
    /// Timestamp is from a locally-monotonic clock but not disciplined.
    LocalMonotonic,
    /// Timestamp is drifting outside the policy tolerance.
    Drifting { estimated_offset_ms: f64 },
    /// Timestamp was replaced or reconstructed from receive time.
    ReceiveTimeSubstituted,
    /// No trustworthy timestamp is available.
    Unavailable,
}

impl TimeQuality {
    pub fn is_usable_for_correlation(&self) -> bool {
        matches!(self, TimeQuality::Disciplined | TimeQuality::LocalMonotonic)
    }
}
