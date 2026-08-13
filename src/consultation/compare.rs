// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Replay-derived comparison of two saved session receipts.

use std::collections::{BTreeMap, BTreeSet};

use super::{ConsultationDetail, ReceiptComparison, ReceiptComparisonEntry};
use crate::Reading;

pub(super) fn compare_details(
    left: &ConsultationDetail,
    right: &ConsultationDetail,
) -> ReceiptComparison {
    let left_readings = readings_by_position(left);
    let right_readings = readings_by_position(right);
    let positions = left_readings
        .keys()
        .chain(right_readings.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    let entries = positions
        .into_iter()
        .map(|position| {
            let left_reading = left_readings.get(&position).copied();
            let right_reading = right_readings.get(&position).copied();
            ReceiptComparisonEntry {
                position,
                left_candidate: left_reading.map(|reading| reading.candidate_id.clone()),
                right_candidate: right_reading.map(|reading| reading.candidate_id.clone()),
                same_candidate: left_reading
                    .zip(right_reading)
                    .map(|(left, right)| left.candidate_id == right.candidate_id),
                left_mode: left_reading.map(|reading| reading.receipt.mode),
                right_mode: right_reading.map(|reading| reading.receipt.mode),
                same_mode: left_reading
                    .zip(right_reading)
                    .map(|(left, right)| left.receipt.mode == right.receipt.mode),
                left_algorithm: left_reading.map(|reading| reading.receipt.algorithm.clone()),
                right_algorithm: right_reading.map(|reading| reading.receipt.algorithm.clone()),
                same_receipt: left_reading
                    .zip(right_reading)
                    .map(|(left, right)| left.receipt == right.receipt),
            }
        })
        .collect();
    ReceiptComparison {
        left_session_id: left.session.id.clone(),
        right_session_id: right.session.id.clone(),
        same_context: left.session.context_digest == right.session.context_digest,
        same_field: left.session.field_digest == right.session.field_digest,
        same_position_names: left
            .session
            .placements
            .iter()
            .map(|placement| &placement.position)
            .eq(right
                .session
                .placements
                .iter()
                .map(|placement| &placement.position)),
        entries,
    }
}

fn readings_by_position<'a>(detail: &'a ConsultationDetail) -> BTreeMap<String, &'a Reading> {
    detail
        .session
        .placements
        .iter()
        .zip(&detail.readings)
        .map(|(placement, reading)| (placement.position.clone(), reading))
        .collect()
}
