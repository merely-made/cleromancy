// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The genet-probe automation surface over the native app.

use super::{NativeApp, SHEET};

impl genet_probe::Automatable for NativeApp {
    fn with_surfaces<R>(&self, f: impl FnOnce(&[genet_probe::ProbeSurface<'_>]) -> R) -> R {
        let Some(runner) = self.runner.as_ref() else {
            return f(&[]);
        };
        let dom = runner.dom();
        let dom_ref = dom.borrow();
        let (width, height) = self.layout_size;
        let surfaces = [genet_probe::ProbeSurface {
            name: "cleromancy",
            dom: &dom_ref,
            rect: [0.0, 0.0, width, height],
            sheet: SHEET,
        }];
        f(&surfaces)
    }

    fn snapshot(&self) -> genet_probe::ProbeSnapshot {
        let observation = self.scenario_observation();
        let mut snapshot = genet_probe::ProbeSnapshot::default()
            .with_field("status", observation.status)
            .with_field("catalog-ready", observation.catalog_ready.to_string())
            .with_field("sessions", observation.sessions.to_string())
            .with_field("readings", observation.readings.to_string())
            .with_field("reflections", observation.reflections.to_string());
        if let Some(ids) = observation.ids {
            snapshot = snapshot
                .with_field("session-id", ids.session_id)
                .with_field("reading-id", ids.reading_id)
                .with_field("reflection-id", ids.reflection_id);
        }
        snapshot
    }

    fn drain_events(&mut self) -> Vec<String> {
        std::mem::take(&mut self.probe_events)
    }

    fn act(&mut self, _label: &str) -> bool {
        false
    }

    fn press(&mut self, x: f32, y: f32) {
        self.cursor = (x, y);
        self.click();
    }

    fn moved(&mut self, x: f32, y: f32) {
        self.cursor = (x, y);
    }

    fn release(&mut self, _x: f32, _y: f32) {}

    fn busy(&mut self) -> Option<bool> {
        if !self.catalog_ready {
            return Some(true);
        }
        Some(self.runner.as_ref().is_some_and(|runner| {
            matches!(
                runner.state().status(),
                crate::ConsultationStatus::SavingReading
                    | crate::ConsultationStatus::SavingSetup
                    | crate::ConsultationStatus::SavingReflection
                    | crate::ConsultationStatus::LoadingSession
            )
        }))
    }
}

impl genet_probe::Driveable for NativeApp {
    fn capture(&mut self, name: &str) -> bool {
        let Some(dir) = self.capture_dir.as_ref() else {
            return false;
        };
        if name.is_empty() || name.contains(['/', '\\']) {
            return false;
        }
        self.pending_capture = Some(dir.join(format!("{name}.png")));
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
        true
    }

    fn app_step(&mut self, line: &str) -> Result<(), String> {
        match line {
            "advance" => self.scenario_advance(),
            _ => Err(format!("unknown Cleromancy scenario verb: {line}")),
        }
    }
}
