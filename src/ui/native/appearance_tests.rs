// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Appearance ownership through the actual native shell and shared workshop.
use super::{
    ApplicationLogic, Child, appearance::*, appearance_host, appearance_view, empty_catalog,
};
use cambium_genet_winit_host::{CloseRequest, Harness, Init, inert_hooks};
use layout_dom_api::LayoutDom;
use tabard::{
    Theme,
    library::ThemeLibraryStore,
    theme::registry::{Mode, ThemeRegistry},
};
use taproot::Selector;

type Host = Harness<appearance_view::ApplicationUi, ApplicationLogic, Child>;

fn theme(path: &std::path::Path, authored: bool) -> Theme {
    let seeds = ThemeRegistry::default()
        .theme_def(tabard::theme::registry::THEME_ID_DEFAULT)
        .unwrap()
        .seeds;
    let mut theme = Theme::new("user:cleromancy-test", "Consultation theme", seeds);
    if authored {
        theme.mode_sheets.insert(
            "dark".into(),
            vec![
                ":root { --tabard-color-bg:#123456; --tabard-color-text:#f1f2f3; }".into(),
                ".cleromancy-consultation { border-top:3px solid #abcdef; }".into(),
            ],
        );
        theme.mode_sheets.insert(
            "custom:garden".into(),
            vec![":root { --tabard-color-bg:#203020; }".into()],
        );
    }
    ThemeLibraryStore::load(path)
        .unwrap()
        .save(&[theme.clone()])
        .unwrap();
    theme
}

fn state(dir: &tempfile::TempDir) -> AppearanceState {
    AppearanceState::load(
        dir.path().join("appearance.json"),
        dir.path().join("themes.json"),
        dir.path().into(),
    )
}

fn mounted(dir: &tempfile::TempDir) -> Host {
    let mut appearance = state(dir);
    let sheet = appearance.take_stylesheet_change().unwrap();
    let state = appearance_view::ApplicationUi {
        consultation: crate::ConsultationUi::new(empty_catalog()),
        mere: Default::default(),
        appearance,
    };
    let mut hooks = inert_hooks();
    hooks.after_dispatch =
        Box::new(|ctx| appearance_host::after_dispatch_with_exporter(ctx, |_| None));
    hooks.focused_text = Box::new(appearance_host::focused_text);
    hooks.close_request = Box::new(appearance_host::close_request);
    let mut host = Host::with_hooks(
        Init {
            state,
            logic: Box::new(|state: &appearance_view::ApplicationUi| {
                appearance_view::root(state, Box::new(cambium::el("span", ())))
            }) as ApplicationLogic,
            sheet,
            fonts: super::fonts::bundled_fonts(),
            images: vec![],
        },
        hooks,
    );
    host.layout_at(1160.0, 760.0);
    host
}

fn click(host: &mut Host, selector: Selector) {
    assert!(
        host.click_on(&selector),
        "actual painted control: {selector:?}"
    );
    host.after_dispatch();
    host.relayout();
}

fn action(host: &mut Host, action: &str) {
    click(
        host,
        Selector::role("button").with_attr("data-action", action),
    );
}

fn color(host: &Host, expected: tinct::Srgb) {
    let node = host
        .with_dom(|dom| taproot::matching_classes(dom, &Selector::class("cleromancy-desktop"))[0]);
    let value = host
        .computed_value(node, "background-color")
        .unwrap()
        .to_ascii_lowercase()
        .replace(' ', "");
    assert!(
        [
            tinct::color_to_hex(expected).to_ascii_lowercase(),
            format!("rgb({},{},{})", expected.r, expected.g, expected.b),
            format!("rgba({},{},{},1)", expected.r, expected.g, expected.b),
        ]
        .contains(&value),
        "painted background: {value}"
    );
}

// Every caller observes a harness already laid out by mounted/layout_at/relayout.
fn matching_role(host: &Host, selector: &Selector) -> Vec<genet_scripted_dom::NodeId> {
    let projection = host.a11y_projection();
    host.with_dom(|dom| taproot::matching_with_projection(dom, selector, &projection))
}

