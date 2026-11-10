// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The published-product fixture and the carrier-request helpers the replay uses.

use super::*;
use cleromancy::{CleromancyHost, Consultation, ContextDraft, SelectionMode};
use mere_overmap::wire::{self as session_item, SessionGraphV1};
use muniment::MemoryBackend;
use serde_json::Value;
use std::sync::Mutex;

struct PublicEntropy(u64);
impl cleromancy::moirai::clotho::EntropySource for PublicEntropy {
    fn next_u64(&mut self) -> Result<u64, cleromancy::ReadingError> {
        self.0 += 1;
        Ok(self.0)
    }
}

// The historical published package exposes graph() but not graph_mut().
// Use the existing thread-local observer only around this synchronous,
// memory-only product fixture. RAII clears it before any owner I/O or replay.
// This uses public APIs supported by both 2851 and the later portable C.
struct ClearCapture;
impl Drop for ClearCapture {
    fn drop(&mut self) {
        mere::kernel::graph::set_captured_delta_hook(None);
    }
}

pub(super) fn product_fixture(
    typed_decks: bool,
    ambient: bool,
    shuffled: bool,
    situation: bool,
    inference: bool,
) -> (Vec<CapturedDelta>, Value, cleromancy::Reading) {
    let captured = Arc::new(Mutex::new(Vec::new()));
    let observed = captured.clone();
    mere::kernel::graph::set_captured_delta_hook(Some(Arc::new(move |delta| {
        observed.lock().unwrap().push(delta.clone());
    })));
    let clear = ClearCapture;
    let (graph, reading) = pollster::block_on(async {
        // Historical 2851 persists only to this transient MemoryBackend.
        // No snapshot is installed in the resident: only captured edits cross its door.
        let mut consultation = Consultation::new(CleromancyHost::empty(MemoryBackend::new()));
        let field = if typed_decks {
            let fields = consultation.install_typed_tarot_at(1).await.unwrap();
            assert_eq!(fields.len(), 2);
            fields[0].clone()
        } else {
            consultation.install_builtin_tarot_at(1).await.unwrap()
        };
        let context = consultation
            .save_context_at(
                ContextDraft::new(
                    "Published owner replay",
                    "Which boundary must survive a new owner?",
                    "ownership",
                ),
                2,
            )
            .await
            .unwrap();
        let detail = consultation
            .read_at_with_entropy(
                &context,
                &field,
                SelectionMode::Calculated,
                3_000,
                3,
                &mut PublicEntropy(0xc1),
            )
            .await
            .unwrap();
        assert_eq!(detail.readings.len(), 1);
        assert_eq!(detail.readings[0].receipt.mode, SelectionMode::Calculated);
        let mut host = consultation.into_host();
        if ambient {
            let context = host.ambient_context(&detail.session.id, "focus").unwrap();
            let item = context
                .data
                .items
                .iter()
                .find(|i| i.id.starts_with("card:"))
                .unwrap()
                .id
                .clone();
            host.examine_ambient(&detail.session.id, "focus", &item, false, 4000)
                .unwrap();
            host.examine_ambient(&detail.session.id, "focus", &item, true, 5000)
                .unwrap();
            let mut view = context.view;
            view.snapshot.recipe.definition.arrangement.kind = "grid.ordered".into();
            view.snapshot.selected_occurrence = Some(item.clone());
            view.roles.insert(
                item.clone(),
                serde_json::from_value(serde_json::json!("pinned")).unwrap(),
            );
            view.foreground.insert(item.clone());
            view.positions.insert(item, [123000, 456000]);
            view.reduced_motion = true;
            host.insert_ambient_view(&view).unwrap();
        }
        let reading = if shuffled {
            let context = host.context_for_digest(&context).unwrap();
            let field = host.field_for_digest(&field).unwrap();
            let drawn = host
                .record_three_card_spread_derived_at_with_entropy(
                    &context,
                    &field,
                    "C5 owner public seed",
                    "cleromancy.owner/shuffle",
                    if situation { 1_792_149_302_000 } else { 6000 },
                    None,
                    &mut PublicEntropy(0xc5),
                )
                .unwrap();
            if situation {
                use cleromancy::situation::*;
                let place = ReadingPlace::new(51_476_900, 0).unwrap();
                // Public synthetic response; live network provenance has
                // a separate product-native receipt.
                let body = r#"{"latitude":51.472404,"longitude":0.011566162,"utc_offset_seconds":0,"current_units":{"time":"unixtime","interval":"seconds","temperature_2m":"°C","relative_humidity_2m":"%","precipitation":"mm","weather_code":"wmo code","wind_speed_10m":"m/s"},"current":{"time":1792149300,"interval":900,"temperature_2m":12.8,"relative_humidity_2m":70,"precipitation":0,"weather_code":3,"wind_speed_10m":4.1}}"#;
                let weather = WeatherObservation::from_response(
                    &place,
                    1_792_149_301_000,
                    weather_url(&place).unwrap(),
                    body.into(),
                )
                .unwrap();
                let snapshot = ReadingSituation::capture(
                    &drawn.0,
                    &SituationPreferences {
                        place: Some(place),
                        weather: true,
                    },
                    Some(weather),
                )
                .unwrap();
                host.insert_reading_situation(&snapshot).unwrap();
            }
            drawn.2[0].clone()
        } else {
            detail.readings[0].clone()
        };
        if inference {
            use cleromancy::inference::*;
            let session = host
                .sessions()
                .unwrap()
                .into_iter()
                .max_by_key(|s| s.created_at_ms)
                .unwrap();
            let inputs = host.inference_inputs(&session.id).unwrap();
            host.insert_generated_inference(&GeneratedInference::formula(inputs.clone()))
                .unwrap();
            // Deliberately synthetic: this fixture qualifies stored model
            // records and receiving authority, not model execution.
            let model = ModelIdentity {
                model_id: "fixture-only/receiving-record".into(),
                revision: "contract-v1".into(),
                loader: "esp-burn-ndarray".into(),
                artifacts: ["config.json", "tokenizer.json", "model.safetensors"]
                    .map(|name| (name.into(), "a".repeat(64)))
                    .into_iter()
                    .collect(),
            };
            let record = GeneratedInference::model(
                inputs,
                model,
                "Synthetic receiving fixture output; no model was executed by this fixture."
                    .into(),
            );
            host.insert_generated_inference(&record).unwrap();
            host.keep_generated_inference(&record.digest()).unwrap();
        }
        (host.graph().clone(), reading)
    });
    drop(clear);
    let edits = std::mem::take(&mut *captured.lock().unwrap());
    assert!(!edits.is_empty(), "the real product emitted captured edits");
    let replay = mere::kernel::graph::replay_captured_deltas(edits.clone());
    assert_eq!(replay.node_count(), graph.node_count());
    assert_eq!(
        serde_json::to_value(replay.facets()).unwrap(),
        serde_json::to_value(graph.facets()).unwrap(),
        "the captured batch retains every product facet"
    );
    // A control for the fixture, not the owner assertion: the real admitted
    // transaction below independently invokes this same published supplier.
    cleromancy::host::validate_divination_graph(&replay).unwrap();
    (
        edits,
        serde_json::to_value(graph.facets()).unwrap(),
        reading,
    )
}

