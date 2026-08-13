// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use cambium::{DomHandle, GenetAppRunner, Key, KeyEvent, Modifiers, NamedKey, PointerClick};
use cleromancy::moirai::clotho::EntropySource;
use cleromancy::{
    AstrologyChart, AstrologyMoment, AstrologyPosition, CleromancyHost, Consultation,
    ConsultationAction, ConsultationContext, ConsultationLayout, ConsultationUi, ConsultationView,
    ReadingError, SpreadPosition, SpreadRelation, SpreadRelationKind, SpreadTemplate,
    consultation_view,
};
use genet_scripted_dom::{NodeId, ScriptedDom};
use layout_dom_api::{LayoutDom, LocalName, Namespace, NodeKind};
use muniment::MemoryBackend;

type Runner = GenetAppRunner<
    ConsultationUi,
    fn(&ConsultationUi) -> ConsultationView,
    ConsultationView,
    ConsultationAction,
>;

#[test]
fn retained_consultation_dispatches_a_complete_reading_and_reflection() {
    let host = CleromancyHost::empty(MemoryBackend::new());
    let mut consultation = Consultation::new(host);
    pollster::block_on(consultation.install_builtin_tarot_at(1)).unwrap();
    let catalog = consultation.catalog().unwrap();

    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::new(
        dom.clone(),
        consultation_view as fn(&ConsultationUi) -> ConsultationView,
        ConsultationUi::new(catalog),
    );

    assert_eq!(
        find_all_attr(&dom.borrow(), runner.root(), "role", "region").len(),
        3
    );
    for region in ["Consultation", "Reading", "Journal"] {
        assert!(find_attr(&dom.borrow(), runner.root(), "aria-label", region).is_some());
        assert!(find_text(&dom.borrow(), runner.root(), region).is_some());
    }

    click(&mut runner, &dom, "data-key", "read");
    assert!(
        runner
            .state()
            .error()
            .unwrap()
            .contains("context label and question")
    );
    assert!(find_attr(&dom.borrow(), runner.root(), "role", "alert").is_some());

    runner.set_focus(None);
    runner.dispatch_key(tab(false));
    assert_eq!(
        accessible_label(&dom.borrow(), runner.focus().unwrap()),
        Some("Context")
    );
    runner.dispatch_key(tab(false));
    assert_eq!(
        accessible_label(&dom.borrow(), runner.focus().unwrap()),
        Some("Context label")
    );
    runner.dispatch_key(tab(false));
    assert_eq!(
        accessible_label(&dom.borrow(), runner.focus().unwrap()),
        Some("Question")
    );

    type_into(&mut runner, &dom, "Context label", "A changing structure");
    type_into(
        &mut runner,
        &dom,
        "Question",
        "What deserves attention now?",
    );
    type_into(&mut runner, &dom, "Tags", "change, reflection");
    type_into(&mut runner, &dom, "Additional facts", "season: late summer");

    let field = find_attr(&dom.borrow(), runner.root(), "aria-label", "Stored field").unwrap();
    let field_box = dom.borrow().dom_children(field).next().unwrap();
    runner.dispatch_click(field_box, PointerClick::at((1.0, 1.0)));
    let option = find_attr(&dom.borrow(), runner.root(), "role", "option").unwrap();
    runner.dispatch_click(option, PointerClick::at((1.0, 1.0)));

    let cast = find_role_with_text(&dom.borrow(), runner.root(), "radio", "Cast").unwrap();
    runner.dispatch_click(cast, PointerClick::at((1.0, 1.0)));
    let three_cards =
        find_role_with_text(&dom.borrow(), runner.root(), "radio", "Three cards").unwrap();
    runner.dispatch_click(three_cards, PointerClick::at((1.0, 1.0)));

    let actions = click(&mut runner, &dom, "data-key", "read");
    let action = one(actions);
    let (context, field_digest, mode, layout) = match action {
        ConsultationAction::Read {
            context,
            field_digest,
            mode,
            layout,
            ..
        } => (context, field_digest, mode, layout),
        other => panic!("expected read action, found {other:?}"),
    };
    let context_digest = match context {
        ConsultationContext::New(draft) => {
            pollster::block_on(consultation.save_context_at(draft, 2)).unwrap()
        }
        ConsultationContext::Existing(_) => panic!("the first reading must author its context"),
    };
    let mut entropy = FixedEntropy::new(7_u64..64);
    assert_eq!(layout, ConsultationLayout::ThreeCard);
    let detail = match layout {
        ConsultationLayout::Single => pollster::block_on(consultation.read_at_with_entropy(
            &context_digest,
            &field_digest,
            mode,
            1_000,
            3,
            &mut entropy,
        ))
        .unwrap(),
        ConsultationLayout::ThreeCard => {
            pollster::block_on(consultation.read_three_card_at_with_entropy(
                &context_digest,
                &field_digest,
                1_000,
                3,
                &mut entropy,
            ))
            .unwrap()
        }
        ConsultationLayout::Authored(_) => {
            panic!("the fixed three-card DOM receipt must not select an authored layout")
        }
    };
    assert_eq!(detail.readings.len(), 3);
    let expected_title = detail.readings[0].title.clone();
    let expected_prompt = detail.readings[0].interpretation.clone();
    let expected_algorithm = detail.readings[0].receipt.algorithm.clone();
    let session_id = detail.session.id.clone();
    let catalog = consultation.catalog().unwrap();
    runner.update(move |ui| ui.present_reading(catalog, detail));

    for position in ["foundation", "tension", "next_step"] {
        assert!(
            find_attr(
                &dom.borrow(),
                runner.root(),
                "data-key",
                &format!("reading-position:{position}"),
            )
            .is_some()
        );
    }

    assert_eq!(
        text_content(
            &dom.borrow(),
            find_attr(&dom.borrow(), runner.root(), "data-key", "result-title").unwrap(),
        ),
        expected_title
    );
    assert_eq!(
        text_content(
            &dom.borrow(),
            find_attr(&dom.borrow(), runner.root(), "data-key", "result-prompt").unwrap(),
        ),
        expected_prompt
    );
    let trigger = find_attr(
        &dom.borrow(),
        runner.root(),
        "id",
        "cleromancy-workings-trigger",
    )
    .unwrap();
    assert_eq!(attr(&dom.borrow(), trigger, "aria-expanded"), Some("false"));
    runner.dispatch_click(trigger, PointerClick::at((1.0, 1.0)));
    let trigger = find_attr(
        &dom.borrow(),
        runner.root(),
        "id",
        "cleromancy-workings-trigger",
    )
    .unwrap();
    assert_eq!(attr(&dom.borrow(), trigger, "aria-expanded"), Some("true"));
    let panel = find_attr(
        &dom.borrow(),
        runner.root(),
        "id",
        "cleromancy-workings-panel",
    )
    .unwrap();
    assert_eq!(attr(&dom.borrow(), panel, "hidden"), None);
    assert!(find_text(&dom.borrow(), panel, "Algorithm").is_some());
    assert!(find_text(&dom.borrow(), panel, &expected_algorithm).is_some());
    assert!(find_text(&dom.borrow(), panel, "Context digest").is_some());
    assert!(find_text(&dom.borrow(), panel, "Bounded sample").is_some());

    let blank = click(&mut runner, &dom, "data-key", "save-reflection");
    assert!(blank.is_empty());
    assert!(
        runner
            .state()
            .error()
            .unwrap()
            .contains("Enter a reflection")
    );
    type_into(
        &mut runner,
        &dom,
        "Reflection",
        "The structure is useful when it remains revisable.",
    );
    let reflection_action = one(click(&mut runner, &dom, "data-key", "save-reflection"));
    let (action_session, body) = match reflection_action {
        ConsultationAction::SaveReflection { session_id, body } => (session_id, body),
        other => panic!("expected reflection action, found {other:?}"),
    };
    assert_eq!(action_session, session_id);
    let reflected = pollster::block_on(consultation.reflect_at_with_entropy(
        &session_id,
        body,
        2_000,
        4,
        &mut entropy,
    ))
    .unwrap();
    let reflection = reflected.reflections[0].clone();
    let catalog = consultation.catalog().unwrap();
    runner.update(move |ui| ui.present_reflection(catalog, reflected));

    let reflection_node = find_attr(
        &dom.borrow(),
        runner.root(),
        "data-key",
        &format!("reflection:{}", reflection.id),
    )
    .unwrap();
    assert_eq!(
        text_content(&dom.borrow(), reflection_node),
        reflection.body
    );
    type_into(
        &mut runner,
        &dom,
        "Reflection",
        "A later follow-up remains separate.",
    );
    let second_reflection = one(click(&mut runner, &dom, "data-key", "save-reflection"));
    let second_body = match second_reflection {
        ConsultationAction::SaveReflection { body, .. } => body,
        other => panic!("expected second reflection action, found {other:?}"),
    };
    let reflected_twice = pollster::block_on(consultation.reflect_at_with_entropy(
        &session_id,
        second_body,
        2_100,
        4,
        &mut entropy,
    ))
    .unwrap();
    assert_eq!(reflected_twice.reflections.len(), 2);
    let second_reflection_id = reflected_twice.reflections[0].id.clone();
    let catalog = consultation.catalog().unwrap();
    runner.update(move |ui| ui.present_reflection(catalog, reflected_twice));
    assert!(
        find_attr(
            &dom.borrow(),
            runner.root(),
            "data-key",
            &format!("reflection:{second_reflection_id}"),
        )
        .is_some()
    );
    assert!(
        find_attr(
            &dom.borrow(),
            runner.root(),
            "data-key",
            &format!("reflection:{}", reflection.id),
        )
        .is_some()
    );

    let mut comparison_entropy = FixedEntropy::new(0x70_u64..0xa0);
    let comparison_session = pollster::block_on(consultation.read_at_with_entropy(
        &context_digest,
        &field_digest,
        mode,
        2_200,
        5,
        &mut comparison_entropy,
    ))
    .unwrap();
    let comparison_id = comparison_session.session.id.clone();
    let current_detail = consultation.detail(&session_id).unwrap();
    let catalog = consultation.catalog().unwrap();
    runner.update(move |ui| ui.present_session(catalog, current_detail));
    let compare_control =
        find_attr(&dom.borrow(), runner.root(), "aria-label", "Compare with").unwrap();
    let compare_box = dom.borrow().dom_children(compare_control).next().unwrap();
    runner.dispatch_click(compare_box, PointerClick::at((1.0, 1.0)));
    let comparison_option = find_role_with_text(
        &dom.borrow(),
        runner.root(),
        "option",
        &format!("Session {}", &comparison_id[..12]),
    )
    .unwrap();
    runner.dispatch_click(comparison_option, PointerClick::at((1.0, 1.0)));
    let comparison_action = one(click(&mut runner, &dom, "data-key", "compare-receipts"));
    let (left_session_id, right_session_id) = match comparison_action {
        ConsultationAction::CompareSessions {
            left_session_id,
            right_session_id,
        } => (left_session_id, right_session_id),
        other => panic!("expected comparison action, found {other:?}"),
    };
    assert_eq!(left_session_id, session_id);
    assert_eq!(right_session_id, comparison_id);
    let comparison = consultation
        .compare_receipts(&left_session_id, &right_session_id)
        .unwrap();
    let detail = consultation.detail(&left_session_id).unwrap();
    let catalog = consultation.catalog().unwrap();
    runner.update(move |ui| ui.present_comparison(catalog, detail, comparison));
    assert!(
        find_attr(
            &dom.borrow(),
            runner.root(),
            "data-key",
            "receipt-comparison-summary",
        )
        .is_some()
    );
    let session_node = find_attr(
        &dom.borrow(),
        runner.root(),
        "data-key",
        &format!("session:{session_id}"),
    )
    .unwrap();
    let select_action = one(runner.dispatch_click(session_node, PointerClick::at((1.0, 1.0))));
    let selected_id = match select_action {
        ConsultationAction::SelectSession { session_id } => session_id,
        other => panic!("expected session action, found {other:?}"),
    };
    let selected = consultation.detail(&selected_id).unwrap();
    let catalog = consultation.catalog().unwrap();
    runner.update(move |ui| ui.present_session(catalog, selected));
    assert_eq!(runner.state().detail().unwrap().session.id, session_id);

    for (label, id) in [
        ("Context label", "cleromancy-context-label"),
        ("Question", "cleromancy-question"),
        ("Tags", "cleromancy-tags"),
        ("Additional facts", "cleromancy-additional-facts"),
        ("Reflection", "cleromancy-reflection"),
    ] {
        let control = find_attr(&dom.borrow(), runner.root(), "id", id).unwrap();
        assert_eq!(attr(&dom.borrow(), control, "aria-label"), Some(label));
        assert!(find_text(&dom.borrow(), control, label).is_some());
    }
}

