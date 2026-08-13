// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

use cambium::{
    AnyView, DetailRow, DetailSection, GenetCtx, GenetElement, RadioGroup, SelectState, TextInput,
    button, detail_panel, disclosure, el, map_action, map_state, radio_group, select,
    text_field_typed, textarea_typed,
};

use crate::tarot::RWS_MAJOR_ARCANA_ID;
use crate::{Field, Reading, SelectionMode};

use super::state::{ConsultationAction, ConsultationUi};

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

fn consultation_region(ui: &ConsultationUi) -> ConsultationView {
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
        Box::new(el::<_, ConsultationUi, ConsultationAction>("h3", "Import chart facts"))
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
                "Copy source-qualified positions as body | longitude millidegrees | latitude millidegrees | retrograde. Cleromancy validates and derives facts; it does not calculate an ephemeris.",
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

fn reading_region(ui: &ConsultationUi) -> ConsultationView {
    let mut children: Vec<ConsultationView> =
        vec![Box::new(el::<_, ConsultationUi, ConsultationAction>(
            "h2", "Reading",
        ))];
    match ui.detail.as_ref() {
        None => children.push(Box::new(
            el::<_, ConsultationUi, ConsultationAction>(
                "p",
                "Enter a context and make a reading to see its prompt and receipt.",
            )
            .attr("class", "empty-reading"),
        )),
        Some(detail) => {
            for (index, (placement, reading)) in detail
                .session
                .placements
                .iter()
                .zip(&detail.readings)
                .enumerate()
            {
                children.push(Box::new(
                    el::<_, ConsultationUi, ConsultationAction>("h3", placement.position.clone())
                        .attr(
                            "data-key",
                            format!("reading-position:{}", placement.position),
                        ),
                ));
                children.push(Box::new(
                    el::<_, ConsultationUi, ConsultationAction>("h4", reading.title.clone()).attr(
                        "data-key",
                        if index == 0 {
                            "result-title".to_string()
                        } else {
                            format!("result-title:{}", placement.position)
                        },
                    ),
                ));
                children.push(Box::new(
                    el::<_, ConsultationUi, ConsultationAction>(
                        "p",
                        reading.interpretation.clone(),
                    )
                    .attr(
                        "data-key",
                        if index == 0 {
                            "result-prompt".to_string()
                        } else {
                            format!("result-prompt:{}", placement.position)
                        },
                    ),
                ));
            }
            let sections = detail
                .session
                .placements
                .iter()
                .zip(&detail.readings)
                .map(|(placement, reading)| receipt_section(&placement.position, reading))
                .collect::<Vec<_>>();
            let details = detail_panel::<cambium::DisclosureState, ()>(&sections);
            let workings = map_action(disclosure(&ui.workings, details), never_disclosure_action);
            let workings = map_state(workings, workings_state);
            children.push(Box::new(workings));
        }
    }

    Box::new(
        el::<_, ConsultationUi, ConsultationAction>("section", children)
            .attr("role", "region")
            .attr("aria-label", "Reading")
            .attr("data-key", "region:reading"),
    )
}

fn journal_region(ui: &ConsultationUi) -> ConsultationView {
    let mut children: Vec<ConsultationView> =
        vec![Box::new(el::<_, ConsultationUi, ConsultationAction>(
            "h2", "Journal",
        ))];
    children.push(labelled_text(
        "Reflection",
        "cleromancy-reflection",
        true,
        &ui.reflection,
        reflection_state,
    ));
    children.push(Box::new(
        button("Add reflection", |ui: &mut ConsultationUi, _| {
            ui.request_reflection()
        })
        .attr("data-key", "save-reflection")
        .attr("aria-label", "Add reflection"),
    ));
    children.push(Box::new(
        el::<_, ConsultationUi, ConsultationAction>(
            "p",
            "Each follow-up is saved as a separate immutable note.",
        )
        .attr("class", "reflection-explanation"),
    ));

    if let Some(detail) = &ui.detail {
        for reflection in &detail.reflections {
            children.push(Box::new(
                el::<_, ConsultationUi, ConsultationAction>("article", reflection.body.clone())
                    .attr("data-key", format!("reflection:{}", reflection.id))
                    .attr("aria-label", "Saved reflection"),
            ));
        }
    }

    children.push(Box::new(el::<_, ConsultationUi, ConsultationAction>(
        "h3",
        "Recent sessions",
    )));
    if ui.catalog.sessions.is_empty() {
        children.push(Box::new(el::<_, ConsultationUi, ConsultationAction>(
            "p",
            "No saved sessions yet.",
        )));
    } else {
        for session in &ui.catalog.sessions {
            let id = session.id.clone();
            let label = format!("Session {}", short_digest(&id));
            children.push(Box::new(
                button(label.clone(), move |ui: &mut ConsultationUi, _| {
                    ui.request_session(id.clone())
                })
                .attr("data-key", format!("session:{}", session.id))
                .attr("aria-label", format!("Open {label}")),
            ));
        }
    }

    let comparison_sessions = ui
        .catalog
        .sessions
        .iter()
        .filter(|session| Some(&session.id) != ui.detail.as_ref().map(|detail| &detail.session.id))
        .collect::<Vec<_>>();
    let comparison_labels = std::iter::once("Choose a saved session".to_string())
        .chain(
            comparison_sessions
                .iter()
                .map(|session| format!("Session {}", short_digest(&session.id))),
        )
        .collect::<Vec<_>>();
    let comparison_refs = comparison_labels
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let comparison_select = map_action(
        select(&ui.comparison_select, &comparison_refs),
        never_select_action,
    );
    let comparison_select = map_state(comparison_select, comparison_select_state);
    children.push(Box::new(el::<_, ConsultationUi, ConsultationAction>(
        "h3",
        "Receipt comparison",
    )));
    children.push(labelled_control(
        "Compare with",
        "cleromancy-compare-with",
        Box::new(comparison_select),
    ));
    children.push(Box::new(
        button("Compare receipts", |ui: &mut ConsultationUi, _| {
            ui.request_comparison()
        })
        .attr("data-key", "compare-receipts")
        .attr("aria-label", "Compare receipts"),
    ));
    if let Some(comparison) = &ui.comparison {
        children.push(Box::new(
            el::<_, ConsultationUi, ConsultationAction>(
                "p",
                format!(
                    "Context: {}; field: {}; position names: {}.",
                    same_or_different(comparison.same_context),
                    same_or_different(comparison.same_field),
                    same_or_different(comparison.same_position_names),
                ),
            )
            .attr("data-key", "receipt-comparison-summary"),
        ));
        for entry in &comparison.entries {
            children.push(Box::new(
                el::<_, ConsultationUi, ConsultationAction>(
                    "article",
                    (
                        el("h4", entry.position.clone()),
                        el(
                            "p",
                            format!(
                                "Candidate: {} / {}. Mode: {} / {}. Receipt: {}.",
                                entry.left_candidate.as_deref().unwrap_or("not present"),
                                entry.right_candidate.as_deref().unwrap_or("not present"),
                                entry.left_mode.map(mode_label).unwrap_or("not present"),
                                entry.right_mode.map(mode_label).unwrap_or("not present"),
                                entry
                                    .same_receipt
                                    .map(same_or_different)
                                    .unwrap_or("not comparable"),
                            ),
                        ),
                    ),
                )
                .attr("data-key", format!("receipt-comparison:{}", entry.position))
                .attr("aria-label", "Receipt comparison entry"),
            ));
        }
    }

    Box::new(
        el::<_, ConsultationUi, ConsultationAction>("section", children)
            .attr("role", "region")
            .attr("aria-label", "Journal")
            .attr("data-key", "region:journal"),
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

fn field_label(field: &Field) -> String {
    if field.system == RWS_MAJOR_ARCANA_ID {
        "Rider-Waite-Smith Major Arcana".to_string()
    } else {
        field.system.clone()
    }
}

fn mode_label(mode: SelectionMode) -> &'static str {
    match mode {
        SelectionMode::Calculated => "Calculated",
        SelectionMode::Cast => "Cast",
        SelectionMode::Derived => "Derived",
    }
}

fn receipt_section(position: &str, reading: &Reading) -> DetailSection {
    let receipt = &reading.receipt;
    let mut rows = vec![
        DetailRow::new("Mode", mode_label(receipt.mode)),
        DetailRow::new("Algorithm", receipt.algorithm.clone()),
        DetailRow::new("Context digest", receipt.context_digest.clone()),
        DetailRow::new("Field digest", receipt.field_digest.clone()),
        DetailRow::new(
            "Qualified weights",
            receipt
                .qualified_weights
                .iter()
                .map(u64::to_string)
                .collect::<Vec<_>>()
                .join(", "),
        ),
        DetailRow::new("Total weight", receipt.total_weight.to_string()),
        DetailRow::new(
            "Bounded sample",
            receipt
                .sample
                .map_or_else(|| "not used".to_string(), |sample| sample.to_string()),
        ),
    ];
    if let Some(derivation) = &receipt.derivation {
        rows.push(DetailRow::new("Derived seed", derivation.seed.clone()));
        rows.push(DetailRow::new("Derived domain", derivation.domain.clone()));
    }
    if let Some(digest) = &receipt.derivation_digest {
        rows.push(DetailRow::new("Derivation digest", digest.clone()));
    }
    rows.push(DetailRow::new(
        "Selected index",
        receipt.selected_index.to_string(),
    ));
    DetailSection::new(format!("{} receipt", position), rows)
}

fn same_or_different(value: bool) -> &'static str {
    if value { "same" } else { "different" }
}

fn short_digest(digest: &str) -> &str {
    digest.get(..12).unwrap_or(digest)
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

fn comparison_select_state(ui: &mut ConsultationUi) -> &mut SelectState {
    &mut ui.comparison_select
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

fn reflection_state(ui: &mut ConsultationUi) -> &mut TextInput {
    &mut ui.reflection
}

fn workings_state(ui: &mut ConsultationUi) -> &mut cambium::DisclosureState {
    &mut ui.workings
}

fn never_text_action(_: &mut TextInput, _: ()) -> ConsultationAction {
    unreachable!("text controls do not bubble unit actions")
}

fn never_select_action(_: &mut SelectState, _: ()) -> ConsultationAction {
    unreachable!("select controls do not bubble unit actions")
}

fn never_radio_action(_: &mut RadioGroup, _: ()) -> ConsultationAction {
    unreachable!("radio controls do not bubble unit actions")
}

fn never_disclosure_action(_: &mut cambium::DisclosureState, _: ()) -> ConsultationAction {
    unreachable!("disclosures do not bubble unit actions")
}
