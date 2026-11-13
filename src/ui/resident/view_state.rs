// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The reading workspace's arrangement (SET_VIEW) and this device's
//! attention (SET_ATTENTION); a read-only archive keeps both session-local.

use super::*;

impl ResidentConsultation {
    /// The read-only archive's session-local view, while it is still attached.
    fn transient(&self) -> Option<&TransientView> {
        self.transient_view.as_ref().filter(|(id, _, _)| {
            self.envelope
                .as_ref()
                .is_some_and(|envelope| envelope.read_only && envelope.session == *id)
        })
    }

    pub(super) fn reading_view(&self) -> incipit::view::ViewIntent {
        if let Some((_, view, _)) = self.transient() {
            return view.clone();
        }
        self.envelope
            .as_ref()
            .and_then(|envelope| envelope.views.get("reading-workspace"))
            .cloned()
            .unwrap_or_default()
    }

    pub(super) fn reading_attention(&self) -> incipit::view::AttentionState {
        if let Some((_, _, attention)) = self.transient() {
            return attention.clone();
        }
        self.envelope
            .as_ref()
            .and_then(|envelope| envelope.attention.get("reading-workspace"))
            .cloned()
            .unwrap_or_default()
    }

    pub(super) async fn set_view(&mut self, view: incipit::view::ViewIntent) -> Result<(), String> {
        let envelope = self
            .envelope
            .as_ref()
            .ok_or("no resident graph is attached")?;
        if envelope.read_only {
            // SET_VIEW on a thaw is an explicit first-edit fork at the owner.
            // Reading and arranging it must instead remain session-local.
            self.model.layout = view.strategy.clone().unwrap_or_default();
            let attention = self.reading_attention();
            self.transient_view = Some((envelope.session, view, attention));
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

    /// Keep focus as this device's attention (Graphshell's C7 split).
    pub(super) async fn remember_focus(&mut self, url: Option<String>) -> Result<(), String> {
        let envelope = self
            .envelope
            .as_ref()
            .ok_or("no resident graph is attached")?;
        let attention = incipit::view::AttentionState {
            focus: url,
            ..self.reading_attention()
        };
        if envelope.read_only {
            // Graphshell's resident would accept SET_ATTENTION on a thaw;
            // Cleromancy does not send it, so a read-only archive stays unwritten.
            let session = envelope.session;
            let view = self.reading_view();
            self.transient_view = Some((session, view, attention));
            return Ok(());
        }
        let intent = chirograph::IntentInvocation {
            session: self.graph_projection.session.clone(),
            target: sceno::InstanceId(0),
            observed_epoch: self.graph_projection.scene.epoch,
            observed_revision: self.graph_projection.scene.revision,
            intent: session_item::SET_ATTENTION_INTENT.into(),
            payload: serde_json::to_vec(&session_item::SetAttentionV1::new(
                "reading-workspace",
                attention,
            ))
            .map_err(err)?,
        };
        self.invoke(intent).await?;
        self.refresh().await
    }

    pub fn restored_selection(&self) -> Option<(crate::ConsultationDetail, Option<String>)> {
        self.reading_attention()
            .focus
            .and_then(|url| self.selected_detail(&url).ok())
    }
}