#[test]
fn retained_consultation_dispatches_authored_layout_and_chart_input_actions() {
    let mut host = CleromancyHost::empty(MemoryBackend::new());
    let field =
        cleromancy::TarotPack::rws_major_arcana().field(cleromancy::TarotQualification::Contextual);
    host.insert_field(&field).unwrap();
    let template = SpreadTemplate::new(
        "Four directions",
        [
            SpreadPosition::new("north", "North"),
            SpreadPosition::new("east", "East"),
            SpreadPosition::new("south", "South"),
            SpreadPosition::new("west", "West"),
        ],
        [SpreadRelation::new(
            "east",
            SpreadRelationKind::Questions,
            "north",
            "tests the north",
        )],
    )
    .unwrap();
    host.insert_spread_template(&template).unwrap();
    let chart = AstrologyChart::new(
        "source-import/v1",
        "example calculator",
        "example ephemeris",
        AstrologyMoment::global("2026-08-08T12:00:00Z"),
        [AstrologyPosition::new("Sun", 135_000, 0)],
    )
    .unwrap();
    host.insert_astrology_chart(&chart, 1_000).unwrap();
    let facts_digest = chart.facts(1_000).unwrap().digest();
    let catalog = Consultation::new(host).catalog().unwrap();

    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::new(
        dom.clone(),
        consultation_view as fn(&ConsultationUi) -> ConsultationView,
        ConsultationUi::new(catalog),
    );
    for (label, value) in [
        ("Layout label", "Compass"),
        ("Layout positions", "here | Here\nthere | There"),
        (
            "Layout relationships",
            "there | supports | here | supports here",
        ),
    ] {
        type_into(&mut runner, &dom, label, value);
    }
    let save_layout = one(click(&mut runner, &dom, "data-key", "save-layout"));
    match save_layout {
        ConsultationAction::SaveSpreadTemplate { draft } => {
            assert_eq!(draft.label, "Compass");
            assert_eq!(draft.positions, "here | Here\nthere | There");
            assert_eq!(draft.relations, "there | supports | here | supports here");
        }
        other => panic!("expected authored layout action, found {other:?}"),
    }
    for (label, value) in [
        ("Calculation algorithm", "source-import/v1"),
        ("Calculation engine", "example calculator"),
        ("Ephemeris source", "example ephemeris"),
        ("UTC instant", "2026-08-08T12:00:00Z"),
        ("Chart positions", "Sun | 135000 | 0 | false"),
    ] {
        type_into(&mut runner, &dom, label, value);
    }
    let save_chart = one(click(&mut runner, &dom, "data-key", "save-chart-facts"));
    match save_chart {
        ConsultationAction::SaveAstrologyChart { draft } => {
            assert_eq!(draft.algorithm, "source-import/v1");
            assert_eq!(draft.positions, "Sun | 135000 | 0 | false");
            assert_eq!(draft.orb_millidegrees, "1000");
        }
        other => panic!("expected chart input action, found {other:?}"),
    }

    type_into(&mut runner, &dom, "Context label", "A four-part concern");
    type_into(&mut runner, &dom, "Question", "Where is this going?");
    let field = find_attr(&dom.borrow(), runner.root(), "aria-label", "Stored field").unwrap();
    let field_box = dom.borrow().dom_children(field).next().unwrap();
    runner.dispatch_click(field_box, PointerClick::at((1.0, 1.0)));
    let field_option = find_role_with_text(
        &dom.borrow(),
        runner.root(),
        "option",
        "Rider-Waite-Smith Major Arcana",
    )
    .unwrap();
    runner.dispatch_click(field_option, PointerClick::at((1.0, 1.0)));
    let cast = find_role_with_text(&dom.borrow(), runner.root(), "radio", "Cast").unwrap();
    runner.dispatch_click(cast, PointerClick::at((1.0, 1.0)));
    let authored =
        find_role_with_text(&dom.borrow(), runner.root(), "radio", "Authored layout").unwrap();
    runner.dispatch_click(authored, PointerClick::at((1.0, 1.0)));
    let layout = find_attr(
        &dom.borrow(),
        runner.root(),
        "aria-label",
        "Authored layout",
    )
    .unwrap();
    let layout_box = dom.borrow().dom_children(layout).next().unwrap();
    runner.dispatch_click(layout_box, PointerClick::at((1.0, 1.0)));
    let layout_option =
        find_role_with_text(&dom.borrow(), runner.root(), "option", "Four directions").unwrap();
    runner.dispatch_click(layout_option, PointerClick::at((1.0, 1.0)));
    let facts = find_attr(
        &dom.borrow(),
        runner.root(),
        "aria-label",
        "Astrology facts",
    )
    .unwrap();
    let facts_box = dom.borrow().dom_children(facts).next().unwrap();
    runner.dispatch_click(facts_box, PointerClick::at((1.0, 1.0)));
    let facts_option =
        find_role_with_text(&dom.borrow(), runner.root(), "option", "Chart").unwrap();
    runner.dispatch_click(facts_option, PointerClick::at((1.0, 1.0)));
    let read = one(click(&mut runner, &dom, "data-key", "read"));
    match read {
        ConsultationAction::Read {
            mode,
            layout,
            astrology_facts_digest,
            ..
        } => {
            assert_eq!(mode, cleromancy::SelectionMode::Cast);
            assert_eq!(layout, ConsultationLayout::Authored(template.id));
            assert_eq!(
                astrology_facts_digest.as_deref(),
                Some(facts_digest.as_str())
            );
        }
        other => panic!("expected authored read action, found {other:?}"),
    }
}