fn radio_colors(host: &Host, checked: tinct::Srgb, background: tinct::Srgb, body: tinct::Srgb) {
    let selected = matching_role(
        host,
        &Selector::role("radio").with_attr("aria-checked", "true"),
    );
    assert_eq!(
        selected.len(),
        4,
        "selection mode, reading shape, place consent and weather consent"
    );
    let unselected = matching_role(
        host,
        &Selector::role("radio").with_attr("aria-checked", "false"),
    );
    assert_eq!(unselected.len(), 6);
    for (nodes, property, expected) in [
        (&selected, "color", checked),
        (&selected, "background-color", background),
        (&unselected, "color", body),
    ] {
        for &node in nodes {
            let (_, _, width, height) = host.painted_rect(node).expect("radio paints");
            assert!(width > 0.0 && height > 0.0);
            let value = host
                .computed_value(node, property)
                .unwrap()
                .to_ascii_lowercase()
                .replace(' ', "");
            assert!(
                [
                    tinct::color_to_hex(expected).to_ascii_lowercase(),
                    format!("rgb({},{},{})", expected.r, expected.g, expected.b),
                    format!("rgba({},{},{},1)", expected.r, expected.g, expected.b),
                ]
                .contains(&value),
                "radio {property}: {value}, expected {expected:?}"
            );
        }
    }
}

#[test]
fn selected_radio_uses_canonical_on_primary_without_recoloring_unchecked_options() {
    let dir = tempfile::tempdir().unwrap();
    let saved = theme(&dir.path().join("themes.json"), false);
    let mut host = mounted(&dir);
    action(&mut host, "toggle-appearance");
    click(
        &mut host,
        Selector::role("button").with_attr("data-appearance-theme", &saved.id),
    );
    for mode in MODES {
        click(
            &mut host,
            Selector::role("button").with_attr("data-appearance-mode", &mode.as_key()),
        );
        let palette = saved.palette_for_mode(&mode).unwrap().palette;
        for (width, height) in [(1160.0, 760.0), (420.0, 1000.0)] {
            host.layout_at(width, height);
            radio_colors(&host, palette.on_primary, palette.primary, palette.text);
        }
    }
}

#[test]
fn selected_radio_uses_authored_on_primary_after_actual_selection_changes() {
    for (primary, on_primary, body) in [
        ("#ffc085", "#342214", "#fff2ea"),
        ("#203040", "#f1f2f3", "#a0b0c0"),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("themes.json");
        let mut saved = theme(&path, false);
        saved.mode_sheets.insert(
            "dark".into(),
            vec![format!(
                ":root {{ --tabard-color-primary:{primary}; --tabard-color-on-primary:{on_primary}; --tabard-color-text:{body}; }}"
            )],
        );
        ThemeLibraryStore::load(&path)
            .unwrap()
            .save(&[saved.clone()])
            .unwrap();
        let mut host = mounted(&dir);
        action(&mut host, "toggle-appearance");
        click(
            &mut host,
            Selector::role("button").with_attr("data-appearance-theme", &saved.id),
        );
        click(
            &mut host,
            Selector::role("button").with_attr("data-appearance-mode", "dark"),
        );
        for (width, height) in [(1160.0, 760.0), (420.0, 1000.0)] {
            host.layout_at(width, height);
            radio_colors(
                &host,
                tinct::color_from_hex(on_primary).unwrap(),
                tinct::color_from_hex(primary).unwrap(),
                tinct::color_from_hex(body).unwrap(),
            );
        }
        host.layout_at(1160.0, 760.0);
        let before = host.state().consultation.mode.selected;
        click(
            &mut host,
            Selector::role("radio").with_attr("aria-checked", "false"),
        );
        assert_ne!(host.state().consultation.mode.selected, before);
        radio_colors(
            &host,
            tinct::color_from_hex(on_primary).unwrap(),
            tinct::color_from_hex(primary).unwrap(),
            tinct::color_from_hex(body).unwrap(),
        );
    }
}

