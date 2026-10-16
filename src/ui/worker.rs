// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The admitted resident command lane for the native consultation window.
//!
//! Cambium state emits a small product action. This worker is the only native
//! UI component allowed to call the async resident boundary,
//! so a slow save never borrows or blocks the window event loop.

use std::path::PathBuf;
use std::sync::mpsc::{self, Sender};

use super::resident::ResidentConsultation;
use djinn::embedded_reservoir::{ApplicationReservoirOptions, ReservoirAttachment};
use graphshell_endpoint::door::app_admission::{AppId, configured_app_endpoint};
use mere_view::{MereViewModel, MereViewRequest};
use std::time::Duration;

#[cfg(feature = "analytic-ephemeris")]
use crate::{AnalyticEphemerisAdapter, calculate_with_adapter};
use crate::{
    Consultation, ConsultationCatalog, ConsultationDetail, ConsultationError, ReceiptComparison,
    SelectionMode,
};

use super::{ConsultationAction, ConsultationContext, ConsultationLayout};

/// A result crossing from the persistence lane back to the native host.
#[derive(Clone, Debug)]
pub(crate) enum WorkerUpdate {
    GrantedContext {
        cards: Vec<crate::granted_ambient::GrantedCard>,
        notice: String,
        complete_request: bool,
    },
    Neighborhood {
        catalog: ConsultationCatalog,
        detail: ConsultationDetail,
        context: crate::ambient::AmbientContext,
    },
    Unavailable {
        message: String,
    },
    GraphSelection {
        catalog: ConsultationCatalog,
        detail: ConsultationDetail,
        position: Option<String>,
    },
    Mere {
        model: MereViewModel,
        catalog: ConsultationCatalog,
    },
    Catalog(ConsultationCatalog),
    SavedNotice(String),
    #[cfg(feature = "analytic-ephemeris")]
    AstrologyChart {
        catalog: ConsultationCatalog,
        facts_digest: String,
    },
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
    commands: Sender<WorkerCommand>,
}

enum WorkerCommand {
    Product(ConsultationAction),
    Mere(MereViewRequest),
}

impl ConsultationWorker {
    pub(crate) fn mere(&self, request: MereViewRequest) -> Result<(), String> {
        self.commands
            .send(WorkerCommand::Mere(request))
            .map_err(|_| "the resident consultation worker is no longer running".into())
    }

    pub(crate) fn command(&self, command: ConsultationAction) -> Result<(), String> {
        self.commands
            .send(WorkerCommand::Product(command))
            .map_err(|_| "the consultation worker is no longer running".to_string())
    }
}

/// Start the resident client worker and publish its picker/history state.
pub(crate) fn spawn(deliver: impl Fn(WorkerUpdate) + Send + 'static) -> ConsultationWorker {
    let (commands, receiver) = mpsc::channel();
    std::thread::Builder::new()
        .name("cleromancy-resident-client".to_string())
        .spawn(move || run(receiver, deliver))
        .expect("start Cleromancy resident client worker");
    ConsultationWorker { commands }
}

