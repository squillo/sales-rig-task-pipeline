# transcript_processor

Application crate that wires the extraction adapters, persistence, and orchestration into a runnable CLI. It is the recommended entry point for end-to-end demos.

## Knowledge Graph

- Crate: transcript_processor
  - bin: transcript_processor (src/main.rs)
  - application
    - ports
      - transcript_extractor_port (re-exported from transcript_extractor)
  - adapters
    - ollama_adapter (LLM via local Ollama)
    - candle_adapter (embedded inference; CPU/GPU via Metal/CUDA)
    - mistral_adapter (HTTP to mistralrs-server; OpenAI-compatible)
    - mistralrs_embed_adapter (optional embedded mistral.rs)
    - rig_adapter (Rig/OpenAI provider)
    - in_memory_task_adapter (demo persistence)
  - domain
    - action_item, task, task_status, task_sort_key, transcript_analysis (cross-crate mapping helpers)

## Usage

Quick demo with Ollama:

```bash
cd transcript_processor
cargo run
```

Candle (embedded):

```bash
# CPU (default)
cd transcript_processor
EXTRACTOR=candle cargo run

# Apple Metal
a) compile with feature
a) cd transcript_processor && cargo run --features metal
b) select device at runtime: EXTRACTOR=candle CANDLE_DEVICE=metal cargo run --features metal

# CUDA (Linux/Windows with CUDA installed)
cd transcript_processor
EXTRACTOR=candle CANDLE_DEVICE=cuda cargo run --features cuda
```

Mistral.rs via local server:

```bash
cd transcript_processor
EXTRACTOR=mistral cargo run --features mistral_rs
```

Rig/OpenAI (requires OPENAI_API_KEY):

```bash
cd transcript_processor
EXTRACTOR=rig OPENAI_API_KEY=sk_... cargo run --features rig_adapter
```

## Documentation

- Tutorial: ../docs/TUTORIAL.md
- Architecture: ../docs/ARCHITECTURE.md
- Orchestration Flow: ../docs/FLOW.md

## Notes

- Run tests from inside the crate directory; features are per-crate.
- On macOS, avoid `--all-features` at the workspace root to prevent enabling CUDA unintentionally.