#[test]
fn neutral_workshop_headings_remain_readable_inside_dark_and_authored_hosts() {
    let neutral = ThemeRegistry::default()
        .theme_def(tabard::theme::registry::THEME_ID_DEFAULT)
        .unwrap()
        .palette_for_mode(&Mode::Light)
        .unwrap()
        .palette;
    let luminance = |color: tinct::Srgb| {
        let linear = |channel: u8| {
            let value = f64::from(channel) / 255.0;
            if value <= 0.04045 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * linear(color.r) + 0.7152 * linear(color.g) + 0.0722 * linear(color.b)
    };
    let ink = luminance(neutral.text);
    let surface = luminance(neutral.surface);
    assert!((ink.max(surface) + 0.05) / (ink.min(surface) + 0.05) >= 4.5);
    let css_colors = |color: tinct::Srgb| {
        [
            tinct::color_to_hex(color).to_ascii_lowercase(),
            format!("rgb({},{},{})", color.r, color.g, color.b),
            format!("rgba({},{},{},1)", color.r, color.g, color.b),
        ]
    };
    for authored in [false, true] {
        let dir = tempfile::tempdir().unwrap();
        let saved = authored.then(|| theme(&dir.path().join("themes.json"), true));
        let mut host = mounted(&dir);
        action(&mut host, "toggle-appearance");
        if let Some(saved) = saved {
            click(
                &mut host,
                Selector::role("button").with_attr("data-appearance-theme", &saved.id),
            );
            click(
                &mut host,
                Selector::role("button").with_attr("data-appearance-mode", "dark"),
            );
            color(&host, tinct::Srgb::rgb(0x12, 0x34, 0x56));
        } else {
            color(&host, tinct::Srgb::rgb(24, 23, 20));
        }
        action(&mut host, "edit-appearance");
        for (width, height) in [(1160.0, 760.0), (420.0, 1000.0)] {
            host.layout_at(width, height);
            let header = host.with_dom(|dom| {
                let workshop =
                    taproot::matching_classes(dom, &Selector::class("tabard-workshop"))[0];
                dom.dom_children(workshop)
                    .find(|node| dom.has_class(*node, "cambium-title-bar"))
                    .expect("shared workshop title bar")
            });
            let editor = host.with_dom(|dom| {
                taproot::matching_classes(dom, &Selector::class("theme-editor"))[0]
            });
            for (container, label, background) in [
                ("brand-title", "Tabard", header),
                ("workspace-pane-heading", "Your theme", editor),
                ("control-group", "Seed colours", editor),
            ] {
                let heading = host.with_dom(|dom| {
                    taproot::matching_classes(dom, &Selector::class(container))
                        .into_iter()
                        .flat_map(|node| dom.dom_children(node))
                        .find(|node| {
                            dom.element_name(*node)
                                .is_some_and(|name| matches!(&*name.local, "h1" | "h2" | "h3"))
                                && dom
                                    .dom_children(*node)
                                    .filter_map(|child| dom.text(child))
                                    .collect::<String>()
                                    == label
                        })
                        .expect("actual shared workshop heading")
                });
                let (_, _, w, h) = host.painted_rect(heading).expect("heading paints");
                assert!(w > 0.0 && h > 0.0, "{label} at {width}px");
                let foreground = host
                    .computed_value(heading, "color")
                    .unwrap()
                    .to_ascii_lowercase()
                    .replace(' ', "");
                assert!(
                    css_colors(neutral.text).contains(&foreground),
                    "{label} foreground at {width}px, authored={authored}: {foreground}"
                );
                let background = host
                    .computed_value(background, "background-color")
                    .unwrap()
                    .to_ascii_lowercase()
                    .replace(' ', "");
                assert!(
                    css_colors(neutral.surface).contains(&background),
                    "{label} background at {width}px: {background}"
                );
            }
        }
    }
}
#[test]
fn unchanged_appearance_does_not_reset_the_callers_stylesheet_on_dispatch() {
    let dir = tempfile::tempdir().unwrap();
    let mut appearance = state(&dir);
    assert!(appearance.take_stylesheet_change().is_some());
    assert!(appearance.take_stylesheet_change().is_none());
    appearance.open = true;
    assert!(appearance.take_stylesheet_change().is_none());
    appearance.begin_edit().unwrap();
    assert!(
        appearance
            .take_stylesheet_change()
            .unwrap()
            .contains(cambium::TITLE_BAR_CSS)
    );
}

#[test]
fn product_identity_keeps_its_seeds_across_canonical_modes_and_reopen() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = state(&dir);
    for mode in MODES {
        state
            .request_select(PRODUCT_DEFAULT, Some(mode.clone()))
            .unwrap();
        state.commit_requested();
        assert_eq!(state.active_id(), PRODUCT_DEFAULT);
        assert_eq!(
            state.applied().unwrap().theme.seeds.primary,
            tinct::Srgb::rgb(215, 180, 106)
        );
        if mode == Mode::Dark {
            assert!(matches!(
                state.applied().unwrap().presentation,
                tabard::ThemePresentation::AuthoredStylesheet(_)
            ));
        } else {
            assert!(matches!(
                state.applied().unwrap().presentation,
                tabard::ThemePresentation::Derived(_)
            ));
        }
        let reopened = AppearanceState::load(
            dir.path().join("appearance.json"),
            dir.path().join("themes.json"),
            dir.path().into(),
        );
        assert_eq!(reopened.active_id(), PRODUCT_DEFAULT);
        assert_eq!(reopened.applied().unwrap().resolved.theme_mode, Some(mode));
    }
}

