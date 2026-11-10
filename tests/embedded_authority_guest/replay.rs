// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The three-owner replay: embedded, daemon, embedded, with tamper refusals.

use super::*;
use super::fixture::{apply_bounded_fixture, apply_checked, disclosed, journal_and_history, product_fixture};

pub(super) async fn owner_replay(
    typed_decks: bool,
    ambient: bool,
    shuffled: bool,
    situation: bool,
    inference: bool,
) {
    let root = tempfile::tempdir().unwrap();
    install_divination_guest(root.path(), persona());
    let unusable = root.path().join("no-attached-client-store");
    std::fs::write(
        &unusable,
        b"the live client cannot open local custody or storage",
    )
    .unwrap();
    let (edits, expected_facets, reading) =
        product_fixture(typed_decks, ambient, shuffled, situation, inference);
    let reading_url = format!("cleromancy://reading/{}", reading.id);
    let reading_id = kernel::graph::Graph::node_namespace_id(&reading_url);
    let mut retained_session = None;
    let mut retained_graph = None;
    let mut retained_logs = None;
    for (phase, mode) in ["embedded", "daemon", "embedded"].into_iter().enumerate() {
        let address = endpoint();
        let process = Process::start(root.path(), &address, mode);
        assert_ne!(process.child.id(), std::process::id());
        let mut attached = ReservoirAttachment::open_application(
            ApplicationReservoirOptions {
                endpoint: address,
                application: AppId::new("validator-fixture"),
            declaration: mere_resident::resident_reservoir::mere_client_declaration(),
                domain: DomainId::new("divination").unwrap(),
                shared_root: unusable.clone(),
                vault_dir: unusable.clone(),
                profile: None,
                persona: Some(persona()),
                session_duration: Duration::from_secs(60),
            },
            &mut NoLocalPrompt,
            |_| async {
                panic!("an attached client must not compose local authority");
                #[allow(unreachable_code)]
                Ok::<MereRoutes, String>(unreachable!())
            },
        )
        .await
        .unwrap();
        assert!(!attached.is_embedded());
        let initial = snapshots(attached.client_mut()).await;
        if phase == 0 {
            let empty = disclosed(attached.client_mut(), &initial[1])
                .await
                .into_graph()
                .unwrap();
            assert_eq!(
                empty.node_count(),
                0,
                "the owner has no fixture-seeded graph"
            );
            if shuffled || typed_decks {
                apply_bounded_fixture(attached.client_mut(), &edits).await;
            } else {
                assert_eq!(
                    apply_checked(attached.client_mut(), &initial[1], edits.clone()).await,
                    IntentResult::Accepted,
                    "real published replay authority accepts the complete product batch"
                );
            }
        }
        let before = snapshots(attached.client_mut()).await;
        let envelope = disclosed(attached.client_mut(), &before[1]).await;
        let session = envelope.session;
        let graph = envelope.clone().into_graph().unwrap();
        assert_eq!(
            serde_json::to_value(graph.facets()).unwrap(),
            expected_facets
        );
        let stored = graph
            .facets()
            .get(
                &reading_id,
                &pandect::facet_store::FacetId::new(cleromancy::host::READING_FACET),
            )
            .unwrap();
        assert_eq!(
            serde_json::from_value::<cleromancy::Reading>(stored.clone()).unwrap(),
            reading
        );
        let graph_state = serde_json::to_value(graph.to_snapshot_at(0)).unwrap();
        let durable = durable_files_for(root.path(), "divination");
        let logs = journal_and_history(&durable);
        if typed_decks {
            let fields: Vec<_> = graph
                .nodes()
                .filter(|(_, node)| node.url().starts_with("cleromancy://field/"))
                .collect();
            assert_eq!(fields.len(), 2);
            for (_, field_node) in &fields {
                let stored = graph
                    .facets()
                    .get(
                        &field_node.id,
                        &pandect::facet_store::FacetId::new(
                            cleromancy::host::FIELD_FACET,
                        ),
                    )
                    .unwrap();
                let field: cleromancy::Field =
                    serde_json::from_value(stored.clone()).unwrap();
                assert_eq!(field.tarot.as_ref().unwrap().cards.len(), 78);
                assert!(field.position_meanings.is_some());
            }
            let (_, field_node) = fields[0];
            let mut forged = graph
                .facets()
                .get(
                    &field_node.id,
                    &pandect::facet_store::FacetId::new(cleromancy::host::FIELD_FACET),
                )
                .unwrap()
                .clone();
            forged["tarot"]["cards"][0]["title"] = serde_json::json!("Forged typed card title");
            let result = apply_checked(
                attached.client_mut(),
                &before[1],
                vec![
                    node(0xc300 + phase as u128),
                    CapturedDelta::ReplaySetNodeFacetById {
                        node_id: field_node.id.to_string(),
                        facet: cleromancy::host::FIELD_FACET.into(),
                        value_json: forged.to_string(),
                    },
                ],
            )
            .await;
            assert!(
                matches!(result, IntentResult::Rejected { .. }),
                "forged deck metadata must be refused: {result:?}"
            );
            let after = snapshots(attached.client_mut()).await;
            assert_eq!(
                serde_json::to_value(&after).unwrap(),
                serde_json::to_value(&before).unwrap()
            );
            assert_eq!(durable_files_for(root.path(), "divination"), durable);
            println!(
                "TYPED DECK REPLAY mode={mode} phase={phase} decks=2 cards-per-deck=78 content-revisions-retained=true metadata-tamper-refused=true durable-bytes-unchanged=true"
            );
        }
        if ambient {
            let host = cleromancy::CleromancyHost::candidate(graph.clone()).unwrap();
            let views = host.ambient_views().unwrap();
            let memories = host.ambient_memories().unwrap();
            assert_eq!(views.len(), 1);
            assert!(views[0].reduced_motion);
            assert_eq!(
                views[0].snapshot.recipe.definition.arrangement.kind,
                "grid.ordered"
            );
            assert_eq!(memories.len(), 1);
            assert!(memories[0].kept);
            assert_eq!(views[0].roles[&memories[0].memory.item.id].id(), "pinned");
            assert!(views[0].foreground.contains(&memories[0].memory.item.id));
            for facet in [
                cleromancy::host::AMBIENT_VIEW_FACET,
                cleromancy::host::AMBIENT_MEMORY_FACET,
            ] {
                let (node_id, mut forged) = graph
                    .facets()
                    .iter()
                    .find_map(|(id, facets)| {
                        facets
                            .iter()
                            .find(|(name, _)| name.as_str() == facet)
                            .map(|(_, v)| (*id, v.clone()))
                    })
                    .unwrap();
                if facet == cleromancy::host::AMBIENT_VIEW_FACET {
                    forged["snapshot"]["selected_occurrence"] =
                        serde_json::json!("undisclosed");
                } else {
                    forged["item"]["body"] = serde_json::json!("not the saved engine result");
                }
                let result = apply_checked(
                    attached.client_mut(),
                    &before[1],
                    vec![CapturedDelta::ReplaySetNodeFacetById {
                        node_id: node_id.to_string(),
                        facet: facet.into(),
                        value_json: forged.to_string(),
                    }],
                )
                .await;
                assert!(
                    matches!(result, IntentResult::Rejected { .. }),
                    "C4 source/curation tamper accepted: {result:?}"
                );
                assert_eq!(
                    serde_json::to_value(snapshots(attached.client_mut()).await).unwrap(),
                    serde_json::to_value(&before).unwrap()
                );
                assert_eq!(durable_files_for(root.path(), "divination"), durable);
            }
            println!(
                "AMBIENT REPLAY mode={mode} phase={phase} focus-roles-foreground-and-spec-retained=true saved-memory=true source-tamper-refused=true durable-bytes-unchanged=true"
            );
        }
        if shuffled {
            let draw = reading.receipt.shuffle.as_ref().unwrap();
            assert_eq!(draw.count, 3);
            let plan = draw
                .shuffle
                .replay(
                    &cleromancy::CleromancyHost::candidate(graph.clone())
                        .unwrap()
                        .context_for_digest(&reading.receipt.context_digest)
                        .unwrap(),
                    &cleromancy::CleromancyHost::candidate(graph.clone())
                        .unwrap()
                        .field_for_digest(&reading.receipt.field_digest)
                        .unwrap(),
                )
                .unwrap();
            assert_eq!(plan.len(), 78);
            let mut forged = stored.clone();
            forged["receipt"]["shuffle"]["orientation"] = serde_json::json!(if reading
                .orientation()
                == cleromancy::Orientation::Reversed
            {
                "upright"
            } else {
                "reversed"
            });
            let result = apply_checked(
                attached.client_mut(),
                &before[1],
                vec![
                    node(0xc500 + phase as u128),
                    CapturedDelta::ReplaySetNodeFacetById {
                        node_id: reading_id.to_string(),
                        facet: cleromancy::host::READING_FACET.into(),
                        value_json: forged.to_string(),
                    },
                ],
            )
            .await;
            assert!(matches!(result, IntentResult::Rejected { .. }));
            assert_eq!(
                serde_json::to_value(snapshots(attached.client_mut()).await).unwrap(),
                serde_json::to_value(&before).unwrap()
            );
            assert_eq!(durable_files_for(root.path(), "divination"), durable);
            println!(
                "SHUFFLED DECK REPLAY mode={mode} phase={phase} full-deck=78 prefix=3 orientation-tamper-refused=true durable-bytes-unchanged=true"
            );
        }
        if situation {
            use cleromancy::situation::*;
            let host = cleromancy::CleromancyHost::candidate(graph.clone()).unwrap();
            let records = host.reading_situations().unwrap();
            assert_eq!(records.len(), 1);
            assert_eq!(
                records[0]
                    .weather
                    .as_ref()
                    .unwrap()
                    .values
                    .temperature_millicelsius,
                12_800
            );
            let facet = SITUATION_FACET;
            let (node_id, mut forged) = graph
                .facets()
                .iter()
                .find_map(|(id, facets)| {
                    facets
                        .iter()
                        .find(|(name, _)| name.as_str() == facet)
                        .map(|(_, v)| (*id, v.clone()))
                })
                .unwrap();
            forged["weather"]["values"]["temperature_millicelsius"] = serde_json::json!(99_999);
            let result = apply_checked(
                attached.client_mut(),
                &before[1],
                vec![
                    node(0xc600 + phase as u128),
                    CapturedDelta::ReplaySetNodeFacetById {
                        node_id: node_id.to_string(),
                        facet: facet.into(),
                        value_json: forged.to_string(),
                    },
                ],
            )
            .await;
            assert!(matches!(result, IntentResult::Rejected { .. }));
            assert_eq!(
                serde_json::to_value(snapshots(attached.client_mut()).await).unwrap(),
                serde_json::to_value(&before).unwrap()
            );
            assert_eq!(durable_files_for(root.path(), "divination"), durable);
            println!(
                "SITUATION REPLAY mode={mode} phase={phase} saved-response-exact=true network-free-replay=true weather-tamper-refused=true durable-bytes-unchanged=true"
            );
        }
        if inference {
            use cleromancy::inference::*;
            let host = cleromancy::CleromancyHost::candidate(graph.clone()).unwrap();
            let records = host.generated_inferences().unwrap();
            assert_eq!(records.len(), 2);
            for record in records {
                let key = graph
                    .get_node_by_url(&format!("cleromancy://inference/{}", record.digest()))
                    .unwrap()
                    .0;
                let id = graph.get_node(key).unwrap().id;
                match record.generation {
                    Generation::Formula { .. } => {
                        assert_eq!(record.output, record.inputs.formula_output())
                    },
                    Generation::Model { .. } => {
                        assert_eq!(record.label, LABEL_MODEL);
                        assert!(
                            record
                                .output
                                .starts_with("Synthetic receiving fixture output")
                        );
                        assert!(
                            graph
                                .node_content_tags(key)
                                .is_some_and(|tags| tags.contains("kept"))
                        );
                    },
                }
                for case in 0..3 {
                    let mut forged = serde_json::to_value(&record).unwrap();
                    if case == 0 {
                        forged["inputs"]["context_text"] =
                            serde_json::json!("fabricated source input");
                    } else if case == 1 {
                        forged["output"] = serde_json::json!("fabricated output");
                    } else {
                        forged["label"] = serde_json::json!("Unlabelled invented output");
                    }
                    let result = apply_checked(
                        attached.client_mut(),
                        &before[1],
                        vec![
                            node(0xc700 + phase as u128 * 6 + case),
                            CapturedDelta::ReplaySetNodeFacetById {
                                node_id: id.to_string(),
                                facet: INFERENCE_FACET.into(),
                                value_json: forged.to_string(),
                            },
                        ],
                    )
                    .await;
                    assert!(matches!(result, IntentResult::Rejected { .. }));
                    assert_eq!(
                        serde_json::to_value(snapshots(attached.client_mut()).await).unwrap(),
                        serde_json::to_value(&before).unwrap()
                    );
                    assert_eq!(durable_files_for(root.path(), "divination"), durable);
                }
            }
            println!(
                "INFERENCE REPLAY mode={mode} phase={phase} formula-exact=true stored-model-label-retained=true explicit-keep-retained=true input-and-disclosure-tamper-refused=true durable-bytes-unchanged=true"
            );
        }
        if let Some(id) = retained_session {
            assert_eq!(session, id, "the same owner-held session reopens");
            assert_eq!(retained_graph.as_ref().unwrap(), &graph_state);
            assert_eq!(retained_logs.as_ref().unwrap(), &logs);
        } else {
            retained_session = Some(session);
            retained_graph = Some(graph_state.clone());
            retained_logs = Some(logs.clone());
        }
        for (case, label) in ["candidate_id", "receipt_total_weight"]
            .into_iter()
            .enumerate()
        {
            let mut tampered = stored.clone();
            if case == 0 {
                tampered["candidate_id"] = serde_json::json!("invented-c1-candidate");
            } else {
                tampered["receipt"]["total_weight"] =
                    serde_json::json!(reading.receipt.total_weight + 1);
            }
            let typed: cleromancy::Reading =
                serde_json::from_value(tampered.clone()).unwrap();
            assert_eq!(typed.schema, reading.schema);
            assert_eq!(typed.receipt.schema, reading.receipt.schema);
            assert_eq!(typed.id, reading.id);
            assert!(typed.schema.ends_with(if shuffled { "/v3" } else { "/v1" }));
            assert!(
                typed
                    .receipt
                    .schema
                    .ends_with(if shuffled { "/v5" } else { "/v1" })
            );
            let result = apply_checked(
                attached.client_mut(),
                &before[1],
                vec![
                    node(0xc100 + phase as u128 * 2 + case as u128),
                    CapturedDelta::ReplaySetNodeFacetById {
                        node_id: reading_id.to_string(),
                        facet: cleromancy::host::READING_FACET.into(),
                        value_json: tampered.to_string(),
                    },
                ],
            )
            .await;
            let IntentResult::Rejected { reason } = result else {
                panic!("owner must refuse supported-schema {label} tamper: {result:?}");
            };
            assert!(
                reason.contains("receipt"),
                "expected replay mismatch, got {reason}"
            );
            assert!(
                !reason.contains("unsupported domain facet"),
                "schema rejection is not this proof"
            );
            let after = snapshots(attached.client_mut()).await;
            assert_eq!(
                serde_json::to_value(&after).unwrap(),
                serde_json::to_value(&before).unwrap()
            );
            let after_envelope = disclosed(attached.client_mut(), &after[1]).await;
            assert_eq!(
                serde_json::to_value(after_envelope).unwrap(),
                serde_json::to_value(&envelope).unwrap()
            );
            let after_bytes = durable_files_for(root.path(), "divination");
            assert_eq!(journal_and_history(&after_bytes), logs);
            assert_eq!(
                after_bytes, durable,
                "rejected whole batch changes no durable session state"
            );
            println!(
                "RECEIPT REPLAY mode={mode} phase={phase} case={label} same-v1-schema=true replay-refused=true projection-revision-and-journal-history-and-durable-bytes-unchanged=true"
            );
        }
        attached.close().await.unwrap();
        process.stop();
    }
}