fn click(runner: &mut Runner, dom: &DomHandle, name: &str, value: &str) -> Vec<ConsultationAction> {
    let node = find_attr(&dom.borrow(), runner.root(), name, value)
        .unwrap_or_else(|| panic!("missing [{name}={value}]"));
    runner.dispatch_click(node, PointerClick::at((1.0, 1.0)))
}

fn type_into(runner: &mut Runner, dom: &DomHandle, label: &str, value: &str) {
    let group = find_attr(&dom.borrow(), runner.root(), "data-control", label)
        .unwrap_or_else(|| panic!("missing control {label}"));
    let tag = if matches!(
        label,
        "Question"
            | "Reflection"
            | "Additional facts"
            | "Layout positions"
            | "Layout relationships"
            | "Chart positions"
    ) {
        "textarea"
    } else {
        "input"
    };
    let node = find_element(&dom.borrow(), group, tag)
        .unwrap_or_else(|| panic!("missing {tag} for {label}"));
    runner.set_focus(Some(node));
    assert!(
        runner
            .dispatch_key(KeyEvent::new(Key::Character(value.to_string())))
            .is_empty()
    );
}

fn one(actions: Vec<ConsultationAction>) -> ConsultationAction {
    let mut actions = actions.into_iter();
    let action = actions.next().expect("one action");
    assert!(actions.next().is_none(), "only one action may bubble");
    action
}

