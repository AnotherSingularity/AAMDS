//! Newtyped identifiers. Prevents accidental cross-type identifier
//! confusion at compile time.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

macro_rules! newtype_id_uuid {
    ($name:ident, $prefix:literal) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub Uuid);
        impl $name {
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }
            pub fn as_uuid(&self) -> Uuid {
                self.0
            }
        }
        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }
        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{}:{}", $prefix, self.0)
            }
        }
    };
}

macro_rules! newtype_id_str {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub String);
        impl $name {
            pub fn new<S: Into<String>>(s: S) -> Self {
                Self(s.into())
            }
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

newtype_id_uuid!(ObservationId, "obs");
newtype_id_uuid!(TrackId, "trk");
newtype_id_uuid!(AlertId, "alt");
newtype_id_uuid!(RelayMessageId, "rly");
newtype_id_uuid!(AuditEventId, "aud");
newtype_id_uuid!(ConfigurationVersionId, "cfg");
newtype_id_uuid!(ModelVersionId, "mdl");

newtype_id_str!(SourceSystemId);
newtype_id_str!(SensorId);
newtype_id_str!(AdapterId);
newtype_id_str!(DestinationId);
newtype_id_str!(ActorId);