#[test]
fn product_dark_copy_supplies_shared_preview_roles_and_exact_product_context_colors() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = state(&dir);
    state.begin_edit().unwrap();
    let tabard::ThemePresentation::AuthoredStylesheet(rules) = state
        .workshop
        .draft_theme()
        .presentation_for_mode(&Mode::Dark)
        .unwrap()
    else {
        panic!("exact product dark sheet");
    };
    let mut specimen = tabard_workshop::StylesheetSpecimen::new(&Mode::Dark, &rules);
    specimen.frame(640, 480);
    let bg = specimen
        .computed_style("application-body", "background-color")
        .unwrap()
        .to_ascii_lowercase()
        .replace(' ', "");
    assert!(
        ["#181714", "rgb(24,23,20)", "rgba(24,23,20,1)"].contains(&bg.as_str()),
        "shared preview background: {bg}"
    );
    assert!(specimen.diagnostics().is_empty());
    assert!(rules.join("\n").contains(
        ".cleromancy-desktop .cleromancy-consultation .reading-card h4 { color: #fff0cf; }"
    ));
}

// Granular preferences use the actual shared editor and native input/store seams.
fn focus_selector(host: &mut Host, selector: Selector) {
    let wanted = matching_role(host, &selector)[0];
    for _ in 0..100 {
        host.tab(true);
        if host.focus() == Some(wanted) {
            return;
        }
    }
    panic!("font control unreachable: {selector:?}");
}

