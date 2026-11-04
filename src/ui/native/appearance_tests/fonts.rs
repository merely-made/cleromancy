// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;

#[test]
fn font_roles_edit_without_apply_then_save_reopen_and_survive_mode_changes() {
    use tabard::typography::FontRole;
    let dir = tempfile::tempdir().unwrap();
    let mut host = mounted(&dir);
    action(&mut host, "toggle-appearance");
    focus_selector(
        &mut host,
        Selector::role("textbox").with_attr("data-font-role", "reading"),
    );
    host.key_injected("IBM Plex Mono");
    assert_eq!(
        host.state()
            .appearance
            .font_editor
            .field(FontRole::Reading)
            .text(),
        "IBM Plex Mono"
    );
    assert!(host.state().appearance.font_preferences().is_empty());
    assert!(!dir.path().join("appearance.json").exists());
    focus_selector(
        &mut host,
        Selector::role("button").with_attr("data-action", "apply-font-roles"),
    );
    host.press_key(&cambium_genet_winit_host::KeyPress::new(
        cambium_genet_winit_host::Key::Named(cambium_genet_winit_host::NamedKey::Enter),
    ));
    host.after_dispatch();
    host.relayout();
    assert_eq!(
        host.state().appearance.font_preferences().0[&FontRole::Reading][0].name(),
        "IBM Plex Mono"
    );
    let before = std::fs::read(dir.path().join("appearance.json")).unwrap();
    assert!(
        host.state()
            .appearance
            .stylesheet()
            .contains("--tabard-font-reading: \"IBM Plex Mono\", serif;")
    );
    let mut reopened = state(&dir);
    assert_eq!(
        reopened.font_preferences(),
        host.state().appearance.font_preferences()
    );
    reopened
        .request_select(PRODUCT_DEFAULT, Some(Mode::Light))
        .unwrap();
    reopened.commit_requested();
    assert_eq!(
        reopened.font_preferences().0[&FontRole::Reading][0].name(),
        "IBM Plex Mono"
    );
    assert_ne!(
        std::fs::read(dir.path().join("appearance.json")).unwrap(),
        before
    );
}

#[test]
fn missing_font_requests_and_failed_writes_preserve_applied_typography() {
    use tabard::typography::FontRole;
    let dir = tempfile::tempdir().unwrap();
    let mut appearance = state(&dir);
    *appearance.font_editor.field_mut(FontRole::Reading) =
        cambium::TextInput::new("Cleromancy Missing Face");
    appearance.font_editor.request_apply();
    appearance.commit_requested();
    assert!(appearance.font_diagnostics()[0].contains("using serif"));
    let reopened = state(&dir);
    assert_eq!(
        reopened.font_preferences().0[&FontRole::Reading][0].name(),
        "Cleromancy Missing Face"
    );
    assert!(
        reopened
            .stylesheet()
            .contains("--tabard-font-reading: serif;")
    );
    let original = appearance.stylesheet();
    std::fs::remove_file(dir.path().join("appearance.json")).unwrap();
    std::fs::create_dir(dir.path().join("appearance.json")).unwrap();
    *appearance.font_editor.field_mut(FontRole::Reading) = cambium::TextInput::new("IBM Plex Mono");
    appearance.font_editor.request_apply();
    appearance.commit_requested();
    assert_eq!(appearance.stylesheet(), original);
    assert!(appearance.notice.as_ref().unwrap().contains("not saved"));
    assert!(dir.path().join("appearance.json").is_dir());
}

#[test]
fn invalid_saved_font_preferences_remain_preserved_and_block_apply() {
    use tabard::typography::FontRole;
    let dir = tempfile::tempdir().unwrap();
    let bytes = br#"{"theme_id":"cleromancy:default","fonts":{"reading":["bad\nname"]}}"#;
    std::fs::write(dir.path().join("appearance.json"), bytes).unwrap();
    let mut appearance = state(&dir);
    *appearance.font_editor.field_mut(FontRole::Reading) = cambium::TextInput::new("IBM Plex Mono");
    appearance.font_editor.request_apply();
    appearance.commit_requested();
    assert!(appearance.font_preferences().is_empty());
    assert!(
        appearance
            .font_editor
            .notice
            .as_ref()
            .unwrap()
            .contains("preserved file")
    );
    assert_eq!(
        std::fs::read(dir.path().join("appearance.json")).unwrap(),
        bytes
    );
}

