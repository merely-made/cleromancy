// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! A saved occasion projected into Mere's scene contract. The same sealed
//! result may occur in several positions; occurrence identity remains local
//! to the session, while the source is interned once, as in Woodshed's Stage.

use crate::ConsultationDetail;
use sceno::{
    Footprint, ProjectedItem, Rect, Representation, Scene, Size2, SourceRef, Transform2, Vec2,
};

pub const CARD_WIDTH: f32 = 164.0;
pub const CARD_HEIGHT: f32 = 136.0;
pub const GAP: f32 = 12.0;

pub struct ReadingScene {
    pub session_id: String,
    pub scene: Scene,
    /// Dense scene instance -> immutable session position.
    pub positions: Vec<String>,
}

pub fn reading_scene(detail: &ConsultationDetail) -> ReadingScene {
    let mut scene = Scene::new();
    let mut positions = Vec::new();
    for (index, placement) in detail.session.placements.iter().enumerate() {
        let source =
            scene.intern_source(SourceRef::new("cleromancy.reading", &placement.reading_id));
        scene.items.push(ProjectedItem {
            source,
            space: Scene::WORLD,
            transform: Transform2::translation(
                (index % 3) as f32 * (CARD_WIDTH + GAP) + CARD_WIDTH / 2.0,
                (index / 3) as f32 * (CARD_HEIGHT + GAP) + CARD_HEIGHT / 2.0,
            ),
            footprint: Footprint::Rect {
                size: Size2::new(CARD_WIDTH, CARD_HEIGHT),
            },
            representation: Representation::Card,
            layer: 0,
            visible: true,
            hit: None,
            channels: Vec::new(),
        });
        positions.push(placement.position.clone());
    }
    scene.bounds = Rect::new(
        Vec2::ZERO,
        Size2::new(
            scene.items.len().clamp(1, 3) as f32 * (CARD_WIDTH + GAP) - GAP,
            scene.items.len().div_ceil(3).max(1) as f32 * (CARD_HEIGHT + GAP),
        ),
    );
    ReadingScene {
        session_id: detail.session.id.clone(),
        scene,
        positions,
    }
}

pub fn position_label(position: &str) -> String {
    let label = position.replace('_', " ");
    let mut chars = label.chars();
    chars
        .next()
        .map(|first| first.to_uppercase().to_string() + chars.as_str())
        .unwrap_or(label)
}

pub fn position_rationale(position: &str) -> &'static str {
    match position {
        "foundation" => {
            "Read this as the ground you are starting from: what supports the situation, or what you are taking for granted."
        },
        "tension" => {
            "Read this against the foundation: what complicates it, challenges it, or asks for a different response."
        },
        "next_step" => {
            "Read this as a possible response to that tension: something to try or attend to, rather than a promised outcome."
        },
        "focus" => {
            "Use this as a single lens on your question. Notice what fits your experience and what does not."
        },
        _ => {
            "Read this through the named position in your chosen layout, alongside the other results and your question."
        },
    }
}
