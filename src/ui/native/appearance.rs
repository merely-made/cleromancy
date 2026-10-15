// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Native-shell appearance. Reading, graph, worker and sync authority remain
//! in the consultation; this adapter owns only an applied Tabard presentation.

use std::path::{Path, PathBuf};
use tabard::theme::choice::{
    FileThemeChoiceStore, InMemoryThemeChoiceStore, ThemeChoice, ThemeChoiceStore,
};
use tabard::theme::registry::{Mode, ThemeSource};
use tabard::theme::seed::default_mode_for_def;
use tabard::{ResolvedThemeChoice, ThemePresentation, resolve_theme_choice};
use tabard_workshop::WorkshopState;

pub(super) const PRODUCT_DEFAULT: &str = "cleromancy:default";
pub(super) const MODES: [Mode; 4] = [Mode::Light, Mode::Dark, Mode::HcLight, Mode::HcDark];

pub(super) struct AppearanceState {
    pub workshop: WorkshopState,
    pub font_editor: tabard_workshop::fonts::FontRoleEditor,
    pub available_fonts: Vec<String>,
    pub open: bool,
    pub editor_open: bool,
    pub close_app: bool,
    pub authoring_available: bool,
    pub notice: Option<String>,
    store: Box<dyn ThemeChoiceStore>,
    selection_path: Option<PathBuf>,
    library_path: Option<PathBuf>,
    protected_root: Option<PathBuf>,
    selection_error: Option<String>,
    library_error: Option<String>,
    applied: Option<ResolvedThemeChoice>,
    pending_choice: Option<ThemeChoice>,
    installed_sheet: Option<String>,
}

impl Default for AppearanceState {
    fn default() -> Self {
        let available_fonts = cambium_genet_winit_host::available_font_families(&super::fonts::bundled_fonts());
        let mut font_editor = tabard_workshop::fonts::FontRoleEditor::new(Default::default());
        font_editor.set_available_families(available_fonts.clone());
        Self {
            workshop: WorkshopState::in_memory(),
            font_editor,
            available_fonts,
            open: false,
            editor_open: false,
            close_app: false,
            authoring_available: false,
            notice: None,
            store: Box::new(InMemoryThemeChoiceStore::new(ThemeChoice::new(
                PRODUCT_DEFAULT,
                None,
            ))),
            selection_path: None,
            library_path: None,
            protected_root: None,
            selection_error: None,
            library_error: None,
            applied: None,
            pending_choice: None,
            installed_sheet: None,
        }
    }
}

impl AppearanceState {
    pub fn load(selection: PathBuf, library: PathBuf, data_root: PathBuf) -> Self {
        let mut state = Self {
            selection_path: Some(selection.clone()),
            library_path: Some(library.clone()),
            protected_root: Some(data_root),
            ..Self::default()
        };
        if state.path_collision() {
            let message = "Appearance preferences, theme library and consultation store need separate paths. Existing files are preserved.".to_owned();
            state.selection_error = Some(message.clone());
            state.library_error = Some(message);
            state.protect_exports();
            return state;
        }
        match WorkshopState::load(library) {
            Ok(workshop) if workshop.registry().theme_def(PRODUCT_DEFAULT).is_none() => {
                state.workshop = workshop;
                state.authoring_available = true;
            },
            Ok(_) => state.library_error = Some("The library contains Cleromancy's reserved default identity. Repair it before authoring; product defaults remain active.".into()),
            Err(error) => state.library_error = Some(format!("The theme library could not be opened: {error}. The file is preserved and authoring is unavailable.")),
        }
        state.protect_exports();
        let existing = selection.exists();
        match FileThemeChoiceStore::load_strict(selection) {
            Ok(store) => {
                if existing {
                    let choice = store.choice();
                    let resolved = if choice.theme_id == PRODUCT_DEFAULT {
                        choice
                            .theme_mode
                            .as_ref()
                            .map(|_| product_resolution(choice))
                            .transpose()
                    } else {
                        resolve_theme_choice(state.workshop.registry(), choice)
                            .map(Some)
                            .map_err(|error| error.to_string())
                    };
                    match resolved {
                        Ok(resolved) => state.applied = resolved,
                        Err(error) => {
                            state.notice = Some(format!(
                                "The saved appearance cannot be rendered: {error}. Product defaults remain active."
                            ))
                        },
                    }
                }
                state.store = Box::new(store);
            },
            Err(error) => {
                state.selection_error = Some(format!(
                    "Appearance preferences could not be read: {error}. Repair the preserved file and reopen Cleromancy before saving a choice."
                ))
            },
        }
        state.font_editor = tabard_workshop::fonts::FontRoleEditor::new(state.store.choice().fonts.clone());
        state.font_editor.set_available_families(state.available_fonts.clone());
        state
    }

