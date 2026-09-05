// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The four surfaces, and the tab bar that is the only way to move between
//! them.
//!
//! Surface choice is view-local: it is derived from one Cambium
//! [`SelectionState`] held by [`crate::ConsultationUi`], and nothing the
//! persistence worker reports may move the person to another surface. Keeping
//! the mapping in one module is what makes that single source of truth
//! checkable — the enum, the tab items, the panel ids, and the feature gate all
//! live here.

use cambium::{SelectionItem, SelectionState};

/// The tab list's own DOM id. Cambium derives each tab's id from it as
/// `{bar}-item-{item id}`, which is what `aria-labelledby` on a surface panel
/// points at.
pub(crate) const SURFACE_BAR_ID: &str = "cleromancy-surfaces";

/// Which surface is in the tree.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ConsultationScreen {
    /// The working surface: consultation form, the current reading, and a
    /// bounded recent trail.
    #[default]
    Today,
    /// The full archive reader.
    Journal,
    /// Stored sky facts. Empty until R3.
    Sky,
    /// Saved chart moments. Empty until R4.
    Chart,
}

impl ConsultationScreen {
    /// Every surface, in tab order.
    pub const ALL: [Self; 4] = [Self::Today, Self::Journal, Self::Sky, Self::Chart];

    /// The stable key used by `data-screen`, region keys, and tab ids.
    pub fn key(self) -> &'static str {
        match self {
            Self::Today => "today",
            Self::Journal => "journal",
            Self::Sky => "sky",
            Self::Chart => "chart",
        }
    }

    /// The visible tab label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Today => "Today",
            Self::Journal => "Journal",
            Self::Sky => "Sky",
            Self::Chart => "Chart",
        }
    }

    fn index(self) -> usize {
        Self::ALL
            .iter()
            .position(|screen| *screen == self)
            .unwrap_or(0)
    }

    fn from_index(index: usize) -> Self {
        Self::ALL.get(index).copied().unwrap_or_default()
    }

    /// The id of the `<main>` panel this tab controls.
    pub(crate) fn panel_id(self) -> String {
        format!("cleromancy-surface-{}", self.key())
    }

    /// The DOM id Cambium's selection bar stamps on this surface's tab.
    ///
    /// See `mere/crates/cambium/cambium/src/selection_bar.rs`: the id is
    /// `{state.id}-item-{item.id}` with non-alphanumerics replaced. Every
    /// surface key here is already lowercase ASCII, so no replacement occurs.
    pub(crate) fn tab_dom_id(self) -> String {
        format!("{SURFACE_BAR_ID}-item-{}", self.key())
    }

    /// Why this surface cannot be opened in this build, when it cannot.
    ///
    /// The `sky-timeline` feature gates the Sky event surface, even though
    /// stored sky facts remain catalogued without compiling an adapter. The
    /// chart surface is unconditional: manual chart import needs no ephemeris
    /// feature.
    pub(crate) fn disabled_reason(self) -> Option<&'static str> {
        match self {
            Self::Sky if !cfg!(feature = "sky-timeline") => {
                Some("The sky surface needs the sky-timeline feature.")
            },
            _ => None,
        }
    }
}

/// The retained selection state backing the surface tab bar.
pub(crate) fn surface_tab_state() -> SelectionState {
    SelectionState::single(ConsultationScreen::Today.index())
        .with_label("Surfaces")
        .with_id(SURFACE_BAR_ID)
}

/// One tab per surface, each controlling its own panel.
pub(crate) fn surface_tab_items() -> Vec<SelectionItem> {
    ConsultationScreen::ALL
        .iter()
        .map(|screen| {
            let item = SelectionItem::new(screen.label())
                .with_id(screen.key())
                .controls(screen.panel_id());
            match screen.disabled_reason() {
                Some(reason) => item.disabled_because(reason),
                None => item,
            }
        })
        .collect()
}

/// The one place a surface is derived from the tab bar's selection.
pub(crate) fn selected_screen(state: &SelectionState) -> ConsultationScreen {
    state
        .selected
        .first()
        .copied()
        .map(ConsultationScreen::from_index)
        .unwrap_or_default()
}

/// Change surfaces from a local link without creating a product command.
pub(crate) fn select_surface(state: &mut SelectionState, screen: ConsultationScreen) {
    state.selected = vec![screen.index()];
}
