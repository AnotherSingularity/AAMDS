//! Health snapshot builder.

use aeon_contracts::health::{
    AdapterHealth, RelayHealth, RuntimeState, SensorFeedHealth, StorageHealth, SystemHealth,
    TimeSourceHealth,
};
use aeon_contracts::version::health_schema;
use time::OffsetDateTime;

/// Snapshot builder — a plain-data builder pattern so the runtime can
/// construct a well-formed `SystemHealth` without touching the wire type
/// directly.
#[derive(Debug, Default)]
pub struct HealthBuilder {
    pub adapters: Vec<AdapterHealth>,
    pub sensor_feeds: Vec<SensorFeedHealth>,
    pub relay: Vec<RelayHealth>,
    pub active_model_ids: Vec<String>,
    pub degraded_capabilities: Vec<String>,
    pub storage: Option<StorageHealth>,
    pub time_source: Option<TimeSourceHealth>,
}

impl HealthBuilder {
    pub fn build(
        self,
        runtime_state: RuntimeState,
        configuration_version: String,
        build_id: String,
    ) -> SystemHealth {
        SystemHealth {
            schema_version: health_schema(),
            captured_at: OffsetDateTime::now_utc(),
            runtime_state,
            adapters: self.adapters,
            sensor_feeds: self.sensor_feeds,
            storage: self.storage.unwrap_or(StorageHealth::Healthy),
            relay: self.relay,
            time_source: self.time_source.unwrap_or(TimeSourceHealth::LocalOnly),
            active_model_ids: self.active_model_ids,
            configuration_version,
            build_id,
            degraded_capabilities: self.degraded_capabilities,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_builder_defaults_are_conservative() {
        let h =
            HealthBuilder::default().build(RuntimeState::Ready, "cfg-0".into(), "build-0".into());
        assert_eq!(h.runtime_state, RuntimeState::Ready);
        assert_eq!(h.storage, StorageHealth::Healthy);
        assert_eq!(h.time_source, TimeSourceHealth::LocalOnly);
        assert!(h.adapters.is_empty());
    }
}
