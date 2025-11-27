# Rig Task Pipeline: End-to-End Tutorial

This tutorial walks you through installing prerequisites, running the transcript processing pipeline, switching adapters/backends (Ollama, Candle, Mistral.rs, Rig/OpenAI), and orchestrating tasks with the graph-based flow. It is designed for fast success on macOS and Linux.

- Time: 20–45 minutes (first run depends on model downloads)
- Audience: Developers evaluating or extending the pipeline

## Prerequisites

- Rust (stable; Edition 2024)
- macOS (Apple Silicon recommended) or Linux
- Recommended: 16GB+ RAM for CPU runs, more for larger models
- Network access to download models (HF Hub, Ollama)

## Repository Layout (Workspace)

```
rig-task-pipeline/
├── transcript_processor/     # App crate (CLI) – compose adapters, run pipeline
├── transcript_extractor/     # Library crate – domain + ports + extraction DTOs
├── task_manager/             # Library crate – domain + ports + persistence adapters
├── task_orchestrator/        # Library crate – graph-based orchestration (flow)
├── docs/                     # Documentation and diagrams
└── README.md                 # Root overview
```

Key principle: Run commands inside the target crate directory (per-crate features). Avoid running workspace-wide tests with heavy features unless you know what you’re doing.

## 1. Quick Start (Ollama Adapter)

The fastest path uses a local Ollama server for LLM inference.

1) Install/start Ollama

```bash
# macOS (Homebrew)
brew install ollama
ollama serve &

# Linux
curl -fsSL https://ollama.com/install.sh | sh
ollama serve &

# Pull a model (example)
ollama pull llama3.2
```

2) Run the demo app

```bash
cd transcript_processor
cargo run
```

- The default configuration uses the Ollama adapter.
- Integration tests (optional):

```bash
cd transcript_processor
cargo test --test integration_ollama_five_person_conversation -- --nocapture
```

## 2. Embedded Inference (Candle Adapter)

For on-device inference without an external service, use the embedded Candle adapter with Phi‑3.5‑mini‑instruct. First run downloads model weights via HF Hub.

- CPU (default):

```bash
cd transcript_processor
cargo test --test integration_candle_complex_conversation -- --nocapture
```

- Apple Silicon (Metal):

```bash
cd transcript_processor
cargo test --features metal --test integration_candle_five_person_conversation -- --nocapture
EXTRACTOR=candle CANDLE_DEVICE=metal cargo run --features metal
```

- NVIDIA CUDA (Linux/Windows with CUDA toolkit/drivers installed):

```bash
cd transcript_processor
cargo test --features cuda --test integration_candle_complex_conversation -- --nocapture
EXTRACTOR=candle CANDLE_DEVICE=cuda cargo run --features cuda
```

Notes:
- On macOS, do not use `--all-features` at the workspace root. Prefer per-crate feature flags.
- The Candle adapter auto-detects an available GPU backend and falls back to CPU if initialization fails. Adjust via `CANDLE_DEVICE=cpu|metal|cuda`.

## 3. Faster Local Inference (Mistral.rs Adapter)

Use Mistral.rs via an OpenAI-compatible local server for speed and lower memory usage (quantization, paged attention).

```bash
# Start a local mistralrs-server (see docs in RESEARCH_MISTRAL_RS.md)

cd transcript_processor
EXTRACTOR=mistral cargo run --features mistral_rs
EXTRACTOR=mistral cargo test --features mistral_rs -- --nocapture
```

Environment (defaults):
- `MISTRALRS_BASE_URL` = http://127.0.0.1:8080
- `MISTRALRS_MODEL` = microsoft/Phi-3.5-mini-instruct

## 4. Rig/OpenAI Provider (Rig Adapter)

Use Rig (rig-core) to call an OpenAI model for schema-enforced extraction.

```bash
cd transcript_processor
EXTRACTOR=rig OPENAI_API_KEY=sk_... cargo run --features rig_adapter
EXTRACTOR=rig OPENAI_API_KEY=sk_... cargo test --features rig_adapter -- --nocapture
```

Optional: `RIG_OPENAI_MODEL` (default `gpt-4o-mini`).

## 5. Orchestrating Tasks with the Graph Flow

The `task_orchestrator` crate assembles a simple flow with a semantic router, enhancement, comprehension test, and a check step that loops until pass/fail.

```rust
#[tokio::main]
async fn main() -> Result<(), String> {
    let ai = transcript_extractor::domain::action_item::ActionItem {
        title: String::from("Write release notes for v0.1"),
        assignee: None,
        due_date: None,
    };
    let task = task_manager::domain::task::Task::from_action_item(&ai, None);
    let updated = task_orchestrator::use_cases::run_task_with_flow::run_task_with_flow(
        "llama3.1",
        "short_answer",
        task,
    ).await?;
    println!("Updated status: {:?}", updated.status);
    Ok(())
}
```

Run crate tests:

```bash
cd task_orchestrator
cargo test
```

## 6. End-to-End: From Transcript to Tasks

1) Provide or load a transcript (plain text)
2) Choose an extractor adapter (Ollama, Candle, Mistral, Rig)
3) Extract ActionItems → map to Tasks → persist with Task Manager (SQLite or in-memory)
4) Orchestrate tasks through the graph flow

Example (pseudo shell):

```bash
# 1) Extract action items
cd transcript_processor
EXTRACTOR=ollama cargo run --quiet < transcript.txt > action_items.json

# 2) Convert to Tasks and persist (library usage or a future CLI subcommand)
#    See task_manager README for examples and SQLx SQLite adapter.

# 3) Run orchestrator to enhance and validate tasks
cd ../task_orchestrator
cargo test -q
```

## 7. Troubleshooting

- CUDA build error (`nvcc` not found) on macOS: do not run `--all-features` at the workspace root; use per-crate flags.
- Large model downloads: See `README.md` for Candle model caching and prefill knobs.
- Ollama service not reachable: confirm `ollama serve` is running and model exists (`ollama list`).

## 8. FAQ

- Q: Can I swap adapters at runtime? A: Yes, set `EXTRACTOR=ollama|candle|mistral|rig`.
- Q: Where are diagrams? A: See ARCHITECTURE.md and FLOW.md in this docs folder.
- Q: Where do I run tests? A: Inside the target crate directory.
