// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0
use super::*;
use crate::ambient::{AmbientContext, AmbientView};
impl<B: Backend> Consultation<B> {
    pub fn ambient_context(
        &self,
        session_id: &str,
        position: &str,
    ) -> Result<AmbientContext, ConsultationError> {
        Ok(self.host.ambient_context(session_id, position)?)
    }
    pub async fn save_ambient_view(&mut self, view: &AmbientView) -> Result<(), ConsultationError> {
        self.ensure_writable()?;
        self.host.insert_ambient_view(view)?;
        self.persist(unix_time_secs()?).await
    }
    pub async fn remember_ambient(
        &mut self,
        session_id: &str,
        position: &str,
        item_id: &str,
        keep: bool,
    ) -> Result<(), ConsultationError> {
        self.ensure_writable()?;
        self.host
            .examine_ambient(session_id, position, item_id, keep, unix_time_ms()?)?;
        self.persist(unix_time_secs()?).await
    }
}
