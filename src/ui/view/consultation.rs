// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The consultation region: context form, field and mode selection, layout
//! authoring, and chart-fact import.

use cambium::{
    RadioGroup, SelectState, TextInput, button, el, map_action, map_state, radio_group, select,
};

use super::{ConsultationView, labelled_control, labelled_text, never_select_action, short_digest};
use crate::Field;
use crate::tarot::RWS_MAJOR_ARCANA_ID;
use crate::ui::state::{ConsultationAction, ConsultationUi};

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
        Box::new(el::<_, ConsultationUi, ConsultationAction>("h2", "Consultation")),
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
            el::<_, ConsultationUi, ConsultationAction>(
                "p",
                "Additional facts use one disclosed name: value line. A changed form creates a new context snapshot; a selected context is reused unchanged.",
            )
            .attr("class", "context-explanation"),
        ),
        labelled_control("Stored field", "cleromancy-field", Box::new(field_select)),
        labelled_control("Selection mode", "cleromancy-mode", Box::new(mode)),
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
        labelled_control("Reading shape", "cleromancy-layout", Box::new(layout)),
        labelled_control(
            "Authored layout",
            "cleromancy-template-select",
            Box::new(template_select),
        ),
        labelled_control(
            "Astrology facts to associate",
            "cleromancy-astrology-facts",
            Box::new(astrology_facts_select),
        ),
        Box::new(
            el::<_, ConsultationUi, ConsultationAction>(
                "p",
                "Calculated follows the highest disclosed qualified weight. Cast makes a fresh choice from operating-system cryptographic randomness. Derived hashes the public seed and domain into a replayable choice; it is not fresh entropy. Derived is single-card only. Multi-position layouts are always cast. Chosen chart facts are recorded as a concurrence, never as a cause or interpretation.",
            )
            .attr("class", "selection-explanation"),
        ),
    ];
    children.extend([
        Box::new(el::<_, ConsultationUi, ConsultationAction>("h3", "Author a layout"))
            as ConsultationView,
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
            el::<_, ConsultationUi, ConsultationAction>(
                "p",
                "Positions use name | label. Relationships use from | supports, contradicts, questions, next_step, or elaborates | to | label.",
            )
            .attr("class", "context-explanation"),
        ) as ConsultationView,
        Box::new(
            button("Save layout", |ui: &mut ConsultationUi, _| {
                ui.request_spread_template()
            })
            .attr("data-key", "save-layout")
            .attr("aria-label", "Save authored layout"),
        ) as ConsultationView,
        Box::new(el::<_, ConsultationUi, ConsultationAction>("h3", "Chart moment"))
            as ConsultationView,
        labelled_text(
            "UTC instant",
            "cleromancy-astrology-instant",
            false,
            &ui.astrology_instant_utc,
            astrology_instant_state,
        ),
        labelled_text(
            "Latitude microdegrees (optional)",
            "cleromancy-astrology-latitude",
            false,
            &ui.astrology_latitude,
            astrology_latitude_state,
        ),
        labelled_text(
            "Longitude microdegrees (optional)",
            "cleromancy-astrology-longitude",
            false,
            &ui.astrology_longitude,
            astrology_longitude_state,
        ),
        labelled_text(
            "Aspect orb millidegrees",
            "cleromancy-astrology-orb",
            false,
            &ui.astrology_orb,
            astrology_orb_state,
        ),
    ]);
    #[cfg(feature = "ephemeris")]
    children.extend(ephemeris_controls(ui));
    children.extend([
        Box::new(el::<_, ConsultationUi, ConsultationAction>("h3", "Import chart"))
            as ConsultationView,
        labelled_text(
            "Calculation algorithm",
            "cleromancy-astrology-algorithm",
            false,
            &ui.astrology_algorithm,
            astrology_algorithm_state,
        ),
        labelled_text(
            "Calculation engine",
            "cleromancy-astrology-engine",
            false,
            &ui.astrology_engine,
            astrology_engine_state,
        ),
        labelled_text(
            "Ephemeris source",
            "cleromancy-astrology-ephemeris",
            false,
            &ui.astrology_ephemeris,
            astrology_ephemeris_state,
        ),
        labelled_text(
            "Chart positions",
            "cleromancy-astrology-positions",
            true,
            &ui.astrology_positions,
            astrology_positions_state,
        ),
        Box::new(
            el::<_, ConsultationUi, ConsultationAction>(
                "p",
                "For a manual import, identify the algorithm, engine, and ephemeris, then copy positions as body | longitude millidegrees | latitude millidegrees | retrograde. Local calculation ignores those manual source and position fields.",
            )
            .attr("class", "context-explanation"),
        ) as ConsultationView,
        Box::new(
            button("Save chart facts", |ui: &mut ConsultationUi, _| {
                ui.request_astrology_chart()
            })
            .attr("data-key", "save-chart-facts")
            .attr("aria-label", "Save imported chart facts"),
        ) as ConsultationView,
        Box::new(
            button("Read", |ui: &mut ConsultationUi, _| ui.request_read())
                .attr("data-key", "read")
                .attr("aria-label", "Read this consultation"),
        ) as ConsultationView,
    ]);

    Box::new(
        el::<_, ConsultationUi, ConsultationAction>("section", children)
            .attr("role", "region")
            .attr("aria-label", "Consultation")
            .attr("data-key", "region:consultation"),
    )
}

