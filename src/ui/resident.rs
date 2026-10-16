// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! One admitted divination client. The resident owns every saved graph and
//! journal. Product actions transform only a complete, revision-bound candidate.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use chirograph::{
    CarrierRequestBody, CarrierResponseBody, IntentResult, PortableCardV1, ProjectionSnapshot,
};
use djinn::embedded_reservoir::ReservoirAttachment;
use mere::kernel::graph::{CapturedDelta, Graph};
use mere_overmap::{route as mere_route, wire as session_item};
use mere_view::{MereViewModel, MereViewRequest};
use muniment::MemoryBackend;

use super::action::ConsultationAction;
use super::worker::{WorkerUpdate, execute};
use crate::{CleromancyHost, Consultation};

pub(crate) struct ResidentConsultation {
    granted: Option<crate::granted_ambient::GrantedAmbient>,
    attachment: ReservoirAttachment,
    sessions: ProjectionSnapshot,
    graph_projection: ProjectionSnapshot,
    envelope: Option<session_item::SessionGraphV1>,
    archive: Option<ProjectionSnapshot>,
    transient_view: Option<(pandect::SessionId, pandect::ViewIntent)>,
    pub model: MereViewModel,
    pub unavailable: bool,
    pub saved_notice: Option<String>,
}

impl ResidentConsultation {
    pub async fn attach(mut attachment: ReservoirAttachment) -> Result<Self, String> {
        let opened = attachment.client_mut().open_session().await.map_err(err)?;
        let (sessions, graph) =
            mere_route::requests(&opened.descriptor).ok_or("resident has no mere projections")?;
        let sessions = attachment
            .client_mut()
            .snapshot(sessions)
            .await
            .map_err(err)?;
        let graph_projection = attachment.client_mut().snapshot(graph).await.map_err(err)?;
        let mut client = Self {
            granted: None,
            attachment,
            sessions,
            graph_projection,
            envelope: None,
            archive: None,
            transient_view: None,
            model: MereViewModel::default(),
            unavailable: false,
            saved_notice: None,
        };
        client.refresh().await?;
        if !client.model.sessions.iter().any(|session| session.attached) {
            client.request(MereViewRequest::Mint).await?;
        }
        if client
            .envelope
            .as_ref()
            .is_some_and(|envelope| !envelope.read_only)
            && client.consultation()?.host().is_empty()
        {
            let (mut consultation, edits) = client.candidate()?;
            consultation
                .install_tarot_deck("cleromancy.tarot/rws-complete-reflective-v1")
                .await
                .map_err(err)?;
            client.commit(edits).await?;
            client.refresh().await?;
        }
        Ok(client)
    }

