// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The owned persistence lane for the native consultation window.
//!
//! Cambium state emits a small product action. This worker is the only native
//! UI component allowed to open Redb or call the async persistence boundary,
//! so a slow save never borrows or blocks the window event loop.

use std::path::PathBuf;
use std::sync::mpsc::{self, Sender};

use muniment::RedbBackend;

use crate::{
    CleromancyHost, Consultation, ConsultationCatalog, ConsultationDetail, ConsultationError,
    ReceiptComparison, SelectionMode,
};

use super::{ConsultationAction, ConsultationContext, ConsultationLayout};

/// A result crossing from the persistence lane back to the native host.
#[derive(Clone, Debug)]
pub(crate) enum WorkerUpdate {
    Catalog(ConsultationCatalog),
    Reading {
        catalog: ConsultationCatalog,
        detail: ConsultationDetail,
    },
    Reflection {
        catalog: ConsultationCatalog,
        detail: ConsultationDetail,
    },
    Session {
        catalog: ConsultationCatalog,
        detail: ConsultationDetail,
    },
    Comparison {
        catalog: ConsultationCatalog,
        detail: ConsultationDetail,
        comparison: ReceiptComparison,
    },
    Error {
        catalog: Option<ConsultationCatalog>,
        message: String,
    },
}

/// A send-only handle held by the native event loop.
///
/// Dropping it closes the command channel and lets the worker leave cleanly
/// after the window closes.
#[derive(Clone, Debug)]
pub(crate) struct ConsultationWorker {
    commands: Sender<ConsultationAction>,
}

impl ConsultationWorker {
    pub(crate) fn command(&self, command: ConsultationAction) -> Result<(), String> {
        self.commands
            .send(command)
            .map_err(|_| "the consultation worker is no longer running".to_string())
    }
}

/// Start the worker and immediately publish the durable picker/history state.
pub(crate) fn spawn(
    store_path: PathBuf,
    deliver: impl Fn(WorkerUpdate) + Send + 'static,
) -> ConsultationWorker {
    let (commands, receiver) = mpsc::channel();
    std::thread::Builder::new()
        .name("cleromancy-store".to_string())
        .spawn(move || run(store_path, receiver, deliver))
        .expect("start Cleromancy persistence worker");
    ConsultationWorker { commands }
}

fn run(
    store_path: PathBuf,
    commands: mpsc::Receiver<ConsultationAction>,
    deliver: impl Fn(WorkerUpdate),
) {
    let backend = match RedbBackend::open(&store_path) {
        Ok(backend) => backend,
        Err(error) => {
            deliver(WorkerUpdate::Error {
                catalog: None,
                message: format!("open local store: {error}"),
            });
            return;
        }
    };
    let mut consultation = match open_consultation(backend.clone()) {
        Ok(consultation) => consultation,
        Err(error) => {
            deliver(WorkerUpdate::Error {
                catalog: None,
                message: format!("open local consultation: {error}"),
            });
            return;
        }
    };

    if consultation.host().is_empty() {
        if let Err(error) = pollster::block_on(consultation.install_builtin_tarot()) {
            deliver(WorkerUpdate::Error {
                catalog: None,
                message: format!("install built-in tarot field: {error}"),
            });
            return;
        }
    }
    match consultation.catalog() {
        Ok(catalog) => deliver(WorkerUpdate::Catalog(catalog)),
        Err(error) => {
            deliver(WorkerUpdate::Error {
                catalog: None,
                message: format!("load local consultation: {error}"),
            });
            return;
        }
    }

    while let Ok(action) = commands.recv() {
        match execute(&mut consultation, action) {
            Ok(update) => deliver(update),
            Err(error) => {
                // A persist failure makes the controller deliberately refuse
                // follow-up writes. Reopen before accepting the next action,
                // never pretending an uncertain graph instance is still live.
                if consultation.is_faulted() {
                    match open_consultation(backend.clone()) {
                        Ok(reopened) => consultation = reopened,
                        Err(reopen_error) => {
                            deliver(WorkerUpdate::Error {
                                catalog: None,
                                message: format!(
                                    "{error}; reopen local consultation: {reopen_error}"
                                ),
                            });
                            continue;
                        }
                    }
                }
                deliver(WorkerUpdate::Error {
                    catalog: consultation.catalog().ok(),
                    message: error.to_string(),
                });
            }
        }
    }
}

fn open_consultation(backend: RedbBackend) -> Result<Consultation<RedbBackend>, ConsultationError> {
    Ok(Consultation::new(pollster::block_on(
        CleromancyHost::open(backend),
    )?))
}

