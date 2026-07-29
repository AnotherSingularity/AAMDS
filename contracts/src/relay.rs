//! Outbound relay envelope. The variants of [`RelayMessageKind`] are the
//! **entire** informational surface Aeon may transmit to external systems.
//!
//! No prohibited-concept variant is or will ever be added here. See
//! `docs/architecture/SCOPE_BOUNDARY.md` and `contracts::prohibited` for
//! the canonical list. The scope-boundary scanner asserts that this enum
//! contains exactly the allowed variants — any addition triggers a
//! deliberate boundary review.

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::ids::{ActorId, DestinationId, RelayMessageId};
use crate::version::{SchemaVersion, relay_schema};

/// The complete permitted outbound-message surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelayMessageKind {
    /// A published track state or track update summary.
    TrackState,
    /// A summary of one or more observations (source, quality, provenance).
    ObservationSummary,
    /// A system-health snapshot for authorized monitoring peers.
    SystemHealth,
    /// An operator alert intended for authorized situational-awareness
    /// consumers.
    Alert,
}

/// Sensitivity / releasability label. Structurally free-form so sponsors
/// can insert their marking scheme; the relay policy validates values.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Classification {
    pub label: String,
    pub caveats: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Releasability {
    pub allowed_communities: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryState {
    Queued,
    InFlight,
    Delivered,
    DeadLettered,
    Rejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AckState {
    None,
    Pending,
    Acknowledged,
    Nacked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayEnvelope {
    pub schema_version: SchemaVersion,
    pub message_id: RelayMessageId,
    pub destination: DestinationId,
    pub kind: RelayMessageKind,
    pub payload_schema: SchemaVersion,
    pub payload_json: serde_json::Value,
    pub payload_digest_hex: String,
    pub classification: Classification,
    pub releasability: Releasability,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub expires_at: OffsetDateTime,
    pub sender: ActorId,
    pub signature_hex: String,
    pub anti_replay_nonce_hex: String,
    pub delivery_state: DeliveryState,
    pub ack_state: AckState,
}

impl RelayEnvelope {
    pub fn schema() -> SchemaVersion { relay_schema() }
}
