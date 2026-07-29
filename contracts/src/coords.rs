//! Coordinate reference frames and canonical geodetic representation.

use serde::{Deserialize, Serialize};

/// The frame a raw observation was reported in. Aeon accepts several and
/// normalizes to [`CanonicalPosition`] (WGS84 geodetic + geodetic-height).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoordinateReference {
    /// WGS84 geodetic (lat, lon, height above ellipsoid).
    Wgs84Geodetic,
    /// Earth-Centred, Earth-Fixed (X, Y, Z).
    Ecef,
    /// Local East-North-Up relative to a declared origin.
    Enu {
        origin_lat_deg: f64,
        origin_lon_deg: f64,
        origin_alt_m: f64,
    },
    /// Sensor-local range-bearing-elevation.
    RangeBearingElevation,
    /// Unknown frame — must be rejected by normalization.
    Unknown,
}

/// The canonical representation Aeon uses internally.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CanonicalPosition {
    pub latitude_deg: f64,
    pub longitude_deg: f64,
    pub altitude_m: f64,
}

/// Coordinate-quality state used to qualify normalization output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoordinateQuality {
    Good,
    Degraded,
    OutOfDeclaredDomain,
    UnknownFrame,
}
