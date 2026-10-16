// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Host configuration supplies a local artifact directory. Execution is ESP's;
//! loading and generation never download weights or send reading data away.
use super::*;
use esp::infer::decoder::{DecoderDevice, DecoderProvider};
use esp::infer::{GenerationRequest, InferenceProvider};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, path::Path};

#[derive(Deserialize)]
struct Manifest {
    model_id: String,
    revision: String,
    artifacts: BTreeMap<String, String>,
}
pub fn configured_label() -> Option<String> {
    let directory = std::env::var_os("CLEROMANCY_MODEL_DIR")?;
    let manifest: Manifest = serde_json::from_slice(
        &std::fs::read(std::path::Path::new(&directory).join("cleromancy-model.json")).ok()?,
    )
    .ok()?;
    Some(format!(
        "Configured local model: {}. Reading data stays on this device; generation is greedy, with a 64-token limit.",
        manifest.model_id
    ))
}
pub fn generate(
    inputs: InferenceInputs,
    directory: &Path,
) -> Result<GeneratedInference, InferenceError> {
    let fail = |e: String| InferenceError(format!("local ESP model: {e}"));
    let manifest: Manifest = serde_json::from_slice(
        &std::fs::read(directory.join("cleromancy-model.json")).map_err(|e| fail(e.to_string()))?,
    )
    .map_err(|e| fail(e.to_string()))?;
    let mut buffers = BTreeMap::new();
    let mut artifacts = BTreeMap::new();
    for name in ["config.json", "tokenizer.json", "model.safetensors"] {
        let bytes = std::fs::read(directory.join(name)).map_err(|e| fail(e.to_string()))?;
        let hash = format!("{:x}", Sha256::digest(&bytes));
        require(
            manifest.artifacts.get(name) == Some(&hash),
            "local model bytes differ from the configured artifact identity",
        )?;
        artifacts.insert(name.into(), hash);
        buffers.insert(name, bytes);
    }
    let model = ModelIdentity {
        model_id: manifest.model_id,
        revision: manifest.revision,
        loader: "esp-burn-ndarray".into(),
        artifacts,
    };
    // Validate identity and request before invoking the shared runtime.
    GeneratedInference::model(inputs.clone(), model.clone(), "pending".into()).validate()?;
    #[allow(deprecated)] // Match the pinned ESP NdArray CPU contract.
    let provider = DecoderProvider::from_bytes(
        &buffers["config.json"],
        &buffers["tokenizer.json"],
        &buffers["model.safetensors"],
        &model.model_id,
        &model.loader,
        &DecoderDevice::ndarray(),
    )
    .map_err(|e| fail(e.to_string()))?;
    let request = GenerationRequest {
        prompt: inputs.prompt(),
        max_tokens: 64,
        temperature: 0.0,
        ..Default::default()
    };
    let output = provider
        .generate(&request)
        .map_err(|e| fail(e.to_string()))?;
    let record = GeneratedInference::model(inputs, model, output);
    record.validate()?;
    Ok(record)
}
