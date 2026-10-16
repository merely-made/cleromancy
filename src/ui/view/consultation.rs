// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The consultation region: context form, field and mode selection, layout
//! authoring, and chart-fact association.

use cambium::{
    RadioGroup, SelectState, TextInput, button, disclosure, el, map_action, map_state, radio_group,
    select,
};

use super::{ConsultationView, labelled_control, labelled_text, never_select_action, short_digest};
use crate::Field;
use crate::tarot::RWS_MAJOR_ARCANA_ID;
use crate::ui::state::ConsultationUi;

pub(super) fn consultation_region(ui: &ConsultationUi) -> ConsultationView {
    let context_labels = std::iter::once("New context".to_string())
        .chain(
            ui.catalog
                .contexts
                .iter()
                .map(|context| context.label.clone()),
        )
        .collect::<Vec<_>>();
    let context_refs = context_labels
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let context_select = map_action(
        select(&ui.context_select, &context_refs),
        never_select_action,
    );
    let context_select = map_state(context_select, context_select_state);

    let field_labels = ui
        .catalog
        .fields
        .iter()
        .map(field_label)
        .collect::<Vec<_>>();
    let field_refs = field_labels.iter().map(String::as_str).collect::<Vec<_>>();
    let field_select = map_action(select(&ui.field_select, &field_refs), never_select_action);
    let field_select = map_state(field_select, field_select_state);

    let die = ui
        .catalog
        .fields
        .get(ui.field_select.selected)
        .is_some_and(|f| f.rules == crate::UNIFORM_DIE_RULE);
    let mode = map_action(
        radio_group(&ui.mode, &["Calculated", "Cast", "Derived"]),
        never_radio_action,
    );
    let mode = map_state(mode, mode_state);
    let layout = map_action(
        radio_group(
            &ui.layout,
            &["Single card", "Three cards", "Authored layout"],
        ),
        never_radio_action,
    );
    let layout = map_state(layout, layout_state);

    let template_labels = std::iter::once("Choose an authored layout".to_string())
        .chain(
            ui.catalog
                .spread_templates
                .iter()
                .map(|template| template.label.clone()),
        )
        .collect::<Vec<_>>();
    let template_refs = template_labels
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let template_select = map_action(
        select(&ui.template_select, &template_refs),
        never_select_action,
    );
    let template_select = map_state(template_select, template_select_state);

    let astrology_labels = std::iter::once("No astrology facts for this reading".to_string())
        .chain(ui.catalog.astrology_facts.iter().map(|facts| {
            format!(
                "Chart {} · {} placements",
                short_digest(&facts.chart_digest),
                facts.placements.len()
            )
        }))
        .collect::<Vec<_>>();
    let astrology_refs = astrology_labels
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let astrology_facts_select = map_action(
        select(&ui.astrology_facts_select, &astrology_refs),
        never_select_action,
    );
    let astrology_facts_select = map_state(astrology_facts_select, astrology_facts_select_state);

    let mut children: Vec<ConsultationView> = vec![
        Box::new(el::<_, ConsultationUi, ()>("h2", "Consultation")),
        labelled_control("Context", "cleromancy-context", Box::new(context_select)),
        labelled_text(
            "Context label",
            "cleromancy-context-label",
            false,
            &ui.context_label,
            context_label_state,
        ),
        labelled_text(
            "Question",
            "cleromancy-question",
            true,
            &ui.question,
            question_state,
        ),
        labelled_control("Stored field", "cleromancy-field", Box::new(field_select)),
        Box::new(
            button("Add tarot decks", |ui: &mut ConsultationUi, _| {
                let action = ui.request_typed_decks();
                ui.record_action(action);
            })
            .attr("data-key", "install-typed-decks")
            .attr("class", "install-decks"),
        ),
        Box::new(
            button("Add a six-sided die", |ui: &mut ConsultationUi, _| {
                let action = ui.request_die();
                ui.record_action(action);
            })
            .attr("data-key", "install-six-sided-die"),
        ),
    ];
    if die {
        for (label, key, selected) in [
            ("Use operating-system randomness", "die-os", 1),
            ("Use a public seed", "die-seed", 2),
        ] {
            children.push(Box::new(
                button(label, move |ui: &mut ConsultationUi, _| {
                    ui.mode.selected = selected;
                    ui.layout.selected = 0;
                })
                .attr("data-key", key)
                .attr(
                    "aria-pressed",
                    if ui.mode.selected == selected {
                        "true"
                    } else {
                        "false"
                    },
                ),
            ));
        }
        children.push(Box::new(el::<_, ConsultationUi, ()>("p", "One roll, one outcome. All six faces have equal weight. Separate rolls may repeat a face. A public seed and domain reproduce the same result with the same saved context. There is no orientation or assigned symbolic meaning.").attr("data-key", "die-method").attr("class", "die-method")));
    } else {
        children.push(labelled_control(
            "Selection mode",
            "cleromancy-mode",
            Box::new(mode),
        ));
        children.push(labelled_control(
            "Reading shape",
            "cleromancy-layout",
            Box::new(layout),
        ));
    }
    let context_example = if die {
        "A question to riff on: What am I exploring with this roll? Add the setting or question you want to remember alongside its outcome.".to_string()
    } else {
        ui.detail.as_ref().and_then(|detail| detail.readings.first()).map(|reading| {
        format!("A question to riff on: Where do I notice {} in my situation today?", reading.title)
    }).unwrap_or_else(|| "A question to riff on: What deserves my attention in this change? Add a concrete detail, such as a decision at work or a habit you are reconsidering.".into());
    };
    children.push(Box::new(
        el::<_, ConsultationUi, ()>("p", context_example)
            .attr("class", "context-explanation")
            .attr("data-key", "context-example"),
    ));
    if !die && ui.layout.selected == 2 {
        children.push(labelled_control(
            "Authored layout",
            "cleromancy-template-select",
            Box::new(template_select),
        ));
    }
    if ui.mode.selected == 2 {
        children.extend([
            labelled_text(
                "Derived seed",
                "cleromancy-derived-seed",
                false,
                &ui.derived_seed,
                derived_seed_state,
            ),
            labelled_text(
                "Derived domain",
                "cleromancy-derived-domain",
                false,
                &ui.derived_domain,
                derived_domain_state,
            ),
        ]);
    }
    children.push(Box::new(
        button(
            "Explore an astrological reading",
            |ui: &mut ConsultationUi, _| {
                super::screen::select_surface(
                    &mut ui.surface_tabs,
                    super::ConsultationScreen::Chart,
                );
            },
        )
        .attr("data-key", "explore-astrology"),
    ));
    children.push(super::situation::controls(ui));
    children.push(Box::new(
        button(
            if die { "Roll the die" } else { "Read" },
            |ui: &mut ConsultationUi, _| {
                let action = ui.request_read();
                ui.record_action(action);
            },
        )
        .attr("data-key", "read")
        .attr("aria-label", "Read this consultation")
        .attr("class", "read-button"),
    ));
    children.push(Box::new(
        el::<_, ConsultationUi, ()>(
            "p",
            "Readings are saved automatically. Add a reflection after you read.",
        )
        .attr("class", "selection-explanation"),
    ));
    children.extend([
        labelled_text(
            "Tags",
            "cleromancy-tags",
            false,
            &ui.tags,
            tags_state,
        ),
        labelled_text(
            "Additional facts",
            "cleromancy-additional-facts",
            true,
            &ui.additional_facts,
            additional_facts_state,
        ),
        Box::new(
            el::<_, ConsultationUi, ()>(
                "p",
                "Additional facts use one disclosed name: value line. A changed form creates a new context snapshot; a selected context is reused unchanged.",
            )
            .attr("class", "context-explanation"),
        ),
        labelled_control(
            "Astrology facts to associate",
            "cleromancy-astrology-facts",
            Box::new(astrology_facts_select),
        ),
        Box::new(
            el::<_, ConsultationUi, ()>(
                "p",
                if die { "The saved question and situation stay with this roll. You can add a reflection after seeing its outcome. Chosen chart facts are saved alongside the roll." } else { "Calculated follows the highest disclosed qualified weight. For tarot, Cast shuffles once using operating-system randomness; Derived shuffles from your public seed and domain. Each card occupies one place in the deck, with upright or reversed orientation, and positions take cards from the top without replacement. Context weights do not bias a deck shuffle. Other collections retain their disclosed selection method. Chosen chart facts are saved alongside the reading." },
            )
            .attr("class", "selection-explanation"),
        ),
    ]);
    let authoring: Vec<ConsultationView> = vec![
        labelled_text(
            "Layout label",
            "cleromancy-template-label",
            false,
            &ui.template_label,
            template_label_state,
        ),
        labelled_text(
            "Layout positions",
            "cleromancy-template-positions",
            true,
            &ui.template_positions,
            template_positions_state,
        ),
        labelled_text(
            "Layout relationships",
            "cleromancy-template-relations",
            true,
            &ui.template_relations,
            template_relations_state,
        ),
        Box::new(
            el::<_, ConsultationUi, ()>(
                "p",
                "Positions use name | label. Relationships use from | supports, contradicts, questions, next_step, or elaborates | to | label.",
            )
            .attr("class", "context-explanation"),
        ) as ConsultationView,
        Box::new(
            button("Save layout", |ui: &mut ConsultationUi, _| {
                let action = ui.request_spread_template();
                ui.record_action(action);
            })
            .attr("data-key", "save-layout")
            .attr("aria-label", "Save authored layout"),
        ) as ConsultationView,
    ];
    children.push(Box::new(disclosure(
        &ui.layout_editor,
        authoring,
        |ui: &mut ConsultationUi| ui.layout_editor.toggle(),
    )));

    Box::new(
        el::<_, ConsultationUi, ()>("section", children)
            .attr("role", "region")
            .attr("aria-label", "Consultation")
            .attr("data-key", "region:consultation"),
    )
}

