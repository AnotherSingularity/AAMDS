//! Uncertainty representations.

use serde::{Deserialize, Serialize};

/// Positional uncertainty as a diagonal covariance in metres.
/// Off-diagonal components are supplied when known.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PositionUncertainty {
    /// Standard deviation in metres, east/north/up.
    pub sigma_east_m: f64,
    pub sigma_north_m: f64,
    pub sigma_up_m: f64,
}

impl PositionUncertainty {
    /// Aeon forbids negative sigma values. This helper is used by the
    /// contract validators and the property tests.
    pub fn is_valid(&self) -> bool {
        self.sigma_east_m.is_finite()
            && self.sigma_north_m.is_finite()
            && self.sigma_up_m.is_finite()
            && self.sigma_east_m >= 0.0
            && self.sigma_north_m >= 0.0
            && self.sigma_up_m >= 0.0
    }
}

/// Velocity uncertainty (m/s).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct VelocityUncertainty {
    pub sigma_east_ms: f64,
    pub sigma_north_ms: f64,
    pub sigma_up_ms: f64,
}

impl VelocityUncertainty {
    pub fn is_valid(&self) -> bool {
        [self.sigma_east_ms, self.sigma_north_ms, self.sigma_up_ms]
            .iter()
            .all(|v| v.is_finite() && *v >= 0.0)
    }
}

/// Confidence must be in `[0.0, 1.0]`. Any value outside that range
/// indicates a contract-validation failure.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Confidence(pub f64);

impl Confidence {
    pub fn new(v: f64) -> Option<Self> {
        if v.is_finite() && (0.0..=1.0).contains(&v) {
            Some(Self(v))
        } else {
            None
        }
    }
    pub fn get(&self) -> f64 {
        self.0
    }
}