fn run(commands: mpsc::Receiver<WorkerCommand>, deliver: impl Fn(WorkerUpdate)) {
    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(error) => {
            deliver(WorkerUpdate::Unavailable {
                message: error.to_string(),
            });
            return;
        },
    };
    let opened = runtime.block_on(async {
        let mut prompt = djinn::startup_vault::NativeOrTerminal {
            native: djinn::identity_ui::SystemNativeIdentityUi::default(),
        };
        let options = ApplicationReservoirOptions {
            endpoint: configured_app_endpoint(),
            application: AppId::new("cleromancy"),
            domain: pandect::DomainId::new("divination").map_err(|e| e.to_string())?,
            shared_root: pandect::shared_root::shared_root(),
            vault_dir: std::env::var_os("CLEROMANCY_VAULT_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(personae::default_vault_dir),
            persona: None,
            profile: None,
            session_duration: Duration::from_secs(24 * 60 * 60),
        };
        let attachment =
            ReservoirAttachment::open_application(options, &mut prompt, |routes| async {
                Ok(routes.with_divination_authority())
            })
            .await
            .map_err(|e| e.to_string())?;
        ResidentConsultation::attach(attachment).await
    });
    let mut client = match opened {
        Ok(client) => client,
        Err(message) => {
            deliver(WorkerUpdate::Unavailable { message });
            return;
        },
    };
    let publish = |client: &ResidentConsultation| -> Result<(), String> {
        let catalog = client
            .consultation()?
            .catalog()
            .map_err(|e| e.to_string())?;
        deliver(WorkerUpdate::Mere {
            model: client.model.clone(),
            catalog: catalog.clone(),
        });
        if let Some((detail, position)) = client.restored_selection() {
            deliver(WorkerUpdate::GraphSelection {
                catalog,
                detail,
                position,
            });
        }
        Ok(())
    };
    if let Err(message) = publish(&client) {
        deliver(WorkerUpdate::Unavailable { message });
        let _ = runtime.block_on(client.close());
        return;
    }
    loop {
        match commands.recv_timeout(Duration::from_millis(250)) {
            Ok(WorkerCommand::Product(action)) => match runtime.block_on(client.action(action)) {
                Ok(update) => {
                    if let Err(message) = publish(&client) {
                        deliver(WorkerUpdate::Unavailable { message });
                        break;
                    }
                    deliver(update);
                    if let Some(message) = client.saved_notice.take() {
                        deliver(WorkerUpdate::SavedNotice(message));
                    }
                },
                Err(message) => {
                    if client.unavailable {
                        deliver(WorkerUpdate::Unavailable { message });
                        break;
                    }
                    deliver(WorkerUpdate::Error {
                        catalog: client.consultation().ok().and_then(|c| c.catalog().ok()),
                        message,
                    });
                },
            },
            Ok(WorkerCommand::Mere(MereViewRequest::Activate(key))) => {
                match runtime.block_on(client.select(&key)) {
                    Ok((detail, position)) => match client
                        .consultation()
                        .and_then(|c| c.catalog().map_err(|e| e.to_string()))
                    {
                        Ok(catalog) => deliver(WorkerUpdate::GraphSelection {
                            catalog,
                            detail,
                            position,
                        }),
                        Err(message) => deliver(WorkerUpdate::Error {
                            catalog: None,
                            message,
                        }),
                    },
                    Err(message) => {
                        if client.unavailable {
                            deliver(WorkerUpdate::Unavailable { message });
                            break;
                        }
                        deliver(WorkerUpdate::Error {
                            catalog: None,
                            message,
                        });
                    },
                }
            },
            Ok(WorkerCommand::Mere(request)) => {
                if let Err(message) = runtime.block_on(client.request(request)) {
                    if client.unavailable {
                        deliver(WorkerUpdate::Unavailable { message });
                        break;
                    }
                    client.model.notice = Some(message);
                }
                if let Err(message) = publish(&client) {
                    deliver(WorkerUpdate::Unavailable { message });
                    break;
                }
            },
            Err(mpsc::RecvTimeoutError::Timeout) => {
                // Poll the existing admitted connection for external changes;
                // the route's own notices include graph, policy and lifecycle.
                if let Some(update) = runtime.block_on(client.poll_granted()) {
                    deliver(update);
                }
                match runtime.block_on(client.take_notice()) {
                    Ok(true) => match runtime.block_on(client.refresh()) {
                        Ok(()) => {
                            if let Err(message) = publish(&client) {
                                deliver(WorkerUpdate::Unavailable { message });
                                break;
                            }
                        },
                        Err(message) => {
                            deliver(WorkerUpdate::Unavailable { message });
                            break;
                        },
                    },
                    Ok(false) => {},
                    Err(message) => {
                        deliver(WorkerUpdate::Unavailable { message });
                        break;
                    },
                }
            },
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
    if let Err(message) = runtime.block_on(client.close()) {
        eprintln!("cleromancy resident close: {message}");
    }
}

pub(crate) fn execute<B: muniment::Backend>(
    consultation: &mut Consultation<B>,
    action: ConsultationAction,
) -> Result<WorkerUpdate, ConsultationError> {
    match action {
        ConsultationAction::OpenNeighborhood {
            session_id,
            position,
        } => neighborhood_update(consultation, &session_id, &position),
        ConsultationAction::SaveNeighborhood { view } => {
            pollster::block_on(consultation.save_ambient_view(&view))?;
            neighborhood_update(consultation, &view.session_id, &view.position)
        },
        ConsultationAction::ExamineNeighbor {
            session_id,
            position,
            item_id,
            keep,
        } => {
            pollster::block_on(consultation.remember_ambient(
                &session_id,
                &position,
                &item_id,
                keep,
            ))?;
            neighborhood_update(consultation, &session_id, &position)
        },
        ConsultationAction::ReadGrantedContext { .. } | ConsultationAction::ClearGrantedContext => {
            unreachable!("handled by the admitted resident client")
        },
        ConsultationAction::InstallTarotDecks => {
            pollster::block_on(consultation.install_typed_tarot())?;
            Ok(WorkerUpdate::Catalog(consultation.catalog()?))
        },
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
                },
            };
            let detail = match layout {
                ConsultationLayout::Single => match (mode, derivation) {
                    (SelectionMode::Derived, Some(selection)) => pollster::block_on(
                        consultation.read_derived(&context_digest, &field_digest, selection),
                    )?,
                    (SelectionMode::Derived, None) => {
                        return Err(ConsultationError::DerivedSelectionRequired);
                    },
                    (_, None) => {
                        pollster::block_on(consultation.read(&context_digest, &field_digest, mode))?
                    },
                    (_, Some(_)) => {
                        return Err(ConsultationError::UnexpectedDerivation);
                    },
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
        },
        ConsultationAction::SaveSpreadTemplate { draft } => {
            pollster::block_on(consultation.save_spread_template(draft))?;
            Ok(WorkerUpdate::Catalog(consultation.catalog()?))
        },
        ConsultationAction::SaveAstrologyChart { draft } => {
            pollster::block_on(consultation.save_astrology_chart(draft))?;
            Ok(WorkerUpdate::Catalog(consultation.catalog()?))
        },
        #[cfg(feature = "analytic-ephemeris")]
        ConsultationAction::CalculateAstrologyChart { draft } => {
            let (moment, orb_millidegrees) = draft.into_moment_and_orb()?;
            let chart = calculate_with_adapter(&AnalyticEphemerisAdapter::new(), &moment)
                .map_err(|error| ConsultationError::Ephemeris(error.to_string()))?;
            let facts_digest = pollster::block_on(
                consultation.save_calculated_astrology_chart(chart, orb_millidegrees),
            )?;
            Ok(WorkerUpdate::AstrologyChart {
                catalog: consultation.catalog()?,
                facts_digest,
            })
        },
        ConsultationAction::SaveReflection { session_id, body } => {
            let detail = pollster::block_on(consultation.reflect(&session_id, body))?;
            let catalog = consultation.catalog()?;
            Ok(WorkerUpdate::Reflection { catalog, detail })
        },
        ConsultationAction::SelectSession { session_id } => {
            let detail = consultation.detail(&session_id)?;
            let catalog = consultation.catalog()?;
            Ok(WorkerUpdate::Session { catalog, detail })
        },
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
        },
    }
}

pub(crate) fn neighborhood_update<B: muniment::Backend>(
    consultation: &Consultation<B>,
    session_id: &str,
    position: &str,
) -> Result<WorkerUpdate, ConsultationError> {
    Ok(WorkerUpdate::Neighborhood {
        catalog: consultation.catalog()?,
        detail: consultation.detail(session_id)?,
        context: consultation.ambient_context(session_id, position)?,
    })
}
