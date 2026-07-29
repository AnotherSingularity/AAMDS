//! System health snapshot.

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::ids::{AdapterId, DestinationId, SensorId};
use crate::version::{health_schema, SchemaVersion};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeState {
    Uninitialized,
    ValidatingConfiguration,
    Starting,
    Ready,
    Degraded,
    Paused,
    ShuttingDown,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterHealth {
    pub adapter: AdapterId,
    pub connected: bool,
    pub last_observation_at: Option<OffsetDateTime>,
    pub error_count_last_hour: u64,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorFeedHealth {
    pub sensor: SensorId,
    pub observations_last_minute: u64,
    pub dropouts_last_hour: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageHealth {
    Healthy,
    LatencyElevated,
    ReadOnly,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayHealth {
    pub destination: DestinationId,
    pub queued: u64,
    pub in_flight: u64,
    pub dead_lettered: u64,
    pub last_delivered_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeSourceHealth {
    Disciplined,
    LocalOnly,
    Drifting,
    Unavailable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    pub schema_version: SchemaVersion,
    #[serde(with = "time::serde::rfc3339")]
    pub captured_at: OffsetDateTime,
    pub runtime_state: RuntimeState,
    pub adapters: Vec<AdapterHealth>,
    pub sensor_feeds: Vec<SensorFeedHealth>,
    pub storage: StorageHealth,
    pub relay: Vec<RelayHealth>,
    pub time_source: TimeSourceHealth,
    pub active_model_ids: Vec<String>,
    pub configuration_version: String,
    pub build_id: String,
    pub degraded_capabilities: Vec<String>,
}

impl SystemHealth {
    pub fn schema() -> SchemaVersion {
        health_schema()
    }
}