fn execute(
    consultation: &mut Consultation<RedbBackend>,
    action: ConsultationAction,
) -> Result<WorkerUpdate, ConsultationError> {
    match action {
        ConsultationAction::Read {
            context,
            field_digest,
            mode,
            derivation,
            layout,
            astrology_facts_digest,
        } => {
            let context_digest = match context {
                ConsultationContext::Existing(digest) => digest,
                ConsultationContext::New(draft) => {
                    pollster::block_on(consultation.save_context(draft))?
                }
            };
            let detail = match layout {
                ConsultationLayout::Single => match (mode, derivation) {
                    (SelectionMode::Derived, Some(selection)) => pollster::block_on(
                        consultation.read_derived(&context_digest, &field_digest, selection),
                    )?,
                    (SelectionMode::Derived, None) => {
                        return Err(ConsultationError::DerivedSelectionRequired);
                    }
                    (_, None) => {
                        pollster::block_on(consultation.read(&context_digest, &field_digest, mode))?
                    }
                    (_, Some(_)) => {
                        return Err(ConsultationError::UnexpectedDerivation);
                    }
                },
                ConsultationLayout::ThreeCard => pollster::block_on(
                    consultation.read_three_card(&context_digest, &field_digest),
                )?,
                ConsultationLayout::Authored(template_id) => pollster::block_on(
                    consultation.read_spread(&context_digest, &field_digest, &template_id),
                )?,
            };
            let detail = match astrology_facts_digest {
                Some(facts_digest) => pollster::block_on(
                    consultation.associate_astrology_facts(&facts_digest, &detail.session.id),
                )?,
                None => detail,
            };
            let catalog = consultation.catalog()?;
            Ok(WorkerUpdate::Reading { catalog, detail })
        }
        ConsultationAction::SaveSpreadTemplate { draft } => {
            pollster::block_on(consultation.save_spread_template(draft))?;
            Ok(WorkerUpdate::Catalog(consultation.catalog()?))
        }
        ConsultationAction::SaveAstrologyChart { draft } => {
            pollster::block_on(consultation.save_astrology_chart(draft))?;
            Ok(WorkerUpdate::Catalog(consultation.catalog()?))
        }
        ConsultationAction::SaveReflection { session_id, body } => {
            let detail = pollster::block_on(consultation.reflect(&session_id, body))?;
            let catalog = consultation.catalog()?;
            Ok(WorkerUpdate::Reflection { catalog, detail })
        }
        ConsultationAction::SelectSession { session_id } => {
            let detail = consultation.detail(&session_id)?;
            let catalog = consultation.catalog()?;
            Ok(WorkerUpdate::Session { catalog, detail })
        }
        ConsultationAction::CompareSessions {
            left_session_id,
            right_session_id,
        } => {
            let detail = consultation.detail(&left_session_id)?;
            let comparison = consultation.compare_receipts(&left_session_id, &right_session_id)?;
            let catalog = consultation.catalog()?;
            Ok(WorkerUpdate::Comparison {
                catalog,
                detail,
                comparison,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;
    use std::time::Duration;

    use super::*;
    use crate::{ContextDraft, SelectionMode};

    #[test]
    fn worker_bootstraps_tarot_and_returns_durable_reading_updates() {
        let temp = tempfile::tempdir().expect("temporary local store");
        let (updates, received) = mpsc::channel();
        let worker = spawn(temp.path().join("cleromancy.redb"), move |update| {
            updates.send(update).expect("test receiver remains open");
        });
        let catalog = match received
            .recv_timeout(Duration::from_secs(10))
            .expect("initial catalog update")
        {
            WorkerUpdate::Catalog(catalog) => catalog,
            update => panic!("expected initial catalog, got {update:?}"),
        };
        assert_eq!(catalog.fields.len(), 1, "first launch installs tarot only");
        assert!(catalog.contexts.is_empty());
        assert!(catalog.sessions.is_empty());

        worker
            .command(ConsultationAction::Read {
                context: ConsultationContext::New(ContextDraft::new(
                    "A changing structure",
                    "What deserves attention now?",
                    "change, reflection",
                )),
                field_digest: catalog.fields[0].digest(),
                mode: SelectionMode::Calculated,
                derivation: None,
                layout: ConsultationLayout::Single,
                astrology_facts_digest: None,
            })
            .expect("worker accepts read");
        let (catalog, detail) = match received
            .recv_timeout(Duration::from_secs(10))
            .expect("reading update")
        {
            WorkerUpdate::Reading { catalog, detail } => (catalog, detail),
            update => panic!("expected reading update, got {update:?}"),
        };
        assert_eq!(catalog.contexts.len(), 1);
        assert_eq!(catalog.sessions.len(), 1);
        assert_eq!(detail.readings.len(), 1);
        let context_digest = detail.session.context_digest.clone();

        worker
            .command(ConsultationAction::SaveReflection {
                session_id: detail.session.id.clone(),
                body: "Keep the useful constraint revisable.".to_string(),
            })
            .expect("worker accepts reflection");
        match received
            .recv_timeout(Duration::from_secs(10))
            .expect("reflection update")
        {
            WorkerUpdate::Reflection { detail, .. } => {
                assert_eq!(detail.reflections.len(), 1);
                assert_eq!(
                    detail.reflections[0].body,
                    "Keep the useful constraint revisable."
                );
            }
            update => panic!("expected reflection update, got {update:?}"),
        }

        worker
            .command(ConsultationAction::Read {
                context: ConsultationContext::Existing(context_digest),
                field_digest: catalog.fields[0].digest(),
                mode: SelectionMode::Cast,
                derivation: None,
                layout: ConsultationLayout::ThreeCard,
                astrology_facts_digest: None,
            })
            .expect("worker accepts three-card cast");
        match received
            .recv_timeout(Duration::from_secs(10))
            .expect("three-card update")
        {
            WorkerUpdate::Reading { detail, .. } => {
                assert_eq!(detail.readings.len(), 3);
                assert_eq!(
                    detail
                        .session
                        .placements
                        .iter()
                        .map(|placement| placement.position.as_str())
                        .collect::<Vec<_>>(),
                    ["foundation", "tension", "next_step"]
                );
            }
            update => panic!("expected three-card reading update, got {update:?}"),
        }
    }
}
