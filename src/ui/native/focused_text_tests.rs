// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;
use cambium::{TextFieldMode, TextInput, caret_field_children, el, on_key};
use cambium_genet_winit_host::{Harness, inert_hooks};
use taproot::Selector;

fn marked_question_view(ui: &ConsultationUi) -> ConsultationView {
    let field = el::<_, ConsultationUi, ()>(
        "div",
        caret_field_children::<ConsultationUi, ()>(&ui.question, &[]),
    )
    .attr("role", "textbox")
    .attr("aria-label", "Question")
    .attr("aria-multiline", "true")
    .attr("data-cambium-text-value", ui.question.text())
    .attr("style", "display:block;width:240px;height:40px");
    let field = on_key(field, |ui: &mut ConsultationUi, event| {
        ui.question.apply_key(&event, TextFieldMode::Multiline);
    });
    Box::new(
        el::<_, ConsultationUi, ()>("label", (el("span", "Question"), field))
            .attr("id", "cleromancy-question"),
    )
}

#[test]
fn marked_div_field_routes_text_into_its_question_slot() {
    let hooks: HostHooks<ConsultationUi, Logic, ConsultationView> = HostHooks {
        focused_text: Box::new(consultation_focused_text),
        ..inert_hooks()
    };
    let mut state = ConsultationUi::new(empty_catalog());
    state.question = TextInput::default();
    let mut harness = Harness::with_hooks(
        Init {
            state,
            logic: marked_question_view as Logic,
            sheet: String::new(),
            fonts: Vec::new(),
            images: Vec::new(),
        },
        hooks,
    );
    harness.layout_at(320.0, 120.0);
    assert!(
        harness.click_on(&Selector::role("textbox").with_attr("aria-label", "Question")),
        "the marked textbox has a painted click target",
    );

    let focused = consultation_focused_text(harness.runner())
        .expect("the marked DIV resolves to the Question slot");
    assert_eq!(
        focused.node,
        harness.focus().expect("the textbox receives focus")
    );
    assert_eq!((focused.get)(harness.state()).text(), "");
    assert_eq!((focused.get)(harness.state()).caret_position().byte, 0);
    harness.key_injected("x");

    let focused = consultation_focused_text(harness.runner())
        .expect("the marked DIV remains routed after input");
    assert_eq!((focused.get)(harness.state()).text(), "x");
    assert_eq!((focused.get)(harness.state()).caret_position().byte, 1);
}
