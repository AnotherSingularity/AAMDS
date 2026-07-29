//! Schema-version tags carried in every serialised contract.
//!
//! When a schema evolves in a compatible way, bump the minor version.
//! When it evolves in an incompatible way, bump the major version *and*
//! add a compatibility test.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaVersion {
    pub name: String,
    pub major: u16,
    pub minor: u16,
}

impl SchemaVersion {
    pub fn new<S: Into<String>>(name: S, major: u16, minor: u16) -> Self {
        Self {
            name: name.into(),
            major,
            minor,
        }
    }
}

/// Canonical versions for the shipped contracts. When you change a schema,
/// bump the constant here and update `docs/verification/TRACEABILITY_MATRIX.md`.
pub fn observation_schema() -> SchemaVersion {
    SchemaVersion::new("observation", 1, 0)
}
pub fn normalized_schema() -> SchemaVersion {
    SchemaVersion::new("normalized", 1, 0)
}
pub fn track_schema() -> SchemaVersion {
    SchemaVersion::new("track", 1, 0)
}
pub fn track_update_schema() -> SchemaVersion {
    SchemaVersion::new("track_update", 1, 0)
}
pub fn health_schema() -> SchemaVersion {
    SchemaVersion::new("health", 1, 0)
}
pub fn alert_schema() -> SchemaVersion {
    SchemaVersion::new("alert", 1, 0)
}
pub fn relay_schema() -> SchemaVersion {
    SchemaVersion::new("relay_envelope", 1, 0)
}
pub fn audit_schema() -> SchemaVersion {
    SchemaVersion::new("audit", 1, 0)
}
