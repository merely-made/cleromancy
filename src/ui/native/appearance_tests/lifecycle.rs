// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;

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
        let node = matching_role(
            &host,
            &Selector::role("button").with_attr("data-action", "toggle-appearance"),
        )[0];
        let (x, y, w, h) = host.painted_rect(node).unwrap();
        assert!(x >= 0.0 && y >= 0.0 && w > 0.0 && h >= 24.0 && x + w <= width + 0.1);
    }
    action(&mut host, "toggle-appearance");
    action(&mut host, "edit-appearance");
    for (width, height) in [(420.0, 1000.0), (1160.0, 760.0)] {
        host.layout_at(width, height);
        for field in ["name", "seed-hex"] {
            let node = matching_role(
                &host,
                &Selector::role("textbox").with_attr("data-field", field),
            )[0];
            let (_, _, w, h) = host.painted_rect(node).unwrap();
            assert!(w >= 40.0 && h >= 33.9, "{field} at {width}px: {w}x{h}");
        }
        for control in ["back-to-consultation", "apply-to-cleromancy"] {
            let node = matching_role(
                &host,
                &Selector::role("button").with_attr("data-action", control),
            )[0];
            let (x, y, w, h) = host.visible_rect(node).unwrap();
            assert!(x >= 0.0 && y >= 0.0 && w > 0.0 && h >= 24.0 && x + w <= width + 0.1);
        }
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
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("scenarios");
    for name in [
        "tabard-authoring.scn",
        "tabard-reopen.scn",
        "tabard-same-id.scn",
        "tabard-radio-foreground.scn",
        "tabard-radio-reopen.scn",
        "tabard-font-roles.scn",
        "tabard-font-reopen.scn",
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