    fn path_collision(&self) -> bool {
        let Some(selection) = &self.selection_path else {
            return false;
        };
        let Some(library) = &self.library_path else {
            return false;
        };
        same_destination(selection, library)
            || self.protected_root.as_ref().is_some_and(|root| {
                let consultation = root.join("cleromancy.redb");
                same_destination(selection, &consultation)
                    || same_destination(library, &consultation)
            })
    }

    fn protect_exports(&mut self) {
        self.workshop
            .set_protected_export_paths(self.selection_path.iter().cloned().collect());
        self.workshop
            .set_protected_export_directories(self.protected_root.iter().cloned().collect());
    }

    /// Reloading the catalog does not publish a changed same-identity definition.
    pub fn reload_library(&mut self) -> Result<(), String> {
        if self.editor_open || self.workshop.has_changes() || self.workshop.has_pending_fields() {
            return Err(
                "Save or discard the workshop changes before reloading the library.".into(),
            );
        }
        if self.path_collision() {
            self.authoring_available = false;
            let message = "Appearance preferences, theme library and consultation store need separate paths. Existing files are preserved.".to_owned();
            self.library_error = Some(message.clone());
            return Err(message);
        }
        let path = self
            .library_path
            .clone()
            .ok_or("No persistent theme library is available.")?;
        let workshop = match WorkshopState::load(path) {
            Ok(workshop) if workshop.registry().theme_def(PRODUCT_DEFAULT).is_none() => workshop,
            Ok(_) => {
                let message = "The library contains Cleromancy's reserved default identity. The file is preserved and authoring is unavailable.".to_owned();
                self.authoring_available = false;
                self.library_error = Some(message.clone());
                return Err(message);
            },
            Err(error) => {
                let message = format!(
                    "The theme library could not be opened: {error}. The file is preserved and authoring is unavailable."
                );
                self.authoring_available = false;
                self.library_error = Some(message.clone());
                return Err(message);
            },
        };
        self.workshop = workshop;
        self.authoring_available = true;
        self.library_error = None;
        self.protect_exports();
        Ok(())
    }

    pub fn applied(&self) -> Option<&ResolvedThemeChoice> {
        self.applied.as_ref()
    }
    pub fn active_id(&self) -> &str {
        self.applied
            .as_ref()
            .map_or(PRODUCT_DEFAULT, |value| value.resolved.theme_id.as_str())
    }
    pub fn modes(&self) -> Vec<Mode> {
        let mut modes = MODES.to_vec();
        if let Some(theme) = self.workshop.registry().theme_def(self.active_id()) {
            for key in theme.mode_sheets.keys() {
                if let Some(mode @ Mode::Custom(_)) = Mode::from_key(key) {
                    if !modes.contains(&mode) {
                        modes.push(mode);
                    }
                }
            }
        }
        modes
    }

    fn candidate(&self, choice: &ThemeChoice) -> Result<Option<ResolvedThemeChoice>, String> {
        if let Some(error) = &self.selection_error {
            return Err(error.clone());
        }
        if choice.theme_id == PRODUCT_DEFAULT {
            return choice
                .theme_mode
                .as_ref()
                .map(|_| product_resolution(choice))
                .transpose();
        }
        let theme = self
            .workshop
            .registry()
            .theme_def(&choice.theme_id)
            .ok_or_else(|| format!("Theme {} is unavailable.", choice.theme_id))?;
        let mode = choice
            .theme_mode
            .clone()
            .unwrap_or_else(|| default_mode_for_def(theme));
        theme
            .presentation_for_mode(&mode)
            .map_err(|error| error.to_string())?;
        resolve_theme_choice(self.workshop.registry(), choice)
            .map(Some)
            .map_err(|error| error.to_string())
    }

    pub fn request_select(&mut self, id: &str, mode: Option<Mode>) -> Result<(), String> {
        let mut choice = ThemeChoice::new(id, mode);
        choice.fonts = self.store.choice().fonts.clone();
        self.candidate(&choice)?;
        self.pending_choice = Some(choice);
        Ok(())
    }

    pub fn font_preferences(&self) -> &tabard::typography::FontPreferences { &self.store.choice().fonts }

