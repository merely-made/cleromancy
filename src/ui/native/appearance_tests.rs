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
            fonts: vec![],
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
    let node =
        host.with_dom(|dom| taproot::matching(dom, &Selector::class("cleromancy-desktop"))[0]);
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

fn radio_colors(host: &Host, checked: tinct::Srgb, background: tinct::Srgb, body: tinct::Srgb) {
    let selected = host.with_dom(|dom| {
        taproot::matching(
            dom,
            &Selector::role("radio").with_attr("aria-checked", "true"),
        )
    });
    assert_eq!(selected.len(), 2, "selection mode and reading shape");
    let unselected = host.with_dom(|dom| {
        taproot::matching(
            dom,
            &Selector::role("radio").with_attr("aria-checked", "false"),
        )
    });
    assert_eq!(unselected.len(), 4);
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
fn modes_authored_css_and_reopen_preserve_consultation_and_action_owner() {
    let dir = tempfile::tempdir().unwrap();
    let theme = theme(&dir.path().join("themes.json"), true);
    let mut host = mounted(&dir);
    let original = host.state().consultation.clone();
    color(&host, tinct::Srgb::rgb(24, 23, 20));
    action(&mut host, "toggle-appearance");
    click(
        &mut host,
        Selector::role("button").with_attr("data-appearance-theme", &theme.id),
    );
    for mode in MODES.into_iter().chain([Mode::Custom("garden".into())]) {
        click(
            &mut host,
            Selector::role("button").with_attr("data-appearance-mode", &mode.as_key()),
        );
        let expected = match mode {
            Mode::Dark => tinct::Srgb::rgb(0x12, 0x34, 0x56),
            Mode::Custom(_) => tinct::Srgb::rgb(0x20, 0x30, 0x20),
            _ => theme.palette_for_mode(&mode).unwrap().palette.bg,
        };
        color(&host, expected);
        assert_eq!(host.state().consultation, original);
        assert!(host.state().consultation.pending_action.is_none());
        let reopened = mounted(&dir);
        color(&reopened, expected);
        assert_eq!(
            reopened
                .state()
                .appearance
                .applied()
                .unwrap()
                .resolved
                .theme_mode,
            Some(mode)
        );
    }
    click(
        &mut host,
        Selector::role("button").with_attr("data-appearance-theme", PRODUCT_DEFAULT),
    );
    color(&host, tinct::Srgb::rgb(24, 23, 20));
    assert_eq!(host.state().consultation, original);
}

#[test]
fn save_and_library_reload_hold_same_identity_until_explicit_apply() {
    let dir = tempfile::tempdir().unwrap();
    let saved = theme(&dir.path().join("themes.json"), false);
    let mut state = state(&dir);
    state.request_select(&saved.id, Some(Mode::Light)).unwrap();
    state.commit_requested();
    let original = state.applied().unwrap().clone();
    let choice_bytes = std::fs::read(dir.path().join("appearance.json")).unwrap();
    state.begin_edit().unwrap();
    *state.workshop.text_field_mut("seed-hex").unwrap() = cambium::TextInput::new("#e42b65");
    state.workshop.apply_hex();
    state.workshop.save();
    assert_ne!(
        state
            .workshop
            .registry()
            .theme_def(&saved.id)
            .unwrap()
            .seeds,
        original.theme.seeds
    );
    assert_eq!(state.applied().unwrap().theme, original.theme);
    state.back();
    state.reload_library().unwrap();
    assert_eq!(state.applied().unwrap().theme, original.theme);
    assert_eq!(
        std::fs::read(dir.path().join("appearance.json")).unwrap(),
        choice_bytes
    );
    state.begin_edit().unwrap();
    assert_eq!(state.workshop.draft_theme().id, saved.id);
    assert!(!state.workshop.has_changes());
    assert_ne!(state.workshop.draft_theme().seeds, original.theme.seeds);
    assert_eq!(state.applied().unwrap().theme, original.theme);
    state.apply_workshop().unwrap();
    assert_eq!(state.applied().unwrap().theme, original.theme);
    state.commit_requested();
    assert_ne!(state.applied().unwrap().theme.seeds, original.theme.seeds);
    let reopened = AppearanceState::load(
        dir.path().join("appearance.json"),
        dir.path().join("themes.json"),
        dir.path().into(),
    );
    assert_eq!(
        reopened.applied().unwrap().theme,
        state.applied().unwrap().theme
    );
}

#[test]
fn corrupt_files_and_failed_choice_write_preserve_bytes_and_applied_presentation() {
    let dir = tempfile::tempdir().unwrap();
    let selection = dir.path().join("appearance.json");
    let library = dir.path().join("themes.json");
    std::fs::write(&selection, b"broken choice").unwrap();
    std::fs::write(&library, b"broken library").unwrap();
    let mut broken = state(&dir);
    assert!(!broken.authoring_available);
    assert!(broken.request_select(PRODUCT_DEFAULT, None).is_err());
    assert!(broken.begin_edit().is_err());
    assert_eq!(std::fs::read(&selection).unwrap(), b"broken choice");
    assert_eq!(std::fs::read(&library).unwrap(), b"broken library");
    assert!(broken.diagnostics().join(" ").contains("preserved"));
    std::fs::remove_file(&selection).unwrap();
    std::fs::remove_file(&library).unwrap();
    let saved = theme(&library, false);
    let mut working = state(&dir);
    working
        .request_select(&saved.id, Some(Mode::Light))
        .unwrap();
    working.commit_requested();
    let applied = working.applied().unwrap().clone();
    std::fs::remove_file(&selection).unwrap();
    std::fs::create_dir(&selection).unwrap();
    working.request_select(&saved.id, Some(Mode::Dark)).unwrap();
    working.commit_requested();
    assert_eq!(working.applied().unwrap().resolved, applied.resolved);
    assert_eq!(
        working.applied().unwrap().presentation,
        applied.presentation
    );
    assert!(working.diagnostics().join(" ").contains("not saved"));
    assert!(selection.is_dir());
    std::fs::write(&library, b"broken after load").unwrap();
    assert!(working.reload_library().is_err());
    assert!(!working.authoring_available);
    assert!(working.begin_edit().is_err());
    assert_eq!(working.applied().unwrap().theme, applied.theme);
    assert_eq!(std::fs::read(&library).unwrap(), b"broken after load");
}

#[test]
fn shared_text_save_apply_and_back_do_not_dispatch_a_consultation_action() {
    let dir = tempfile::tempdir().unwrap();
    let mut host = mounted(&dir);
    let original = host.state().consultation.clone();
    action(&mut host, "toggle-appearance");
    action(&mut host, "edit-appearance");
    assert!(host.state().appearance.editor_open);
    click(
        &mut host,
        Selector::role("textbox").with_attr("data-field", "name"),
    );
    host.key_injected(" studio");
    assert!(
        host.state()
            .appearance
            .workshop
            .draft_theme()
            .name
            .contains("studio")
    );
    action(&mut host, "apply-to-cleromancy");
    assert_eq!(host.state().appearance.active_id(), PRODUCT_DEFAULT);
    action(&mut host, "save");
    assert_eq!(host.state().appearance.active_id(), PRODUCT_DEFAULT);
    action(&mut host, "apply-to-cleromancy");
    assert_ne!(host.state().appearance.active_id(), PRODUCT_DEFAULT);
    action(&mut host, "back-to-consultation");
    assert!(!host.state().appearance.editor_open);
    assert_eq!(host.state().consultation, original);
    assert!(host.state().consultation.pending_action.is_none());
    assert!(dir.path().join("appearance.json").is_file());
}

#[test]
fn cancel_native_close_then_discard_back_preserves_parent() {
    let dir = tempfile::tempdir().unwrap();
    let mut host = mounted(&dir);
    action(&mut host, "toggle-appearance");
    action(&mut host, "edit-appearance");
    host.request_close(CloseRequest::Native);
    host.relayout();
    assert!(!host.close_requested());
    assert!(host.state().appearance.close_app);
    action(&mut host, "cancel-close");
    assert!(!host.state().appearance.close_app);
    action(&mut host, "back-to-consultation");
    action(&mut host, "discard-close");
    assert!(!host.close_requested());
    assert!(!host.state().appearance.editor_open);
}

#[test]
fn native_save_close_saves_definition_without_applying_or_mutating_consultation() {
    let dir = tempfile::tempdir().unwrap();
    let mut host = mounted(&dir);
    let original = host.state().consultation.clone();
    action(&mut host, "toggle-appearance");
    action(&mut host, "edit-appearance");
    host.request_close(CloseRequest::Native);
    host.relayout();
    action(&mut host, "save-close");
    assert!(host.close_requested());
    assert!(dir.path().join("themes.json").is_file());
    assert!(!dir.path().join("appearance.json").exists());
    assert_eq!(host.state().consultation, original);
}

#[test]
fn shared_fields_and_parent_controls_keep_real_geometry_at_420_and_1160() {
    let dir = tempfile::tempdir().unwrap();
    let mut host = mounted(&dir);
    for (width, height) in [(420.0, 1000.0), (1160.0, 760.0)] {
        host.layout_at(width, height);
        let node = host.with_dom(|dom| {
            taproot::matching(
                dom,
                &Selector::role("button").with_attr("data-action", "toggle-appearance"),
            )[0]
        });
        let (x, y, w, h) = host.painted_rect(node).unwrap();
        assert!(x >= 0.0 && y >= 0.0 && w > 0.0 && h >= 24.0 && x + w <= width + 0.1);
    }
    action(&mut host, "toggle-appearance");
    action(&mut host, "edit-appearance");
    for (width, height) in [(420.0, 1000.0), (1160.0, 760.0)] {
        host.layout_at(width, height);
        for field in ["name", "seed-hex"] {
            let node = host.with_dom(|dom| {
                taproot::matching(
                    dom,
                    &Selector::role("textbox").with_attr("data-field", field),
                )[0]
            });
            let (_, _, w, h) = host.painted_rect(node).unwrap();
            assert!(w >= 40.0 && h >= 33.9, "{field} at {width}px: {w}x{h}");
        }
        for control in ["back-to-consultation", "apply-to-cleromancy"] {
            let node = host.with_dom(|dom| {
                taproot::matching(
                    dom,
                    &Selector::role("button").with_attr("data-action", control),
                )[0]
            });
            let (x, y, w, h) = host.visible_rect(node).unwrap();
            assert!(x >= 0.0 && y >= 0.0 && w > 0.0 && h >= 24.0 && x + w <= width + 0.1);
        }
    }
}

#[test]
fn neutral_workshop_headings_remain_readable_inside_dark_and_authored_hosts() {
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
                let workshop = taproot::matching(dom, &Selector::class("tabard-workshop"))[0];
                dom.dom_children(workshop)
                    .find(|node| dom.has_class(*node, "cambium-title-bar"))
                    .expect("shared workshop title bar")
            });
            let editor = host.with_dom(|dom| {
                taproot::matching(dom, &Selector::class("theme-editor"))[0]
            });
            for (container, label, background) in [
                ("brand-title", "Tabard", header),
                ("editor-intro", "Your theme", editor),
                ("control-group", "Seed colours", editor),
            ] {
                let heading = host.with_dom(|dom| {
                    taproot::matching(dom, &Selector::class(container))
                        .into_iter()
                        .flat_map(|node| dom.dom_children(node))
                        .find(|node| {
                            dom.element_name(*node).is_some_and(|name| {
                                matches!(name.local.as_ref(), "h1" | "h2" | "h3")
                            }) && dom
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
                    ["#242c39", "rgb(36,44,57)", "rgba(36,44,57,1)"]
                        .contains(&foreground.as_str()),
                    "{label} foreground at {width}px, authored={authored}: {foreground}"
                );
                let background = host
                    .computed_value(background, "background-color")
                    .unwrap()
                    .to_ascii_lowercase()
                    .replace(' ', "");
                assert!(
                    ["#fff", "#ffffff", "rgb(255,255,255)", "rgba(255,255,255,1)"]
                        .contains(&background.as_str()),
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
fn exports_cannot_replace_future_application_data_or_overridden_preferences() {
    let dir = tempfile::tempdir().unwrap();
    let data = dir.path().join("application");
    let selection = dir.path().join("separate-choice.json");
    let mut state = AppearanceState::load(
        selection.clone(),
        dir.path().join("themes.json"),
        data.clone(),
    );
    for target in [
        selection.clone(),
        data.join("cleromancy.redb"),
        data.join("future/session.json"),
    ] {
        state.workshop.request_export();
        let artifact = state.workshop.take_export().unwrap();
        state.workshop.complete_export(artifact, Some(target));
        assert!(state.workshop.status().contains("protected"));
        assert!(state.workshop.replacement_path().is_none());
    }
    assert!(!selection.exists());
    assert!(!data.exists());
    state.workshop.request_export();
    let artifact = state.workshop.take_export().unwrap();
    state.workshop.complete_export(artifact, None);
    assert!(!selection.exists());
    assert_eq!(state.active_id(), PRODUCT_DEFAULT);
}

#[test]
fn appearance_fixtures_keep_the_shared_scenario_lane_and_durable_product_contract() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("receipts");
    for name in [
        "tabard-authoring.scn",
        "tabard-reopen.scn",
        "tabard-same-id.scn",
        "tabard-radio-foreground.scn",
        "tabard-radio-reopen.scn",
    ] {
        let source = std::fs::read_to_string(root.join(name)).unwrap();
        taproot::Scenario::parse(&source).unwrap();
        assert!(source.contains("assert event durable"));
        assert!(source.contains("assert snap reflections >= 1"));
    }
}

#[test]
fn identical_missing_paths_refuse_preference_and_library_writes() {
    let dir = tempfile::tempdir().unwrap();
    let same = dir.path().join("not-created/appearance.json");
    let mut state = AppearanceState::load(same.clone(), same.clone(), dir.path().join("data"));
    assert!(!state.authoring_available);
    assert!(state.begin_edit().is_err());
    assert!(
        state
            .request_select(PRODUCT_DEFAULT, Some(Mode::Light))
            .is_err()
    );
    assert!(!same.exists());
}

#[cfg(unix)]
#[test]
fn missing_nested_and_existing_aliases_preserve_owned_files() {
    let dir = tempfile::tempdir().unwrap();
    let alias = dir.path().join("alias");
    std::os::unix::fs::symlink(dir.path(), &alias).unwrap();
    let target = dir.path().join("new/nested/appearance.json");
    let mut state = AppearanceState::load(
        target.clone(),
        alias.join("new/nested/appearance.json"),
        dir.path().join("data"),
    );
    assert!(!state.authoring_available);
    assert!(
        state
            .request_select(PRODUCT_DEFAULT, Some(Mode::Dark))
            .is_err()
    );
    assert!(!target.exists());
    let selection = dir.path().join("appearance.json");
    std::fs::write(&selection, b"existing owned bytes").unwrap();
    let mut existing = AppearanceState::load(
        selection.clone(),
        alias.join("appearance.json"),
        dir.path().join("data"),
    );
    assert!(existing.begin_edit().is_err());
    assert!(existing.request_select(PRODUCT_DEFAULT, None).is_err());
    assert_eq!(std::fs::read(selection).unwrap(), b"existing owned bytes");
}

#[test]
fn overrides_and_exports_preserve_the_existing_consultation_store() {
    let dir = tempfile::tempdir().unwrap();
    let data = dir.path().join("data");
    std::fs::create_dir(&data).unwrap();
    let database = data.join("cleromancy.redb");
    std::fs::write(&database, b"consultation bytes").unwrap();
    for (selection, library) in [
        (database.clone(), dir.path().join("themes.json")),
        (dir.path().join("appearance.json"), database.clone()),
    ] {
        let mut blocked = AppearanceState::load(selection, library, data.clone());
        assert!(!blocked.authoring_available);
        assert!(blocked.request_select(PRODUCT_DEFAULT, None).is_err());
    }
    let mut valid = AppearanceState::load(
        dir.path().join("appearance.json"),
        dir.path().join("themes.json"),
        data.clone(),
    );
    valid.workshop.request_export();
    let artifact = valid.workshop.take_export().unwrap();
    valid
        .workshop
        .complete_export(artifact, Some(database.clone()));
    assert!(valid.workshop.status().contains("protected"));
    assert!(valid.workshop.replacement_path().is_none());
    assert_eq!(std::fs::read(database).unwrap(), b"consultation bytes");
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

#[test]
fn mounted_save_back_edit_apply_reuses_saved_identity_without_early_publication() {
    let dir = tempfile::tempdir().unwrap();
    let saved = theme(&dir.path().join("themes.json"), false);
    let mut host = mounted(&dir);
    let consultation = host.state().consultation.clone();
    action(&mut host, "toggle-appearance");
    click(
        &mut host,
        Selector::role("button").with_attr("data-appearance-theme", &saved.id),
    );
    click(
        &mut host,
        Selector::role("button").with_attr("data-appearance-mode", "light"),
    );
    let applied = host.state().appearance.applied().unwrap().clone();
    action(&mut host, "edit-appearance");
    click(
        &mut host,
        Selector::role("textbox").with_attr("data-field", "seed-hex"),
    );
    host.set_modifiers(cambium_genet_winit_host::Modifiers {
        ctrl: true,
        ..Default::default()
    });
    host.key_char("a");
    host.set_modifiers(Default::default());
    host.key_injected("#e42b65");
    action(&mut host, "apply-hex");
    action(&mut host, "save");
    assert_ne!(
        host.state()
            .appearance
            .workshop
            .registry()
            .theme_def(&saved.id)
            .unwrap()
            .seeds,
        applied.theme.seeds
    );
    action(&mut host, "back-to-consultation");
    assert!(!host.state().appearance.editor_open);
    assert_eq!(host.state().appearance.applied().unwrap(), &applied);
    action(&mut host, "edit-appearance");
    assert_eq!(host.state().appearance.workshop.draft_theme().id, saved.id);
    assert!(!host.state().appearance.workshop.has_changes());
    assert_eq!(host.state().appearance.applied().unwrap(), &applied);
    action(&mut host, "apply-to-cleromancy");
    action(&mut host, "back-to-consultation");
    assert!(!host.state().appearance.editor_open);
    assert_eq!(host.state().appearance.active_id(), saved.id);
    assert_ne!(
        host.state().appearance.applied().unwrap().theme.seeds,
        applied.theme.seeds
    );
    assert_eq!(host.state().consultation, consultation);
}
