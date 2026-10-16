// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Product-authored symbolic prompts over saved chart facts. This view never
//! changes the ephemeris output or presents an interpretation as a sky fact.
use super::ConsultationView;
use crate::AstrologyFacts;
use crate::ui::{
    screen::{self, ConsultationScreen},
    state::ConsultationUi,
};
use cambium::{button, el};

pub(super) fn reading(ui: &ConsultationUi, facts: &AstrologyFacts) -> ConsultationView {
    let pack = crate::content::astrology_pack();
    let mut children: Vec<ConsultationView> = vec![
        Box::new(el::<_, ConsultationUi, ()>("h3", "Astrological reading")),
        Box::new(el::<_, ConsultationUi, ()>("p", pack.intro.clone())),
    ];
    for prompt in pack.prompts(facts) {
        children.push(Box::new(
            el::<_, ConsultationUi, ()>("article", (el("h4", prompt.title), el("p", prompt.text)))
                .attr("class", "astrology-prompt")
                .attr("data-key", prompt.key),
        ));
    }
    children.push(Box::new(el::<_, ConsultationUi, ()>(
        "h4",
        "How this reading was made",
    )));
    children.push(Box::new(
        el::<_, ConsultationUi, ()>("p", pack.rationale(facts))
            .attr("data-key", "astrology-rationale"),
    ));
    children.push(Box::new(
        el::<_, ConsultationUi, ()>(
            "p",
            format!(
                "{} · edition {} · {} · {}",
                pack.id, pack.version, pack.author, pack.date
            ),
        )
        .attr("data-key", "astrology-content-edition")
        .attr("data-content-digest", pack.digest()),
    ));
    let digest = facts.digest();
    let available = ui
        .catalog
        .astrology_facts
        .iter()
        .any(|item| item.digest() == digest);
    if available {
        children.push(Box::new(
            button(
                "Draw cards alongside this chart",
                move |ui: &mut ConsultationUi, _| {
                    if let Some(index) = ui
                        .catalog
                        .astrology_facts
                        .iter()
                        .position(|item| item.digest() == digest)
                    {
                        ui.astrology_facts_select.selected = index + 1;
                        screen::select_surface(&mut ui.surface_tabs, ConsultationScreen::Today);
                    }
                },
            )
            .attr("data-key", "read-with-chart"),
        ));
    }
    Box::new(el::<_, ConsultationUi, ()>("div", children).attr("data-key", "astrological-reading"))
}