    pub async fn refresh(&mut self) -> Result<(), String> {
        self.unavailable = true;
        // A failed read clears transient material too. Publish it again only
        // after the complete graph proves the same read-only attachment.
        let transient_view = self.transient_view.take();
        let opened = self
            .attachment
            .client_mut()
            .open_session()
            .await
            .map_err(err)?;
        let (sessions, graph) =
            mere_route::requests(&opened.descriptor).ok_or("resident has no mere projections")?;
        let sessions = self
            .attachment
            .client_mut()
            .snapshot(sessions)
            .await
            .map_err(err)?;
        let graph_projection = self
            .attachment
            .client_mut()
            .snapshot(graph)
            .await
            .map_err(err)?;
        let mut cards = HashMap::new();
        for request in mere_route::card_requests(&sessions) {
            let bytes = self
                .attachment
                .client_mut()
                .resource(request.session, request.resource)
                .await
                .map_err(err)?;
            verify_resource(request.resource, &bytes)?;
            cards.insert(request.resource, bytes);
        }
        let mut model = self.model.clone();
        mere_route::fill(&mut model, &sessions, &graph_projection, &cards);
        let mut archive_projection = None;
        if let Some(request) = mere_route::archive_request(&opened.descriptor) {
            let archive = self
                .attachment
                .client_mut()
                .snapshot(request)
                .await
                .map_err(err)?;
            let mut archive_cards = HashMap::new();
            for request in mere_route::card_requests(&archive) {
                let bytes = self
                    .attachment
                    .client_mut()
                    .resource(request.session, request.resource)
                    .await
                    .map_err(err)?;
                verify_resource(request.resource, &bytes)?;
                archive_cards.insert(request.resource, bytes);
            }
            mere_route::fill_archive(&mut model, &sessions, &archive, &archive_cards);
            archive_projection = Some(archive);
        }
        let mut staged_envelope = None;
        for request in mere_route::card_requests(&graph_projection) {
            let bytes = self
                .attachment
                .client_mut()
                .resource(request.session.clone(), request.resource)
                .await
                .map_err(err)?;
            verify_resource(request.resource, &bytes)?;
            if let Ok(card) = serde_json::from_slice::<PortableCardV1>(&bytes) {
                if let Some(media) = card.media.first() {
                    let bytes = self
                        .attachment
                        .client_mut()
                        .resource(request.session, *media)
                        .await
                        .map_err(err)?;
                    verify_resource(*media, &bytes)?;
                    if let Ok(envelope) =
                        serde_json::from_slice::<session_item::SessionGraphV1>(&bytes)
                    {
                        let attached = model
                            .sessions
                            .iter()
                            .find(|session| session.attached)
                            .ok_or("graph envelope has no attached session")?;
                        verify_envelope(
                            &envelope,
                            attached.id,
                            graph_projection.scene.epoch,
                            graph_projection.scene.revision,
                        )?;
                        staged_envelope = Some(envelope);
                        break;
                    }
                }
            }
        }
        if model.sessions.iter().any(|session| session.attached) && staged_envelope.is_none() {
            return Err("resident did not disclose its complete attached graph".into());
        }
        restore_scoped_layout(&mut model, staged_envelope.as_ref());
        self.transient_view = transient_view.filter(|(id, _)| {
            staged_envelope
                .as_ref()
                .is_some_and(|envelope| envelope.read_only && envelope.session == *id)
        });
        if let Some((_, view)) = &self.transient_view {
            model.layout = view.strategy.clone().unwrap_or_default();
        }
        self.sessions = sessions;
        self.graph_projection = graph_projection;
        self.model = model;
        self.archive = archive_projection;
        self.envelope = staged_envelope;
        self.unavailable = false;
        Ok(())
    }

    pub fn consultation(&self) -> Result<Consultation<MemoryBackend>, String> {
        let graph = match &self.envelope {
            Some(envelope) => envelope.clone().into_graph()?,
            None => Graph::new(),
        };
        Ok(Consultation::new(
            CleromancyHost::candidate(graph).map_err(err)?,
        ))
    }

    fn candidate(
        &self,
    ) -> Result<(Consultation<MemoryBackend>, Arc<Mutex<Vec<CapturedDelta>>>), String> {
        let mut consultation = self.consultation()?;
        let edits = Arc::new(Mutex::new(Vec::new()));
        consultation
            .host_mut_candidate()
            .record_edits(edits.clone());
        Ok((consultation, edits))
    }

    pub async fn action(&mut self, action: ConsultationAction) -> Result<WorkerUpdate, String> {
        self.saved_notice = None;
        match &action {
            ConsultationAction::ClearGrantedContext => {
                if let Some(reader) = self.granted.take() {
                    let _ = reader.close().await;
                }
                return Ok(WorkerUpdate::GrantedContext {
                    cards: Vec::new(),
                    notice: "Neighboring context cleared.".into(),
                    complete_request: true,
                });
            },
            ConsultationAction::ReadGrantedContext { domain } => {
                if let Some(reader) = self.granted.take() {
                    let _ = reader.close().await;
                }
                let opened = tokio::time::timeout(
                    std::time::Duration::from_secs(2),
                    crate::granted_ambient::GrantedAmbient::open_at(
                        &graphshell_endpoint::door::app_admission::configured_app_endpoint(),
                        domain,
                    ),
                )
                .await
                .unwrap_or_else(|_| Err("ambient source connection timed out".into()));
                match opened {
                    Ok(reader) => {
                        self.granted = Some(reader);
                        let mut update =
                            self.poll_granted()
                                .await
                                .unwrap_or(WorkerUpdate::GrantedContext {
                                    cards: Vec::new(),
                                    notice: "No granted context is available.".into(),
                                    complete_request: true,
                                });
                        if let WorkerUpdate::GrantedContext {
                            complete_request, ..
                        } = &mut update
                        {
                            *complete_request = true;
                        }
                        return Ok(update);
                    },
                    Err(error) => {
                        return Ok(WorkerUpdate::GrantedContext {
                            cards: Vec::new(),
                            notice: format!("No ambient grant or readable source: {error}"),
                            complete_request: true,
                        });
                    },
                }
            },
            _ => {},
        }
        self.refresh().await?;
        let (mut consultation, edits) = self.candidate()?;
        let update = execute(&mut consultation, action).map_err(err)?;
        self.commit(edits).await?;
        self.refresh().await?;
        let selected = match &update {
            WorkerUpdate::Neighborhood { detail, .. }
            | WorkerUpdate::Reading { detail, .. }
            | WorkerUpdate::Reflection { detail, .. }
            | WorkerUpdate::Session { detail, .. }
            | WorkerUpdate::Comparison { detail, .. } => {
                Some(format!("cleromancy://session/{}", detail.session.id))
            },
            _ => None,
        };
        if let Some(url) = selected {
            let same_occasion = self.restored_selection().is_some_and(|(detail, _)| {
                url == format!("cleromancy://session/{}", detail.session.id)
            });
            if !same_occasion {
                let focus = self.remember_focus(Some(url)).await;
                if let Some(notice) = postcommit_focus_notice(self.unavailable, focus)? {
                    self.saved_notice = Some(notice.clone());
                    self.model.notice = Some(notice);
                }
            }
        }
        owner_update(&self.consultation()?, update).map_err(|message| {
            self.unavailable = true;
            message
        })
    }

