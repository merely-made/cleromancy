// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0
use super::ConsultationView;
use crate::{
    ConsultationDetail,
    inference::Generation,
    ui::{ConsultationAction, ConsultationUi},
};
use cambium::{button, el};
pub(super) fn generated(ui: &ConsultationUi, detail: &ConsultationDetail) -> ConsultationView {
    let mut children: Vec<ConsultationView> = vec![
        Box::new(el::<_, ConsultationUi, ()>("h3", "Reflection prompts")),
        Box::new(el::<_, ConsultationUi, ()>(
            "p",
            "Generate a question from this reading and up to eight earlier occasions. The formula counts repeated cards; the local model offers its own wording. Saved reflections are included. Each prompt is labelled, stored separately, and starts in Recent memory. Keep explicitly promotes it to Saved memory.",
        )),
    ];
    children.push(Box::new(el::<_, ConsultationUi, ()>(
        "p",
        ui.local_model_label.clone(),
    )));
    for (label, key, model) in [
        ("Generate a history prompt", "inference-formula", false),
        ("Generate with local model", "inference-model", true),
    ] {
        let session_id = detail.session.id.clone();
        children.push(Box::new(
            button(label, move |ui: &mut ConsultationUi, _| {
                ui.neighborhood_action(ConsultationAction::GenerateInference {
                    session_id: session_id.clone(),
                    model,
                });
            })
            .attr("data-key", key),
        ));
    }
    for record in &detail.inferences {
        let method = match &record.generation {
            Generation::Formula { method } => {
                format!("Method: {method}. Recomputed exactly from the saved input selection.")
            },
            Generation::Model {
                method,
                model,
                max_tokens,
                temperature_milli,
                ..
            } => format!(
                "Method: {method}. Model: {} at {}; runtime {}; at most {max_tokens} tokens, temperature {}. This is the stored output; reopening does not run the model.",
                model.model_id,
                model.revision,
                model.loader,
                *temperature_milli as f64 / 1000.0
            ),
        };
        let session_id = detail.session.id.clone();
        let digest = record.digest();
        let mut items: Vec<ConsultationView> = vec![
            Box::new(el::<_, ConsultationUi, ()>("h4", record.label.clone())),
            Box::new(
                el::<_, ConsultationUi, ()>("p", record.output.clone())
                    .attr("data-key", format!("inference-output:{}", record.digest())),
            ),
            Box::new(el::<_, ConsultationUi, ()>("p", match &record.generation {
                Generation::Formula { .. } => "Counts repeated cards in the selected earlier occasions; this prompt recomputes exactly.".to_string(),
                Generation::Model { model, .. } => format!("Generated locally by {}. The saved output reopens without running the model.", model.model_id),
            })),
            Box::new(el::<_, ConsultationUi, ()>(
                "p",
                format!(
                    "Uses this reading, {} earlier occasions and {} saved reflections. Questions and card prompts use excerpts of up to 160 characters; reflections use up to 240.",
                    record.inputs.history_session_ids.len(),
                    record.inputs.reflection_ids.len()
                ),
            )),
            Box::new(el::<_, ConsultationUi, ()>(
                "p",
                if detail.kept_inferences.contains(&record.digest()) {
                    "Saved memory"
                } else {
                    "Recent memory"
                },
            )),
            Box::new(
                button("Keep this prompt", move |ui: &mut ConsultationUi, _| {
                    ui.neighborhood_action(ConsultationAction::KeepInference {
                        session_id: session_id.clone(),
                        digest: digest.clone(),
                    });
                })
                .attr("data-key", format!("inference-keep:{}", record.digest())),
            ),
        ];
        let digest = record.digest();
        let expanded = ui.expanded_inference_inputs.contains(&digest);
        let toggle_digest = digest.clone();
        items.push(Box::new(
            button(
                if expanded {
                    "Hide generation inputs"
                } else {
                    "Inspect generation inputs"
                },
                move |ui: &mut ConsultationUi, _| {
                    if !ui.expanded_inference_inputs.remove(&toggle_digest) {
                        ui.expanded_inference_inputs.insert(toggle_digest.clone());
                    }
                },
            )
            .attr("data-key", format!("inference-inspect:{digest}")),
        ));
        if expanded {
            items.push(Box::new(
                el::<_, ConsultationUi, ()>("p", method).attr("class", "inference-inputs"),
            ));
            items.push(Box::new(
                el::<_, ConsultationUi, ()>("p", record.inputs.context_text.clone())
                    .attr("class", "inference-inputs"),
            ));
            items.push(Box::new(
                el::<_, ConsultationUi, ()>(
                    "p",
                    serde_json::to_string(&record.generation).expect("generation serializes"),
                )
                .attr("class", "inference-inputs"),
            ));
        }
        children.push(Box::new(
            el::<_, ConsultationUi, ()>("article", items).attr(
                "class",
                match record.generation {
                    Generation::Formula { .. } => "generated-inference formula-inference",
                    Generation::Model { .. } => "generated-inference model-inference",
                },
            ),
        ));
    }
    Box::new(
        el::<_, ConsultationUi, ()>("section", children)
            .attr("class", "reading-inferences")
            .attr("data-key", "reading-inferences")
            .attr("aria-label", "Generated reflection prompts"),
    )
}
