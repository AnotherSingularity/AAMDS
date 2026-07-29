//! Provenance chain.
//!
//! Attached to every material output. Retains the source observation
//! identifiers, adapter identity, timestamps, transformation chain, and
//! integrity state so the audit log can reconstruct *how* a value came to be.

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::ids::{AdapterId, ObservationId, SensorId, SourceSystemId};

/// Integrity state of a value's chain of custody.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Integrity {
    /// Signature verified, source authenticated.
    Verified,
    /// Signature missing but source policy permits unsigned in this context.
    Unsigned,
    /// Signature check failed. Value is untrusted.
    SignatureInvalid { reason: String },
    /// Source could not be authenticated.
    SourceUnknown,
    /// Value derives from other values; integrity is the *minimum* of
    /// the contributing integrities.
    Derived { min_upstream: Box<Integrity> },
}

/// One step of a transformation chain (e.g. "reproject WGS84 -> ECEF v2").
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransformationStep {
    pub operation: String,
    pub version: String,
    #[serde(with = "time::serde::rfc3339")]
    pub applied_at: OffsetDateTime,
}

/// Provenance record attached to every normalized observation, track update,
/// alert, and relay envelope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Provenance {
    pub source_observations: Vec<ObservationId>,
    pub source_system: SourceSystemId,
    pub sensor: SensorId,
    pub adapter: AdapterId,
    pub adapter_version: String,
    #[serde(with = "time::serde::rfc3339")]
    pub receive_timestamp: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub source_timestamp: OffsetDateTime,
    pub normalization_version: String,
    pub coordinate_transformation_version: String,
    pub track_algorithm_version: Option<String>,
    pub model_versions: Vec<String>,
    pub configuration_version: String,
    pub build_id: String,
    pub processing_sequence: u64,
    pub integrity: Integrity,
    pub transformation_chain: Vec<TransformationStep>,
}
