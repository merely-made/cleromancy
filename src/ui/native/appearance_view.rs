// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Native application chrome around the reusable consultation view.

use super::appearance::{AppearanceState, PRODUCT_DEFAULT};
use crate::{ConsultationUi, consultation_view};
use cambium::{AnyView, GenetCtx, GenetElement, PointerClick, button, el, lens, text};

pub(super) struct ApplicationUi {
    pub consultation: ConsultationUi,
    pub appearance: AppearanceState,
}

pub(super) type Child = Box<dyn AnyView<ApplicationUi, (), GenetCtx, GenetElement>>;

pub(super) fn root(state: &ApplicationUi, captions: Child) -> Child {
    let chrome = cambium::title_bar(
        Box::new(
            el("span", text("C"))
                .attr("class", "cleromancy-ornament")
                .attr("aria-hidden", "true"),
        ),
        Box::new(el("span", text("Cleromancy"))),
        Box::new(
            button(
                "Appearance",
                |state: &mut ApplicationUi, _: PointerClick| {
                    state.appearance.open = !state.appearance.open;
                },
            )
            .attr("class", "cleromancy-appearance-toggle")
            .attr("data-action", "toggle-appearance")
            .attr("aria-expanded", state.appearance.open.to_string()),
        ),
        captions,
    );
    let mut children = vec![chrome];
    if state.appearance.editor_open {
        children.push(editor(state));
    } else {
        if state.appearance.open {
            children.push(panel(state));
        }
        children.push(Box::new(lens(
            |ui: &mut ConsultationUi| consultation_view(ui),
            |state: &mut ApplicationUi| &mut state.consultation,
        )));
    }
    Box::new(el("div", children).attr("class", "cleromancy-desktop"))
}

fn notices(state: &ApplicationUi, class: &str) -> Vec<Child> {
    state
        .appearance
        .diagnostics()
        .into_iter()
        .map(|message| {
            Box::new(
                el("p", text(message))
                    .attr("class", class)
                    .attr("role", "status"),
            ) as Child
        })
        .collect()
}

fn panel(state: &ApplicationUi) -> Child {
    let mut themes: Vec<Child> = vec![Box::new(
        button(
            "Cleromancy default",
            |state: &mut ApplicationUi, _: PointerClick| {
                state.appearance.notice =
                    state.appearance.request_select(PRODUCT_DEFAULT, None).err();
            },
        )
        .attr("data-appearance-theme", PRODUCT_DEFAULT)
        .attr(
            "aria-pressed",
            (state.appearance.active_id() == PRODUCT_DEFAULT).to_string(),
        ),
    )];
    themes.extend(
        state
            .appearance
            .workshop
            .registry()
            .list()
            .into_iter()
            .filter(|theme| theme.id != PRODUCT_DEFAULT)
            .map(|theme| {
                let id = theme.id.clone();
                Box::new(
                    button(
                        theme.name.clone(),
                        move |state: &mut ApplicationUi, _: PointerClick| {
                            state.appearance.notice =
                                state.appearance.request_select(&id, None).err();
                        },
                    )
                    .attr("data-appearance-theme", theme.id.clone())
                    .attr(
                        "aria-pressed",
                        (state.appearance.active_id() == theme.id).to_string(),
                    ),
                ) as Child
            }),
    );
    let modes: Vec<Child> = state
        .appearance
        .modes()
        .into_iter()
        .map(|mode| {
            let key = mode.as_key();
            let active = state
                .appearance
                .applied()
                .is_some_and(|value| value.resolved.theme_mode.as_ref() == Some(&mode));
            Box::new(
                button(
                    mode.label(),
                    move |state: &mut ApplicationUi, _: PointerClick| {
                        let id = state.appearance.active_id().to_owned();
                        state.appearance.notice = state
                            .appearance
                            .request_select(&id, Some(mode.clone()))
                            .err();
                    },
                )
                .attr("data-appearance-mode", key)
                .attr("aria-pressed", active.to_string()),
            ) as Child
        })
        .collect();
    let mut children: Vec<Child> = vec![
        Box::new(el("h2", text("Appearance"))),
        Box::new(
            el("div", themes)
                .attr("class", "cleromancy-appearance-options")
                .attr("aria-label", "Theme"),
        ),
        Box::new(
            el("div", modes)
                .attr("class", "cleromancy-appearance-options")
                .attr("aria-label", "Presentation mode"),
        ),
    ];
    children.extend(notices(state, "cleromancy-appearance-notice"));
    if state.appearance.authoring_available {
        children.push(Box::new(
            button(
                "Edit themes…",
                |state: &mut ApplicationUi, _: PointerClick| {
                    state.appearance.notice = state.appearance.begin_edit().err();
                },
            )
            .attr("data-action", "edit-appearance"),
        ));
    }
    children.push(Box::new(
        button(
            "Reload theme library",
            |state: &mut ApplicationUi, _: PointerClick| {
                state.appearance.notice = match state.appearance.reload_library() {
                    Ok(()) => Some(
                        "Theme library reloaded. Apply a choice to change the active appearance."
                            .into(),
                    ),
                    Err(error) => Some(error),
                };
            },
        )
        .attr("data-action", "reload-theme-library"),
    ));
    Box::new(
        el("section", children)
            .attr("class", "cleromancy-appearance")
            .attr("aria-label", "Application appearance"),
    )
}

