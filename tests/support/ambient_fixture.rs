// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0
use cleromancy::{
    AstrologyChartDraft, CleromancyHost, Consultation, ContextDraft, SelectionMode,
    SpreadTemplateDraft,
};
use mere::kernel::graph::Graph;
use muniment::MemoryBackend;
pub struct ZeroEntropy;
impl cleromancy::moirai::clotho::EntropySource for ZeroEntropy {
    fn next_u64(&mut self) -> Result<u64, cleromancy::ReadingError> {
        Ok(0)
    }
}
pub fn fixture() -> (Consultation<MemoryBackend>, cleromancy::ConsultationDetail) {
    fixture_on(CleromancyHost::candidate(Graph::new()).unwrap())
}
pub fn fixture_on(
    host: CleromancyHost<MemoryBackend>,
) -> (Consultation<MemoryBackend>, cleromancy::ConsultationDetail) {
    let mut c = Consultation::new(host);
    let fields = pollster::block_on(c.install_typed_tarot_at(1)).unwrap();
    for (index, question) in [(1, "Where did I begin?"), (2, "What did I learn?")] {
        let context = pollster::block_on(
            c.save_context_at(ContextDraft::new("Earlier occasion", question, ""), index),
        )
        .unwrap();
        let earlier = pollster::block_on(c.read_at_with_entropy(
            &context,
            &fields[0],
            SelectionMode::Calculated,
            index * 1000,
            index,
            &mut ZeroEntropy,
        ))
        .unwrap();
        pollster::block_on(c.reflect_at_with_entropy(
            &earlier.session.id,
            format!("Earlier reflection {index}"),
            index * 1000 + 10,
            index,
            &mut ZeroEntropy,
        ))
        .unwrap();
    }
    let context = pollster::block_on(c.save_context_at(
        ContextDraft::new(
            "Current occasion",
            "What can I bring to this beginning?",
            "",
        ),
        3,
    ))
    .unwrap();
    let template=pollster::block_on(c.save_spread_template_at(SpreadTemplateDraft::new("Three concerns","foundation | What I must release\nattention | What needs attention\nnext | A next step").with_relations("foundation | questions | attention | release and attention\nattention | next_step | next | a possible next step"),3)).unwrap();
    // An explicit synthetic arrangement of replayable calculated readings.
    // No fixture entropy is represented as an operating-system random draw.
    let mut host = c.into_host();
    let context = host.context_for_digest(&context).unwrap();
    let field = host.field_for_digest(&fields[0]).unwrap();
    let template = host
        .spread_templates()
        .unwrap()
        .into_iter()
        .find(|t| t.id == template)
        .unwrap();
    let reading = cleromancy::ReadingEngine::calculate(&context, &field).unwrap();
    let session = cleromancy::ReadingSession::with_placements(
        3000,
        "c4202610000000000000000000000000",
        context.digest(),
        field.digest(),
        template
            .positions
            .iter()
            .map(|p| cleromancy::ReadingPlacement {
                position: p.name.clone(),
                reading_id: reading.id.clone(),
            })
            .collect(),
        None,
    )
    .unwrap();
    host.insert_session(&context, &field, std::slice::from_ref(&reading), &session)
        .unwrap();
    host.insert_spread(
        &session,
        std::slice::from_ref(&reading),
        &template,
        &cleromancy::Spread::new(&template, &session).unwrap(),
    )
    .unwrap();
    let mut c = Consultation::new(host);
    let detail = c.detail(&session.id).unwrap();
    let facts = pollster::block_on(c.save_astrology_chart_at(
        AstrologyChartDraft {
            algorithm: "source-import/v1".into(),
            engine: "public synthetic chart 1.0".into(),
            ephemeris: "recorded synthetic positions".into(),
            instant_utc: "2026-10-16T12:00:00Z".into(),
            latitude_microdegrees: String::new(),
            longitude_microdegrees: String::new(),
            orb_millidegrees: "1000".into(),
            positions: "Sun | 195000 | 0 | false\nMoon | 225500 | 0 | false".into(),
        },
        3,
    ))
    .unwrap();
    let detail =
        pollster::block_on(c.associate_astrology_facts_at(&facts, &detail.session.id, 3010, 3))
            .unwrap();
    pollster::block_on(c.reflect_at_with_entropy(
        &detail.session.id,
        "I can return to this question after comparing the earlier occasions.".into(),
        3020,
        3,
        &mut ZeroEntropy,
    ))
    .unwrap();
    let detail = c.detail(&detail.session.id).unwrap();
    (c, detail)
}
