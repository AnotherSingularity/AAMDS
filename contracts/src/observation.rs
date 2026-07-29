//! Raw and normalized observations.

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::coords::{CanonicalPosition, CoordinateQuality, CoordinateReference};
use crate::ids::{AdapterId, ObservationId, SensorId, SourceSystemId};
use crate::provenance::{Integrity, Provenance, TransformationStep};
use crate::time_kind::TimeQuality;
use crate::uncertainty::{Confidence, PositionUncertainty, VelocityUncertainty};
use crate::unknown::Known;
use crate::version::{normalized_schema, observation_schema, SchemaVersion};

/// A single raw sensor measurement, as reported by an adapter before
/// canonicalisation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawObservation {
    pub schema_version: SchemaVersion,
    pub observation_id: ObservationId,
    pub source_system_id: SourceSystemId,
    pub sensor_id: SensorId,
    pub adapter_id: AdapterId,
    pub adapter_version: String,
    #[serde(with = "time::serde::rfc3339")]
    pub source_timestamp: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub receive_timestamp: OffsetDateTime,
    pub sequence_number: u64,
    pub coordinate_reference: CoordinateReference,
    pub measurement: RawMeasurement,
    pub measurement_uncertainty: Known<PositionUncertainty>,
    pub velocity_uncertainty: Known<VelocityUncertainty>,
    pub classification_claims: Vec<String>,
    pub quality_indicators: QualityIndicators,
    pub integrity: Integrity,
    pub raw_source_blob_digest: Option<String>,
}

impl RawObservation {
    pub fn schema() -> SchemaVersion {
        observation_schema()
    }
}

/// The measurement payload as reported. Kept in native units so the
/// normalization layer can record the exact transformation applied.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RawMeasurement {
    Position {
        x: f64,
        y: f64,
        z: f64,
    },
    RangeBearingElevation {
        range_m: f64,
        bearing_deg: f64,
        elevation_deg: f64,
    },
    /// A track-declared position with vendor semantics — the normalization
    /// layer must interpret it via the adapter capability declaration.
    VendorNative {
        payload_json: String,
    },
}

/// Sensor-supplied quality hints. Adapters populate what they know; missing
/// fields become `Known::Unknown` — never silently zero.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityIndicators {
    pub time_quality: TimeQuality,
    pub declared_snr_db: Known<f64>,
    pub declared_confidence: Known<Confidence>,
}

/// A raw observation after normalization to Aeon's canonical form.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedObservation {
    pub schema_version: SchemaVersion,
    pub source_observation: ObservationId,
    #[serde(with = "time::serde::rfc3339")]
    pub canonical_timestamp: OffsetDateTime,
    pub time_quality: TimeQuality,
    pub position: Known<CanonicalPosition>,
    pub position_uncertainty: Known<PositionUncertainty>,
    pub velocity_uncertainty: Known<VelocityUncertainty>,
    pub coordinate_quality: CoordinateQuality,
    pub classification_claims: Vec<String>,
    pub declared_confidence: Known<Confidence>,
    pub validation_notes: Vec<String>,
    pub transformation_chain: Vec<TransformationStep>,
    pub normalization_version: String,
    pub provenance: Provenance,
}

impl NormalizedObservation {
    pub fn schema() -> SchemaVersion {
        normalized_schema()
    }
}
