// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Cleromancy's adapter for a Graphshell session already admitted by its host.
//!
//! The surrounding Graphshell session loop retains the delegation chain and
//! checks expiry and revocation. This adapter receives only the projected
//! session name and subject, which it uses to scope Cleromancy's own endpoint
//! and Servitor petitions. It does not authenticate a new caller.

#[cfg(feature = "personal-sync")]
use std::path::Path;
use std::sync::{Arc, Mutex};

use graphshell::lifecycle::{AdmittedEndpointContext, BindAdmittedSession};
use graphshell::native::endpoint_catalog::{ResidentEndpointCatalog, ResidentEndpointCatalogError};
#[cfg(feature = "personal-sync")]
use graphshell::personal_sync::SyncProjection;
use graphshell_endpoint::{
    IntentSink, PresentationSource, ProjectionCatalog, ProjectionNoticeSource, ProjectionSource,
};
use chirograph::{
    CarrierNotice, EndpointDescriptor, IntentInvocation, IntentResult, ProjectionRequest,
    ProjectionSnapshot, ResourceRequest, ResourceResponse,
};
use muniment::Backend;
use servitor::Subject;
#[cfg(feature = "personal-sync")]
use thiserror::Error;

use crate::app::CleromancyAppSessionState;
use crate::{AppError, CleromancyApp, HostError};
#[cfg(feature = "personal-sync")]
use crate::{
    CleromancySyncBatch, CleromancySyncError, CleromancySyncImport, CleromancySyncSelection,
    CleromancySyncSettings, CleromancySyncSettingsError, sync_settings_path,
};

impl<B: Backend> BindAdmittedSession for CleromancyApp<B> {
    fn bind_admitted_session(mut self, context: &AdmittedEndpointContext) -> Self {
        self.host.bind_projection_session(context.session().clone());
        self.bind_intent_subject(Subject::new(context.subject()));
        self
    }
}

/// One durable Cleromancy graph and Servitor authority shared by resident
/// endpoints. Each endpoint receives its own projection state at open time.
pub struct CleromancySessionAuthority<B> {
    app: Arc<Mutex<CleromancyApp<B>>>,
    #[cfg(feature = "personal-sync")]
    sync_selection: Arc<Mutex<CleromancySyncSelection>>,
}

/// A failure while opening Cleromancy's resident graph with its local sync
/// consent applied.
#[cfg(feature = "personal-sync")]
#[derive(Debug, Error)]
pub enum CleromancyResidentOpenError {
    #[error(transparent)]
    Host(#[from] HostError),
    #[error(transparent)]
    SyncSettings(#[from] CleromancySyncSettingsError),
}

impl<B> Clone for CleromancySessionAuthority<B> {
    fn clone(&self) -> Self {
        Self {
            app: Arc::clone(&self.app),
            #[cfg(feature = "personal-sync")]
            sync_selection: Arc::clone(&self.sync_selection),
        }
    }
}

impl<B: Backend + Send + 'static> CleromancySessionAuthority<B> {
    pub fn new(app: CleromancyApp<B>) -> Self {
        Self {
            app: Arc::new(Mutex::new(app)),
            #[cfg(feature = "personal-sync")]
            sync_selection: Arc::new(Mutex::new(CleromancySyncSelection::Off)),
        }
    }

    /// Open Cleromancy's durable graph and apply its local sync consent before
    /// the resident authority becomes available. The data root owns only the
    /// product's consent file; Graphshell identity and transport stay outside
    /// this constructor.
    #[cfg(feature = "personal-sync")]
    pub async fn open_with_local_sync_settings(
        backend: B,
        data_root: &Path,
    ) -> Result<Self, CleromancyResidentOpenError> {
        let settings = CleromancySyncSettings::load(&sync_settings_path(data_root))?;
        let host = crate::CleromancyHost::open(backend).await?;
        let authority = Self::new(CleromancyApp::new(host));
        authority.apply_sync_settings(&settings)?;
        Ok(authority)
    }

    pub fn endpoint(&self, context: &AdmittedEndpointContext) -> CleromancySessionEndpoint<B> {
        CleromancySessionEndpoint {
            authority: self.clone(),
            state: Mutex::new(ResidentSessionState {
                app: CleromancyAppSessionState::admitted(
                    context.session().clone(),
                    Subject::new(context.subject()),
                ),
                last_notified: None,
            }),
        }
    }

    /// Register a factory that gives every already-admitted session a fresh
    /// endpoint over this authority's durable reading graph.
    pub fn register_catalog(
        &self,
        catalog: &mut ResidentEndpointCatalog,
        id: impl Into<String>,
        label: impl Into<String>,
    ) -> Result<(), ResidentEndpointCatalogError> {
        let authority = self.clone();
        catalog.register_notifying(id, label, move |context| Ok(authority.endpoint(context)))
    }

    /// Flush the durable reading graph after one or more resident writes.
    ///
    /// Persistence remains an explicit host policy. Holding the authority
    /// lock across the store operation makes the saved graph one coherent
    /// point in the same mutation order the admitted endpoints observe.
    pub async fn persist(&self, saved_at_secs: u64) -> Result<(), HostError> {
        let mut app = self
            .app
            .lock()
            .expect("Cleromancy resident authority lock poisoned");
        app.host.persist(saved_at_secs).await
    }

    /// Export only the user-selected local truth through Cleromancy's
    /// existing personal-sync mapping. This does not author, sign, or send an
    /// operation.
    #[cfg(feature = "personal-sync")]
    pub fn export_sync_batch(
        &self,
        selection: CleromancySyncSelection,
    ) -> Result<CleromancySyncBatch, CleromancySyncError> {
        let app = self
            .app
            .lock()
            .expect("Cleromancy resident authority lock poisoned");
        crate::sync::export_sync_batch(&app.host, selection)
    }

    /// The selection presently applied to this resident authority. It starts
    /// at `Off`, even when Graphshell's personal-sync transport is available.
    #[cfg(feature = "personal-sync")]
    pub fn sync_selection(&self) -> CleromancySyncSelection {
        *self
            .sync_selection
            .lock()
            .expect("Cleromancy resident sync selection lock poisoned")
    }

    /// Apply product-owned local consent to the live resident authority.
    /// Persistence stays with [`CleromancySyncSettings`].
    #[cfg(feature = "personal-sync")]
    pub fn apply_sync_settings(
        &self,
        settings: &CleromancySyncSettings,
    ) -> Result<(), CleromancySyncSettingsError> {
        settings.validate()?;
        *self
            .sync_selection
            .lock()
            .expect("Cleromancy resident sync selection lock poisoned") = settings.selection;
        Ok(())
    }

    /// Export through the selection currently applied to this authority.
    #[cfg(feature = "personal-sync")]
    pub fn export_selected_sync_batch(&self) -> Result<CleromancySyncBatch, CleromancySyncError> {
        self.export_sync_batch(self.sync_selection())
    }

    /// Materialize selected, already-admitted personal graph truth into this
    /// authority. The existing importer validates the complete projection
    /// before it mutates the local reading graph.
    #[cfg(feature = "personal-sync")]
    pub fn import_sync_projection(
        &self,
        projection: &SyncProjection,
        selection: CleromancySyncSelection,
    ) -> Result<CleromancySyncImport, CleromancySyncError> {
        let mut app = self
            .app
            .lock()
            .expect("Cleromancy resident authority lock poisoned");
        crate::sync::import_sync_projection(&mut app.host, projection, selection)
    }

    /// Materialize a projection through the selection currently applied to
    /// this authority.
    #[cfg(feature = "personal-sync")]
    pub fn import_selected_sync_projection(
        &self,
        projection: &SyncProjection,
    ) -> Result<CleromancySyncImport, CleromancySyncError> {
        self.import_sync_projection(projection, self.sync_selection())
    }
}

/// The session-local endpoint selected by a Graphshell resident catalog.
///
/// Its state owns only projection ephemera and the Servitor subject. The
/// shared authority serializes mutations of Cleromancy's durable graph.
pub struct CleromancySessionEndpoint<B> {
    authority: CleromancySessionAuthority<B>,
    state: Mutex<ResidentSessionState>,
}

struct ResidentSessionState {
    app: CleromancyAppSessionState,
    last_notified: Option<(scenotime::SceneEpoch, scenotime::Revision)>,
}

impl<B: Backend + Send + 'static> CleromancySessionEndpoint<B> {
    fn with_app<R>(&self, operation: impl FnOnce(&mut CleromancyApp<B>) -> R) -> R {
        let mut app = self
            .authority
            .app
            .lock()
            .expect("Cleromancy resident authority lock poisoned");
        let mut state = self
            .state
            .lock()
            .expect("Cleromancy resident endpoint lock poisoned");
        app.with_session_state(&mut state.app, operation)
    }

