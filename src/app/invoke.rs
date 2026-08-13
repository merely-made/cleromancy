// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Lowering advertised Graphshell commands through the bound Servitor subject.

use graphshell_protocol::{IntentInvocation, IntentResult};
use muniment::Backend;

use super::{AppError, CleromancyApp};
use crate::intents::{
    AstrologyReadingConcurrenceIntentPayload, COMPOSE_READING_INTENT, COMPOSE_READING_SCHEMA,
    CREATE_CONCURRENCE_INTENT, CREATE_CONCURRENCE_SCHEMA, CompositionLayout, FieldSelection,
    READ_INTENT, READ_SCHEMA, ROLL_INTENT, ROLL_SCHEMA, ReadingCompositionIntentPayload,
    ReadingIntentPayload, RollIntentPayload, SELECT_INTENT, SELECT_SCHEMA,
    THREE_CARD_SPREAD_INTENT, THREE_CARD_SPREAD_INTENT_SCHEMA, ThreeCardSpreadIntentPayload,
    die_field, scope_for,
};
use crate::moirai::clotho::EntropySource;
use crate::{HostError, ReadingEngine, ReadingError};

impl<B: Backend> CleromancyApp<B> {
    /// Lower an advertised Graphshell command through the bound Servitor
    /// subject. Production dispatch supplies OS entropy; tests may inject a
    /// deterministic source without creating a second command path.
    pub fn invoke_with_entropy(
        &mut self,
        intent: IntentInvocation,
        entropy: &mut impl EntropySource,
    ) -> Result<IntentResult, AppError> {
        if intent.session != self.host.session() {
            return Err(AppError::Intent(
                "intent names another projection session".to_string(),
            ));
        }
        let Some((epoch, revision)) = self.host.active_revision() else {
            return Err(AppError::Intent(
                "intent arrived before a projection snapshot".to_string(),
            ));
        };
        if intent.observed_epoch != epoch || intent.observed_revision != revision {
            return Ok(IntentResult::Stale {
                current_epoch: epoch,
                current_revision: revision,
            });
        }
        if !self
            .host
            .intent_was_advertised(intent.target, &intent.intent)
        {
            return Ok(rejected(
                "target or intent was not advertised by this snapshot",
            ));
        }
        if intent.payload.len() > self.intent_limits.max_payload_bytes {
            return Ok(rejected("payload exceeds the configured byte limit"));
        }
        let Some(subject) = self.intent_subject else {
            return Ok(rejected(
                "the containing transport did not bind an authenticated subject",
            ));
        };
        let scope = scope_for(&intent.intent)
            .ok_or_else(|| AppError::Intent("advertised intent has no scope".to_string()))?;

        if intent.intent == CREATE_CONCURRENCE_INTENT {
            let payload = match serde_json::from_slice::<AstrologyReadingConcurrenceIntentPayload>(
                &intent.payload,
            ) {
                Ok(payload) => payload,
                Err(error) => {
                    return Ok(rejected(format!("invalid concurrence payload: {error}")));
                }
            };
            if payload.schema != CREATE_CONCURRENCE_SCHEMA {
                return Ok(rejected(
                    "concurrence payload schema does not match the intent",
                ));
            }
            if invalid_digest(&payload.astrology_facts_digest)
                || invalid_digest(&payload.reading_session_id)
            {
                return Ok(rejected(
                    "selected facts digest or reading session ID is invalid",
                ));
            }
            if !self.host.concurrence_target_matches(
                intent.target,
                &payload.astrology_facts_digest,
                &payload.reading_session_id,
            ) {
                return Ok(rejected(
                    "the target card is not one selected member of the pattern occasion",
                ));
            }
            match self.host.validate_astrology_reading_concurrence_members(
                &payload.astrology_facts_digest,
                &payload.reading_session_id,
            ) {
                Ok(()) => {}
                Err(HostError::MissingReadingDependency { .. }) => {
                    return Ok(rejected(
                        "selected facts or reading session was not found in graph truth",
                    ));
                }
                Err(error) => return Err(error.into()),
            }
            if self
                .servitors
                .petition_write(subject, scope, "cleromancy.create-concurrence request")
                .is_err()
            {
                return Ok(rejected("Servitor refused the bound subject"));
            }
            match self.host.create_astrology_reading_concurrence(
                &payload.astrology_facts_digest,
                &payload.reading_session_id,
            ) {
                Ok(_) => {
                    self.pending_notice = true;
                    return Ok(IntentResult::Accepted);
                }
                Err(HostError::MissingReadingDependency { .. }) => {
                    return Ok(rejected(
                        "selected facts or reading session was not found in graph truth",
                    ));
                }
                Err(error) => return Err(error.into()),
            }
        }

        let context = self
            .host
            .context_for_instance(intent.target)
            .ok_or_else(|| AppError::Intent("advertised context disappeared".to_string()))?;

        if intent.intent == COMPOSE_READING_INTENT {
            let payload =
                match serde_json::from_slice::<ReadingCompositionIntentPayload>(&intent.payload) {
                    Ok(payload) => payload,
                    Err(error) => {
                        return Ok(rejected(format!("invalid composition payload: {error}")));
                    }
                };
            if payload.schema != COMPOSE_READING_SCHEMA {
                return Ok(rejected(
                    "composition payload schema does not match the intent",
                ));
            }
            let ReadingCompositionIntentPayload {
                field: selection,
                layout,
                mode,
                enrichment,
                client_token,
                ..
            } = payload;
            let field = match selection {
                FieldSelection::Inline { field } => field,
                FieldSelection::Stored { digest } => {
                    if digest.trim().is_empty() || digest.len() > 128 {
                        return Ok(rejected(
                            "stored field digest is outside the configured intent limits",
                        ));
                    }
                    match self.host.field_for_digest(&digest) {
                        Ok(field) => field,
                        Err(HostError::MissingReadingDependency { .. }) => {
                            return Ok(rejected("stored field was not found in graph truth"));
                        }
                        Err(error) => return Err(error.into()),
                    }
                }
            };
            if field.candidates.is_empty()
                || field.candidates.len() > self.intent_limits.max_candidates
            {
                return Ok(rejected(
                    "candidate count is outside the configured intent limits",
                ));
            }
            if invalid_client_token(
                client_token.as_deref(),
                self.intent_limits.max_client_token_bytes,
            ) {
                return Ok(rejected(
                    "client token is outside the configured intent limits",
                ));
            }
            if mode == crate::SelectionMode::Derived {
                return Ok(rejected(
                    "derived selection needs a disclosed seed and domain and is not available through this intent",
                ));
            }
            if layout == CompositionLayout::ThreeCard && mode != crate::SelectionMode::Cast {
                return Ok(rejected(
                    "the three-card layout requires cast selection mode",
                ));
            }
            if layout == CompositionLayout::ThreeCard && enrichment.is_some() {
                return Ok(rejected("the three-card layout does not accept enrichment"));
            }
            if self
                .servitors
                .petition_write(subject, scope, "cleromancy.compose-reading request")
                .is_err()
            {
                return Ok(rejected("Servitor refused the bound subject"));
            }
            match layout {
                CompositionLayout::Single => {
                    let result = match (mode, enrichment.as_ref()) {
                        (crate::SelectionMode::Calculated, Some(evidence)) => {
                            ReadingEngine::calculate_enriched(&context, &field, evidence)
                        }
                        (crate::SelectionMode::Calculated, None) => {
                            ReadingEngine::calculate(&context, &field)
                        }
                        (crate::SelectionMode::Cast, Some(evidence)) => {
                            ReadingEngine::cast_enriched_with(&context, &field, evidence, entropy)
                        }
                        (crate::SelectionMode::Cast, None) => {
                            ReadingEngine::cast_with(&context, &field, entropy)
                        }
                        (crate::SelectionMode::Derived, _) => {
                            return Ok(rejected(
                                "derived selection needs a disclosed seed and domain and is not available through this intent",
                            ));
                        }
                    };
                    let reading = match result {
                        Ok(reading) => reading,
                        Err(ReadingError::Entropy(error)) => {
                            return Err(AppError::Intent(format!("entropy failed: {error}")));
                        }
                        Err(error) => {
                            return Ok(rejected(format!(
                                "composition request is invalid: {error}"
                            )));
                        }
                    };
                    self.host.record_reading_session_with_entropy(
                        &context,
                        &field,
                        &reading,
                        client_token,
                        entropy,
                    )?;
                }
                CompositionLayout::ThreeCard => {
                    match self.host.record_three_card_spread_with_entropy(
                        &context,
                        &field,
                        client_token,
                        entropy,
                    ) {
                        Ok(_) => {}
                        Err(HostError::Reading(ReadingError::Entropy(error))) => {
                            return Err(AppError::Intent(format!("entropy failed: {error}")));
                        }
                        Err(HostError::Reading(error)) => {
                            return Ok(rejected(format!(
                                "composition request is invalid: {error}"
                            )));
                        }
                        Err(error) => return Err(error.into()),
                    }
                }
            }
            self.pending_notice = true;
            return Ok(IntentResult::Accepted);
        }

        if intent.intent == THREE_CARD_SPREAD_INTENT {
            let payload =
                match serde_json::from_slice::<ThreeCardSpreadIntentPayload>(&intent.payload) {
                    Ok(payload) => payload,
                    Err(error) => return Ok(rejected(format!("invalid spread payload: {error}"))),
                };
            if payload.schema != THREE_CARD_SPREAD_INTENT_SCHEMA {
                return Ok(rejected("spread payload schema does not match the intent"));
            }
            if payload.field.candidates.is_empty()
                || payload.field.candidates.len() > self.intent_limits.max_candidates
            {
                return Ok(rejected(
                    "candidate count is outside the configured intent limits",
                ));
            }
            if invalid_client_token(
                payload.client_token.as_deref(),
                self.intent_limits.max_client_token_bytes,
            ) {
                return Ok(rejected(
                    "client token is outside the configured intent limits",
                ));
            }
            if self
                .servitors
                .petition_write(subject, scope, "cleromancy.three-card-spread request")
                .is_err()
            {
                return Ok(rejected("Servitor refused the bound subject"));
            }
            match self.host.record_three_card_spread_with_entropy(
                &context,
                &payload.field,
                payload.client_token,
                entropy,
            ) {
                Ok(_) => {
                    self.pending_notice = true;
                    return Ok(IntentResult::Accepted);
                }
                Err(HostError::Reading(ReadingError::Entropy(error))) => {
                    return Err(AppError::Intent(format!("entropy failed: {error}")));
                }
                Err(HostError::Reading(error)) => {
                    return Ok(rejected(format!("spread request is invalid: {error}")));
                }
                Err(error) => return Err(error.into()),
            }
        }

        let (field, reading, client_token) = match intent.intent.as_str() {
            READ_INTENT | SELECT_INTENT => {
                let payload = match serde_json::from_slice::<ReadingIntentPayload>(&intent.payload)
                {
                    Ok(payload) => payload,
                    Err(error) => return Ok(rejected(format!("invalid reading payload: {error}"))),
                };
                let expected_schema = if intent.intent == READ_INTENT {
                    READ_SCHEMA
                } else {
                    SELECT_SCHEMA
                };
                if payload.schema != expected_schema {
                    return Ok(rejected("reading payload schema does not match the intent"));
                }
                if payload.field.candidates.is_empty()
                    || payload.field.candidates.len() > self.intent_limits.max_candidates
                {
                    return Ok(rejected(
                        "candidate count is outside the configured intent limits",
                    ));
                }
                if invalid_client_token(
                    payload.client_token.as_deref(),
                    self.intent_limits.max_client_token_bytes,
                ) {
                    return Ok(rejected(
                        "client token is outside the configured intent limits",
                    ));
                }
                if self
                    .servitors
                    .petition_write(subject, scope, format!("{} request", intent.intent))
                    .is_err()
                {
                    return Ok(rejected("Servitor refused the bound subject"));
                }
                let result = match (intent.intent.as_str(), payload.enrichment.as_ref()) {
                    (READ_INTENT, Some(evidence)) => {
                        ReadingEngine::calculate_enriched(&context, &payload.field, evidence)
                    }
                    (READ_INTENT, None) => ReadingEngine::calculate(&context, &payload.field),
                    (SELECT_INTENT, Some(evidence)) => ReadingEngine::cast_enriched_with(
                        &context,
                        &payload.field,
                        evidence,
                        entropy,
                    ),
                    (SELECT_INTENT, None) => {
                        ReadingEngine::cast_with(&context, &payload.field, entropy)
                    }
                    _ => unreachable!("the match is limited to read and select"),
                };
                match result {
                    Ok(reading) => (payload.field, reading, payload.client_token),
                    Err(ReadingError::Entropy(error)) => {
                        return Err(AppError::Intent(format!("entropy failed: {error}")));
                    }
                    Err(error) => {
                        return Ok(rejected(format!("reading request is invalid: {error}")));
                    }
                }
            }
            ROLL_INTENT => {
                let payload = match serde_json::from_slice::<RollIntentPayload>(&intent.payload) {
                    Ok(payload) => payload,
                    Err(error) => return Ok(rejected(format!("invalid roll payload: {error}"))),
                };
                if payload.schema != ROLL_SCHEMA {
                    return Ok(rejected("roll payload schema does not match the intent"));
                }
                if payload.sides < 2 || payload.sides > self.intent_limits.max_die_sides {
                    return Ok(rejected(
                        "die sides are outside the configured intent limits",
                    ));
                }
                if invalid_client_token(
                    payload.client_token.as_deref(),
                    self.intent_limits.max_client_token_bytes,
                ) {
                    return Ok(rejected(
                        "client token is outside the configured intent limits",
                    ));
                }
                if self
                    .servitors
                    .petition_write(subject, scope, "cleromancy.roll request")
                    .is_err()
                {
                    return Ok(rejected("Servitor refused the bound subject"));
                }
                let field = die_field(payload.sides, payload.label.as_deref());
                match ReadingEngine::cast_with(&context, &field, entropy) {
                    Ok(reading) => (field, reading, payload.client_token),
                    Err(ReadingError::Entropy(error)) => {
                        return Err(AppError::Intent(format!("entropy failed: {error}")));
                    }
                    Err(error) => {
                        return Ok(rejected(format!("roll request is invalid: {error}")));
                    }
                }
            }
            _ => return Ok(rejected("intent is not implemented")),
        };
        self.host.record_reading_session_with_entropy(
            &context,
            &field,
            &reading,
            client_token,
            entropy,
        )?;
        self.pending_notice = true;
        Ok(IntentResult::Accepted)
    }
}

fn invalid_client_token(client_token: Option<&str>, max_bytes: usize) -> bool {
    client_token.is_some_and(|token| token.is_empty() || token.len() > max_bytes)
}

fn invalid_digest(value: &str) -> bool {
    value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

fn rejected(reason: impl Into<String>) -> IntentResult {
    IntentResult::Rejected {
        reason: reason.into(),
    }
}
