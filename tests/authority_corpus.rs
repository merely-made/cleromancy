// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The authority corpus (cycle breaks P3, spike): the graphs of the seven
//! tests that move from Mere's `embedded_reservoir_validation`, as captured
//! delta batches, each with its native verdict, plus single-leaf tampers of
//! every batch with theirs.
//!
//! The conformance test runs the same cases through the wasm guest (`authority-guest/`),
//! installed as the owner approves it into Mere's guest store, and demands the same verdict
//! and the same refusal text from the guest as from the native validator. Set
//! `CLEROMANCY_AUTHORITY_CORPUS` to a directory to write the cases as files; without it the
//! first test writes nothing and checks only that the untampered fixtures are admitted and
//! that tampering is refused. `CLEROMANCY_AUTHORITY_TAMPERS` is the tampers per fixture
//! (3 for the first test, 150 for the conformance run).

use std::sync::{Arc, Mutex};

mod common {
    #[allow(dead_code)]
    pub mod authority_pack {
        include!("common/authority_pack.rs");
    }
}

use cleromancy::{CleromancyHost, Consultation, ContextDraft, SelectionMode};
use mere::kernel::graph::CapturedDelta;
use muniment::MemoryBackend;
use serde_json::Value;

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

fn product_fixture(
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

/// The native verdict the resident's validator reaches over a batch: replay it
/// into a graph, then validate the complete candidate.
fn verdict(edits: Vec<CapturedDelta>) -> Result<(), String> {
    let graph = mere::kernel::graph::replay_captured_deltas(edits);
    cleromancy::host::validate_divination_graph(&graph).map_err(|error| error.to_string())
}

/// Every leaf (string or number) of a JSON value, by path, in order.
fn leaves(value: &Value, path: &mut Vec<String>, out: &mut Vec<Vec<String>>) {
    match value {
        Value::Object(map) => {
            for (key, inner) in map {
                path.push(key.clone());
                leaves(inner, path, out);
                path.pop();
            }
        },
        Value::Array(items) => {
            for (index, inner) in items.iter().enumerate() {
                path.push(index.to_string());
                leaves(inner, path, out);
                path.pop();
            }
        },
        Value::String(_) | Value::Number(_) => out.push(path.clone()),
        _ => {},
    }
}

fn leaf_mut<'a>(value: &'a mut Value, path: &[String]) -> &'a mut Value {
    path.iter().fold(value, |at, key| match at {
        Value::Object(map) => map.get_mut(key).unwrap(),
        Value::Array(items) => &mut items[key.parse::<usize>().unwrap()],
        _ => unreachable!(),
    })
}

fn tamper(leaf: &mut Value) {
    match leaf {
        Value::String(text) => text.push('x'),
        Value::Number(number) => {
            *leaf = match number.as_i64() {
                Some(whole) => Value::from(whole + 1),
                None => Value::from(number.as_f64().unwrap_or(0.0) + 1.0),
            }
        },
        _ => {},
    }
}

/// One corpus case: a captured batch (as JSON, so a tamper is a changed leaf) and the
/// native verdict over it.
struct Case {
    name: String,
    deltas: Value,
    native: Result<(), String>,
}

/// The six product graphs of the seven moving tests and `per_fixture` single-leaf tampers of
/// each, with their native verdicts. A tamper that no longer parses as a batch, or whose
/// stable id no longer parses (which panics inside the kernel's replay before any validator
/// runs), cannot reach the validator over the resident's wire and is not a case.
fn corpus(per_fixture: usize) -> Vec<Case> {
    let fixtures = [
        ("published-reading", [false, false, false, false, false]),
        ("typed-decks", [true, false, false, false, false]),
        ("ambient", [true, true, false, false, false]),
        ("shuffled", [true, false, true, false, false]),
        ("situation", [true, false, true, true, false]),
        ("inference", [true, false, true, false, true]),
    ];
    std::panic::set_hook(Box::new(|_| {}));
    let mut cases = Vec::new();
    for (name, [typed, ambient, shuffled, situation, inference]) in fixtures {
        let (edits, _facets, _reading) =
            product_fixture(typed, ambient, shuffled, situation, inference);
        assert_eq!(verdict(edits.clone()), Ok(()), "{name} is admitted");
        let whole = serde_json::to_value(&edits).unwrap();
        let mut all = Vec::new();
        leaves(&whole, &mut Vec::new(), &mut all);
        // Spread the sampled leaves evenly over the batch.
        let step = (all.len() / per_fixture.max(1)).max(1);
        let mut variants = vec![(name.to_string(), whole.clone())];
        for (n, path) in all.iter().step_by(step).take(per_fixture).enumerate() {
            let mut changed = whole.clone();
            tamper(leaf_mut(&mut changed, path));
            variants.push((format!("{name}-tamper-{n:03}"), changed));
        }
        for (case, value) in variants {
            let Ok(batch) = serde_json::from_value::<Vec<CapturedDelta>>(value.clone()) else {
                continue;
            };
            let Ok(native) = std::panic::catch_unwind(|| verdict(batch)) else {
                continue;
            };
            cases.push(Case { name: case, deltas: value, native });
        }
    }
    cases
}