#[cfg(feature = "ephemeris")]
fn ephemeris_controls(ui: &ConsultationUi) -> Vec<ConsultationView> {
    use crate::EphemerisStatus;

    let status = match &ui.ephemeris_status {
        EphemerisStatus::Missing => {
            "NASA/JPL DE440s is not installed. Installation downloads 31 MiB from NAIF and verifies its exact checksum before use.".to_string()
        }
        EphemerisStatus::Ready { path } => {
            format!("Verified NASA/JPL DE440s ready at {}.", path.display())
        }
        EphemerisStatus::Invalid { path, detail } => format!(
            "The local ephemeris at {} is invalid: {detail}. Installing again preserves it as a rejected file.",
            path.display()
        ),
    };
    let mut controls = vec![Box::new(
        el::<_, ConsultationUi, ConsultationAction>("p", status)
            .attr("class", "context-explanation")
            .attr("data-key", "ephemeris-status"),
    ) as ConsultationView];
    if !ui.ephemeris_status.is_ready() {
        controls.push(Box::new(
            button("Install NASA ephemeris", |ui: &mut ConsultationUi, _| {
                ui.request_ephemeris_install()
            })
            .attr("data-key", "install-ephemeris")
            .attr("aria-label", "Install verified NASA JPL ephemeris"),
        ));
    }
    controls.push(Box::new(
        button("Calculate and save chart", |ui: &mut ConsultationUi, _| {
            ui.request_calculated_astrology_chart()
        })
        .attr("data-key", "calculate-chart")
        .attr("aria-label", "Calculate and save astrology chart"),
    ));
    controls
}

fn field_label(field: &Field) -> String {
    if field.system == RWS_MAJOR_ARCANA_ID {
        "Rider-Waite-Smith Major Arcana".to_string()
    } else {
        field.system.clone()
    }
}

fn never_radio_action(_: &mut RadioGroup, _: ()) -> ConsultationAction {
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

fn astrology_algorithm_state(ui: &mut ConsultationUi) -> &mut TextInput {
    &mut ui.astrology_algorithm
}

fn astrology_engine_state(ui: &mut ConsultationUi) -> &mut TextInput {
    &mut ui.astrology_engine
}

fn astrology_ephemeris_state(ui: &mut ConsultationUi) -> &mut TextInput {
    &mut ui.astrology_ephemeris
}

fn astrology_instant_state(ui: &mut ConsultationUi) -> &mut TextInput {
    &mut ui.astrology_instant_utc
}

fn astrology_latitude_state(ui: &mut ConsultationUi) -> &mut TextInput {
    &mut ui.astrology_latitude
}

fn astrology_longitude_state(ui: &mut ConsultationUi) -> &mut TextInput {
    &mut ui.astrology_longitude
}

fn astrology_orb_state(ui: &mut ConsultationUi) -> &mut TextInput {
    &mut ui.astrology_orb
}

fn astrology_positions_state(ui: &mut ConsultationUi) -> &mut TextInput {
    &mut ui.astrology_positions
}
