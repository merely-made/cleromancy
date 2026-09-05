// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Pure Cambium view construction over [`ConsultationUi`], one submodule per
//! semantic region.

use cambium::{
    AnyView, GenetCtx, GenetElement, SelectState, SelectionState, TabActivation, TextInput, el,
    map_action, map_state, tab_bar, text_field_typed, textarea_typed,
};

use super::screen::{self, ConsultationScreen};
use super::state::ConsultationUi;
use crate::SelectionMode;

mod chart;
mod consultation;
mod journal;
mod reading;
mod shared;
#[cfg(feature = "sky-timeline")]
mod sky;
mod trail;

use chart::chart_region;
use consultation::consultation_region;
use journal::journal_region;
use reading::reading_region;
use shared::placeholder_region;
#[cfg(feature = "sky-timeline")]
use sky::sky_region;
use trail::trail_region;

pub type ConsultationView = Box<dyn AnyView<ConsultationUi, (), GenetCtx, GenetElement>>;

pub fn consultation_view(ui: &ConsultationUi) -> ConsultationView {
    let mut chrome = Vec::new();
    chrome.push(Box::new(
        el::<_, ConsultationUi, ()>(
            "header",
            (
                el("p", "Cleromancy").attr("class", "eyebrow"),
                el("h1", "Local consultation"),
                el(
                    "p",
                    "A private reading with its context, selection, and workings kept together.",
                ),
            ),
        )
        .attr("class", "app-header"),
    ) as ConsultationView);
    chrome.push(Box::new(
        el::<_, ConsultationUi, ()>("p", ui.status.label())
            .attr("role", "status")
            .attr("aria-live", "polite")
            .attr("data-key", "consultation-status"),
    ));
    if let Some(error) = &ui.error {
        chrome.push(Box::new(
            el::<_, ConsultationUi, ()>("p", error.clone())
                .attr("role", "alert")
                .attr("data-key", "consultation-error"),
        ));
    }
    // The tab bar is the only surface switch. It carries no
    // `ConsultationAction`: Cambium mutates the `SelectionState` it is lensed
    // onto and nothing else, so activating a tab never reaches the persistence
    // worker.
    let items = screen::surface_tab_items();
    let tabs = map_action(
        tab_bar(&ui.surface_tabs, &items, TabActivation::Automatic),
        never_tab_action,
    );
    chrome.push(Box::new(map_state(tabs, surface_tabs_state)));

    let screen = ui.screen();
    let regions = match screen {
        ConsultationScreen::Today => vec![
            consultation_region(ui),
            reading_region(ui),
            trail_region(ui),
        ],
        ConsultationScreen::Journal => vec![journal_region(ui)],
        ConsultationScreen::Sky => {
            #[cfg(feature = "sky-timeline")]
            {
                vec![sky_region(ui)]
            }
            #[cfg(not(feature = "sky-timeline"))]
            {
                vec![placeholder_region(
                    "Sky",
                    "sky",
                    "The sky surface needs the sky-timeline feature.",
                )]
            }
        },
        ConsultationScreen::Chart => vec![chart_region(ui)],
    };

    Box::new(
        el::<_, ConsultationUi, ()>(
            "div",
            vec![
                Box::new(el::<_, ConsultationUi, ()>("div", chrome)) as ConsultationView,
                // One `<main>` per surface is the tab panel: its id is what the
                // active tab's `aria-controls` names, and it points back at the
                // tab through `aria-labelledby`.
                Box::new(
                    el::<_, ConsultationUi, ()>("main", regions)
                        .attr("class", "cleromancy-regions")
                        .attr("role", "tabpanel")
                        .attr("id", screen.panel_id())
                        .attr("aria-labelledby", screen.tab_dom_id()),
                ) as ConsultationView,
            ],
        )
        .attr("class", "cleromancy-consultation")
        .attr("data-screen", screen.key()),
    )
}

fn surface_tabs_state(ui: &mut ConsultationUi) -> &mut SelectionState {
    &mut ui.surface_tabs
}

fn never_tab_action(_: &mut SelectionState, _: ()) {
    unreachable!("the surface tab bar does not bubble unit actions")
}

fn labelled_control(label: &str, id: &str, control: ConsultationView) -> ConsultationView {
    Box::new(
        el::<_, ConsultationUi, ()>(
            "div",
            vec![
                Box::new(
                    el::<_, ConsultationUi, ()>("span", label.to_string())
                        .attr("class", "control-label"),
                ) as ConsultationView,
                control,
            ],
        )
        .attr("class", "control")
        .attr("id", id.to_string()),
    )
}

fn labelled_text(
    label: &str,
    id: &str,
    multiline: bool,
    input: &TextInput,
    state: fn(&mut ConsultationUi) -> &mut TextInput,
) -> ConsultationView {
    let field = if multiline {
        textarea_typed(input)
    } else {
        text_field_typed(input)
    };
    let field = map_action(field, never_text_action);
    let field = map_state(field, state);
    Box::new(
        el::<_, ConsultationUi, ()>(
            "label",
            (
                el("span", label.to_string()).attr("class", "control-label"),
                field,
            ),
        )
        .attr("class", "control")
        .attr("id", id.to_string())
        .attr("aria-label", label.to_string())
        .attr("data-control", label.to_string()),
    )
}

fn mode_label(mode: SelectionMode) -> &'static str {
    match mode {
        SelectionMode::Calculated => "Calculated",
        SelectionMode::Cast => "Cast",
        SelectionMode::Derived => "Derived",
    }
}

fn short_digest(digest: &str) -> &str {
    digest.get(..12).unwrap_or(digest)
}

fn never_text_action(_: &mut TextInput, _: ()) {
    unreachable!("text controls do not bubble unit actions")
}

fn never_select_action(_: &mut SelectState, _: ()) {
    unreachable!("select controls do not bubble unit actions")
}
