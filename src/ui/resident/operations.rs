// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;

impl ResidentConsultation {
    pub async fn attach(mut attachment: ReservoirAttachment) -> Result<Self, String> {
        let reading =
            materialization::read(attachment.client_mut(), "cleromancy", GraphRead::Complete)
                .await?;
        let model = materialized_model(&reading, MereViewModel::default())?;
        let mut client = Self {
            personal: None,
            granted: None,
            attachment,
            sessions: reading.sessions,
            graph_projection: reading.graph,
            envelope: reading.graph_material,
            archive: reading.archive,
            transient_view: None,
            model,
            unavailable: false,
            saved_notice: None,
        };
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
        let reading = materialization::read(
            self.attachment.client_mut(),
            "cleromancy",
            GraphRead::Complete,
        )
        .await?;
        let mut model = materialized_model(&reading, self.model.clone())?;
        let staged_envelope = reading.graph_material;
        self.transient_view = transient_view.filter(|(id, _, _)| {
            staged_envelope
                .as_ref()
                .is_some_and(|envelope| envelope.read_only && envelope.session == *id)
        });
        if let Some((_, view, _)) = &self.transient_view {
            model.layout = view.strategy.clone().unwrap_or_default();
        }
        self.sessions = reading.sessions;
        self.graph_projection = reading.graph;
        self.model = model;
        self.archive = reading.archive;
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

    pub(super) fn candidate(
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
        if matches!(
            &action,
            ConsultationAction::RefreshBirthFacts
                | ConsultationAction::CreateSubject { .. }
                | ConsultationAction::SaveBirthFacts { .. }
                | ConsultationAction::CreateNatal { .. }
        ) {
            return self.personal_action(action).await;
        }

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
                        &ostiary::app_admission::configured_app_endpoint(),
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

    pub(super) async fn personal_action(
        &mut self,
        action: ConsultationAction,
    ) -> Result<WorkerUpdate, String> {
        let result = self.personal_action_inner(action).await;
        match result {
            Ok(value) => Ok(value),
            Err(notice) => {
                if let Some(client) = self.personal.take() {
                    let _ = client.close().await;
                }
                Ok(WorkerUpdate::BirthFacts {
                    catalog: None,
                    notice,
                })
            },
        }
    }
    pub(super) async fn personal_action_inner(
        &mut self,
        action: ConsultationAction,
    ) -> Result<WorkerUpdate, String> {
        if self.personal.is_none() {
            self.personal = Some(
                mere_resident::personal_facts::PersonalFactsClient::open(
                    &ostiary::app_admission::configured_app_endpoint(),
                    ostiary::app_admission::AppId::new("cleromancy"),
                    crate::granted_ambient::declaration(),
                )
                .await?,
            );
        }
        let client = self.personal.as_mut().unwrap();
        match action {
            ConsultationAction::RefreshBirthFacts => {},
            ConsultationAction::CreateSubject { label } => {
                client.create_contact(&label).await?;
            },
            ConsultationAction::SaveBirthFacts { facts } => {
                client.save_birth(&facts).await?;
            },
            ConsultationAction::CreateNatal {
                resource,
                question,
                fold,
                houses,
                orientation,
                aspects,
            } => {
                let catalog = client.catalog().await?;
                let birth=catalog.births.iter().find(|b|b.resource==resource).ok_or("Selected birth revision is no longer available from the admitted shared domain")?.clone();
                let label = catalog
                    .subjects
                    .iter()
                    .find(|s| s.subject == birth.facts.subject)
                    .ok_or("Subject is unavailable")?
                    .label
                    .clone();
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_err(err)?
                    .as_millis() as u64;
                let reading = crate::natal::NatalReading::new(
                    catalog.persona,
                    label,
                    birth,
                    question,
                    fold,
                    now,
                    natal_nonce(),
                )?;
                let reading = reading.with_supplement(houses, orientation)?;
                let reading = if let Some(rules) = aspects {
                    reading.with_aspects(rules)?
                } else {
                    reading
                };
                self.refresh().await?;
                let (mut candidate, edits) = self.candidate()?;
                candidate
                    .host_mut_candidate()
                    .insert_natal_reading(&reading)
                    .map_err(err)?;
                self.commit(edits).await?;
                self.refresh().await?;
                let link = dramatis::resources::SubjectResourceAssociation::about(
                    reading.birth.facts.subject.clone(),
                    dramatis::resources::ResourceRef {
                        domain: "divination".into(),
                        id: format!("cleromancy://natal/{}", reading.id),
                        revision: reading.id.clone(),
                    },
                );
                if let Some(client) = self.personal.as_mut() {
                    if let Err(error) = client.associate(&link).await {
                        self.saved_notice = Some(format!(
                            "Natal reading saved; shared subject link could not be saved: {error}"
                        ));
                    }
                }
                return Ok(WorkerUpdate::Catalog(
                    self.consultation()?.catalog().map_err(err)?,
                ));
            },
            _ => return Err("Unsupported shared birth action".into()),
        }
        Ok(WorkerUpdate::BirthFacts {
            catalog: Some(client.catalog().await?),
            notice: "Shared birth facts refreshed. Corrections create a new revision.".into(),
        })
    }

    pub(super) async fn commit(
        &mut self,
        edits: Arc<Mutex<Vec<CapturedDelta>>>,
    ) -> Result<(), String> {
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

    pub(super) async fn invoke(
        &mut self,
        intent: chirograph::IntentInvocation,
    ) -> Result<(), String> {
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

    pub(super) fn discard_transient_view(&mut self) {
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

    pub async fn close(mut self) -> Result<(), String> {
        if let Some(client) = self.personal.take() {
            let _ = client.close().await;
        }
        self.attachment.close().await.map_err(err)
    }
}