fn tampers_per_fixture(default: usize) -> usize {
    std::env::var("CLEROMANCY_AUTHORITY_TAMPERS")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

#[test]
fn the_seven_tests_graphs_and_their_tampers_have_native_verdicts() {
    let out = std::env::var_os("CLEROMANCY_AUTHORITY_CORPUS").map(std::path::PathBuf::from);
    if let Some(dir) = &out {
        std::fs::create_dir_all(dir).unwrap();
    }
    // A written corpus wants breadth; the ordinary run only needs the controls.
    let cases = corpus(tampers_per_fixture(if out.is_some() { 40 } else { 3 }));
    let refused = cases.iter().filter(|case| case.native.is_err()).count();
    if let Some(dir) = &out {
        for case in &cases {
            let record = serde_json::json!({
                "name": case.name,
                "deltas": case.deltas,
                "native_ok": case.native.is_ok(),
                "native_error": case.native.clone().err(),
            });
            std::fs::write(
                dir.join(format!("{}.json", case.name)),
                serde_json::to_vec(&record).unwrap(),
            )
            .unwrap();
        }
    }
    assert!(cases.len() > 6, "tampered cases exist");
    assert!(refused > 0, "tampering is refused at least once ({refused}/{})", cases.len());
}

/// The guest, installed as the owner approves it, gives the native validator's verdict and
/// its refusal text on every case; none traps. Prints the measurements the plan records.
#[test]
fn the_guest_agrees_with_the_native_validator_on_every_case() {
    use mere_resident::authority_guest::GuestStore;
    use mere_resident::domain_registry::Authority;
    use std::time::{Duration, Instant};

    let persona = personae::PersonaId::from_uuid(uuid::Uuid::from_u128(0x4155_5448));
    let root = tempfile::tempdir().unwrap();
    let pin = common::authority_pack::install_divination_guest(root.path(), persona);
    let registration = GuestStore::at(root.path(), persona).registrations().remove(0);
    assert_eq!((registration.domain.as_str(), registration.authority.kind()), ("divination", "guest"));
    let Authority::Guest(guest) = &registration.authority else { unreachable!() };
    let runtime = guest.runtime().expect("the approved guest loads").clone();
    let component_bytes = runtime.component_bytes();
    assert!(
        component_bytes <= authority_cap(),
        "the component is {component_bytes} bytes against the 8 MiB ceiling (16 MiB cap)"
    );

    let cases = corpus(tampers_per_fixture(150));
    let (mut agree, mut same_text, mut admitted, mut refused) = (0usize, 0usize, 0usize, 0usize);
    let mut disagreements = Vec::new();
    let (mut native_times, mut guest_times) = (Vec::new(), Vec::new());
    for case in &cases {
        let batch: Vec<CapturedDelta> = serde_json::from_value(case.deltas.clone()).unwrap();
        let graph = mere::kernel::graph::replay_captured_deltas(batch.clone());
        let started = Instant::now();
        let native = verdict(batch);
        native_times.push(started.elapsed());
        let started = Instant::now();
        let guest = registration.validate(&graph);
        guest_times.push(started.elapsed());
        assert_eq!(native.is_ok(), case.native.is_ok(), "{}: native reproduces", case.name);
        if native.is_ok() == guest.is_ok() {
            agree += 1;
            if native == guest {
                same_text += 1;
            }
        } else {
            disagreements.push(case.name.clone());
        }
        if guest.is_ok() { admitted += 1 } else { refused += 1 }
    }
    let stats = runtime.stats();
    native_times.sort();
    guest_times.sort();
    let pick = |times: &[Duration], p: f64| times[((times.len() - 1) as f64 * p).round() as usize];
    eprintln!(
        "GUEST-CONFORMANCE pin={} cases={} admitted={admitted} refused={refused} agree={agree} same-text={same_text}",
        pin.component,
        cases.len()
    );
    eprintln!(
        "GUEST-CONFORMANCE component={component_bytes} bytes ({:.2} MiB) trapped={} max-fuel={} max-memory={:.1} MiB max-candidate={} bytes",
        component_bytes as f64 / 1_048_576.0,
        stats.trapped,
        stats.max_fuel,
        stats.max_memory as f64 / 1_048_576.0,
        stats.max_candidate
    );
    eprintln!(
        "GUEST-CONFORMANCE per-call native p50={:?} p90={:?} max={:?}; resident total (encode+guest) p50={:?} p90={:?} max={:?}",
        pick(&native_times, 0.5),
        pick(&native_times, 0.9),
        native_times[native_times.len() - 1],
        pick(&guest_times, 0.5),
        pick(&guest_times, 0.9),
        guest_times[guest_times.len() - 1]
    );
    eprintln!(
        "GUEST-CONFORMANCE encode total={:?} max={:?} mean={:?}; guest total={:?} max={:?} mean={:?}",
        stats.encode,
        stats.max_encode,
        stats.encode / stats.calls as u32,
        stats.guest,
        stats.max_guest,
        stats.guest / stats.calls as u32
    );
    assert!(disagreements.is_empty(), "the guest disagreed with native on {disagreements:?}");
    assert_eq!(same_text, cases.len(), "every refusal carries the native text");
    assert_eq!(stats.trapped, 0, "no case trapped the guest");
    assert!(refused > 0 && admitted > 0);
}

/// The component's size ceiling: half the 16 MiB pack cap (D12), so growth is noticed long
/// before the cap. The component built at 5.15 MiB (the spike's, with more dependencies, 7.45).
fn authority_cap() -> usize {
    8 * 1024 * 1024
}

/// What the resident could send a guest as "the complete candidate": the
/// kernel's `GraphSnapshot` (`Graph::to_snapshot_at`) is the serde form of a
/// whole graph. This records whether it carries the product's facets, which
/// the validator reads. See the spike's findings.
#[test]
fn the_graph_snapshot_wire_and_the_facets() {
    let (edits, facets, _) = product_fixture(true, false, true, false, false);
    let graph = mere::kernel::graph::replay_captured_deltas(edits);
    let snapshot = graph.to_snapshot_at(0);
    let json = serde_json::to_vec(&snapshot).unwrap();
    let back = serde_json::from_slice(&json).unwrap();
    let restored = mere::kernel::graph::Graph::try_from_snapshot(&back).unwrap();
    let kept = serde_json::to_value(restored.facets()).unwrap() == facets;
    let verdict = cleromancy::host::validate_divination_graph(&restored).map_err(|e| e.to_string());
    // The facet store serialises on its own and installs onto a restored graph.
    let facet_bytes = serde_json::to_vec(graph.facets()).unwrap();
    let mut with_facets = mere::kernel::graph::Graph::try_from_snapshot(&back).unwrap();
    with_facets
        .install_facets(serde_json::from_slice(&facet_bytes).unwrap())
        .unwrap();
    let both = cleromancy::host::validate_divination_graph(&with_facets).map_err(|e| e.to_string());
    eprintln!(
        "SNAPSHOT-WIRE: snapshot {} bytes, facets kept: {kept}, validator over it: {verdict:?}; \
         facet store {} bytes, validator over snapshot plus facets: {both:?}",
        json.len(),
        facet_bytes.len()
    );
    assert!(!kept, "the snapshot alone drops the facets");
    assert_eq!(both, Ok(()), "snapshot plus facet store is a complete candidate");
}