fn field_label(field: &Field) -> String {
    if let Some(deck) = &field.tarot {
        format!("{} ({} cards)", deck.title, deck.cards.len())
    } else if field.rules == crate::UNIFORM_DIE_RULE {
        format!("{}-sided die", field.candidates.len())
    } else if field.system == RWS_MAJOR_ARCANA_ID {
        "Rider-Waite-Smith Major Arcana".to_string()
    } else {
        field.system.clone()
    }
}

fn never_radio_action(_: &mut RadioGroup, _: ()) {
    unreachable!("radio controls do not bubble unit actions")
}

fn context_select_state(ui: &mut ConsultationUi) -> &mut SelectState {
    &mut ui.context_select
}

fn field_select_state(ui: &mut ConsultationUi) -> &mut SelectState {
    &mut ui.field_select
}

fn mode_state(ui: &mut ConsultationUi) -> &mut RadioGroup {
    &mut ui.mode
}

fn derived_seed_state(ui: &mut ConsultationUi) -> &mut TextInput {
    &mut ui.derived_seed
}

fn derived_domain_state(ui: &mut ConsultationUi) -> &mut TextInput {
    &mut ui.derived_domain
}

fn layout_state(ui: &mut ConsultationUi) -> &mut RadioGroup {
    &mut ui.layout
}

fn template_select_state(ui: &mut ConsultationUi) -> &mut SelectState {
    &mut ui.template_select
}

fn astrology_facts_select_state(ui: &mut ConsultationUi) -> &mut SelectState {
    &mut ui.astrology_facts_select
}

fn context_label_state(ui: &mut ConsultationUi) -> &mut TextInput {
    &mut ui.context_label
}

fn question_state(ui: &mut ConsultationUi) -> &mut TextInput {
    &mut ui.question
}

fn tags_state(ui: &mut ConsultationUi) -> &mut TextInput {
    &mut ui.tags
}

fn additional_facts_state(ui: &mut ConsultationUi) -> &mut TextInput {
    &mut ui.additional_facts
}

fn template_label_state(ui: &mut ConsultationUi) -> &mut TextInput {
    &mut ui.template_label
}

fn template_positions_state(ui: &mut ConsultationUi) -> &mut TextInput {
    &mut ui.template_positions
}

fn template_relations_state(ui: &mut ConsultationUi) -> &mut TextInput {
    &mut ui.template_relations
}