#[test]
fn shared_mere_and_focused_reading_share_the_ordinary_wide_viewport() {
    let dir = tempfile::tempdir().unwrap();
    let mut host = mounted(&dir);
    let mut consultation = crate::Consultation::new(
        crate::CleromancyHost::candidate(mere::kernel::graph::Graph::new()).unwrap(),
    );
    let field = pollster::block_on(consultation.install_builtin_tarot_at(1)).unwrap();
    let context = pollster::block_on(consultation.save_context_at(
        crate::ContextDraft::new("A shared workspace", "What needs attention?", "reading"),
        2,
    ))
    .unwrap();
    let detail =
        pollster::block_on(consultation.read(&context, &field, crate::SelectionMode::Calculated))
            .unwrap();
    let catalog = consultation.catalog().unwrap();
    host.update(|state| {
        state.consultation.present_reading(catalog, detail);
        state.mere.width = 420;
        state.mere.height = 718;
        state.mere.wide = true;
        state.mere.model.title = "Divination".into();
        state.mere.model.can_mint = true;
    });
    host.relayout();
    action(&mut host, "toggle-mere");
    let node = |class: &str| {
        host.with_dom(|dom| taproot::matching_classes(dom, &Selector::class(class))[0])
    };
    let panel = host.painted_rect(node("mere-view")).unwrap();
    let reader = host.painted_rect(node("cleromancy-mere-reading")).unwrap();
    assert!(panel.2 > 0.0 && reader.2 > 0.0);
    assert!(
        panel.0 + panel.2 <= reader.0 + 1.0,
        "panel and reader are side by side"
    );
    assert!((panel.1 - reader.1).abs() < 1.0);
    let focused =
        host.with_dom(|dom| taproot::matching_classes(dom, &Selector::class("reading-card"))[0]);
    let card = host
        .visible_rect(focused)
        .expect("the selected card is visible beside the shared panel");
    assert!(card.2 > 0.0 && card.3 > 0.0);
    click(
        &mut host,
        Selector::role("button").with_attr("data-request", "mint"),
    );
    assert_eq!(
        host.state().mere.pending,
        Some(mere_view::MereViewRequest::Mint)
    );
    assert!(host.state().consultation.pending_action.is_none());
}

#[test]
fn mere_refresh_preserves_pending_until_atomic_command_response_restores_selection() {
    let dir = tempfile::tempdir().unwrap();
    let mut ui = appearance_view::ApplicationUi {
        consultation: crate::ConsultationUi::new(empty_catalog()),
        mere: Default::default(),
        appearance: state(&dir),
    };
    let mut consultation = crate::Consultation::new(
        crate::CleromancyHost::candidate(mere::kernel::graph::Graph::new()).unwrap(),
    );
    let field = pollster::block_on(consultation.install_builtin_tarot_at(1)).unwrap();
    let context = pollster::block_on(consultation.save_context_at(
        crate::ContextDraft::new("Pending restore", "What survives a refresh?", "lifecycle"),
        2,
    ))
    .unwrap();
    let detail =
        pollster::block_on(consultation.read(&context, &field, crate::SelectionMode::Calculated))
            .unwrap();
    let catalog = consultation.catalog().unwrap();
    let session = detail.session.id.clone();
    let position = detail.session.placements[0].position.clone();
    let reading = detail.session.placements[0].reading_id.clone();
    ui.mere.busy = true;

    // A queued/polled publication may arrive after the next command starts.
    super::apply_mere_publication(
        &mut ui,
        Default::default(),
        catalog.clone(),
        Some((detail.clone(), Some(position.clone()))),
        false,
    );
    assert!(
        ui.mere.busy,
        "an unsolicited refresh cannot acknowledge restore"
    );
    assert_eq!(ui.consultation.detail().unwrap().session.id, session);

    // Nor may an empty refresh release the wait before its restored selection.
    ui.consultation.clear_selection();
    super::apply_mere_publication(&mut ui, Default::default(), catalog.clone(), None, false);
    assert!(ui.mere.busy);
    assert!(ui.consultation.detail().is_none());

    let mut model = mere_view::MereViewModel::default();
    model.title = "Restore acknowledged".into();
    super::apply_mere_publication(
        &mut ui,
        model,
        catalog,
        Some((detail, Some(position.clone()))),
        true,
    );
    assert!(!ui.mere.busy, "the command response releases its own wait");
    assert_eq!(ui.mere.model.title, "Restore acknowledged");
    assert_eq!(ui.consultation.detail().unwrap().session.id, session);
    assert_eq!(ui.consultation.reading_focus.get(&session), Some(&position));
    assert_eq!(
        ui.mere.view.focused,
        Some(
            mere::kernel::graph::Graph::node_namespace_id(&format!(
                "cleromancy://reading/{reading}"
            ),)
            .to_string(),
        )
    );
}

#[path = "appearance_tests/lifecycle.rs"]
mod lifecycle;

#[path = "appearance_tests/fonts.rs"]
mod fonts;