fn tab(shift: bool) -> KeyEvent {
    KeyEvent::with_mods(
        Key::Named(NamedKey::Tab),
        Modifiers {
            shift,
            ..Default::default()
        },
    )
}

fn attr<'a>(dom: &'a ScriptedDom, node: NodeId, name: &str) -> Option<&'a str> {
    dom.attribute(node, &Namespace::from(""), &LocalName::from(name))
}

fn accessible_label(dom: &ScriptedDom, mut node: NodeId) -> Option<&str> {
    loop {
        if let Some(label) = attr(dom, node, "aria-label") {
            return Some(label);
        }
        node = dom.parent(node)?;
    }
}

fn find_element(dom: &ScriptedDom, node: NodeId, name: &str) -> Option<NodeId> {
    if dom.kind(node) == NodeKind::Element
        && dom
            .element_name(node)
            .is_some_and(|element| element.local.as_ref() == name)
    {
        return Some(node);
    }
    dom.dom_children(node)
        .find_map(|child| find_element(dom, child, name))
}

fn find_attr(dom: &ScriptedDom, node: NodeId, name: &str, value: &str) -> Option<NodeId> {
    if attr(dom, node, name) == Some(value) {
        return Some(node);
    }
    dom.dom_children(node)
        .find_map(|child| find_attr(dom, child, name, value))
}