    async fn commit(&mut self, edits: Arc<Mutex<Vec<CapturedDelta>>>) -> Result<(), String> {
        let edits = std::mem::take(&mut *edits.lock().map_err(err)?);
        if edits.is_empty() {
            return Ok(());
        }
        let envelope = self
            .envelope
            .as_ref()
            .ok_or("no writable resident session is attached")?;
        let intent = chirograph::IntentInvocation {
            session: self.graph_projection.session.clone(),
            target: sceno::InstanceId(0),
            observed_epoch: self.graph_projection.scene.epoch,
            observed_revision: self.graph_projection.scene.revision,
            intent: session_item::APPLY_EDITS_INTENT.into(),
            payload: serde_json::to_vec(&session_item::ApplyEditsV1::checked(
                envelope.revision,
                edits,
            ))
            .map_err(err)?,
        };
        self.invoke(intent).await
    }

    async fn invoke(&mut self, intent: chirograph::IntentInvocation) -> Result<(), String> {
        let response = match self
            .attachment
            .client_mut()
            .request_body(CarrierRequestBody::Intent(intent))
            .await
        {
            Ok(response) => response,
            Err(error) => {
                self.unavailable = true;
                self.discard_transient_view();
                return Err(err(error));
            },
        };
        match response {
            CarrierResponseBody::Intent(IntentResult::Accepted) => Ok(()),
            CarrierResponseBody::Intent(result) => {
                self.discard_transient_view();
                Err(mere_route::declined(&result)
                    .unwrap_or_else(|| "resident refused the action".into()))
            },
            _ => {
                self.unavailable = true;
                self.discard_transient_view();
                Err("resident returned an unexpected action response".into())
            },
        }
    }

    fn discard_transient_view(&mut self) {
        self.transient_view = None;
        restore_scoped_layout(&mut self.model, self.envelope.as_ref());
    }

    pub async fn request(&mut self, request: MereViewRequest) -> Result<(), String> {
        let minted = matches!(&request, MereViewRequest::Mint);
        let before_sessions: Vec<_> = self
            .model
            .sessions
            .iter()
            .map(|session| session.id)
            .collect();
        let intent = mere_route::intent(&request, &self.sessions).or_else(|| {
            self.archive
                .as_ref()
                .and_then(|archive| mere_route::archive_intent(&request, &self.sessions, archive))
        });
        if let Some(intent) = intent {
            self.invoke(intent).await?;
        } else if let MereViewRequest::Layout(layout) = request {
            let mut view = self.reading_view();
            view.strategy = Some(layout);
            self.set_view(view).await?;
        } else {
            return Err("this view action is not available".into());
        }
        self.refresh().await?;
        // Only an explicit successful mint initializes a new session. Browsing
        // an archive or switching an existing session never forks to add data.
        if minted {
            let id = newly_minted_session(&before_sessions, &self.model)?;
            let switch = mere_route::intent(
                &MereViewRequest::Session(id, mere_view::SessionStep::Switch),
                &self.sessions,
            )
            .ok_or("resident did not advertise switching to the minted session")?;
            self.invoke(switch).await?;
            self.refresh().await?;
            let envelope = self
                .envelope
                .as_ref()
                .ok_or("minted session has no complete graph")?;
            if envelope.session != id || envelope.read_only {
                return Err("resident did not attach the new mutable session".into());
            }
            let (mut consultation, edits) = self.candidate()?;
            if !consultation.host().is_empty() {
                return Err("the newly minted session changed before initialization".into());
            }
            consultation
                .install_tarot_deck("cleromancy.tarot/rws-complete-reflective-v1")
                .await
                .map_err(err)?;
            self.commit(edits).await?;
            self.refresh().await?;
        }
        Ok(())
    }

