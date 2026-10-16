// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0
use super::{ConsultationAction, ConsultationStatus, ConsultationUi};
use crate::ambient::*;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct AmbientUi {
    pub neighbor_domain: cambium::TextInput,
    pub granted_cards: Vec<crate::granted_ambient::GrantedCard>,
    pub granted_notice: String,
    pub context: Option<AmbientContext>,
    pub swatch: Option<cambium::GraphCanvasSwatch<String, RelationFamily>>,
    pub generation: u64,
    pub width: u32,
    pub ticks: u64,
    pub error: Option<String>,
    pub drag_basis: [[i32; 2]; 2],
    pub dragging: Option<(String, (f32, f32))>,
    pub back: Option<(String, String)>,
    pub drafts: BTreeMap<(String, String), AmbientView>,
}
impl AmbientUi {
    pub fn close(&mut self) {
        if let Some(context) = self.context.take() {
            self.drafts.insert(
                (context.data.session_id, context.data.position),
                context.view,
            );
        }
        self.swatch = None;
        self.generation += 1;
        self.dragging = None;
    }
    pub fn present(&mut self, mut context: AmbientContext) {
        if let Some(old) = self.context.take() {
            self.drafts
                .insert((old.data.session_id, old.data.position), old.view);
        }
        if let Some(view) = self.drafts.remove(&(
            context.data.session_id.clone(),
            context.data.position.clone(),
        )) {
            context.view = view;
            context.view.rebind(&context.data);
        }
        self.context = Some(context);
        self.generation += 1;
        self.error = None;
    }
}
impl ConsultationUi {
    pub fn present_neighborhood(
        &mut self,
        catalog: crate::ConsultationCatalog,
        detail: crate::ConsultationDetail,
        context: AmbientContext,
    ) {
        self.present_session(catalog, detail);
        self.reading_focus.insert(
            context.data.session_id.clone(),
            context.data.position.clone(),
        );
        self.ambient.present(context);
    }
    pub fn neighborhood_view(&self) -> Option<&AmbientView> {
        self.ambient.context.as_ref().map(|c| &c.view)
    }
    pub fn neighborhood_data(&self) -> Option<&CardNeighborhood> {
        self.ambient.context.as_ref().map(|c| &c.data)
    }
    pub(crate) fn neighborhood_action(&mut self, action: ConsultationAction) {
        if !self.status.is_busy() {
            self.status = ConsultationStatus::SavingSetup;
            self.record_action(Some(action));
        }
    }
    pub(crate) fn focus_neighbor(&mut self, id: String) {
        if let Some(context) = &mut self.ambient.context {
            if context.data.item(&id).is_some() {
                context.view.snapshot.selected_occurrence = Some(id.clone());
                if let Some(swatch) = &mut self.ambient.swatch {
                    swatch.selected = Some(id);
                }
            }
        }
    }
}