fn find_all_attr(dom: &ScriptedDom, node: NodeId, name: &str, value: &str) -> Vec<NodeId> {
    let mut found = Vec::new();
    if attr(dom, node, name) == Some(value) {
        found.push(node);
    }
    for child in dom.dom_children(node) {
        found.extend(find_all_attr(dom, child, name, value));
    }
    found
}

fn find_text(dom: &ScriptedDom, node: NodeId, text: &str) -> Option<NodeId> {
    if dom.kind(node) == NodeKind::Text && dom.text(node) == Some(text) {
        return Some(node);
    }
    dom.dom_children(node)
        .find_map(|child| find_text(dom, child, text))
}

fn find_role_with_text(dom: &ScriptedDom, node: NodeId, role: &str, text: &str) -> Option<NodeId> {
    if attr(dom, node, "role") == Some(role) && text_content(dom, node).contains(text) {
        return Some(node);
    }
    dom.dom_children(node)
        .find_map(|child| find_role_with_text(dom, child, role, text))
}

fn text_content(dom: &ScriptedDom, node: NodeId) -> String {
    let mut out = dom.text(node).unwrap_or_default().to_string();
    for child in dom.dom_children(node) {
        out.push_str(&text_content(dom, child));
    }
    out
}

struct FixedEntropy {
    words: VecDeque<u64>,
}

impl FixedEntropy {
    fn new(words: impl IntoIterator<Item = u64>) -> Self {
        Self {
            words: words.into_iter().collect(),
        }
    }
}

impl EntropySource for FixedEntropy {
    fn next_u64(&mut self) -> Result<u64, ReadingError> {
        self.words
            .pop_front()
            .ok_or_else(|| ReadingError::Entropy("fixed source exhausted".to_string()))
    }
}
