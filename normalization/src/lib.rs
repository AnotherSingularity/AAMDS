//! Canonicalisation of raw sensor observations.
//!
//! The normalization layer never overwrites source values. It produces a
//! [`NormalizedObservation`] that carries a canonical timestamp, a canonical
//! position (WGS84 geodetic), a normalised uncertainty, and the full
//! transformation chain that was applied.
//!
//! Every failure mode is a typed error and the rejected input is left
//! intact for the persistence layer to record.

#![forbid(unsafe_code)]

use aeon_contracts::coords::{CanonicalPosition, CoordinateQuality, CoordinateReference};
use aeon_contracts::observation::{NormalizedObservation, RawMeasurement, RawObservation};
use aeon_contracts::provenance::{Integrity, Provenance, TransformationStep};
use aeon_contracts::unknown::Known;
use aeon_contracts::version::normalized_schema;
use time::OffsetDateTime;

pub const NORMALIZATION_VERSION: &str = "aeon-normalization/1.0";
pub const COORD_TRANSFORM_VERSION: &str = "aeon-coords/1.0";

#[derive(Debug, thiserror::Error)]
pub enum NormalizationError {
    #[error("unknown coordinate frame")]
    UnknownCoordinateFrame,
    #[error(
        "vendor-native payload requires a coordinate-transform capability that was not declared"
    )]
    UnsupportedVendorPayload,
    #[error("latitude out of range: {0}")]
    LatOutOfRange(f64),
    #[error("longitude out of range: {0}")]
    LonOutOfRange(f64),
    #[error("negative sigma detected")]
    NegativeSigma,
    #[error("timestamp is not finite")]
    NonFiniteTimestamp,
}

