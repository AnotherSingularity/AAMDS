//! Replay adapter — reads recorded observations from a JSON-lines file
//! and re-emits them in-order. Used by deterministic replay.

use aeon_contracts::observation::RawObservation;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use crate::adapter::{AdapterDiagnostic, AdapterError, SensorAdapter};
use crate::capability::AdapterCapability;

#[derive(Debug)]
pub struct ReplayAdapter {
    path: PathBuf,
    reader: Option<BufReader<File>>,
    line_buf: String,
    forwarded: u64,
    rejected: u64,
    last_seq: Option<u64>,
    last_time_ns: Option<i128>,
    connected: bool,
    /// If true, an observation whose timestamp is older than the previous
    /// observation's timestamp yields `AdapterError::ClockRollback`.
    pub reject_clock_rollback: bool,
    /// If true, a repeated sequence number yields `AdapterError::DuplicateSequence`.
    pub reject_duplicate_sequence: bool,
}

impl ReplayAdapter {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            reader: None,
            line_buf: String::new(),
            forwarded: 0,
            rejected: 0,
            last_seq: None,
            last_time_ns: None,
            connected: false,
            reject_clock_rollback: true,
            reject_duplicate_sequence: true,
        }
    }
}

impl SensorAdapter for ReplayAdapter {
    fn capability(&self) -> AdapterCapability {
        AdapterCapability {
            name: "aeon-replay".into(),
            adapter_version: env!("CARGO_PKG_VERSION").into(),
            supported_coordinate_frames: vec![
                aeon_contracts::coords::CoordinateReference::Wgs84Geodetic,
                aeon_contracts::coords::CoordinateReference::Ecef,
            ],
            supplies_velocity: true,
            supplies_classification: true,
            max_observations_per_second: u32::MAX,
            supports_backpressure: true,
            supports_reconnect: true,
            supports_signed_source: true,
        }
    }

    fn validate_configuration(&self) -> Result<(), AdapterError> {
        if !self.path.exists() {
            return Err(AdapterError::InvalidConfiguration(format!(
                "replay path does not exist: {}",
                self.path.display()
            )));
        }
        Ok(())
    }

    fn connect(&mut self) -> Result<(), AdapterError> {
        let f = File::open(&self.path)?;
        self.reader = Some(BufReader::new(f));
        self.connected = true;
        Ok(())
    }

    fn next_observation(&mut self) -> Result<Option<RawObservation>, AdapterError> {
        let Some(r) = self.reader.as_mut() else {
            return Err(AdapterError::Connect("not connected".into()));
        };
        loop {
            self.line_buf.clear();
            let n = r.read_line(&mut self.line_buf)?;
            if n == 0 {
                return Ok(None);
            }
            let line = self.line_buf.trim();
            if line.is_empty() {
                continue;
            }
            let obs: RawObservation = match serde_json::from_str(line) {
                Ok(o) => o,
                Err(e) => {
                    self.rejected += 1;
                    return Err(AdapterError::Malformed(e.to_string()));
                }
            };
            if self.reject_duplicate_sequence {
                if let Some(prev) = self.last_seq {
                    if obs.sequence_number == prev {
                        self.rejected += 1;
                        return Err(AdapterError::DuplicateSequence(prev));
                    }
                }
            }
            if self.reject_clock_rollback {
                if let Some(prev_ns) = self.last_time_ns {
                    let now_ns = obs.source_timestamp.unix_timestamp_nanos();
                    if now_ns < prev_ns {
                        self.rejected += 1;
                        return Err(AdapterError::ClockRollback {
                            seen_ns: prev_ns,
                            received_ns: now_ns,
                        });
                    }
                }
            }
            self.last_seq = Some(obs.sequence_number);
            self.last_time_ns = Some(obs.source_timestamp.unix_timestamp_nanos());
            self.forwarded += 1;
            return Ok(Some(obs));
        }
    }

    fn diagnostic(&self) -> AdapterDiagnostic {
        AdapterDiagnostic {
            connected: self.connected,
            observations_forwarded: self.forwarded,
            observations_rejected: self.rejected,
            last_error: None,
            last_sequence: self.last_seq,
        }
    }

    fn shutdown(&mut self) -> Result<(), AdapterError> {
        self.reader = None;
        self.connected = false;
        Ok(())
    }
}
