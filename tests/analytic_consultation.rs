// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![cfg(feature = "analytic-ephemeris")]

use cleromancy::{
    AnalyticEphemerisAdapter, AstrologyMoment, CleromancyHost, Consultation,
    calculate_with_adapter,
};
use muniment::RedbBackend;

/// The calculated-chart path needs no kernel, so this runs in the ordinary
/// suite rather than as an ignored test.
#[test]
fn calculated_chart_becomes_durable_consultation_truth() {
    let adapter = AnalyticEphemerisAdapter::new();
    let moment = AstrologyMoment::at("2024-04-08T18:00:00Z", 32_776_700, -96_797_000);
    let chart = calculate_with_adapter(&adapter, &moment).expect("calculate chart");

    let temporary = tempfile::tempdir().expect("temporary local store");
    let backend =
        RedbBackend::open(temporary.path().join("cleromancy.redb")).expect("open local store");
    let host = pollster::block_on(CleromancyHost::open(backend.clone())).expect("open host");
    let mut consultation = Consultation::new(host);
    let facts_digest = pollster::block_on(consultation.save_calculated_astrology_chart_at(
        chart.clone(),
        1_000,
        1,
    ))
    .expect("save calculated chart");
    drop(consultation);

    let reopened =
        Consultation::new(pollster::block_on(CleromancyHost::open(backend)).expect("reopen host"));
    let catalog = reopened.catalog().expect("load durable catalog");
    assert_eq!(catalog.astrology_facts.len(), 1);
    assert_eq!(catalog.astrology_facts[0].digest(), facts_digest);
    assert_eq!(
        reopened
            .host()
            .astrology_chart_for_digest(&chart.digest())
            .expect("load stored chart"),
        chart
    );
}