    pub fn font_diagnostics(&self) -> Vec<String> {
        let mut messages = Vec::new();
        for resolved in self.font_preferences().resolve(&self.available_fonts, &super::fonts::defaults()) {
            let missing: Vec<_> = resolved.requested.iter().filter(|family| !family.is_generic() && !self.available_fonts.iter().any(|name| name.eq_ignore_ascii_case(family.name()))).map(|family| family.name()).collect();
            if !missing.is_empty() { messages.push(format!("{}: {} unavailable here; using {} with glyph fallback. Saved requests are retained.", resolved.role.label(), missing.join(", "), resolved.family.name())); }
        }
        messages
    }

    pub fn commit_requested(&mut self) {
        if let Some(fonts) = self.font_editor.take_requested() {
            let mut choice = self.store.choice().clone();
            choice.fonts = fonts;
            match self.candidate(&choice) {
                Ok(_) => self.pending_choice = Some(choice),
                Err(error) => self.font_editor.notice = Some(error),
            }
        }
        let Some(choice) = self.pending_choice.take() else {
            return;
        };
        let result = (|| {
            if self.path_collision() {
                return Err("Appearance was not saved: preferences, library and consultation store need separate paths.".to_owned());
            }
            let candidate = self.candidate(&choice)?;
            if let Some(parent) = self.selection_path.as_ref().and_then(|path| path.parent()) {
                std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
            }
            self.store
                .set_choice(choice)
                .map_err(|error| format!("Appearance was not saved: {error}"))?;
            self.font_editor.applied(self.store.choice().fonts.clone());
            self.applied = candidate;
            Ok::<_, String>(())
        })();
        self.notice = Some(match result {
            Ok(()) if self.store.is_persistent() => {
                "Saved appearance applied to Cleromancy.".into()
            },
            Ok(()) => "Appearance applied for this session.".into(),
            Err(error) => error,
        });
    }

    pub fn begin_edit(&mut self) -> Result<(), String> {
        if !self.authoring_available {
            return Err("Reload a readable theme library before authoring.".into());
        }
        if self.editor_open {
            return Ok(());
        }
        if self.workshop.has_changes() || self.workshop.has_pending_fields() {
            return Err(
                "Save or discard the current workshop changes before opening another definition."
                    .into(),
            );
        }
        let (theme, mode) = match &self.applied {
            Some(applied) => (
                self.workshop
                    .registry()
                    .theme_def(&applied.resolved.theme_id)
                    .cloned()
                    .unwrap_or_else(|| applied.theme.clone()),
                applied.resolved.theme_mode.clone(),
            ),
            None => (product_theme(), Some(Mode::Dark)),
        };
        self.workshop.edit_definition(&theme, mode)?;
        self.workshop.cancel_close();
        self.editor_open = true;
        self.close_app = false;
        Ok(())
    }

    pub fn apply_workshop(&mut self) -> Result<(), String> {
        let choice = self.workshop.saved_choice()?;
        if choice.theme_id == PRODUCT_DEFAULT {
            return Err("Make a new copy of Cleromancy's default before applying it.".into());
        }
        self.request_select(&choice.theme_id, choice.theme_mode)
    }

    pub fn back(&mut self) {
        self.close_app = false;
        if self.workshop.request_close() {
            self.workshop.cancel_close();
            self.editor_open = false;
        }
    }

    pub fn prepare_close(&mut self) -> bool {
        if !self.editor_open {
            return true;
        }
        self.close_app = true;
        if !self.workshop.request_close() {
            return false;
        }
        self.workshop.cancel_close();
        self.editor_open = false;
        self.close_app = false;
        true
    }

    pub fn sync_editor(&mut self, exit: bool) -> bool {
        if !self.editor_open {
            return false;
        }
        if exit {
            let close = self.close_app;
            self.workshop.cancel_close();
            self.editor_open = false;
            self.close_app = false;
            return close;
        }
        if !self.workshop.close_requested() {
            self.close_app = false;
        }
        false
    }

    pub fn stylesheet(&self) -> String {
        if self.editor_open {
            return format!(
                "{}\n{}\n{}\n{}",
                tabard_workshop::workshop_stylesheet(),
                cambium::TITLE_BAR_CSS,
                super::appearance_view::CHROME_CSS,
                super::appearance_view::EDITOR_CSS
            );
        }
        let base = format!(
            "{}\n{}\n{}\n{}\n{}",
            super::SHEET,
            mere_view::MERE_VIEW_CSS,
            cambium::GRAPH_CANVAS_SWATCH_CSS,
            cambium::TITLE_BAR_CSS,
            super::appearance_view::CHROME_CSS
        );
        let themed = match self.applied.as_ref().map(|value| &value.presentation) {
            None => base,
            Some(ThemePresentation::Derived(mode)) => {
                format!("{base}\n{}", mode.css_custom_properties())
            },
            Some(ThemePresentation::AuthoredStylesheet(rules)) => {
                format!("{base}\n{}", rules.join("\n"))
            },
        };
        format!("{themed}\n{}\n{}", tabard_workshop::fonts::FONT_ROLE_CSS, self.font_preferences().css_custom_properties(&self.available_fonts, &super::fonts::defaults()))
    }

