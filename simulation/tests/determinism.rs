//! Deterministic-replay proof: same inputs → same trace digest.

use aeon_simulation::determinism::trace_digest;
use aeon_simulation::pipeline::run_pipeline;
use aeon_simulation::scenarios::{crossing_two_tracks, single_clean_track};
use time::OffsetDateTime;

fn far_future() -> OffsetDateTime {
    // Fixed clock so latency guard never trips.
    OffsetDateTime::UNIX_EPOCH + time::Duration::days(365 * 10)
}

#[test]
fn single_clean_track_is_deterministic_across_runs() {
    let mut s1 = single_clean_track();
    let out1 = run_pipeline(
        &s1.manifest.name,
        &mut s1.adapter,
        s1.policy.clone(),
        far_future,
    )
    .unwrap();
    let mut s2 = single_clean_track();
    let out2 = run_pipeline(
        &s2.manifest.name,
        &mut s2.adapter,
        s2.policy.clone(),
        far_future,
    )
    .unwrap();
    assert_eq!(
        trace_digest(&out1),
        trace_digest(&out2),
        "same inputs must yield same trace"
    );
    assert!(!out1.updates.is_empty(), "scenario must produce updates");
}

#[test]
fn crossing_scenario_is_deterministic_across_runs() {
    let mut s1 = crossing_two_tracks();
    let out1 = run_pipeline(
        &s1.manifest.name,
        &mut s1.adapter,
        s1.policy.clone(),
        far_future,
    )
    .unwrap();
    let mut s2 = crossing_two_tracks();
    let out2 = run_pipeline(
        &s2.manifest.name,
        &mut s2.adapter,
        s2.policy.clone(),
        far_future,
    )
    .unwrap();
    assert_eq!(trace_digest(&out1), trace_digest(&out2));
}

#[test]
fn different_scenarios_produce_different_digests() {
    let mut a = single_clean_track();
    let out_a = run_pipeline(
        &a.manifest.name,
        &mut a.adapter,
        a.policy.clone(),
        far_future,
    )
    .unwrap();
    let mut b = crossing_two_tracks();
    let out_b = run_pipeline(
        &b.manifest.name,
        &mut b.adapter,
        b.policy.clone(),
        far_future,
    )
    .unwrap();
    assert_ne!(trace_digest(&out_a), trace_digest(&out_b));
}
