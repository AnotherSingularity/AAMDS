//! Track and TrackUpdate contracts.

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::coords::CanonicalPosition;
use crate::ids::{ObservationId, SensorId, SourceSystemId, TrackId};
use crate::provenance::{Integrity, Provenance};
use crate::uncertainty::{Confidence, PositionUncertainty, VelocityUncertainty};
use crate::unknown::Known;
use crate::version::{track_schema, track_update_schema, SchemaVersion};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrackStatus {
    Tentative,
    Active,
    Coasting,
    Stale,
    Retired,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KinematicState {
    pub position: Known<CanonicalPosition>,
    pub velocity_east_ms: Known<f64>,
    pub velocity_north_ms: Known<f64>,
    pub velocity_up_ms: Known<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StateUncertainty {
    pub position: Known<PositionUncertainty>,
    pub velocity: Known<VelocityUncertainty>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClassificationHypothesis {
    pub label: String,
    pub confidence: Confidence,
    pub supporting_source_count: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceContribution {
    pub source_system: SourceSystemId,
    pub sensor: SensorId,
    pub weight: f64,
    pub last_contributed_at: OffsetDateTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictState {
    None,
    ClassificationConflict,
    PositionConflict,
    MultipleConflicts,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FreshnessState {
    Fresh,
    Aging,
    Stale,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub schema_version: SchemaVersion,
    pub track_id: TrackId,
    pub status: TrackStatus,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub last_updated_at: OffsetDateTime,
    pub kinematic_state: KinematicState,
    pub state_uncertainty: StateUncertainty,
    pub classification_hypotheses: Vec<ClassificationHypothesis>,
    pub confidence: Confidence,
    pub source_contributions: Vec<SourceContribution>,
    pub conflict_state: ConflictState,
    pub freshness_state: FreshnessState,
    pub integrity: Integrity,
    pub provenance_root: Provenance,
    pub track_algorithm_version: String,
}

impl Track {
    pub fn schema() -> SchemaVersion {
        track_schema()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CorrelationRationale {
    NewTrack,
    AssociatedByGate,
    AssociatedByClassification,
    ManualOperatorAssociation,
    NoAssociation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackUpdate {
    pub schema_version: SchemaVersion,
    pub track_id: TrackId,
    #[serde(with = "time::serde::rfc3339")]
    pub processing_timestamp: OffsetDateTime,
    pub deterministic_sequence: u64,
    pub prior_state: Option<Box<Track>>,
    pub new_state: Track,
    pub contributing_observations: Vec<ObservationId>,
    pub rejected_observations: Vec<(ObservationId, String)>,
    pub correlation_rationale: CorrelationRationale,
    pub confidence_delta: f64,
    pub uncertainty_delta_m: f64,
    pub conflict_indicators: Vec<String>,
    pub algorithm_version: String,
    pub model_references: Vec<String>,
}

impl TrackUpdate {
    pub fn schema() -> SchemaVersion {
        track_update_schema()
    }
}
