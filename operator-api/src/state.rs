//! Shared state for the read-oriented API.

use aeon_contracts::alert::Alert;
use aeon_contracts::health::SystemHealth;
use aeon_contracts::track::Track;
use aeon_track_management::TrackEngine;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct ApiState {
    pub engine: Arc<Mutex<TrackEngine>>,
    pub health: Arc<Mutex<SystemHealth>>,
    pub alerts: Arc<Mutex<Vec<Alert>>>,
    pub build_id: String,
    pub configuration_version: String,
    pub runtime_id: String,
    pub replay_mode: Arc<Mutex<bool>>,
}

impl ApiState {
    pub fn tracks(&self) -> Vec<Track> {
        self.engine.lock().unwrap().tracks().cloned().collect()
    }
}