pub(super) async fn disclosed(
    client: &mut AppBrokerClient,
    snapshot: &ProjectionSnapshot,
) -> SessionGraphV1 {
    let offer = &snapshot.presentation.offers_for(InstanceId(0)).unwrap()[0];
    let bytes = client
        .resource(snapshot.session.clone(), offer.resource)
        .await
        .unwrap();
    assert_eq!(chirograph::ContentHash::of(&bytes), offer.resource);
    let card: chirograph::PortableCardV1 = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(
        card.media.len(),
        1,
        "session card advertises its complete graph"
    );
    let media = card.media[0];
    let bytes = client
        .resource(snapshot.session.clone(), media)
        .await
        .unwrap();
    assert_eq!(chirograph::ContentHash::of(&bytes), media);
    let envelope: SessionGraphV1 = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(envelope.schema, session_item::SESSION_GRAPH_SCHEMA);
    assert_eq!(envelope.application, "validator-fixture");
    assert_eq!(envelope.epoch, snapshot.scene.epoch);
    assert_eq!(envelope.revision, snapshot.scene.revision);
    assert!(!envelope.read_only);
    envelope
}

// Fixture seeding may contain several complete decks. Each bounded
// transaction remains a fully valid candidate; no frame cap or refusal
// guard is widened to fit a combined acceptance setup packet.
pub(super) async fn apply_bounded_fixture(client: &mut AppBrokerClient, edits: &[CapturedDelta]) {
    let mut accepted: Vec<CapturedDelta> = Vec::new();
    let mut offset = 0;
    while offset < edits.len() {
        let current = snapshots(client).await;
        let mut choice = None;
        for end in (offset + 1..=edits.len()).rev() {
            let batch = edits[offset..end].to_vec();
            let payload =
                serde_json::to_vec(&ApplyEditsV1::checked(current[1].scene.revision, batch))
                    .unwrap();
            if serde_json::to_vec(&payload).unwrap().len() > 900_000 {
                continue;
            }
            let candidate = mere::kernel::graph::replay_captured_deltas(
                accepted
                    .iter()
                    .cloned()
                    .chain(edits[offset..end].iter().cloned()),
            );
            if cleromancy::host::validate_divination_graph(&candidate).is_ok() {
                choice = Some(end);
                break;
            }
        }
        let end =
            choice.expect("fixture must have a replay-valid prefix below the carrier limit");
        assert_eq!(
            apply_checked(client, &current[1], edits[offset..end].to_vec()).await,
            IntentResult::Accepted
        );
        println!(
            "BOUNDED PRODUCT FIXTURE deltas={} replay-valid-candidate=true",
            end - offset
        );
        accepted.extend_from_slice(&edits[offset..end]);
        offset = end;
    }
}