fn editor(state: &ApplicationUi) -> Child {
    let mut children: Vec<Child> = vec![Box::new(
        el(
            "div",
            (
                button(
                    "Back to consultation",
                    |state: &mut ApplicationUi, _: PointerClick| state.appearance.back(),
                )
                .attr("data-action", "back-to-consultation"),
                button(
                    "Apply to Cleromancy",
                    |state: &mut ApplicationUi, _: PointerClick| {
                        state.appearance.notice = state.appearance.apply_workshop().err();
                    },
                )
                .attr("data-action", "apply-to-cleromancy"),
                el(
                    "p",
                    text("Save the definition here, then apply it to Cleromancy."),
                ),
            ),
        )
        .attr("class", "cleromancy-editor-controls"),
    )];
    children.extend(notices(state, "cleromancy-editor-notice"));
    children.push(Box::new(lens(
        |workshop: &mut tabard_workshop::WorkshopState| tabard_workshop::workshop_view(workshop),
        |state: &mut ApplicationUi| &mut state.appearance.workshop,
    )));
    Box::new(el("div", children).attr("class", "cleromancy-editor"))
}

pub(super) const CHROME_CSS: &str = "
.cleromancy-desktop { display:flex;flex-direction:column;height:100vh;min-height:0;overflow:hidden;color:var(--tabard-color-text,#f1ede4);background:var(--tabard-color-bg,#181714);font-family:sans-serif; }
.cleromancy-desktop > .cleromancy-consultation { flex:1;min-height:0;overflow:auto; }
.cleromancy-desktop > .cambium-title-bar { --titlebar-min-height:42px;background:var(--tabard-color-surface,#24221d);color:var(--tabard-color-text,#f1ede4); }
.cleromancy-ornament { display:block;padding:4px 6px;font-size:18px; }
.cleromancy-appearance-toggle { display:block;padding:6px 10px;border:1px solid currentColor;border-radius:5px;background:transparent;color:inherit; }
.cleromancy-appearance { flex-shrink:0;max-height:36vh;overflow:auto;padding:14px 24px;background:var(--tabard-color-surface,#24221d);color:var(--tabard-color-text,#f1ede4); }
.cleromancy-appearance-options { display:flex;flex-wrap:wrap;gap:8px;margin:10px 0; }
.cleromancy-appearance button { display:block;padding:8px 12px;border:1px solid currentColor;background:transparent;color:inherit; }
.cleromancy-appearance button[aria-pressed=\"true\"] { background:var(--tabard-color-primary,#6e522a);color:var(--tabard-color-on-primary,#f7f2e7); }
.cleromancy-appearance button:focus-visible,.cleromancy-appearance-toggle:focus-visible { box-shadow:inset 0 0 0 2px currentColor; }
.cleromancy-appearance-notice { display:block;line-height:1.5;margin:8px 0; }
";

pub(super) const EDITOR_CSS: &str = "
.cleromancy-editor { display:flex;flex:1;flex-direction:column;min-height:0;color:#242c39; }
.cleromancy-editor > .tabard-workshop { flex:1;min-height:0; }
.cleromancy-editor-controls { display:flex;flex-wrap:wrap;align-items:center;gap:10px;padding:10px 18px;background:#fff;color:#253247; }
.cleromancy-editor-controls button { display:block;padding:8px 12px;border:1px solid currentColor; }
.cleromancy-editor-controls p { margin:0; }
.cleromancy-editor-notice { display:block;padding:8px 18px;margin:0;background:#fff4d8;color:#493d25; }
@media(max-width:480px) { .cleromancy-editor-controls { display:flex;flex-direction:column;align-items:stretch;padding:8px 10px;gap:6px; } .cleromancy-editor-controls button { min-height:34px; } }
";

// An editable built-in definition captures the exact existing dark product
// roles. The built-in itself is immutable; the workshop requires a user copy.
pub(super) const PRODUCT_DEFAULT_CSS: &str = r#"
:root { --tabard-color-bg:#181714; --tabard-color-surface:#24221d; --tabard-color-surface-2:#302d26; --tabard-color-surface-hover:#574124; --tabard-color-text:#f1ede4; --tabard-color-text-header:#e4cf9f; --tabard-color-text-dim:#d7c9a9; --tabard-color-text-disabled:#625d50; --tabard-color-primary:#6e522a; --tabard-color-on-primary:#f7f2e7; --tabard-color-danger:#ffd8d2; }
.cleromancy-desktop .cleromancy-consultation { color: #f1ede4; background: #181714; }
.cleromancy-desktop .cleromancy-consultation .app-header { border-bottom: 1px solid #625d50; }
.cleromancy-desktop .cleromancy-consultation .selection-item { border: 1px solid #625d50; color: #d7c9a9; }
.cleromancy-desktop .cleromancy-consultation .selection-item.selected { color: #f7f2e7; background: #6e522a; }
.cleromancy-desktop .cleromancy-consultation .selection-item[aria-disabled='true'] { color: #8b8474; }
.cleromancy-desktop .cleromancy-consultation .eyebrow { color: #d7b46a; }
.cleromancy-desktop .cleromancy-consultation section[role='region'] { border: 1px solid #625d50; background: #24221d; }
.cleromancy-desktop .cleromancy-consultation .control-label { color: #d7c9a9; }
.cleromancy-desktop .cleromancy-consultation input, .cleromancy-desktop .cleromancy-consultation textarea, .cleromancy-desktop .cleromancy-consultation select, .cleromancy-desktop .cleromancy-consultation button, .cleromancy-desktop .cleromancy-consultation [role='textbox'] { border: 1px solid #7b725f; color: #f7f2e7; background: #302d26; }
.cleromancy-desktop .cleromancy-consultation button { background: #6e522a; }
.cleromancy-desktop .cleromancy-consultation button:focus, .cleromancy-desktop .cleromancy-consultation input:focus, .cleromancy-desktop .cleromancy-consultation textarea:focus, .cleromancy-desktop .cleromancy-consultation select:focus, .cleromancy-desktop .cleromancy-consultation [role='textbox']:focus { outline: 2px solid #d7b46a; }
.cleromancy-desktop .cleromancy-consultation [role='alert'] { color: #ffd8d2; background: #542d29; }
.cleromancy-desktop .cleromancy-consultation [role='status'] { color: #d7c9a9; }
.cleromancy-desktop .cleromancy-consultation .selection-explanation, .cleromancy-desktop .cleromancy-consultation .empty-reading { color: #c4bcad; }
.cleromancy-desktop .cleromancy-consultation .chart-ecliptic-labels { color: #d7c9a9; }
.cleromancy-desktop .cleromancy-consultation .chart-ecliptic-explanation { color: #c4bcad; }
.cleromancy-desktop .cleromancy-consultation .reading-question { color: #e4cf9f; }
.cleromancy-desktop .cleromancy-consultation .reading-card { border: 1px solid #897550; background: #302b22; }
.cleromancy-desktop .cleromancy-consultation .reading-card h3 { color: #d7b46a; }
.cleromancy-desktop .cleromancy-consultation .reading-card h4 { color: #fff0cf; }
.cleromancy-desktop .cleromancy-consultation .context-explanation, .cleromancy-desktop .cleromancy-consultation .reflection-explanation, .cleromancy-desktop .cleromancy-consultation .trail-explanation { color: #c4bcad; }
.cleromancy-desktop .cleromancy-consultation [role='radio'] { border: 1px solid #7b725f; }
.cleromancy-desktop .cleromancy-consultation [role='radio'][aria-checked='true'] { background: #6e522a; border-color: #d7b46a; }
.cleromancy-desktop .cleromancy-consultation .control [role='combobox'] { border: 1px solid #7b725f; background: #302d26; }
.cleromancy-desktop .cleromancy-consultation .select-list { border: 1px solid #897550; background: #302d26; }
.cleromancy-desktop .cleromancy-consultation .select-option[aria-selected='true'] { background: #6e522a; }
.cleromancy-desktop .cleromancy-consultation .scene-card { background: #302b22; }
.cleromancy-desktop .cleromancy-consultation .scene-card[aria-pressed='true'] { background: #574124; border: 2px solid #e4cf9f; }
.cleromancy-desktop .cleromancy-consultation .scene-position { color: #d7b46a; }
.cleromancy-desktop .cleromancy-consultation .scene-title { color: #fff0cf; }
.cleromancy-desktop .cleromancy-consultation .reading-card .position-rationale { color: #d7c9a9; }
.cleromancy-desktop .cleromancy-consultation .astrology-prompt { border-left: 2px solid #897550; background: #302b22; }
.cleromancy-desktop .cleromancy-consultation .astrology-prompt h4 { color: #fff0cf; }
"#;
