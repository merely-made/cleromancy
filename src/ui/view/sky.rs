// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Stored sky-day event facts, never a live astronomy calculation.

use cambium::{
    ListRow, ListSection, SelectState, disclosure, el, map_action, map_state, sectioned_list,
    select,
};

use super::{ConsultationView, labelled_control, never_select_action, short_digest};
use crate::SkyDayFacts;
use crate::ui::state::ConsultationUi;

pub(super) fn sky_region(ui: &ConsultationUi) -> ConsultationView {
    let mut children: Vec<ConsultationView> =
        vec![Box::new(el::<_, ConsultationUi, ()>("h2", "Sky events"))];
    if ui.catalog.sky_day_facts.is_empty() {
        children.push(Box::new(el::<_, ConsultationUi, ()>(
            "p",
            "No stored sky-day facts yet.",
        )));
        return region(children);
    }

    let labels = ui
        .catalog
        .sky_day_facts
        .iter()
        .map(record_label)
        .collect::<Vec<_>>();
    let options = labels.iter().map(String::as_str).collect::<Vec<_>>();
    let selected = map_action(select(&ui.sky.selected_day, &options), never_select_action);
    children.push(labelled_control(
        "Stored civil day",
        "cleromancy-sky-day",
        Box::new(map_state(selected, sky_day_state)),
    ));

    let index = ui
        .sky
        .selected_day
        .selected
        .min(ui.catalog.sky_day_facts.len() - 1);
    let facts = &ui.catalog.sky_day_facts[index];
    record_summary(facts, &mut children);
    event_ledger(facts, &mut children);
    policy_and_provenance(ui, facts, &mut children);
    region(children)
}

fn region(children: Vec<ConsultationView>) -> ConsultationView {
    Box::new(
        el::<_, ConsultationUi, ()>("section", children)
            .attr("role", "region")
            .attr("aria-label", "Sky")
            .attr("data-key", "region:sky")
            .attr("id", "cleromancy-region-sky"),
    )
}

fn record_summary(facts: &SkyDayFacts, children: &mut Vec<ConsultationView>) {
    let day = civil_day(facts);
    children.push(Box::new(
        el::<_, ConsultationUi, ()>("h3", format!("Record {day}"))
            .attr("data-key", "sky-record-day"),
    ));
    children.push(Box::new(
        el::<_, ConsultationUi, ()>(
            "p",
            format!(
                "UTC civil-day projection: {day}. Observer WGS84: latitude {} microdegrees, longitude {} microdegrees, height {} millimeters.",
                facts.observer.latitude_microdegrees,
                facts.observer.longitude_microdegrees,
                facts.observer.height_millimeters,
            ),
        )
        .attr("data-key", "sky-observer"),
    ));
    children.push(Box::new(
        el::<_, ConsultationUi, ()>("p", format!("Record digest: {}", facts.digest()))
            .attr("data-key", "sky-record-digest"),
    ));
}

fn event_ledger(facts: &SkyDayFacts, children: &mut Vec<ConsultationView>) {
    children.push(Box::new(el::<_, ConsultationUi, ()>(
        "p",
        "Event intervals use Terrestrial Time (TT) Julian days. The civil-day labels above are UTC projections of this stored record.",
    ).attr("data-key", "sky-time-scale")));
    let rows = facts
        .facts
        .iter()
        .map(|fact| {
            ListRow::muted(format!(
                "{}: TT Julian day {:?} to {:?}.",
                fact_name(fact.kind),
                fact.interval.start_julian_day,
                fact.interval.end_julian_day,
            ))
        })
        .collect::<Vec<_>>();
    let sections = [ListSection::new(
        format!("Stored events for {}", civil_day(facts)),
        rows,
    )];
    children.push(Box::new(
        el::<_, ConsultationUi, ()>(
            "div",
            sectioned_list(&sections, |_ui: &mut ConsultationUi, _, _| ()),
        )
        .attr("data-key", "sky-event-ledger"),
    ));
}

fn policy_and_provenance(
    ui: &ConsultationUi,
    facts: &SkyDayFacts,
    children: &mut Vec<ConsultationView>,
) {
    let mut detail: Vec<ConsultationView> = vec![
        Box::new(el::<_, ConsultationUi, ()>("h4", "Numerical policy")),
        Box::new(
            el::<_, ConsultationUi, ()>(
                "p",
                format!(
                    "Phase search: step {} seconds; tolerance {} milliseconds.",
                    facts.policy.phase_search.step_seconds,
                    facts.policy.phase_search.tolerance_milliseconds,
                ),
            )
            .attr("data-key", "sky-policy-phase-search"),
        ),
        Box::new(
            el::<_, ConsultationUi, ()>(
                "p",
                format!(
                    "Twilight policy: {:?}; altitude {} millidegrees; search step {} seconds; tolerance {} milliseconds.",
                    facts.policy.twilight.measurement,
                    facts.policy.twilight.altitude_millidegrees,
                    facts.policy.twilight.search.step_seconds,
                    facts.policy.twilight.search.tolerance_milliseconds,
                ),
            )
            .attr("data-key", "sky-policy-twilight"),
        ),
        Box::new(
            el::<_, ConsultationUi, ()>(
                "p",
                format!(
                    "Earth orientation: authority {}; snapshot {}; approximation {:?}.",
                    facts.policy.earth_orientation.authority,
                    facts.policy.earth_orientation.snapshot,
                    facts.policy.earth_orientation.approximation,
                ),
            )
            .attr("data-key", "sky-policy-earth-orientation"),
        ),
        Box::new(el::<_, ConsultationUi, ()>("h4", "Per-event provenance")),
    ];
    for fact in &facts.facts {
        detail.push(Box::new(
            el::<_, ConsultationUi, ()>(
                "p",
                format!(
                    "{}: model {}; provider {}; provider snapshot {}; transform {}; earth orientation {}.",
                    fact_name(fact.kind),
                    fact.provenance.model,
                    fact.provenance.provider,
                    fact.provenance.provider_snapshot.as_deref().unwrap_or("not recorded"),
                    fact.provenance.transform,
                    fact.provenance.earth_orientation,
                ),
            )
            .attr("data-key", format!("sky-provenance:{}", fact_key(fact.kind))),
        ));
    }
    children.push(Box::new(
        el::<_, ConsultationUi, ()>(
            "div",
            disclosure(
                &ui.sky.policy_and_provenance,
                detail,
                |ui: &mut ConsultationUi| ui.sky.policy_and_provenance.toggle(),
            ),
        )
        .attr("data-key", "sky-policy-and-provenance"),
    ));
}

fn record_label(facts: &SkyDayFacts) -> String {
    format!("{} ({})", civil_day(facts), short_digest(&facts.digest()))
}

fn civil_day(facts: &SkyDayFacts) -> String {
    format!(
        "{:04}-{:02}-{:02}",
        facts.civil_day.year, facts.civil_day.month, facts.civil_day.day,
    )
}

fn fact_name(kind: crate::SkyFactKind) -> &'static str {
    match kind {
        crate::SkyFactKind::NewMoon => "New Moon",
        crate::SkyFactKind::Dawn => "Dawn",
        crate::SkyFactKind::Dusk => "Dusk",
    }
}

fn fact_key(kind: crate::SkyFactKind) -> &'static str {
    match kind {
        crate::SkyFactKind::NewMoon => "new-moon",
        crate::SkyFactKind::Dawn => "dawn",
        crate::SkyFactKind::Dusk => "dusk",
    }
}

fn sky_day_state(ui: &mut ConsultationUi) -> &mut SelectState {
    &mut ui.sky.selected_day
}
