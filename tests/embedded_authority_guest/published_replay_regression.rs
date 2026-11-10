// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The published-replay regression: the same three real owner processes, driven by a fixture built with the published product APIs.

use super::*;

mod fixture;
mod replay;

use replay::owner_replay;

#[tokio::test(flavor = "multi_thread")]
async fn published_reading_replay_and_same_schema_tamper_survive_three_real_owners() {
    owner_replay(false, false, false, false, false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn typed_decks_and_reading_survive_three_owners_and_refuse_metadata_tamper() {
    owner_replay(true, false, false, false, false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn ambient_view_and_kept_memory_survive_three_owners_and_refuse_source_tamper() {
    owner_replay(true, true, false, false, false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn shuffled_deck_survives_three_owners_and_refuses_orientation_tamper() {
    owner_replay(true, false, true, false, false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn recorded_situation_and_weather_survive_three_owners_and_refuse_tamper() {
    owner_replay(true, false, true, true, false).await;
}
#[tokio::test(flavor = "multi_thread")]
async fn generated_inferences_survive_three_owners_and_refuse_input_and_output_tamper() {
    owner_replay(true, false, true, false, true).await;
}
