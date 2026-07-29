//! Immutable audit event.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use time::OffsetDateTime;

use crate::ids::{ActorId, AuditEventId};
use crate::version::{audit_schema, SchemaVersion};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventType {
    RuntimeStateChanged,
    AdapterConnected,
    AdapterDisconnected,
    ObservationReceived,
    ObservationRejected,
    NormalizationApplied,
    TrackCreated,
    TrackUpdated,
    TrackRetired,
    CorrectionApplied,
    ModelActivated,
    ModelRolledBack,
    RelayQueued,
    RelayDelivered,
    RelayRejected,
    RelayReplayAttemptRejected,
    AlertRaised,
    AlertAcknowledged,
    ConfigurationValidated,
    ConfigurationActivated,
    ConfigurationRejected,
    ReplaySessionStarted,
    ReplaySessionEnded,
    ScopeBoundaryViolationAttempt,
    IntegrityViolation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub schema_version: SchemaVersion,
    pub event_id: AuditEventId,
    pub actor: ActorId,
    pub runtime_id: String,
    pub event_type: AuditEventType,
    pub target: String,
    pub before_ref: Option<String>,
    pub after_ref: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub occurred_at: OffsetDateTime,
    pub sequence: u64,
    pub correlation_id: Option<String>,
    pub integrity_digest_hex: String,
    pub security_context: String,
}

impl AuditEvent {
    pub fn schema() -> SchemaVersion {
        audit_schema()
    }

    /// Compute the canonical integrity digest for this event, chaining the
    /// digest of the previous event so tampering is detectable.
    pub fn compute_digest_hex(
        actor: &ActorId,
        runtime_id: &str,
        event_type: AuditEventType,
        target: &str,
        sequence: u64,
        occurred_at_unix_nanos: i128,
        previous_digest_hex: &str,
    ) -> String {
        let mut h = Sha256::new();
        h.update(actor.as_str().as_bytes());
        h.update(runtime_id.as_bytes());
        h.update(
            serde_json::to_string(&event_type)
                .unwrap_or_default()
                .as_bytes(),
        );
        h.update(target.as_bytes());
        h.update(sequence.to_be_bytes());
        h.update(occurred_at_unix_nanos.to_be_bytes());
        h.update(previous_digest_hex.as_bytes());
        hex::encode(h.finalize())
    }
}
