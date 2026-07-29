//! Destination and release policy.

use aeon_contracts::ids::DestinationId;
use aeon_contracts::relay::RelayMessageKind;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DestinationPolicy {
    pub destination: DestinationId,
    /// Kinds this destination is authorized to receive. Must be a subset
    /// of `allowlist::ALLOWED_KINDS`.
    pub allowed_kinds: Vec<RelayMessageKind>,
    /// Allowed classification labels (opaque string comparison).
    pub allowed_classification_labels: Vec<String>,
    /// Communities this destination belongs to. A relay envelope is only
    /// authorised when its releasability includes at least one of these.
    pub communities: Vec<String>,
    /// Maximum payload size accepted by this destination, in bytes.
    pub max_payload_bytes: usize,
    /// The public key the destination expects signatures to be verifiable
    /// against — carried opaquely; verification is out of scope for the
    /// baseline (sponsor-supplied HSM / KMS integration).
    pub public_key_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayPolicy {
    pub destinations: HashMap<String, DestinationPolicy>,
    /// One-way export mode: if true, no acknowledgment is accepted from
    /// destinations (used for diode / cross-domain export gateways).
    pub one_way_export: bool,
    /// Retention (seconds) for the anti-replay nonce cache.
    pub anti_replay_ttl_seconds: u32,
    /// Rate limit (messages per minute per destination).
    pub max_messages_per_minute_per_destination: u32,
    /// Maximum queue depth before dead-lettering.
    pub max_queue_depth_per_destination: u32,
}

impl RelayPolicy {
    pub fn destination(&self, id: &DestinationId) -> Option<&DestinationPolicy> {
        self.destinations.get(id.as_str())
    }
}
