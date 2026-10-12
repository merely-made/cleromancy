// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Shared workshop hooks over the ordinary consultation host.

use super::appearance_view::ApplicationUi;
use super::{ConsultationCtx, ConsultationRunner};
use cambium_genet_winit_host::{FocusedTextSlot, choose_save_path};

pub(super) fn after_dispatch(ctx: &mut ConsultationCtx<'_>) {
    after_dispatch_with_exporter(ctx, |artifact| {
        choose_save_path(
            "Export theme",
            &artifact.suggested_name,
            &[if artifact.format == tabard_workshop::ExportFormat::Css {
                "css"
            } else {
                "json"
            }],
        )
    });
}

pub(super) fn after_dispatch_with_exporter(
    ctx: &mut ConsultationCtx<'_>,
    mut chooser: impl FnMut(&tabard_workshop::ExportArtifact) -> Option<std::path::PathBuf>,
) {
    let mut close = false;
    ctx.runner.update(|state| {
        state.appearance.commit_requested();
        let exit = state.appearance.editor_open
            && tabard_workshop::native_host::sync_and_export(
                &mut state.appearance.workshop,
                &mut chooser,
            );
        close = state.appearance.sync_editor(exit);
    });
    refresh_stylesheet(ctx);
    if close {
        *ctx.close = true;
    }
}

pub(super) fn refresh_stylesheet(ctx: &mut ConsultationCtx<'_>) {
    let mut sheet = None;
    ctx.runner
        .update(|state| sheet = state.appearance.take_stylesheet_change());
    if let Some(sheet) = sheet {
        *ctx.set_sheet = Some(sheet);
    }
}

pub(super) fn focused_text(runner: &ConsultationRunner) -> Option<FocusedTextSlot<ApplicationUi>> {
    let node = runner.focus()?;
    if runner.state().appearance.editor_open {
        let field = tabard_workshop::native_host::focused_field(
            &runner.dom(),
            node,
            &runner.state().appearance.workshop,
        )?;
        let mutable = field.clone();
        return Some(FocusedTextSlot {
            node,
            get: Box::new(move |state| {
                state
                    .appearance
                    .workshop
                    .text_field(&field)
                    .expect("registered workshop field")
            }),
            get_mut: Box::new(move |state| {
                state
                    .appearance
                    .workshop
                    .text_field_mut(&mutable)
                    .expect("registered workshop field")
            }),
        });
    }
    if let Some(field) = tabard_workshop::fonts::focused_font_field(&runner.dom(), node) {
        return Some(FocusedTextSlot {
            node,
            get: Box::new(move |state| state.appearance.font_editor.text_field(field)),
            get_mut: Box::new(move |state| state.appearance.font_editor.text_field_mut(field)),
        });
    }
    let FocusedTextSlot { get, get_mut, .. } = super::consultation_text_slot(&runner.dom(), node)?;
    Some(FocusedTextSlot {
        node,
        get: Box::new(move |state| get(&state.consultation)),
        get_mut: Box::new(move |state| get_mut(&mut state.consultation)),
    })
}

pub(super) fn close_request(
    ctx: &mut ConsultationCtx<'_>,
    _: cambium_genet_winit_host::CloseRequest,
) -> cambium_genet_winit_host::CloseDisposition {
    let mut allow = false;
    ctx.runner
        .update(|state| allow = state.appearance.prepare_close());
    refresh_stylesheet(ctx);
    if allow {
        cambium_genet_winit_host::CloseDisposition::Exit
    } else {
        cambium_genet_winit_host::CloseDisposition::KeepVisible
    }
}