/// Normalise a single raw observation into its canonical form.
///
/// * Time: canonical timestamp = source timestamp (UTC). If the raw
///   observation lacks a trustworthy source timestamp, callers should
///   substitute receive time *and* mark the time quality accordingly
///   before calling.
/// * Coordinates: only `Wgs84Geodetic` is fully supported at this baseline;
///   `Ecef` is validated but marked `CoordinateQuality::Degraded` because
///   this baseline does not ship an ECEF→geodetic implementation.
///   `RangeBearingElevation` requires sensor-local origin metadata that
///   is not carried on the raw payload in this baseline — such payloads
///   yield `CoordinateQuality::OutOfDeclaredDomain` and a `Known::Unknown`
///   canonical position.
/// * Original values are preserved on the returned `NormalizedObservation`
///   and on the transformation chain.
pub fn normalize(
    raw: &RawObservation,
    mut provenance: Provenance,
) -> Result<NormalizedObservation, NormalizationError> {
    let mut chain = Vec::<TransformationStep>::new();
    let mut validation_notes = Vec::<String>::new();
    let now = OffsetDateTime::now_utc();

    // ------- Coordinate normalisation -------
    let (position, coordinate_quality) = match (raw.coordinate_reference, &raw.measurement) {
        (
            CoordinateReference::Wgs84Geodetic,
            RawMeasurement::Position {
                x: lon,
                y: lat,
                z: alt,
            },
        ) => {
            let lat = *lat;
            let lon = *lon;
            let alt = *alt;
            if !(-90.0..=90.0).contains(&lat) {
                return Err(NormalizationError::LatOutOfRange(lat));
            }
            if !(-180.0..=180.0).contains(&lon) {
                return Err(NormalizationError::LonOutOfRange(lon));
            }
            chain.push(TransformationStep {
                operation: "identity_wgs84_geodetic".into(),
                version: COORD_TRANSFORM_VERSION.into(),
                applied_at: now,
            });
            (
                Known::Known {
                    value: CanonicalPosition {
                        latitude_deg: lat,
                        longitude_deg: lon,
                        altitude_m: alt,
                    },
                },
                CoordinateQuality::Good,
            )
        }
        (CoordinateReference::Ecef, RawMeasurement::Position { .. }) => {
            chain.push(TransformationStep {
                operation: "ecef_to_wgs84_geodetic_unavailable".into(),
                version: COORD_TRANSFORM_VERSION.into(),
                applied_at: now,
            });
            validation_notes.push("ECEF conversion not implemented at this baseline".into());
            (
                Known::Unavailable {
                    reason: "ecef_conversion_unimplemented".into(),
                },
                CoordinateQuality::Degraded,
            )
        }
        (CoordinateReference::RangeBearingElevation, _) => {
            validation_notes
                .push("range-bearing-elevation requires sensor-local origin metadata".into());
            (Known::Unknown, CoordinateQuality::OutOfDeclaredDomain)
        }
        (CoordinateReference::Enu { .. }, _) => {
            validation_notes.push("ENU frame accepted only via vendor-native adapter path".into());
            (Known::Unknown, CoordinateQuality::OutOfDeclaredDomain)
        }
        (CoordinateReference::Unknown, _) => {
            return Err(NormalizationError::UnknownCoordinateFrame);
        }
        (_, RawMeasurement::VendorNative { .. }) => {
            return Err(NormalizationError::UnsupportedVendorPayload);
        }
        (_, RawMeasurement::RangeBearingElevation { .. }) => {
            validation_notes.push(
                "RBE measurement without RBE frame — treated as out of declared domain".into(),
            );
            (Known::Unknown, CoordinateQuality::OutOfDeclaredDomain)
        }
    };

    // ------- Uncertainty normalisation -------
    let position_uncertainty = match &raw.measurement_uncertainty {
        Known::Known { value } if !value.is_valid() => {
            return Err(NormalizationError::NegativeSigma);
        }
        other => other.clone(),
    };
    let velocity_uncertainty = match &raw.velocity_uncertainty {
        Known::Known { value } if !value.is_valid() => {
            return Err(NormalizationError::NegativeSigma);
        }
        other => other.clone(),
    };

    // ------- Integrity propagation -------
    provenance.integrity = match raw.integrity.clone() {
        Integrity::Verified => Integrity::Verified,
        Integrity::Unsigned => Integrity::Unsigned,
        Integrity::SignatureInvalid { reason } => Integrity::Derived {
            min_upstream: Box::new(Integrity::SignatureInvalid { reason }),
        },
        Integrity::SourceUnknown => Integrity::Derived {
            min_upstream: Box::new(Integrity::SourceUnknown),
        },
        Integrity::Derived { min_upstream } => Integrity::Derived { min_upstream },
    };

    chain.push(TransformationStep {
        operation: "normalise_v1".into(),
        version: NORMALIZATION_VERSION.into(),
        applied_at: now,
    });

    Ok(NormalizedObservation {
        schema_version: normalized_schema(),
        source_observation: raw.observation_id,
        canonical_timestamp: raw.source_timestamp,
        time_quality: raw.quality_indicators.time_quality,
        position,
        position_uncertainty,
        velocity_uncertainty,
        coordinate_quality,
        classification_claims: raw.classification_claims.clone(),
        declared_confidence: raw.quality_indicators.declared_confidence.clone(),
        validation_notes,
        transformation_chain: chain,
        normalization_version: NORMALIZATION_VERSION.into(),
        provenance,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use aeon_contracts::ids::*;
    use aeon_contracts::observation::{QualityIndicators, RawMeasurement, RawObservation};
    use aeon_contracts::provenance::{Integrity, Provenance};
    use aeon_contracts::time_kind::TimeQuality;
    use aeon_contracts::uncertainty::PositionUncertainty;
    use aeon_contracts::version::observation_schema;

    fn raw_pos(lat: f64, lon: f64) -> RawObservation {
        RawObservation {
            schema_version: observation_schema(),
            observation_id: ObservationId::new(),
            source_system_id: SourceSystemId::new("s"),
            sensor_id: SensorId::new("r"),
            adapter_id: AdapterId::new("a"),
            adapter_version: "1".into(),
            source_timestamp: OffsetDateTime::UNIX_EPOCH,
            receive_timestamp: OffsetDateTime::UNIX_EPOCH,
            sequence_number: 1,
            coordinate_reference: CoordinateReference::Wgs84Geodetic,
            measurement: RawMeasurement::Position {
                x: lon,
                y: lat,
                z: 0.0,
            },
            measurement_uncertainty: Known::Known {
                value: PositionUncertainty {
                    sigma_east_m: 1.0,
                    sigma_north_m: 1.0,
                    sigma_up_m: 1.0,
                },
            },
            velocity_uncertainty: Known::Unknown,
            classification_claims: vec![],
            quality_indicators: QualityIndicators {
                time_quality: TimeQuality::Disciplined,
                declared_snr_db: Known::Unknown,
                declared_confidence: Known::Unknown,
            },
            integrity: Integrity::Verified,
            raw_source_blob_digest: None,
        }
    }

    fn prov() -> Provenance {
        Provenance {
            source_observations: vec![],
            source_system: SourceSystemId::new("s"),
            sensor: SensorId::new("r"),
            adapter: AdapterId::new("a"),
            adapter_version: "1".into(),
            receive_timestamp: OffsetDateTime::UNIX_EPOCH,
            source_timestamp: OffsetDateTime::UNIX_EPOCH,
            normalization_version: NORMALIZATION_VERSION.into(),
            coordinate_transformation_version: COORD_TRANSFORM_VERSION.into(),
            track_algorithm_version: None,
            model_versions: vec![],
            configuration_version: "cfg-1".into(),
            build_id: "b-1".into(),
            processing_sequence: 1,
            integrity: Integrity::Verified,
            transformation_chain: vec![],
        }
    }

    #[test]
    fn wgs84_position_normalises_to_canonical() {
        let r = raw_pos(40.0, -74.0);
        let n = normalize(&r, prov()).unwrap();
        match n.position {
            Known::Known { value } => {
                assert_eq!(value.latitude_deg, 40.0);
                assert_eq!(value.longitude_deg, -74.0);
            }
            other => panic!("expected Known::Known, got {other:?}"),
        }
        assert_eq!(n.coordinate_quality, CoordinateQuality::Good);
        assert!(n.transformation_chain.len() >= 2);
    }

    #[test]
    fn latitude_out_of_range_is_rejected() {
        let r = raw_pos(100.0, 0.0);
        assert!(matches!(
            normalize(&r, prov()),
            Err(NormalizationError::LatOutOfRange(_))
        ));
    }

    #[test]
    fn negative_sigma_is_rejected() {
        let mut r = raw_pos(0.0, 0.0);
        r.measurement_uncertainty = Known::Known {
            value: PositionUncertainty {
                sigma_east_m: -1.0,
                sigma_north_m: 1.0,
                sigma_up_m: 1.0,
            },
        };
        assert!(matches!(
            normalize(&r, prov()),
            Err(NormalizationError::NegativeSigma)
        ));
    }

    #[test]
    fn ecef_input_is_marked_degraded_not_silently_defaulted() {
        let mut r = raw_pos(0.0, 0.0);
        r.coordinate_reference = CoordinateReference::Ecef;
        let n = normalize(&r, prov()).unwrap();
        assert_eq!(n.coordinate_quality, CoordinateQuality::Degraded);
        assert!(matches!(n.position, Known::Unavailable { .. }));
    }

    #[test]
    fn unknown_frame_is_rejected() {
        let mut r = raw_pos(0.0, 0.0);
        r.coordinate_reference = CoordinateReference::Unknown;
        assert!(matches!(
            normalize(&r, prov()),
            Err(NormalizationError::UnknownCoordinateFrame)
        ));
    }

    #[test]
    fn integrity_derives_downstream() {
        let mut r = raw_pos(0.0, 0.0);
        r.integrity = Integrity::SignatureInvalid {
            reason: "bad".into(),
        };
        let n = normalize(&r, prov()).unwrap();
        match n.provenance.integrity {
            Integrity::Derived { min_upstream } => {
                assert!(matches!(*min_upstream, Integrity::SignatureInvalid { .. }));
            }
            other => panic!("expected derived, got {other:?}"),
        }
    }
}
