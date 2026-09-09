// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Shared semantic harness helpers for Cleromancy's retained DOM receipts.

#![allow(dead_code)]

use cambium_genet_winit_host::{Harness, HostHooks, Init, KeyPress, NamedKey, inert_hooks};
use cleromancy::{ConsultationAction, ConsultationUi, ConsultationView, consultation_view};
use genet_probe::Selector;
use genet_scripted_dom::{NodeId, ScriptedDom};
use layout_dom_api::{LayoutDom, LocalName, Namespace, NodeKind};

pub type Logic = fn(&ConsultationUi) -> ConsultationView;
pub type App = Harness<ConsultationUi, Logic, ConsultationView>;

/// Exercise the same stylesheet as the native window.
pub const SHEET: &str = include_str!("../../src/ui/consultation.css");

pub fn harness(catalog: cleromancy::ConsultationCatalog) -> App {
    let hooks: HostHooks<ConsultationUi, Logic, ConsultationView> = HostHooks {
        focused_text: Box::new(cleromancy::ui::native::consultation_focused_text),
        ..inert_hooks()
    };
    let mut harness = Harness::with_hooks(
        Init {
            state: ConsultationUi::new(catalog),
            logic: consultation_view as Logic,
            sheet: SHEET.to_string(),
        },
        hooks,
    );
    // The full consultation form is intentionally tall. Keeping it in one
    // laid-out frame makes every semantic selector hit a real painted box.
    harness.layout_at(1600.0, 4000.0);
    harness
}

pub fn click(harness: &mut App, selector: Selector) -> Option<ConsultationAction> {
    assert!(
        harness.click_on(&selector),
        "semantic selector missed: {selector:?}"
    );
    take_action(harness)
}

pub fn click_key(harness: &mut App, key: &str) -> Option<ConsultationAction> {
    click(harness, Selector::role("button").with_attr("data-key", key))
}

pub fn select(harness: &mut App, label: &str, option: &str) {
    assert!(
        harness.click_on(&Selector::role("combobox").containing(label)),
        "missing combobox {label}"
    );
    assert!(
        harness.click_on(&Selector::role("option").containing(option)),
        "missing option {option} for {label}"
    );
    harness.with_dom(|dom| {
        let selected = find_attr(dom, dom.document(), "aria-label", label).unwrap();
        assert_eq!(
            attr(dom, selected, "aria-expanded"),
            Some("false"),
            "{label} did not close after selecting {option}"
        );
        assert!(
            text_content(dom, selected).contains(option),
            "{label} did not select {option}: {}",
            text_content(dom, selected)
        );
    });
}

/// Activate a surface tab by its visible label, and prove it took.
///
/// Surface choice is view-local, so the helper also proves that it did not
/// emit a `ConsultationAction`.
pub fn switch_surface(harness: &mut App, label: &str) {
    let id = format!("cleromancy-surfaces-item-{}", label.to_ascii_lowercase());
    assert!(
        harness.click_on(&Selector::role("tab").with_attr("id", &id)),
        "missing surface tab {label}"
    );
    assert_eq!(
        attr_at_id(harness, &id, "aria-selected").as_deref(),
        Some("true"),
        "surface tab {label} did not become selected"
    );
    assert!(
        take_action(harness).is_none(),
        "switching to {label} emitted a ConsultationAction"
    );
}

/// Move between surface tabs by the tab list's roving-focus keyboard contract.
///
/// Cambium skips disabled items and, with automatic activation, selects the
/// focused enabled tab. As with pointer activation, this is view-local.
pub fn next_surface(harness: &mut App) {
    harness.press_key(&KeyPress::named(NamedKey::ArrowRight));
    assert!(
        take_action(harness).is_none(),
        "keyboard surface traversal emitted a ConsultationAction"
    );
}

pub fn choose(harness: &mut App, label: &str) {
    assert!(
        harness.click_on(&Selector::role("radio").containing(label)),
        "missing radio {label}"
    );
}

pub fn type_into(harness: &mut App, label: &str, value: &str) {
    let node = harness.with_dom(|dom| {
        let group = find_attr(dom, dom.document(), "data-control", label)
            .unwrap_or_else(|| panic!("missing control {label}"));
        find_element(dom, group, "input")
            .or_else(|| find_element(dom, group, "textarea"))
            .unwrap_or_else(|| panic!("missing text field for {label}"))
    });
    let (x, y, width, height) = harness
        .painted_rect(node)
        .unwrap_or_else(|| panic!("text field {label} has no painted box"));
    harness.click_at(x + width / 2.0, y + height / 2.0);
    assert!(
        cleromancy::ui::native::consultation_focused_text(harness.runner()).is_some(),
        "text field {label} did not acquire the production text slot"
    );
    for (index, line) in value.split('\n').enumerate() {
        if index > 0 {
            harness.press_key(&KeyPress::named(NamedKey::Enter));
        }
        if !line.is_empty() {
            harness.key_injected(line);
        }
    }
    let actual = harness.with_dom(|dom| {
        let group = find_attr(dom, dom.document(), "data-control", label)?;
        let field =
            find_element(dom, group, "input").or_else(|| find_element(dom, group, "textarea"))?;
        Some(text_content(dom, field))
    });
    assert_eq!(
        actual.as_deref(),
        Some(value),
        "text field {label} did not retain injected text"
    );
}

