// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0
#[path = "support/ambient_fixture.rs"]
mod support;
use cleromancy::{ambient::*, ambient_scene::*};
#[test]
fn shared_projection_and_dynamics_keep_source_occurrences_and_position_pins() {
    let (c, detail) = support::fixture();
    let mut context = c.ambient_context(&detail.session.id, "foundation").unwrap();
    let repeated = context
        .data
        .items
        .iter()
        .find(|i| i.id.ends_with(":attention"))
        .unwrap()
        .id
        .clone();
    context
        .view
        .roles
        .insert(repeated.clone(), seiche::Role::Pinned);
    let mut runtime = NeighborhoodRuntime::new(&context.data, &context.view).unwrap();
    let index = runtime.compiled.projection.instance_by_occurrence[&repeated].0 as usize;
    let position = runtime.scene().items[index].transform.translate;
    let sources = runtime.scene().sources.clone();
    let mapping = runtime.compiled.projection.instance_by_occurrence.clone();
    for _ in 0..24 {
        runtime.tick().unwrap();
    }
    assert_eq!(runtime.scene().items[index].transform.translate, position);
    assert_eq!(runtime.scene().sources, sources);
    assert_eq!(runtime.compiled.projection.instance_by_occurrence, mapping);
    assert!(
        runtime
            .scene()
            .items
            .iter()
            .all(|i| i.transform.translate.x.is_finite() && i.transform.translate.y.is_finite())
    );
    runtime.retain_pose(&mut context.view);
    context.view.paused = true;
    context.view.snapshot.selected_occurrence = Some(repeated.clone());
    context.view.foreground.insert(repeated.clone());
    let reopened: AmbientView =
        serde_json::from_slice(&serde_json::to_vec(&context.view).unwrap()).unwrap();
    assert_eq!(context.view, reopened);
    let reopened = NeighborhoodRuntime::new(&context.data, &reopened).unwrap();
    let [x, y] = context.view.positions[&repeated];
    assert_eq!(
        reopened.scene().items[index].transform.translate,
        sceno::Vec2::new(x as f32 / 1000.0, y as f32 / 1000.0),
        "reopen preserves the documented thousandth-unit pose"
    );
    let swatch = reopened.swatch(&context.data, &context.view, 360);
    assert_eq!(swatch.selected, Some(repeated));
    assert_eq!(swatch.graph.nodes.len(), runtime.scene().items.len());
    assert_eq!(
        c.detail(&detail.session.id).unwrap(),
        detail,
        "motion never draws or rewrites a reading"
    );
}
#[test]
fn both_shared_scenes_and_living_motion_bind_and_filters_only_select_disclosed_relations() {
    let (c, detail) = support::fixture();
    let mut context = c.ambient_context(&detail.session.id, "foundation").unwrap();
    for kind in ["spiral", "grid.default", "grid.ordered"] {
        context.view.snapshot.recipe.definition.arrangement.kind = kind.into();
        let runtime = NeighborhoodRuntime::new(&context.data, &context.view).unwrap();
        assert!(!runtime.scene().items.is_empty());
        if kind == "grid.ordered" {
            assert!(
                runtime
                    .scene()
                    .items
                    .iter()
                    .map(|i| i.transform.translate.y.to_bits())
                    .collect::<std::collections::BTreeSet<_>>()
                    .len()
                    > 1,
                "ordered grid must wrap into multiple rows"
            );
        }
    }
    context.view.snapshot.recipe.definition.dynamics =
        Some(supported_motion("flock.boids").unwrap());
    let mut runtime = NeighborhoodRuntime::new(&context.data, &context.view).unwrap();
    for _ in 0..8 {
        runtime.tick().unwrap();
    }
    context.view.families = std::collections::BTreeSet::from([RelationFamily::History]);
    let disclosure = disclosed(&context.data, &context.view);
    assert!(
        disclosure
            .relationships
            .iter()
            .all(|r| r.provenance.provider == RelationFamily::History.engine())
    );
    assert!(
        disclosure
            .dataset
            .occurrences
            .iter()
            .all(|i| context.data.item(&i.occurrence_id).is_some())
    );
}
