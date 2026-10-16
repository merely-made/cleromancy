// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Read-only pre-C5 compatibility vector emitter; no store or UI.
use cleromancy::{Candidate, DerivedSelection, Field, ReadingEngine, UNIFORM_DIE_RULE, a0_fixture};
fn main() {
    let (context, field) = a0_fixture();
    let die = Field::new(
        "fixture.derived-die/v1",
        UNIFORM_DIE_RULE,
        (1..=6).map(|face| {
            Candidate::new(
                format!("face-{face}"),
                format!("Face {face}"),
                format!("Face {face} is the disclosed result."),
            )
        }),
    );
    let mut readings = vec![ReadingEngine::calculate(&context, &field).unwrap()];
    for (seed, domain, field) in [
        ("public-fixture-seed", "cleromancy.test/derived", &field),
        ("roll-2026-08-08", "cleromancy.test/die", &die),
        ("isocosm-compatibility", "generator-choice", &field),
    ] {
        readings.push(
            ReadingEngine::derive(
                &context,
                field,
                &DerivedSelection::new(seed, domain).unwrap(),
            )
            .unwrap(),
        );
    }
    println!("{}", serde_json::to_string_pretty(&readings).unwrap());
}
