// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Product-authored symbolic prompts over saved chart facts. This view never
//! changes the ephemeris output or presents an interpretation as a sky fact.
use super::ConsultationView;
use crate::ui::{
    screen::{self, ConsultationScreen},
    state::ConsultationUi,
};
use crate::{AspectKind, AstrologyFacts, ZodiacSign};
use cambium::{button, el};

pub(super) fn reading(ui: &ConsultationUi, facts: &AstrologyFacts) -> ConsultationView {
    let mut children: Vec<ConsultationView> = vec![
        Box::new(el::<_, ConsultationUi, ()>("h3", "Astrological reading")),
        Box::new(el::<_, ConsultationUi, ()>(
            "p",
            "Read the chart as a set of symbolic prompts. Start with a placement that catches your attention, then consider how the aspects connect its themes.",
        )),
    ];
    let mut placements = facts.placements.iter().collect::<Vec<_>>();
    let order = [
        "sun", "moon", "mercury", "venus", "mars", "jupiter", "saturn", "uranus", "neptune",
        "pluto",
    ];
    placements.sort_by_key(|placement| {
        order
            .iter()
            .position(|body| *body == placement.body.to_ascii_lowercase())
            .unwrap_or(order.len())
    });
    for placement in placements {
        let Some(topic) = body_theme(&placement.body) else {
            continue;
        };
        let (approach, question) = sign_theme(placement.sign);
        let mut words = format!("Consider {topic} through {approach}. {question}");
        if placement.retrograde == Some(true) {
            words.push_str(" This body is marked retrograde in the saved chart. As a symbolic lens, revisit an old approach to this theme before pushing ahead.");
        }
        children.push(Box::new(
            el::<_, ConsultationUi, ()>(
                "article",
                (
                    el(
                        "h4",
                        format!(
                            "{} in {:?}",
                            crate::reading_scene::position_label(&placement.body),
                            placement.sign
                        ),
                    ),
                    el("p", words),
                ),
            )
            .attr("class", "astrology-prompt")
            .attr("data-key", format!("astrology-prompt:{}", placement.body)),
        ));
    }
    for aspect in &facts.aspects {
        let (Some(first), Some(second)) = (body_theme(&aspect.first), body_theme(&aspect.second))
        else {
            continue;
        };
        let lens = match aspect.kind {
            AspectKind::Conjunction => {
                "Bring these themes together. Where do they reinforce each other, and where might they become difficult to separate?"
            },
            AspectKind::Sextile => {
                "Look for an opening between these themes. What small act could help them cooperate?"
            },
            AspectKind::Square => {
                "Look for friction between these themes. What adjustment would make room for both?"
            },
            AspectKind::Trine => {
                "Look for ease between these themes. What available strength could you put to deliberate use?"
            },
            AspectKind::Opposition => {
                "Consider these themes as two ends of a conversation. What does each need from the other?"
            },
        };
        children.push(Box::new(
            el::<_, ConsultationUi, ()>(
                "article",
                (
                    el(
                        "h4",
                        format!("{} · {:?} · {}", aspect.first, aspect.kind, aspect.second),
                    ),
                    el("p", format!("Consider {first} alongside {second}. {lens}")),
                ),
            )
            .attr("class", "astrology-prompt"),
        ));
    }
    children.push(Box::new(el::<_, ConsultationUi, ()>(
        "h4",
        "How this reading was made",
    )));
    children.push(Box::new(el::<_, ConsultationUi, ()>("p", format!(
        "The saved chart supplies the positions. Each sign occupies a 30-degree part of the zodiac; aspects connect bodies near particular angles, allowing a margin of {:.1} degrees in this chart. Cleromancy's symbolic prompts (edition 1) pair a body with a theme, a sign with an approach, and an aspect with a relationship. These are authored invitations to reflect, not predictions or measured effects. This reading concerns the saved moment; it does not infer a birth chart, houses, or a rising sign.", facts.orb_millidegrees as f64 / 1000.0
    )).attr("data-key", "astrology-rationale")));
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

fn body_theme(body: &str) -> Option<&'static str> {
    Some(match body.to_ascii_lowercase().as_str() {
        "sun" => "purpose and self-expression",
        "moon" => "emotional needs and familiar habits",
        "mercury" => "thinking, learning, and communication",
        "venus" => "affection, pleasure, and what you value",
        "mars" => "initiative, desire, and the use of effort",
        "jupiter" => "growth, confidence, and possibility",
        "saturn" => "limits, responsibility, and patient work",
        "uranus" => "freedom and changes to established patterns",
        "neptune" => "imagination, ideals, and uncertainty",
        "pluto" => "power, endings, and deep change",
        _ => return None,
    })
}

fn sign_theme(sign: ZodiacSign) -> (&'static str, &'static str) {
    match sign {
        ZodiacSign::Aries => (
            "initiative and directness",
            "Where would taking a first step clarify things?",
        ),
        ZodiacSign::Taurus => (
            "steadiness and tangible needs",
            "What deserves patient care, and what are you holding too tightly?",
        ),
        ZodiacSign::Gemini => (
            "curiosity and conversation",
            "What changes when you ask a different question?",
        ),
        ZodiacSign::Cancer => (
            "care and belonging",
            "What helps you feel supported enough to respond honestly?",
        ),
        ZodiacSign::Leo => (
            "creative expression and visibility",
            "What would you make or say if you allowed it to matter?",
        ),
        ZodiacSign::Virgo => (
            "attention and useful practice",
            "What small adjustment would improve the whole?",
        ),
        ZodiacSign::Libra => (
            "reciprocity and balance",
            "Whose perspective could help you make a fairer choice?",
        ),
        ZodiacSign::Scorpio => (
            "depth and trust",
            "What needs an honest encounter rather than a quick explanation?",
        ),
        ZodiacSign::Sagittarius => (
            "exploration and meaning",
            "Which assumption would benefit from a wider view?",
        ),
        ZodiacSign::Capricorn => (
            "structure and commitment",
            "What can you sustain, and what responsibility is actually yours?",
        ),
        ZodiacSign::Aquarius => (
            "independence and collective possibility",
            "What pattern could you question with others?",
        ),
        ZodiacSign::Pisces => (
            "sensitivity and imagination",
            "Where could compassion help, and where would a clearer boundary help?",
        ),
    }
}