    fn reading_view(&self) -> pandect::ViewIntent {
        if let Some((id, view)) = &self.transient_view
            && self
                .envelope
                .as_ref()
                .is_some_and(|envelope| envelope.read_only && envelope.session == *id)
        {
            return view.clone();
        }
        self.envelope
            .as_ref()
            .and_then(|envelope| envelope.views.get("reading-workspace"))
            .cloned()
            .unwrap_or_default()
    }

    async fn set_view(&mut self, view: pandect::ViewIntent) -> Result<(), String> {
        let envelope = self
            .envelope
            .as_ref()
            .ok_or("no resident graph is attached")?;
        if envelope.read_only {
            // SET_VIEW on a thaw is an explicit first-edit fork at the owner.
            // Reading and arranging it must instead remain session-local.
            self.model.layout = view.strategy.clone().unwrap_or_default();
            self.transient_view = Some((envelope.session, view));
            return Ok(());
        }
        let intent = chirograph::IntentInvocation {
            session: self.graph_projection.session.clone(),
            target: sceno::InstanceId(0),
            observed_epoch: self.graph_projection.scene.epoch,
            observed_revision: self.graph_projection.scene.revision,
            intent: session_item::SET_VIEW_INTENT.into(),
            payload: serde_json::to_vec(&session_item::SetViewV1::new("reading-workspace", view))
                .map_err(err)?,
        };
        self.invoke(intent).await?;
        self.refresh().await
    }

    async fn remember_focus(&mut self, url: Option<String>) -> Result<(), String> {
        let mut view = self.reading_view();
        view.focus = url;
        self.set_view(view).await
    }

    pub fn restored_selection(&self) -> Option<(crate::ConsultationDetail, Option<String>)> {
        self.reading_view()
            .focus
            .and_then(|url| self.selected_detail(&url).ok())
    }

    pub async fn select(
        &mut self,
        key: &str,
    ) -> Result<(crate::ConsultationDetail, Option<String>), String> {
        self.refresh().await?;
        self.selected_detail(key)?;
        let consultation = self.consultation()?;
        let url = consultation
            .host()
            .graph()
            .nodes()
            .find(|(_, node)| node.id.to_string() == key || node.url() == key)
            .map(|(_, node)| node.url().to_string())
            .ok_or("selected node is missing")?;
        self.remember_focus(Some(url)).await?;
        self.selected_detail(key)
    }

    pub fn selected_detail(
        &self,
        key: &str,
    ) -> Result<(crate::ConsultationDetail, Option<String>), String> {
        let consultation = self.consultation()?;
        let host = consultation.host();
        let (node_key, _) = host
            .graph()
            .nodes()
            .find(|(_, node)| node.id.to_string() == key || node.url() == key)
            .ok_or("selected node is no longer in the attached session")?;
        if let Some(value) = host.facet_value(node_key, crate::host::SESSION_FACET) {
            let session: crate::ReadingSession =
                serde_json::from_value(value.clone()).map_err(err)?;
            return consultation
                .detail(&session.id)
                .map(|detail| (detail, None))
                .map_err(err);
        }
        if let Some(value) = host.facet_value(node_key, crate::host::READING_FACET) {
            let reading: crate::Reading = serde_json::from_value(value.clone()).map_err(err)?;
            let session = host
                .sessions()
                .map_err(err)?
                .into_iter()
                .find(|session| {
                    session
                        .placements
                        .iter()
                        .any(|placement| placement.reading_id == reading.id)
                })
                .ok_or("selected reading has no saved occasion")?;
            let position = session
                .placements
                .iter()
                .find(|p| p.reading_id == reading.id)
                .map(|p| p.position.clone());
            return consultation
                .detail(&session.id)
                .map(|detail| (detail, position))
                .map_err(err);
        }
        Err("Select a saved occasion or a card occurrence to open its reading.".into())
    }

