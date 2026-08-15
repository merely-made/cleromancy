// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The composed Graphshell product endpoint: Cleromancy owns readings,
//! Graphshell owns the portable presentation client, and Servitor stays
//! reachable at an explicit authority seam. `invoke` holds intent dispatch.

use graphshell::view::{ProjectionLayoutView, ProjectionReceiptView, render_projection_receipt};
use graphshell_client::{
    ClientState, PresentationResolution, ResolvedPresentation, SnapshotApplyError,
};
use graphshell_endpoint::PresentationSource;
use chirograph::{
    CapabilityProfile, CarrierNotice, IntentInvocation, IntentResult, PresentationCapability,
};
use muniment::Backend;
use servitor::Subject;
use thiserror::Error;

use crate::enrichment::{EnrichmentError, EnrichmentReport, ExternalProjection, mount_carrier};
#[cfg(all(feature = "graphshell-admission", not(target_arch = "wasm32")))]
use crate::host::CleromancyProjectionState;
use crate::intents::IntentLimits;
use crate::moirai::clotho::OsEntropy;
use crate::{CleromancyHost, HostError, ServitorAccess};

mod invoke;

#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    Host(#[from] HostError),
    #[error("Graphshell refused the projection snapshot: {0:?}")]
    Snapshot(SnapshotApplyError),
    #[error("Graphshell client: {0}")]
    Client(String),
    #[error("Cleromancy intent: {0}")]
    Intent(String),
}

/// A small product composition: Cleromancy owns readings, Graphshell owns the
/// portable presentation client, and Servitor stays reachable at an explicit
/// authority seam.
pub struct CleromancyApp<B> {
    pub host: CleromancyHost<B>,
    client: ClientState,
    servitors: ServitorAccess,
    intent_subject: Option<Subject>,
    intent_limits: IntentLimits,
    pending_notice: bool,
}

/// The connection-local half of a [`CleromancyApp`].
///
/// A resident authority owns the reading graph, Servitor table, and intent
/// limits once. Every admitted endpoint carries one of these instead, so its
/// session, action targets, and revision bell cannot overwrite another peer's.
#[cfg(all(feature = "graphshell-admission", not(target_arch = "wasm32")))]
pub(crate) struct CleromancyAppSessionState {
    projection: CleromancyProjectionState,
    intent_subject: Option<Subject>,
    pending_notice: bool,
}

#[cfg(all(feature = "graphshell-admission", not(target_arch = "wasm32")))]
impl CleromancyAppSessionState {
    pub(crate) fn admitted(
        session: chirograph::ProjectionSession,
        subject: Subject,
    ) -> Self {
        Self {
            projection: CleromancyProjectionState::for_session(session),
            intent_subject: Some(subject),
            pending_notice: false,
        }
    }
}

impl<B: Backend> CleromancyApp<B> {
    pub fn new(host: CleromancyHost<B>) -> Self {
        Self {
            host,
            client: ClientState::default(),
            servitors: ServitorAccess::new(),
            intent_subject: None,
            intent_limits: IntentLimits::default(),
            pending_notice: false,
        }
    }

    /// Run one resident endpoint operation with that endpoint's local state.
    ///
    /// The swaps keep the existing local-app API intact while the resident
    /// authority serializes access to the shared graph and Servitor audit.
    #[cfg(all(feature = "graphshell-admission", not(target_arch = "wasm32")))]
    pub(crate) fn with_session_state<R>(
        &mut self,
        state: &mut CleromancyAppSessionState,
        operation: impl FnOnce(&mut Self) -> R,
    ) -> R {
        self.host.swap_projection_state(&mut state.projection);
        std::mem::swap(&mut self.intent_subject, &mut state.intent_subject);
        std::mem::swap(&mut self.pending_notice, &mut state.pending_notice);
        let result = operation(self);
        std::mem::swap(&mut self.intent_subject, &mut state.intent_subject);
        std::mem::swap(&mut self.pending_notice, &mut state.pending_notice);
        self.host.swap_projection_state(&mut state.projection);
        result
    }

    pub fn client(&self) -> &ClientState {
        &self.client
    }

    pub fn servitors(&self) -> &ServitorAccess {
        &self.servitors
    }

    pub fn servitors_mut(&mut self) -> &mut ServitorAccess {
        &mut self.servitors
    }

    /// Bind the peer identity proved by the containing transport. Intent
    /// payloads never get to assert their own subject.
    pub fn bind_intent_subject(&mut self, subject: Subject) {
        self.intent_subject = Some(subject);
    }

    pub fn clear_intent_subject(&mut self) {
        self.intent_subject = None;
    }

    pub fn intent_limits(&self) -> IntentLimits {
        self.intent_limits
    }

    pub fn set_intent_limits(&mut self, limits: IntentLimits) {
        self.intent_limits = limits;
    }

    pub(crate) fn intents_are_bound(&self) -> bool {
        self.intent_subject.is_some()
    }

