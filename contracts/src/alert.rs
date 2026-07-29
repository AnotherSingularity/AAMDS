//! Operator alerts.
//!
//! Alert *recommended actions* are limited to system operation and review
//! (e.g. "acknowledge", "inspect adapter logs", "trigger replay"). Alerts
//! never carry any prohibited-concept recommendation — that is a
//! scope-boundary constraint enforced by `contracts::prohibited` and the
//! boundary scanner in `verification/scope-boundary`.

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::ids::AlertId;
use crate::version::{alert_schema, SchemaVersion};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertCategory {
    AdapterFailure,
    SensorDropout,
    ClockDrift,
    StorageDegraded,
    RelayDegraded,
    IntegrityViolation,
    ConfigurationInvalid,
    ModelUnavailable,
    ModelOutOfDistribution,
    ScopeBoundaryViolationAttempt,
    HighReleaseLatency,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertAcknowledgment {
    Unacknowledged,
    Acknowledged,
    Suppressed,
}

/// Recommended operator action, limited to system-operation review verbs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecommendedOperatorAction {
    Acknowledge,
    InspectAdapterLogs,
    InspectStorage,
    InspectRelayQueue,
    TriggerReplay,
    ReviewConfiguration,
    ReviewModelActivation,
    ContactMaintainer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub schema_version: SchemaVersion,
    pub alert_id: AlertId,
    pub severity: AlertSeverity,
    pub category: AlertCategory,
    pub affected_component: String,
    pub human_summary: String,
    pub machine_reason: String,
    #[serde(with = "time::serde::rfc3339")]
    pub first_occurrence: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub last_occurrence: OffsetDateTime,
    pub occurrence_count: u32,
    pub acknowledgment: AlertAcknowledgment,
    pub recommended_action: RecommendedOperatorAction,
}

impl Alert {
    pub fn schema() -> SchemaVersion {
        alert_schema()
    }
}