#[test]
fn role_families_shape_real_bundled_faces_and_authored_css_retains_precedence() {
    use std::collections::BTreeMap;
    use tabard::typography::{FontFamily, FontRole};
    let dir = tempfile::tempdir().unwrap();
    let mut appearance = state(&dir);
    let families = appearance.available_fonts.clone();
    assert!(families.iter().any(|name| name == "IBM Plex Mono"));
    assert!(families.iter().any(|name| name == "Source Serif 4"));
    *appearance.font_editor.field_mut(FontRole::Reading) = cambium::TextInput::new("IBM Plex Mono");
    *appearance.font_editor.field_mut(FontRole::Heading) =
        cambium::TextInput::new("Source Serif 4");
    appearance.font_editor.request_apply();
    appearance.commit_requested();
    let probe = "\n.font-probe { position:absolute;display:block;top:0;left:0;font-size:20px; } .font-probe p,.font-probe h4 { display:block;position:absolute;top:0;left:0;font-size:20px;font-weight:400;white-space:nowrap;margin:0; }";
    let mut hooks = inert_hooks();
    hooks.after_dispatch = Box::new(move |ctx| {
        appearance_host::after_dispatch_with_exporter(ctx, |_| None);
        if let Some(sheet) = ctx.set_sheet.as_mut() {
            sheet.push_str(probe);
        }
    });
    let mut host = Host::with_hooks(
        Init {
            sheet: format!("{}{}", appearance.stylesheet(), probe),
            state: appearance_view::ApplicationUi {
                consultation: crate::ConsultationUi::new(empty_catalog()),
                mere: Default::default(),
                appearance,
            },
            logic: Box::new(|_: &appearance_view::ApplicationUi| {
                Box::new(
                    cambium::el(
                        "div",
                        (
                            cambium::el("p", cambium::text("00000000")).attr("class", "mono-probe"),
                            cambium::el("h4", cambium::text("iiiiiiii"))
                                .attr("class", "serif-narrow"),
                            cambium::el("h4", cambium::text("WWWWWWWW"))
                                .attr("class", "serif-wide"),
                        ),
                    )
                    .attr("class", "reading-card font-probe"),
                ) as Child
            }) as ApplicationLogic,
            fonts: super::super::fonts::bundled_fonts(),
            images: vec![],
        },
        hooks,
    );
    host.layout_at(900.0, 300.0);
    let width = |host: &Host, class: &str| {
        let node = host.with_dom(|dom| taproot::matching_classes(dom, &Selector::class(class))[0]);
        host.painted_rect(node).unwrap().2
    };
    assert!(
        (width(&host, "mono-probe") - 96.0).abs() < 1.0,
        "Plex's measured 600/1000 em advance"
    );
    assert!(width(&host, "serif-wide") > 1.8 * width(&host, "serif-narrow"));
    host.layout_at(420.0, 500.0);
    assert!(
        (width(&host, "mono-probe") - 96.0).abs() < 1.0,
        "resource ledger survives resize"
    );
    // Explicit authored family declarations have normal cascade authority.
    let mut theme = theme(&dir.path().join("themes.json"), true);
    theme
        .mode_sheets
        .get_mut("dark")
        .unwrap()
        .push(".reading-card p { font-family:'Source Serif 4'; }".into());
    ThemeLibraryStore::load(dir.path().join("themes.json"))
        .unwrap()
        .save(&[theme.clone()])
        .unwrap();
    host.update(|state| {
        state.appearance.reload_library().unwrap();
        state
            .appearance
            .request_select(&theme.id, Some(Mode::Dark))
            .unwrap();
    });
    host.after_dispatch();
    host.relayout();
    assert!((width(&host, "mono-probe") - 96.0).abs() > 1.0);
    // Ordered available families remain in CSS for per-glyph engine fallback.
    let ordered = tabard::typography::FontPreferences(BTreeMap::from([(
        FontRole::Reading,
        vec![
            FontFamily::new("IBM Plex Mono").unwrap(),
            FontFamily::new("Source Serif 4").unwrap(),
        ],
    )]));
    assert!(
        ordered
            .css_custom_properties(&families, &super::super::fonts::defaults())
            .contains("\"IBM Plex Mono\", \"Source Serif 4\", serif")
    );
}

#[test]
fn shared_font_picker_search_and_selection_remain_an_application_draft() {
    use tabard::typography::FontRole;
    let dir = tempfile::tempdir().unwrap();
    let mut host = mounted(&dir);
    host.layout_at(420.0, 1000.0);
    action(&mut host, "toggle-appearance");
    focus_selector(
        &mut host,
        Selector::role("button").with_attr("data-font-choose", "reading"),
    );
    let enter = || {
        cambium_genet_winit_host::KeyPress::new(cambium_genet_winit_host::Key::Named(
            cambium_genet_winit_host::NamedKey::Enter,
        ))
    };
    host.press_key(&enter());
    focus_selector(
        &mut host,
        Selector::role("textbox").with_attr("data-font-search", "true"),
    );
    host.key_injected("IBM Plex Mono");
    assert_eq!(
        host.state().appearance.font_editor.search.text(),
        "IBM Plex Mono"
    );
    focus_selector(
        &mut host,
        Selector::role("button").with_attr("data-font-family", "IBM Plex Mono"),
    );
    host.press_key(&enter());
    assert_eq!(
        host.state()
            .appearance
            .font_editor
            .field(FontRole::Reading)
            .text(),
        "IBM Plex Mono"
    );
    assert!(host.state().appearance.font_preferences().is_empty());
    assert!(!dir.path().join("appearance.json").exists());
    focus_selector(
        &mut host,
        Selector::role("button").with_attr("data-action", "apply-font-roles"),
    );
    host.press_key(&enter());
    host.after_dispatch();
    assert_eq!(
        host.state().appearance.font_preferences().0[&FontRole::Reading][0].name(),
        "IBM Plex Mono"
    );
}