    fn record_snapshot(&self, revision: Option<(scenotime::SceneEpoch, scenotime::Revision)>) {
        if let Some(revision) = revision {
            self.state
                .lock()
                .expect("Cleromancy resident endpoint lock poisoned")
                .last_notified = Some(revision);
        }
    }
}

impl<B: Backend + Send + 'static> ProjectionCatalog for CleromancySessionEndpoint<B> {
    fn describe(&self) -> EndpointDescriptor {
        self.with_app(|app| app.describe())
    }
}

impl<B: Backend + Send + 'static> ProjectionSource for CleromancySessionEndpoint<B> {
    type Error = AppError;

    fn snapshot(&mut self, request: ProjectionRequest) -> Result<ProjectionSnapshot, Self::Error> {
        let (snapshot, revision) = self.with_app(|app| {
            let snapshot = app.snapshot(request);
            let revision = app.host.last_snapshot_revision();
            (snapshot, revision)
        });
        if snapshot.is_ok() {
            self.record_snapshot(revision);
        }
        snapshot
    }
}

impl<B: Backend + Send + 'static> PresentationSource for CleromancySessionEndpoint<B> {
    type Error = AppError;

    fn resource(&mut self, request: ResourceRequest) -> Result<ResourceResponse, Self::Error> {
        self.with_app(|app| app.resource(request))
    }
}

impl<B: Backend + Send + 'static> IntentSink for CleromancySessionEndpoint<B> {
    type Error = AppError;

    fn invoke(&mut self, intent: IntentInvocation) -> Result<IntentResult, Self::Error> {
        self.with_app(|app| app.invoke(intent))
    }
}

impl<B: Backend + Send + 'static> ProjectionNoticeSource for CleromancySessionEndpoint<B> {
    type Error = AppError;

    fn poll_notice(&mut self) -> Result<Option<CarrierNotice>, Self::Error> {
        let (observed, current, session) = self.with_app(|app| {
            (
                app.host.last_snapshot_revision(),
                app.host.current_projection_revision(),
                app.host.session(),
            )
        });
        let Some(observed) = observed else {
            return Ok(None);
        };
        if current <= observed {
            return Ok(None);
        }

        let mut state = self
            .state
            .lock()
            .expect("Cleromancy resident endpoint lock poisoned");
        if state.last_notified == Some(current) {
            return Ok(None);
        }
        state.last_notified = Some(current);
        Ok(Some(CarrierNotice {
            session,
            epoch: current.0,
            revision: current.1,
        }))
    }
}