    pub(crate) fn take_projection_notice(&mut self) -> Option<CarrierNotice> {
        if !std::mem::take(&mut self.pending_notice) {
            return None;
        }
        let (epoch, revision) = self.host.active_revision()?;
        Some(CarrierNotice {
            session: self.host.session(),
            epoch,
            revision,
        })
    }

    pub fn invoke(&mut self, intent: IntentInvocation) -> Result<IntentResult, AppError> {
        self.invoke_with_entropy(intent, &mut OsEntropy)
    }

    /// Mount the local endpoint through the same snapshot/resource split a
    /// remote Graphshell endpoint uses.
    pub fn mount_local(&mut self) -> Result<Vec<ResolvedPresentation>, AppError> {
        let session = self.host.session();
        let snapshot = self
            .host
            .build_snapshot_with_actions(self.intents_are_bound())?;
        let resources = snapshot
            .presentation
            .offers
            .values()
            .flatten()
            .map(|offer| offer.resource)
            .collect::<std::collections::BTreeSet<_>>();
        self.client
            .apply_snapshot(snapshot)
            .map_err(AppError::Snapshot)?;
        for resource in resources {
            let response = self.host.resource(chirograph::ResourceRequest {
                session: session.clone(),
                resource,
            })?;
            self.client
                .apply_resource(response)
                .map_err(|error| AppError::Client(format!("resource: {error:?}")))?;
        }

        let mounted = self
            .client
            .mounted(&session)
            .ok_or_else(|| AppError::Client("mounted scene disappeared".to_string()))?;
        let instances = mounted
            .scene
            .active_items_in_order()
            .into_iter()
            .map(|(instance, _)| instance)
            .collect::<Vec<_>>();
        let profile = CapabilityProfile::new([PresentationCapability::PortableCard]);
        instances
            .into_iter()
            .map(
                |instance| match self.client.resolve(&session, instance, &profile) {
                    Ok(PresentationResolution::Ready(presentation)) => Ok(presentation),
                    Ok(PresentationResolution::NeedsResource(_)) => Err(AppError::Client(
                        "advertised presentation resource was not fetched".to_string(),
                    )),
                    Err(error) => Err(AppError::Client(format!("presentation: {error:?}"))),
                },
            )
            .collect()
    }

    pub fn receipt_html(&mut self) -> Result<String, AppError> {
        let presentations = self.mount_local()?;
        let session = self.host.session();
        let mounted = self
            .client
            .mounted(&session)
            .ok_or_else(|| AppError::Client("mounted scene disappeared".to_string()))?;
        Ok(render_projection_receipt(&ProjectionReceiptView {
            eyebrow: "Cleromancy · local reading graph".to_string(),
            title: "Qualified readings, with their workings".to_string(),
            lede: "The context and calculation remain visible. Externally qualified readings carry the disclosed cards and exact weight additions; cast readings disclose a bounded random sample without claiming that entropy is meaning.".to_string(),
            session: session.0,
            status: format!("{:?}", mounted.status).to_lowercase(),
            presentations,
            layout: Some(ProjectionLayoutView::from_scene(&mounted.scene)),
            intents: Vec::new(),
        }))
    }

    /// Mount a source-owned projection in the same Graphshell client as the
    /// local reading graph. No external node or facet enters `self.host`.
    pub fn mount_external(
        &mut self,
        carrier: &mut impl chirograph::Carrier,
        projection_index: usize,
    ) -> Result<ExternalProjection, EnrichmentError> {
        mount_carrier(&mut self.client, carrier, projection_index)
    }

    pub fn enrichment_receipt_html(
        &self,
        projection: &ExternalProjection,
        report: &EnrichmentReport,
    ) -> Result<String, EnrichmentError> {
        let mounted = self
            .client
            .mounted(&projection.session)
            .ok_or(EnrichmentError::MissingMount)?;
        let matches = report
            .matches
            .iter()
            .map(|matched| {
                let digest = matched.source_digest.chars().take(12).collect::<String>();
                format!(
                    "{}#{} [{}]",
                    matched.presentation,
                    digest,
                    matched.terms.join(", ")
                )
            })
            .collect::<Vec<_>>();
        let correlation = if matches.is_empty() {
            "No lexical overlap was found.".to_string()
        } else {
            format!("Disclosed overlaps: {}.", matches.join("; "))
        };
        Ok(render_projection_receipt(&ProjectionReceiptView {
            eyebrow: "Cleromancy A1 · external Graphshell projection".to_string(),
            title: format!("{} remains source-owned", projection.endpoint_label),
            lede: format!(
                "{} Cleromancy computed {} and did not import the source graph or change reading weights.",
                correlation, report.algorithm
            ),
            session: projection.session.0.clone(),
            status: format!("{:?}", mounted.status).to_lowercase(),
            presentations: projection.presentations.clone(),
            layout: Some(ProjectionLayoutView::from_scene(&mounted.scene)),
            intents: Vec::new(),
        }))
    }
}
