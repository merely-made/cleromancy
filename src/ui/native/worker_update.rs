// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;

pub(super) fn apply_worker_update(
    runner: &mut ConsultationRunner,
    state: &mut NativeState,
    update: WorkerUpdate,
) {
    match update {
        WorkerUpdate::BirthFacts { catalog, notice } => runner.update(move |state| {
            let ui = &mut state.consultation;
            let subject = ui
                .birth_catalog
                .as_ref()
                .and_then(|c| c.subjects.get(ui.birth_subject))
                .map(|s| s.subject.clone());
            let birth = ui
                .birth_catalog
                .as_ref()
                .and_then(|c| c.births.get(ui.birth_revision))
                .map(|b| b.resource.clone());
            ui.birth_subject = catalog
                .as_ref()
                .and_then(|c| {
                    c.subjects
                        .iter()
                        .position(|s| Some(&s.subject) == subject.as_ref())
                })
                .unwrap_or(0);
            ui.birth_revision = catalog
                .as_ref()
                .and_then(|c| {
                    c.births
                        .iter()
                        .position(|b| Some(&b.resource) == birth.as_ref())
                })
                .unwrap_or(0);
            let failed = catalog.is_none();
            ui.birth_catalog = catalog;
            ui.birth_notice = notice.clone();
            state.consultation.status = super::super::ConsultationStatus::Ready;
            state.consultation.error = failed.then_some(notice);
        }),

        WorkerUpdate::GrantedContext {
            cards,
            notice,
            complete_request,
        } => runner.update(move |state| {
            state.consultation.ambient.granted_cards = cards;
            state.consultation.ambient.granted_notice = notice;
            if complete_request {
                state.consultation.status = super::super::ConsultationStatus::Ready;
            }
        }),
        WorkerUpdate::Neighborhood {
            catalog,
            detail,
            context,
        } => {
            runner.update(move |state| {
                state
                    .consultation
                    .present_neighborhood(catalog, detail, context)
            });
        },
        WorkerUpdate::Unavailable { message } => {
            state.catalog_ready = true;
            state
                .probe_events
                .push(format!("resident unavailable: {message}"));
            runner.update(|state| {
                state.mere.busy = false;
                state.mere.view.focused = None;
                state.mere.view.hovered = None;
                state.mere.model.graph = Default::default();
                state.mere.model.sessions.clear();
                state.mere.model.can_mint = false;
                state.mere.model.actions.clear();
                state.mere.pending = None;
                state.mere.model.archive = None;
                state.consultation.birth_catalog = None;
                state.consultation.clear_selection();
                state.consultation.replace_catalog(empty_catalog());
                state.mere.model.status =
                    mere_view::ViewStatus::Unavailable(mere_view::StatusNote {
                        message: message.clone(),
                        action: None,
                    });
                state.consultation.present_error(message);
            });
        },
        WorkerUpdate::Mere {
            model,
            catalog,
            selection,
            complete_request,
        } => {
            state.catalog_ready = true;
            runner.update(|state| {
                apply_mere_publication(state, model, catalog, selection, complete_request);
            });
        },
        WorkerUpdate::GraphSelection {
            catalog,
            detail,
            position,
        } => {
            state
                .probe_events
                .push(format!("shared graph selected {}", detail.session.id));
            runner.update(|state| {
                state.mere.busy = false;
                let session = detail.session.id.clone();
                let focus_url = position
                    .as_ref()
                    .and_then(|position| {
                        detail
                            .session
                            .placements
                            .iter()
                            .find(|p| &p.position == position)
                    })
                    .map(|p| format!("cleromancy://reading/{}", p.reading_id))
                    .unwrap_or_else(|| format!("cleromancy://session/{session}"));
                state.mere.view.focused =
                    Some(kernel::graph::Graph::node_namespace_id(&focus_url).to_string());
                state.consultation.present_external_session(catalog, detail);
                if let Some(position) = position {
                    state.consultation.reading_focus.insert(session, position);
                }
            });
        },
        WorkerUpdate::Die {
            catalog,
            field_digest,
        } => {
            state.catalog_ready = true;
            runner.update(|state| state.consultation.present_die(catalog, &field_digest));
        },
        WorkerUpdate::Catalog(catalog) => {
            state.catalog_ready = true;
            runner.update(|state| {
                let ui = &mut state.consultation;
                ui.replace_catalog(catalog);
                if ui.status.is_busy() {
                    ui.status = crate::ui::state::ConsultationStatus::Ready;
                }
            });
        },
        #[cfg(feature = "analytic-ephemeris")]
        WorkerUpdate::AstrologyChart {
            catalog,
            facts_digest,
        } => {
            state
                .probe_events
                .push(format!("durable astrology chart saved {facts_digest}"));
            runner.update(move |state| {
                let ui = &mut state.consultation;
                ui.present_calculated_chart(catalog, facts_digest);
            });
        },
        WorkerUpdate::Reading { catalog, detail } => {
            state
                .probe_events
                .push(format!("durable reading saved {}", detail.session.id));
            runner.update(move |state| {
                let ui = &mut state.consultation;
                ui.present_reading(catalog, detail);
            });
        },
        WorkerUpdate::Reflection { catalog, detail } => {
            state
                .probe_events
                .push(format!("durable reflection saved {}", detail.session.id));
            runner.update(move |state| {
                let ui = &mut state.consultation;
                ui.present_reflection(catalog, detail);
            });
        },
        WorkerUpdate::Session { catalog, detail } => {
            runner.update(|state| state.mere.busy = false);
            state
                .probe_events
                .push(format!("durable session recovered {}", detail.session.id));
            runner.update(move |state| {
                let ui = &mut state.consultation;
                ui.present_session(catalog, detail);
            });
        },
        WorkerUpdate::Comparison {
            catalog,
            detail,
            comparison,
        } => {
            state.probe_events.push(format!(
                "durable receipts compared {} {}",
                comparison.left_session_id, comparison.right_session_id
            ));
            runner.update(move |state| {
                let ui = &mut state.consultation;
                ui.present_comparison(catalog, detail, comparison);
            });
        },
        WorkerUpdate::SavedNotice(message) => runner.update(move |state| {
            state.consultation.status = super::super::ConsultationStatus::SavedNotice(message);
        }),
        WorkerUpdate::Error { catalog, message } => runner.update(move |state| {
            state.mere.busy = false;
            let ui = &mut state.consultation;
            if let Some(catalog) = catalog {
                ui.replace_catalog(catalog);
            }
            ui.present_error(message);
        }),
    }
}
