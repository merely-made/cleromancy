// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! One admitted divination client. The resident owns every saved graph and
//! journal. Product actions transform only a complete, revision-bound candidate.

use std::sync::{Arc, Mutex};

use chirograph::{CarrierRequestBody, CarrierResponseBody, IntentResult, ProjectionSnapshot};
use kernel::graph::{CapturedDelta, Graph};
use mere_overmap::{
    materialization::{self, GraphRead, MaterializedMere},
    route as mere_route, wire as session_item,
};
use mere_resident::embedded_reservoir::ReservoirAttachment;
use mere_view::{MereViewModel, MereViewRequest};
use muniment::MemoryBackend;

use super::action::ConsultationAction;
use super::worker::{WorkerUpdate, execute};
use crate::{CleromancyHost, Consultation};

pub(crate) struct ResidentConsultation {
    personal: Option<mere_resident::personal_facts::PersonalFactsClient>,
    granted: Option<crate::granted_ambient::GrantedAmbient>,
    attachment: ReservoirAttachment,
    sessions: ProjectionSnapshot,
    graph_projection: ProjectionSnapshot,
    envelope: Option<session_item::SessionGraphV1>,
    archive: Option<ProjectionSnapshot>,
    transient_view: Option<(pandect::SessionId, incipit::view::ViewIntent)>,
    pub model: MereViewModel,
    pub unavailable: bool,
    pub saved_notice: Option<String>,
}

// The graph commit has already been acknowledged. A recoverable view refusal
// must not turn a saved draw into an error that invites a duplicate retry.
fn postcommit_focus_notice(
    unavailable: bool,
    focus: Result<(), String>,
) -> Result<Option<String>, String> {
    match focus {
        Ok(()) => Ok(None),
        Err(message) if unavailable => Err(message),
        Err(message) => Ok(Some(format!("Reading saved; focus not saved: {message}"))),
    }
}

fn owner_update(
    consultation: &Consultation<MemoryBackend>,
    result: WorkerUpdate,
) -> Result<WorkerUpdate, String> {
    let catalog = consultation.catalog().map_err(err)?;
    // A second admitted writer may have advanced the same graph after our
    // acknowledgement. Re-project current owner truth instead of publishing
    // the candidate captured before that commit.
    let refreshed =
        match result {
            WorkerUpdate::Neighborhood {
                detail, context, ..
            } => Some(
                super::worker::neighborhood_update(
                    consultation,
                    &detail.session.id,
                    &context.data.position,
                )
                .map_err(err)?,
            ),
            WorkerUpdate::Reading { detail, .. } => consultation
                .detail(&detail.session.id)
                .ok()
                .map(|detail| WorkerUpdate::Reading {
                    catalog: catalog.clone(),
                    detail,
                }),
            WorkerUpdate::Reflection { detail, .. } => consultation
                .detail(&detail.session.id)
                .ok()
                .map(|detail| WorkerUpdate::Reflection {
                    catalog: catalog.clone(),
                    detail,
                }),
            WorkerUpdate::Session { detail, .. } => consultation
                .detail(&detail.session.id)
                .ok()
                .map(|detail| WorkerUpdate::Session {
                    catalog: catalog.clone(),
                    detail,
                }),
            WorkerUpdate::Comparison {
                detail, comparison, ..
            } => consultation
                .detail(&detail.session.id)
                .ok()
                .and_then(|detail| {
                    consultation
                        .compare_receipts(&comparison.left_session_id, &comparison.right_session_id)
                        .ok()
                        .map(|comparison| WorkerUpdate::Comparison {
                            catalog: catalog.clone(),
                            detail,
                            comparison,
                        })
                }),
            WorkerUpdate::Die { field_digest, .. } => Some(WorkerUpdate::Die {
                catalog: catalog.clone(),
                field_digest,
            }),
            #[cfg(feature = "analytic-ephemeris")]
            WorkerUpdate::AstrologyChart { facts_digest, .. } => {
                Some(WorkerUpdate::AstrologyChart {
                    catalog: catalog.clone(),
                    facts_digest,
                })
            },
            _ => None,
        };
    Ok(refreshed.unwrap_or(WorkerUpdate::Catalog(catalog)))
}

fn restore_scoped_layout(
    model: &mut MereViewModel,
    envelope: Option<&session_item::SessionGraphV1>,
) {
    // The staged graph belongs to the newly attached session. A missing own
    // view must restore the shared default, never retain the previous session's
    // layout from the cloned model or borrow another application's view.
    model.layout = envelope
        .and_then(|envelope| envelope.views.get("reading-workspace"))
        .and_then(|view| view.strategy.clone())
        .unwrap_or_default();
}

fn newly_minted_session(
    before: &[pandect::SessionId],
    model: &MereViewModel,
) -> Result<pandect::SessionId, String> {
    let mut added = model
        .sessions
        .iter()
        .filter(|session| !session.trashed && !before.contains(&session.id));
    let id = added
        .next()
        .ok_or("resident did not disclose the minted session")?
        .id;
    if added.next().is_some() {
        return Err("sessions changed concurrently; choose the new session explicitly before initializing it".into());
    }
    Ok(id)
}

// Only publish shared transport material after product replay validation.
fn materialized_model(
    reading: &MaterializedMere,
    mut model: MereViewModel,
) -> Result<MereViewModel, String> {
    reading.fill_model(&mut model);
    if let Some(envelope) = &reading.graph_material {
        let attached = model
            .sessions
            .iter()
            .find(|session| session.attached)
            .ok_or("graph envelope has no attached session")?;
        verify_envelope(
            envelope,
            attached.id,
            reading.graph.scene.epoch,
            reading.graph.scene.revision,
        )?;
    }
    restore_scoped_layout(&mut model, reading.graph_material.as_ref());
    Ok(model)
}

fn verify_envelope(
    envelope: &session_item::SessionGraphV1,
    session: pandect::SessionId,
    epoch: scenotime::SceneEpoch,
    revision: scenotime::Revision,
) -> Result<(), String> {
    materialization::validate_graph_material(envelope, "cleromancy", session, epoch, revision)?;
    crate::host::validate_divination_graph(&envelope.clone().into_graph()?).map_err(err)
}

fn err(error: impl std::fmt::Display) -> String {
    error.to_string()
}

#[cfg(test)]
mod tests;

fn natal_nonce() -> String {
    use rand_core::RngCore;
    let mut bytes = [0u8; 16];
    rand_core::OsRng.fill_bytes(&mut bytes);
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

mod operations;
