# C7 bounded local model setup

Model: HuggingFaceTB/SmolLM2-135M-Instruct, revision
12fd25f77366fa6b3b4b768ec3050bf629380bac, Apache-2.0.
Official source: https://huggingface.co/HuggingFaceTB/SmolLM2-135M-Instruct

The host configuration CLEROMANCY_MODEL_DIR points to a local directory
containing config.json, tokenizer.json, model.safetensors and
cleromancy-model.json. The manifest records model_id, revision and the SHA-256
artifacts map. Every artifact is verified before executing the shared ESP
NdArray decoder. Generation performs no network calls or implicit downloads.
The default application build includes local-inference; portable validation
contains no model runtime. Both the request and the output are sealed separately
from the reading. Reopening never requires model files or ESP execution.

The model request uses greedy generation, a 64-token cap, the fixed versioned
chat template, and the exact saved context excerpts. These prompts are
reflection examples and initially enter Alembic Recent memory. Keep is explicit.
The bounded model choice is not a general model manager or a quality claim.
