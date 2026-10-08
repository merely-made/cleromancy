// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

use cambium::{GenetCtx, GenetElement, TextField, TextInput, attr_qual};
use layout_dom_api::{LayoutDom, LayoutDomMut};
use meristem::{MessageCtx, MessageResult, Mut, View, ViewMarker};

// The current public field wrapper has no attribute setter. Keep its builder
// and routing intact while attaching the consumer's existing visible name.
pub(super) struct NamedText {
    field: TextField,
    name: String,
}

pub(super) fn named_text(field: TextField, name: &str) -> NamedText {
    NamedText {
        field,
        name: name.to_owned(),
    }
}

impl NamedText {
    fn apply_name(&self, dom: &cambium::DomHandle, node: genet_scripted_dom::NodeId) {
        let qual = attr_qual("aria-label");
        let mut dom = dom.borrow_mut();
        if dom.attribute(node, &qual.ns, &qual.local) != Some(self.name.as_str()) {
            dom.set_attribute(node, qual, self.name.clone());
        }
    }
}

impl ViewMarker for NamedText {}

impl View<TextInput, (), GenetCtx> for NamedText {
    type Element = GenetElement;
    type ViewState = <TextField as View<TextInput, (), GenetCtx>>::ViewState;

    fn build(&self, ctx: &mut GenetCtx, input: &mut TextInput) -> (GenetElement, Self::ViewState) {
        let (element, state) = self.field.build(ctx, input);
        self.apply_name(&element.dom, element.node);
        (element, state)
    }

    fn rebuild(
        &self,
        prev: &Self,
        state: &mut Self::ViewState,
        ctx: &mut GenetCtx,
        mut element: Mut<'_, GenetElement>,
        input: &mut TextInput,
    ) {
        self.field
            .rebuild(&prev.field, state, ctx, element.reborrow_mut(), input);
        self.apply_name(&element.dom, *element.node);
    }

    fn teardown(
        &self,
        state: &mut Self::ViewState,
        ctx: &mut GenetCtx,
        element: Mut<'_, GenetElement>,
    ) {
        self.field.teardown(state, ctx, element);
    }

    fn message(
        &self,
        state: &mut Self::ViewState,
        message: &mut MessageCtx,
        element: Mut<'_, GenetElement>,
        input: &mut TextInput,
    ) -> MessageResult<()> {
        self.field.message(state, message, element, input)
    }
}
