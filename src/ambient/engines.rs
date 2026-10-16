// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0
use super::*;
use crate::{Reading, narrative::position_meaning};

pub(super) fn correspondence(
    detail: &ConsultationDetail,
    reading: &Reading,
    catalog: &ConsultationCatalog,
    out: &mut Builder,
) {
    let Some(deck) = &detail.field.tarot else {
        return;
    };
    let Some(card) = deck.card(&reading.candidate_id) else {
        return;
    };
    let root_digest = detail.field.digest();
    for field in &catalog.fields {
        let field_digest = field.digest();
        let Some(other_deck) = &field.tarot else {
            continue;
        };
        for other in &other_deck.cards {
            if field_digest == root_digest && other.id == card.id {
                continue;
            }
            let item = AmbientItem {
                id: format!("card:{}:{}", field_digest, other.id),
                title: format!("{} · {}", other.title, other_deck.title),
                body: other.interpretation.text.clone(),
                source: format!("cleromancy://field/{}#card/{}", field_digest, other.id),
                session_id: None,
                position: None,
            };
            let evidence = vec![
                format!("cleromancy://field/{}", root_digest),
                item.source.clone(),
            ];
            if other.key == card.key {
                out.add(item.clone(), RelationFamily::Correspondence, "typed-archetype-crosswalk/v1", "The same typed card identity in another saved deck; printed numbering may differ.".into(), evidence.clone());
            }
            if field_digest == root_digest {
                if other.key.arcana() == card.key.arcana() {
                    out.add(item.clone(),RelationFamily::Correspondence,"typed-arcana-equality/v1",format!("Same typed arcana: {:?}. This is a category relationship, not a similarity score.",card.key.arcana()),evidence.clone());
                }
                if other.key.suit().is_some() && other.key.suit() == card.key.suit() {
                    out.add(
                        item.clone(),
                        RelationFamily::Correspondence,
                        "typed-suit-equality/v1",
                        format!("Same typed suit: {:?}.", card.key.suit().unwrap()),
                        evidence.clone(),
                    );
                }
                if other.number.is_some() && other.number == card.number {
                    out.add(
                        item.clone(),
                        RelationFamily::Correspondence,
                        "printed-number-equality/v1",
                        format!(
                            "Same printed number: {}. This does not assert the same archetype.",
                            card.number.unwrap()
                        ),
                        evidence.clone(),
                    );
                }
                for a in &card.correspondences {
                    for b in &other.correspondences {
                        if a.tradition != b.tradition
                            || a.version != b.version
                            || a.source_id != b.source_id
                        {
                            continue;
                        }
                        let Some(source) = deck.source(&a.source_id) else {
                            continue;
                        };
                        let mut e = evidence.clone();
                        e.push(source.url.clone());
                        let credit = format!(
                            "{} · {} · {} · {}",
                            a.tradition, source.title, source.date, a.version
                        );
                        for (axis, shared) in [
                            (
                                "element",
                                a.elements
                                    .iter()
                                    .filter(|v| b.elements.contains(v))
                                    .map(|v| format!("{v:?}"))
                                    .collect::<Vec<_>>(),
                            ),
                            (
                                "planet",
                                a.planets
                                    .iter()
                                    .filter(|v| b.planets.contains(v))
                                    .map(|v| format!("{v:?}"))
                                    .collect(),
                            ),
                            (
                                "sign",
                                a.signs
                                    .iter()
                                    .filter(|v| b.signs.contains(v))
                                    .map(|v| format!("{v:?}"))
                                    .collect(),
                            ),
                        ] {
                            if !shared.is_empty() {
                                out.add(item.clone(), RelationFamily::Correspondence, &format!("tradition-{axis}-equality/v1"),
                        format!("Shared {axis}: {}. {credit}. Only explicit transcribed values are compared.", shared.join(", ")), e.clone());
                            }
                        }
                    }
                }
            }
        }
    }
}
fn key(detail: &ConsultationDetail, reading: &Reading) -> Option<crate::cards::CardKey> {
    detail
        .field
        .tarot
        .as_ref()?
        .card(&reading.candidate_id)
        .map(|c| c.key)
}
pub(super) fn history(
    detail: &ConsultationDetail,
    reading: &Reading,
    history: &[ConsultationDetail],
    out: &mut Builder,
) {
    for previous in history {
        if previous.session.id == detail.session.id
            || previous.session.created_at_ms > detail.session.created_at_ms
        {
            continue;
        }
        for placement in &previous.session.placements {
            let Some(other) = previous
                .readings
                .iter()
                .find(|r| r.id == placement.reading_id)
            else {
                continue;
            };
            let same = match (key(detail, reading), key(previous, other)) {
                (Some(a), Some(b)) => a == b,
                _ => {
                    detail.field.digest() == previous.field.digest()
                        && reading.candidate_id == other.candidate_id
                },
            };
            if !same {
                continue;
            }
            let meaning = position_meaning(previous, &placement.position);
            let question = previous
                .context
                .facts
                .get("question")
                .unwrap_or(&previous.context.label);
            let item = AmbientItem {
                id: format!("history:{}:{}", previous.session.id, placement.position),
                title: format!("{} · {}", other.title, meaning.label),
                body: format!("{question}\n{}\n{}", meaning.meaning, other.interpretation),
                source: format!("cleromancy://reading/{}", other.id),
                session_id: Some(previous.session.id.clone()),
                position: Some(placement.position.clone()),
            };
            out.add(item, RelationFamily::History, "prior-card-occurrence/v1", "An earlier saved occurrence of this typed card. Position occurrences remain distinct.".into(),
                vec![format!("cleromancy://reading/{}", reading.id), format!("cleromancy://session/{}", previous.session.id)]);
            for reflection in &previous.reflections {
                out.add(AmbientItem { id: format!("reflection:{}:{}", reflection.id, placement.position), title: format!("Reflection · {}", meaning.label), body: reflection.body.clone(),
                    source: format!("cleromancy://reflection/{}", reflection.id), session_id: Some(previous.session.id.clone()), position: Some(placement.position.clone()) },
                    RelationFamily::History, "saved-reflection-on-occurrence/v1", "A saved reflection on the reading session containing this earlier occurrence; it is not a claim that the note names this particular card.".into(),
                    vec![format!("cleromancy://session/{}", previous.session.id)]);
            }
        }
    }
}
pub(super) fn situation(
    detail: &ConsultationDetail,
    catalog: &ConsultationCatalog,
    out: &mut Builder,
) {
    if let Some(saved) = &detail.situation {
        let source = format!("cleromancy://situation/{}", saved.digest());
        out.add(AmbientItem{id:format!("situation:{}:moon",saved.digest()),title:format!("Moon · {}",saved.sky.phase_band),
            body:format!("{:.1}% illuminated at {}. {}",saved.sky.illuminated_parts_per_million as f64/10_000.,crate::situation::utc_label(saved.instant_ms),saved.sky.provider),
            source:source.clone(),session_id:None,position:None},RelationFamily::Situation,&saved.sky.method,
            "The locally computed sky recorded for this reading occasion; inspection does not recompute a current sky.".into(),vec![source.clone()]);
        if let Some(weather) = &saved.weather {
            out.add(AmbientItem{id:format!("situation:{}:weather",saved.digest()),title:"Saved weather model estimate".into(),
                body:format!("{:.1} °C; wind {:.2} m/s. Valid {}, retrieved {}. {}",weather.values.temperature_millicelsius as f64/1000.,weather.values.wind_millimeters_per_second as f64/1000.,crate::situation::utc_label(weather.values.valid_at_ms),crate::situation::utc_label(weather.retrieved_at_ms),weather.provider),
                source:source.clone(),session_id:None,position:None},RelationFamily::Situation,&weather.method,
                "An explicitly requested network response retained with the draw; this view only reads the stored evidence.".into(),vec![source]);
        }
    }
    out.add(AmbientItem { id: format!("time:{}", detail.session.id), title: "Recorded draw time".into(), body: format!("{} milliseconds since the Unix epoch. This timestamp is stored with the reading session.", detail.session.created_at_ms),
        source: format!("cleromancy://session/{}", detail.session.id), session_id: None, position: None }, RelationFamily::Situation,
        "saved-reading-time/v1", "The time actually recorded with this reading.".into(), vec![format!("cleromancy://session/{}", detail.session.id)]);
    for concurrence in &detail.concurrences {
        for member in &concurrence.members {
            for facts in &catalog.astrology_facts {
                if member.address != format!("cleromancy://astrology/facts/{}", facts.digest()) {
                    continue;
                }
                for placement in &facts.placements {
                    out.add(AmbientItem { id: format!("sky:{}:{}", facts.digest(), placement.body), title: format!("{} · {:?}", placement.body, placement.sign),
                    body: format!("{} in {:?} at {:.3}°. Stored observation from {} using {}. Saved alongside the reading; association does not establish causation.", placement.body, placement.sign,
                        placement.degree_millidegrees as f64 / 1000.0, catalog.astrology_charts.iter().find(|c| c.facts.chart_digest == facts.chart_digest).map(|c| c.chart.moment.instant_utc.as_str()).unwrap_or("stored chart"), facts.algorithm), source: member.address.clone(), session_id: None, position: None },
                    RelationFamily::Situation, "saved-astrology-concurrence/v1", "A recorded chart fact explicitly associated with this reading; no current sky is fetched.".into(),
                    vec![format!("cleromancy://concurrence/{}", concurrence.id), member.address.clone()]);
                }
            }
        }
    }
}
pub(super) fn spread(detail: &ConsultationDetail, position: &str, out: &mut Builder) {
    for placement in &detail.session.placements {
        if placement.position == position {
            continue;
        }
        let Some(reading) = detail
            .readings
            .iter()
            .find(|r| r.id == placement.reading_id)
        else {
            continue;
        };
        let meaning = position_meaning(detail, &placement.position);
        let relations = detail
            .spread_template
            .as_ref()
            .map(|t| {
                t.relations
                    .iter()
                    .filter(|r| {
                        (r.from == position && r.to == placement.position)
                            || (r.to == position && r.from == placement.position)
                    })
                    .map(|r| format!("{:?}: {}", r.kind, r.label))
                    .collect::<Vec<_>>()
                    .join("; ")
            })
            .unwrap_or_default();
        out.add(
            AmbientItem {
                id: format!("reading:{}:{}", detail.session.id, placement.position),
                title: format!("{} · {}", meaning.label, reading.title),
                body: format!("{}\n{}", meaning.meaning, reading.interpretation),
                source: format!("cleromancy://reading/{}", reading.id),
                session_id: Some(detail.session.id.clone()),
                position: Some(placement.position.clone()),
            },
            RelationFamily::Spread,
            "saved-spread-position/v1",
            if relations.is_empty() {
                "Another position in this saved reading; co-membership is not an inferred synergy."
                    .into()
            } else {
                format!("Saved authored relation: {relations}")
            },
            vec![format!("cleromancy://session/{}", detail.session.id)],
        );
    }
}