    pub fn take_stylesheet_change(&mut self) -> Option<String> {
        let sheet = self.stylesheet();
        if self.installed_sheet.as_ref() == Some(&sheet) {
            return None;
        }
        self.installed_sheet = Some(sheet.clone());
        Some(sheet)
    }

    pub fn diagnostics(&self) -> Vec<String> {
        let mut values: Vec<String> = self
            .applied
            .iter()
            .flat_map(|value| value.diagnostics.iter().map(ToString::to_string))
            .collect();
        for message in [&self.selection_error, &self.library_error, &self.notice]
            .into_iter()
            .flatten()
        {
            if !values.contains(message) {
                values.push(message.clone());
            }
        }
        values
    }
}

fn product_theme() -> tabard::Theme {
    let mut seeds = tabard::theme::registry::ThemeRegistry::default()
        .theme_def(tabard::theme::registry::THEME_ID_DEFAULT)
        .expect("shared default theme")
        .seeds;
    seeds.primary = tinct::Srgb::rgb(215, 180, 106);
    seeds.neutral = tinct::Srgb::rgb(24, 23, 20);
    seeds.dark = true;
    let mut theme = tabard::Theme::new(PRODUCT_DEFAULT, "Cleromancy default", seeds);
    theme.source = ThemeSource::BuiltIn;
    theme.mode_sheets.insert(
        "dark".into(),
        vec![super::appearance_view::PRODUCT_DEFAULT_CSS.into()],
    );
    theme
}

// Resolve the longest existing ancestor, including aliases of paths whose
// parent directories do not exist yet. This is a local configuration guard;
// workshop export authority and library generation protection stay shared.
fn same_destination(left: &Path, right: &Path) -> bool {
    fn identity(path: &Path) -> Option<PathBuf> {
        let mut ancestor = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir().ok()?.join(path)
        };
        let mut suffix = Vec::new();
        let mut result = loop {
            if let Ok(path) = ancestor.canonicalize() {
                break path;
            }
            suffix.push(
                ancestor
                    .components()
                    .next_back()?
                    .as_os_str()
                    .to_os_string(),
            );
            if !ancestor.pop() {
                return None;
            }
        };
        for segment in suffix.iter().rev() {
            match Path::new(segment).components().next()? {
                std::path::Component::CurDir => {},
                std::path::Component::ParentDir => {
                    result.pop();
                },
                _ => result.push(segment),
            }
        }
        if cfg!(any(target_os = "macos", target_os = "windows")) {
            result = PathBuf::from(result.as_os_str().to_string_lossy().to_lowercase());
        }
        Some(result)
    }
    left == right
        || matches!((identity(left), identity(right)), (Some(left), Some(right)) if left == right)
}

fn product_resolution(choice: &ThemeChoice) -> Result<ResolvedThemeChoice, String> {
    let theme = product_theme();
    let mode = choice.theme_mode.clone().unwrap_or(Mode::Dark);
    let presentation = theme
        .presentation_for_mode(&mode)
        .map_err(|error| error.to_string())?;
    Ok(ResolvedThemeChoice {
        requested: choice.clone(),
        resolved: ThemeChoice { fonts: choice.fonts.clone(), ..ThemeChoice::new(PRODUCT_DEFAULT, Some(mode)) },
        theme,
        presentation,
        diagnostics: vec![],
    })
}

pub(super) fn paths(data_root: &Path) -> (PathBuf, PathBuf) {
    let selection = std::env::var_os("CLEROMANCY_APPEARANCE_STORE")
        .map(PathBuf::from)
        .unwrap_or_else(|| data_root.join("appearance.json"));
    let library = std::env::var_os("CLEROMANCY_THEME_LIBRARY")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            if std::env::var_os("CLEROMANCY_ROOT").is_some() {
                data_root.join("themes.json")
            } else {
                dirs::data_local_dir()
                    .unwrap_or_else(|| data_root.to_path_buf())
                    .join("mere/tabard/themes.json")
            }
        });
    (selection, library)
}
