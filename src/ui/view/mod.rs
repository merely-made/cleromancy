// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Pure Cambium view construction over [`ConsultationUi`], one submodule per
//! semantic region.

use cambium::{AnyView, GenetCtx, GenetElement, SelectState, TextInput, el, map_action, map_state,
    text_field_typed, textarea_typed};

use super::state::{ConsultationAction, ConsultationUi};
use crate::SelectionMode;

mod consultation;
mod journal;
mod reading;

use consultation::consultation_region;
use journal::journal_region;
use reading::reading_region;

pub type ConsultationView =
    Box<dyn AnyView<ConsultationUi, ConsultationAction, GenetCtx, GenetElement>>;

pub fn consultation_view(ui: &ConsultationUi) -> ConsultationView {
    let mut chrome = Vec::new();
    chrome.push(Box::new(
        el::<_, ConsultationUi, ConsultationAction>(
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
        el::<_, ConsultationUi, ConsultationAction>("p", ui.status.label())
            .attr("role", "status")
            .attr("aria-live", "polite")
            .attr("data-key", "consultation-status"),
    ));
    if let Some(error) = &ui.error {
        chrome.push(Box::new(
            el::<_, ConsultationUi, ConsultationAction>("p", error.clone())
                .attr("role", "alert")
                .attr("data-key", "consultation-error"),
        ));
    }
    let regions = vec![
        consultation_region(ui),
        reading_region(ui),
        journal_region(ui),
    ];

    Box::new(
        el::<_, ConsultationUi, ConsultationAction>(
            "div",
            vec![
                Box::new(el::<_, ConsultationUi, ConsultationAction>("div", chrome))
                    as ConsultationView,
                Box::new(
                    el::<_, ConsultationUi, ConsultationAction>("main", regions)
                        .attr("class", "cleromancy-regions"),
                ) as ConsultationView,
            ],
        )
        .attr("class", "cleromancy-consultation")
        .attr("data-screen", ui.screen.key()),
    )
}

fn labelled_control(label: &str, id: &str, control: ConsultationView) -> ConsultationView {
    Box::new(
        el::<_, ConsultationUi, ConsultationAction>(
            "div",
            vec![
                Box::new(
                    el::<_, ConsultationUi, ConsultationAction>("span", label.to_string())
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
        el::<_, ConsultationUi, ConsultationAction>(
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

fn never_text_action(_: &mut TextInput, _: ()) -> ConsultationAction {
    unreachable!("text controls do not bubble unit actions")
}

fn never_select_action(_: &mut SelectState, _: ()) -> ConsultationAction {
    unreachable!("select controls do not bubble unit actions")
}