    pub async fn take_notice(&mut self) -> Result<bool, String> {
        let notice = self
            .attachment
            .client_mut()
            .take_notice()
            .await
            .map(|notice| notice.is_some());
        match notice {
            Ok(notice) => Ok(notice),
            Err(error) => {
                self.discard_transient_view();
                self.unavailable = true;
                Err(err(error))
            },
        }
    }

    pub async fn poll_granted(&mut self) -> Option<WorkerUpdate> {
        let reader = self.granted.as_mut()?;
        match tokio::time::timeout(std::time::Duration::from_secs(2), reader.refresh()).await {
            Ok(Ok(true)) => Some(WorkerUpdate::GrantedContext {
                cards: reader.cards().to_vec(),
                notice: "Read-only context from the resident’s live ambient grant.".into(),
                complete_request: false,
            }),
            Ok(Ok(false)) => None,
            result => {
                let notice = match result {
                    Ok(Err(error)) => format!("Ambient context withdrawn: {error}"),
                    _ => "Ambient source timed out; previous context was withdrawn.".into(),
                };
                self.granted = None;
                Some(WorkerUpdate::GrantedContext {
                    cards: Vec::new(),
                    notice,
                    complete_request: false,
                })
            },
        }
    }

    pub async fn close(self) -> Result<(), String> {
        self.attachment.close().await.map_err(err)
    }
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

fn verify_envelope(
    envelope: &session_item::SessionGraphV1,
    session: pandect::SessionId,
    epoch: scenotime::SceneEpoch,
    revision: scenotime::Revision,
) -> Result<(), String> {
    if envelope.session != session {
        return Err("graph envelope names another attached session".into());
    }
    if envelope.epoch != epoch || envelope.revision != revision {
        return Err("graph envelope revision differs from its projection".into());
    }
    if envelope.application != "cleromancy" {
        return Err("graph envelope belongs to another application view".into());
    }
    crate::host::validate_divination_graph(&envelope.clone().into_graph()?).map_err(err)
}

fn verify_resource(expected: chirograph::ContentHash, bytes: &[u8]) -> Result<(), String> {
    if chirograph::ContentHash::of(bytes) != expected {
        return Err("resident resource bytes do not match their content hash".into());
    }
    Ok(())
}

fn err(error: impl std::fmt::Display) -> String {
    error.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use scenotime::{Revision, SceneEpoch};

    #[test]
    fn installed_die_survives_owner_refresh_with_authoritative_catalog() {
        let mut c = Consultation::new(CleromancyHost::empty(MemoryBackend::new()));
        let digest = pollster::block_on(c.install_six_sided_die()).unwrap();
        let stale = Consultation::new(CleromancyHost::empty(MemoryBackend::new()))
            .catalog()
            .unwrap();
        match owner_update(
            &c,
            WorkerUpdate::Die {
                catalog: stale,
                field_digest: digest.clone(),
            },
        )
        .unwrap()
        {
            WorkerUpdate::Die {
                catalog,
                field_digest,
            } => {
                assert_eq!(field_digest, digest);
                assert_eq!(catalog.fields.len(), 1);
                assert_eq!(catalog.fields[0].digest(), digest);
            },
            other => panic!("installed field selection was lost: {other:?}"),
        }
    }

    fn envelope() -> session_item::SessionGraphV1 {
        let graph = Graph::new();
        session_item::SessionGraphV1 {
            schema: session_item::SESSION_GRAPH_SCHEMA.into(),
            application: "cleromancy".into(),
            read_only: false,
            views: Default::default(),
            session: serde_json::from_str("\"00000000-0000-0000-0000-000000000001\"").unwrap(),
            epoch: SceneEpoch(1),
            revision: Revision(4),
            graph: graph.to_snapshot(),
            facets: graph.facets().clone(),
        }
    }

    #[test]
    fn session_without_own_view_clears_previous_saved_layout() {
        let mut original = envelope();
        original.views.insert(
            "reading-workspace".into(),
            pandect::ViewIntent {
                strategy: Some("grid.default".into()),
                ..Default::default()
            },
        );
        let mut model = MereViewModel::default();
        restore_scoped_layout(&mut model, Some(&original));
        assert_eq!(model.layout, "grid.default");

        let mut next = envelope();
        next.session = serde_json::from_str("\"00000000-0000-0000-0000-000000000002\"").unwrap();
        next.views.insert(
            "graphshell.mere".into(),
            pandect::ViewIntent {
                strategy: Some("spiral.default".into()),
                ..Default::default()
            },
        );
        restore_scoped_layout(&mut model, Some(&next));
        assert_eq!(model.layout, MereViewModel::default().layout);

        // Returning to the original session restores its own saved choice.
        restore_scoped_layout(&mut model, Some(&original));
        assert_eq!(model.layout, "grid.default");
        next.views
            .insert("reading-workspace".into(), pandect::ViewIntent::default());
        restore_scoped_layout(&mut model, Some(&next));
        assert_eq!(model.layout, MereViewModel::default().layout);
        restore_scoped_layout(&mut model, Some(&original));
        restore_scoped_layout(&mut model, None);
        assert_eq!(model.layout, MereViewModel::default().layout);
    }

    #[cfg(unix)]
    #[test]
    fn readonly_archive_layout_selection_and_comparison_never_fork_or_write() {
        use super::super::action::{ConsultationContext, ConsultationLayout};
        use castellan::authority::PersonaeHost;
        use castellan::custody::{IdentityVault, InMemoryStorage, Profile};
        use djinn::embedded_reservoir::ReservoirAttachmentOptions;
        use graphshell_endpoint::door::app_admission::AppId;
        use graphshell_endpoint::door::identity::VaultProtectionView;
        use mere_view::{ArchiveRequest, SessionStep};
        use personae::{Ed25519Keypair, PersonaId, ProfileId};

        fn bytes(
            root: &std::path::Path,
        ) -> std::collections::BTreeMap<std::path::PathBuf, Vec<u8>> {
            fn visit(
                root: &std::path::Path,
                dir: &std::path::Path,
                result: &mut std::collections::BTreeMap<std::path::PathBuf, Vec<u8>>,
            ) {
                for entry in std::fs::read_dir(dir).unwrap() {
                    let path = entry.unwrap().path();
                    if path.is_dir() {
                        visit(root, &path, result);
                    } else {
                        result.insert(
                            path.strip_prefix(root).unwrap().into(),
                            std::fs::read(path).unwrap(),
                        );
                    }
                }
            }
            let mut result = std::collections::BTreeMap::new();
            visit(root, root, &mut result);
            result
        }

        let root = tempfile::tempdir().unwrap();
        let address = format!(
            "/tmp/c1-readonly-{}.sock",
            root.path().file_name().unwrap().to_string_lossy()
        );
        let persona = PersonaId::from_uuid("00000000-0000-0000-0000-00c120261014".parse().unwrap());
        let identity = Arc::new(PersonaeHost::new(
            IdentityVault::with_profile(
                InMemoryStorage::new(),
                Profile::new(
                    ProfileId("c1-public-readonly-fixture".into()),
                    "C1 public fixture",
                    Ed25519Keypair::from_seed([0xc1; 32]),
                ),
            ),
            None,
            VaultProtectionView::Ephemeral,
        ));
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(async {
            let attachment = ReservoirAttachment::open(
                ReservoirAttachmentOptions {
                    endpoint: address,
                    application: AppId::new("cleromancy"),
                    applications: vec![AppId::new("cleromancy")],
                    shared_root: root.path().into(),
                    persona,
                    domain: pandect::DomainId::new("divination").unwrap(),
                    session_duration: std::time::Duration::from_secs(60),
                },
                || async { Ok(identity) },
            )
            .await
            .unwrap();
            let mut client = ResidentConsultation::attach(attachment).await.unwrap();
            let original = client.envelope.as_ref().unwrap().session;
            let field = client.consultation().unwrap().catalog().unwrap().fields[0].digest();
            let mut occasions = Vec::new();
            for question in ["What is retained?", "What remains transient?"] {
                let update = client
                    .action(ConsultationAction::Read {
                        context: ConsultationContext::New(crate::ContextDraft::new(
                            "C1 archive reading",
                            question,
                            "ownership",
                        )),
                        field_digest: field.clone(),
                        mode: crate::SelectionMode::Calculated,
                        derivation: None,
                        layout: ConsultationLayout::Single,
                        astrology_facts_digest: None,
                        situation: Default::default(),
                    })
                    .await
                    .unwrap();
                let WorkerUpdate::Reading { detail, .. } = update else {
                    panic!("expected reading")
                };
                occasions.push(detail.session.id);
            }
            client
                .request(MereViewRequest::Archive(ArchiveRequest::Save(original)))
                .await
                .unwrap();
            let archive = client.model.archive.as_ref().unwrap().entries[0]
                .key
                .clone();
            client
                .request(MereViewRequest::Archive(ArchiveRequest::Open(archive)))
                .await
                .unwrap();
            let thaw = client.envelope.as_ref().unwrap().session;
            assert!(client.envelope.as_ref().unwrap().read_only);
            let count = client.model.sessions.len();
            let before = bytes(root.path());
            let original_view = client.envelope.as_ref().unwrap().views.clone();

            client
                .request(MereViewRequest::Layout("grid.default".into()))
                .await
                .unwrap();
            assert_eq!(client.model.layout, "grid.default");
            client
                .action(ConsultationAction::SelectSession {
                    session_id: occasions[0].clone(),
                })
                .await
                .unwrap();
            assert_eq!(
                client.restored_selection().unwrap().0.session.id,
                occasions[0]
            );
            client
                .action(ConsultationAction::CompareSessions {
                    left_session_id: occasions[1].clone(),
                    right_session_id: occasions[0].clone(),
                })
                .await
                .unwrap();
            client
                .select(&format!("cleromancy://session/{}", occasions[0]))
                .await
                .unwrap();
            client.refresh().await.unwrap();
            assert_eq!(client.model.layout, "grid.default");
            assert_eq!(client.envelope.as_ref().unwrap().session, thaw);
            assert!(client.envelope.as_ref().unwrap().read_only);
            assert_eq!(client.envelope.as_ref().unwrap().views, original_view);
            assert_eq!(client.model.sessions.len(), count);
            assert_eq!(
                bytes(root.path()),
                before,
                "read-only presentation must send no owner write"
            );

            // A real owner refusal must discard the transient focus and
            // layout together, without accepting any first-edit fork.
            let reading = client
                .consultation()
                .unwrap()
                .detail(&occasions[0])
                .unwrap()
                .readings[0]
                .clone();
            let node = Graph::node_namespace_id(&format!("cleromancy://reading/{}", reading.id));
            let mut damaged = serde_json::to_value(reading).unwrap();
            damaged["candidate_id"] = serde_json::json!("invented-readonly-candidate");
            let rejected = chirograph::IntentInvocation {
                session: client.graph_projection.session.clone(),
                target: sceno::InstanceId(0),
                observed_epoch: client.graph_projection.scene.epoch,
                observed_revision: client.graph_projection.scene.revision,
                intent: session_item::APPLY_EDITS_INTENT.into(),
                payload: serde_json::to_vec(&session_item::ApplyEditsV1::checked(
                    client.envelope.as_ref().unwrap().revision,
                    vec![CapturedDelta::ReplaySetNodeFacetById {
                        node_id: node.to_string(),
                        facet: crate::host::READING_FACET.into(),
                        value_json: damaged.to_string(),
                    }],
                ))
                .unwrap(),
            };
            assert!(client.invoke(rejected).await.is_err());
            assert!(
                !client.unavailable,
                "the route remains readable after domain refusal"
            );
            assert!(client.transient_view.is_none());
            assert_eq!(
                client.model.layout,
                original_view
                    .get("reading-workspace")
                    .and_then(|view| view.strategy.clone())
                    .unwrap_or_default()
            );
            client.refresh().await.unwrap();
            assert_eq!(client.envelope.as_ref().unwrap().session, thaw);
            assert!(client.envelope.as_ref().unwrap().read_only);
            assert_eq!(client.model.sessions.len(), count);
            assert_eq!(
                bytes(root.path()),
                before,
                "failed first edit changes no archive session state"
            );

            client
                .request(MereViewRequest::Session(original, SessionStep::Switch))
                .await
                .unwrap();
            assert!(
                client.transient_view.is_none(),
                "a view cannot cross attachments"
            );
            assert!(!client.envelope.as_ref().unwrap().read_only);
            client
                .request(MereViewRequest::Layout("phyllotaxis.default".into()))
                .await
                .unwrap();
            assert_eq!(
                client.envelope.as_ref().unwrap().views["reading-workspace"]
                    .strategy
                    .as_deref(),
                Some("phyllotaxis.default")
            );
            assert_eq!(client.model.sessions.len(), count);
            assert_ne!(
                bytes(root.path()),
                before,
                "an admitted writable layout is persisted"
            );
            client.close().await.unwrap();
        });
    }

    #[test]
    fn acknowledged_reading_survives_view_refusal_but_unavailable_still_purges() {
        let refused = postcommit_focus_notice(false, Err("stale view revision".into())).unwrap();
        assert_eq!(
            refused.as_deref(),
            Some("Reading saved; focus not saved: stale view revision")
        );
        assert_eq!(postcommit_focus_notice(false, Ok(())).unwrap(), None);
        assert_eq!(
            postcommit_focus_notice(true, Err("access revoked".into())),
            Err("access revoked".into())
        );
    }

    #[test]
    fn complete_graph_must_match_the_attached_projection_before_cache_publication() {
        let prior = envelope();
        let before = serde_json::to_vec(&prior).unwrap();
        verify_envelope(&prior, prior.session, prior.epoch, prior.revision).unwrap();
        for (session, epoch, revision) in [
            (
                serde_json::from_str("\"00000000-0000-0000-0000-000000000002\"").unwrap(),
                prior.epoch,
                prior.revision,
            ),
            (prior.session, SceneEpoch(2), prior.revision),
            (prior.session, prior.epoch, Revision(5)),
        ] {
            assert!(verify_envelope(&prior, session, epoch, revision).is_err());
            assert_eq!(serde_json::to_vec(&prior).unwrap(), before);
        }
        let mut other_app = prior.clone();
        other_app.application = "graphshell".into();
        assert!(verify_envelope(&other_app, prior.session, prior.epoch, prior.revision).is_err());
        let mut future = prior.clone();
        future.schema = "mere.session.graph/v9".into();
        assert!(verify_envelope(&future, prior.session, prior.epoch, prior.revision).is_err());
    }

    #[test]
    fn acknowledged_result_uses_current_owner_records_instead_of_precommit_candidate() {
        let mut owner = Consultation::new(CleromancyHost::candidate(Graph::new()).unwrap());
        let field = pollster::block_on(owner.install_builtin_tarot_at(1)).unwrap();
        let context = pollster::block_on(owner.save_context_at(
            crate::ContextDraft::new("A boundary", "What remains visible?", "ownership"),
            2,
        ))
        .unwrap();
        let detail =
            pollster::block_on(owner.read(&context, &field, crate::SelectionMode::Calculated))
                .unwrap();
        let id = detail.session.id.clone();
        let candidate = WorkerUpdate::Reading {
            catalog: owner.catalog().unwrap(),
            detail,
        };
        // A peer advances the same owner graph between commit and UI delivery.
        pollster::block_on(owner.reflect(&id, "A concurrent peer reflection".into())).unwrap();
        match owner_update(&owner, candidate).unwrap() {
            WorkerUpdate::Reading { detail, catalog } => {
                assert_eq!(detail.reflections.len(), 1);
                assert_eq!(detail.reflections[0].body, "A concurrent peer reflection");
                assert_eq!(catalog, owner.catalog().unwrap());
                assert_eq!(detail, owner.detail(&id).unwrap());
            },
            other => panic!("expected current owner reading, got {other:?}"),
        }
    }

    #[test]
    fn mint_switches_only_to_one_new_live_identity_and_refuses_ambiguity() {
        let id = |n: u8| {
            serde_json::from_str(&format!("\"00000000-0000-0000-0000-{:012}\"", n)).unwrap()
        };
        let row = |n: u8| mere_view::SessionEntry {
            id: id(n),
            name: "Same visible name".into(),
            detail: None,
            attached: false,
            trashed: false,
            steps: vec![mere_view::SessionStep::Switch],
        };
        let mut model = MereViewModel {
            sessions: vec![row(1), row(2)],
            ..Default::default()
        };
        assert_eq!(newly_minted_session(&[id(1)], &model).unwrap(), id(2));
        model.sessions.push(row(3));
        assert!(newly_minted_session(&[id(1)], &model).is_err());
        assert!(newly_minted_session(&[id(1), id(2), id(3)], &model).is_err());
        model.sessions[2].trashed = true;
        assert_eq!(newly_minted_session(&[id(1)], &model).unwrap(), id(2));
    }

    #[test]
    fn resource_content_address_is_checked_before_decoding() {
        let bytes = serde_json::to_vec(&envelope()).unwrap();
        let hash = chirograph::ContentHash::of(&bytes);
        verify_resource(hash, &bytes).unwrap();
        assert!(verify_resource(hash, b"tampered").is_err());
        assert!(verify_resource(hash, b"").is_err());
    }
}