pub(super) async fn apply_checked(
    client: &mut AppBrokerClient,
    snapshot: &ProjectionSnapshot,
    edits: Vec<CapturedDelta>,
) -> IntentResult {
    let response = client
        .request_body(CarrierRequestBody::Intent(IntentInvocation {
            session: snapshot.session.clone(),
            target: InstanceId(0),
            observed_epoch: snapshot.scene.epoch,
            observed_revision: snapshot.scene.revision,
            intent: APPLY_EDITS_INTENT.into(),
            payload: serde_json::to_vec(&ApplyEditsV1::checked(snapshot.scene.revision, edits))
                .unwrap(),
        }))
        .await
        .unwrap();
    let CarrierResponseBody::Intent(result) = response else {
        panic!("unexpected owner response: {response:?}");
    };
    result
}

pub(super) fn journal_and_history(files: &BTreeMap<PathBuf, Vec<u8>>) -> BTreeMap<PathBuf, Vec<u8>> {
    let logs: BTreeMap<_, _> = files
        .iter()
        .filter(|(path, _)| {
            matches!(
                path.file_name().and_then(|name| name.to_str()),
                Some("journal.jsonl" | "changes.jsonl")
            )
        })
        .map(|(path, bytes)| (path.clone(), bytes.clone()))
        .collect();
    for name in ["journal.jsonl", "changes.jsonl"] {
        assert!(
            logs.iter().any(
                |(path, bytes)| path.file_name().and_then(|name| name.to_str()) == Some(name)
                    && !bytes.is_empty()
            ),
            "accepted product batch must retain nonempty {name}"
        );
    }
    logs
}