pub fn take_action(harness: &mut App) -> Option<ConsultationAction> {
    let mut action = None;
    harness.update(|ui| action = ui.take_pending_action());
    action
}

pub fn one(action: Option<ConsultationAction>) -> ConsultationAction {
    action.expect("one ConsultationAction")
}

pub fn has_attr(harness: &App, name: &str, value: &str) -> bool {
    harness.with_dom(|dom| find_attr(dom, dom.document(), name, value).is_some())
}

pub fn has_key(harness: &App, key: &str) -> bool {
    has_attr(harness, "data-key", key)
}

pub fn count_attr(harness: &App, name: &str, value: &str) -> usize {
    harness.with_dom(|dom| count_matching_attr(dom, dom.document(), name, value))
}

pub fn attr_at_key(harness: &App, key: &str, name: &str) -> Option<String> {
    harness.with_dom(|dom| {
        find_attr(dom, dom.document(), "data-key", key)
            .and_then(|node| attr(dom, node, name).map(str::to_string))
    })
}

pub fn has_text_at_key(harness: &App, key: &str, text: &str) -> bool {
    harness.with_dom(|dom| {
        find_attr(dom, dom.document(), "data-key", key)
            .is_some_and(|node| find_text(dom, node, text).is_some())
    })
}

pub fn text_at_key(harness: &App, key: &str) -> String {
    harness.with_dom(|dom| {
        let node = find_attr(dom, dom.document(), "data-key", key)
            .unwrap_or_else(|| panic!("missing data-key {key}"));
        text_content(dom, node)
    })
}

pub fn attr_at_id(harness: &App, id: &str, name: &str) -> Option<String> {
    harness.with_dom(|dom| {
        find_attr(dom, dom.document(), "id", id)
            .and_then(|node| attr(dom, node, name).map(str::to_string))
    })
}

pub fn rect_at_key(harness: &App, key: &str) -> (f32, f32, f32, f32) {
    let node = harness
        .with_dom(|dom| find_attr(dom, dom.document(), "data-key", key))
        .unwrap_or_else(|| panic!("missing {key}"));
    harness
        .painted_rect(node)
        .unwrap_or_else(|| panic!("unpainted {key}"))
}

pub fn attr_at_role(harness: &App, role: &str, name: &str) -> Option<String> {
    harness.with_dom(|dom| {
        find_attr(dom, dom.document(), "role", role)
            .and_then(|node| attr(dom, node, name).map(str::to_string))
    })
}

pub fn accessible_label(harness: &App, node: NodeId) -> Option<String> {
    harness.with_dom(|dom| {
        let mut node = node;
        loop {
            if let Some(label) = attr(dom, node, "aria-label") {
                return Some(label.to_string());
            }
            node = dom.parent(node)?;
        }
    })
}

pub fn has_text(harness: &App, text: &str) -> bool {
    harness.with_dom(|dom| find_text(dom, dom.document(), text).is_some())
}

pub fn has_text_in_id(harness: &App, id: &str, text: &str) -> bool {
    harness.with_dom(|dom| {
        find_attr(dom, dom.document(), "id", id)
            .is_some_and(|node| find_text(dom, node, text).is_some())
    })
}

fn attr<'a>(dom: &'a ScriptedDom, node: NodeId, name: &str) -> Option<&'a str> {
    dom.attribute(node, &Namespace::from(""), &LocalName::from(name))
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

fn count_matching_attr(dom: &ScriptedDom, node: NodeId, name: &str, value: &str) -> usize {
    usize::from(attr(dom, node, name) == Some(value))
        + dom
            .dom_children(node)
            .map(|child| count_matching_attr(dom, child, name, value))
            .sum::<usize>()
}

fn find_text(dom: &ScriptedDom, node: NodeId, text: &str) -> Option<NodeId> {
    if dom.kind(node) == NodeKind::Text && dom.text(node) == Some(text) {
        return Some(node);
    }
    dom.dom_children(node)
        .find_map(|child| find_text(dom, child, text))
}

fn text_content(dom: &ScriptedDom, node: NodeId) -> String {
    let mut out = dom.text(node).unwrap_or_default().to_string();
    for child in dom.dom_children(node) {
        out.push_str(&text_content(dom, child));
    }
    out
}
