## **task_id: TP-20251106-7**

status: in-progress

## **Agent Operational Protocol**

**This protocol governs all AI agent actions against this task plan.**

1. **Single Source of Record:** This TASK_PLAN.md is the single source of record. You (the AI agent) MUST update this plan *before and after* every operation.
2. **Mandatory Sub-Tasking:** For any non-trivial task (e.g., implementing a new file, researching an API, defining a struct), you MUST follow this workflow:
  1. **Create Sub-Task:** Create a new, separate task document for that specific item (e.g., SUB_TASK_5_3_InMemoryTaskAdapter.md).
  2. **Detail Plan:** This new sub-task document must contain a high-resolution plan for *that item only* (e.g., code to be written, tests required, research findings).
  3. **Link:** Update the main plan item to link to this new sub-task document.
    * **Example:** - [ ] 5.3. Define InMemoryTaskAdapter... (See: SUB_TASK_5_3_InMemoryTaskAdapter.md)
3. **Completion Workflow:**
  1. Execute the work as detailed in the sub-task document.
  2. Once the work is complete and verified, you MUST return to *this* main TASK_PLAN.md.
  3. Mark the main plan item as complete (- [x]).
  4. Update the ## Current Step section in this document to reflect the next task.

# **Task: Create transcript_processor Crate and Pipeline**

## **Description**

Build a new Rust crate, transcript_processor, that uses a Hexagonal Architecture (enforced by the Squillo/hexser crate) to parse unstructured meeting transcripts. The pipeline must be able to use:

1. The rig crate (with Ollama) for schema-enforced extraction.
2. The candle framework (with Phi-2) for embedded, in-process extraction.

The pipeline will extract structured data (Action Items) and **persist them as Tasks in a robust, history-aware task management system.**

**Project Note:** It is critically important to use the Context7 MCP server for all operations involving specific, non-standard libraries or frameworks to ensure compatibility and centralized management.

## **Plan**

* [x] 1. Decompose low-resolution goal into a high-resolution task plan.
* [ ] 2. **Phase 1: Project & Domain Setup**
  * [ ] 2.1. Initialize the new Rust library crate: transcript_processor.
  * [ ] 2.2. Update transcript_processor/Cargo.toml to include dependencies:
    * **rig stack:**
      * rig-core (for Extractor, Agent, OllamaModel)
    * **candle stack:**
      * candle-core
      * candle-nn
      * candle-transformers
      * tokenizers
      * hf-hub (for fetching models)
      * anyhow (for candle error handling)
    * **Core:**
      * serde (with derive feature)
      * serde_json (for candle adapter parsing)
      * schemars (with derive feature)
      * tokio (with full feature, for main.rs)
      * hexser (with macros feature)
      * async-trait (for async traits in ports)
    * **Task Management:**
      * uuid (with v4 and serde features)
      * parking_lot (for a performant Mutex in the adapter)
      * chrono (with serde feature, for timestamps)
* [ ] 3. **Phase 2: Define Core Domain (The Hexagon)**
  * (Tasks 3.1 - 3.10 remain unchanged from original plan)
  * [ ] 3.1. Create module file: src/domain/mod.rs.
  * [ ] 3.2. Define ActionItem struct in src/domain/action_item.rs.
  * [ ] 3.3. Define ChecklistItem struct in src/domain/checklist_item.rs.
  * [ ] 3.4. Define TranscriptAnalysis struct in src/domain/transcript_analysis.rs.
  * [ ] 3.5. Define TaskStatus enum in src/domain/task_status.rs.
  * [ ] 3.6. Define Task struct in src/domain/task.rs.
  * [ ] 3.7. Define TaskRevision struct in src/domain/task_revision.rs.
  * [ ] 3.8. Define TaskSortKey enum in src/domain/task_sort_key.rs.
  * [ ] 3.9. Define SortOrder enum in src/domain/sort_order.rs.
  * [ ] 3.10. Update src/lib.rs to declare and publicly export all new domain types.
* [ ] 4. **Phase 3: Define Application Layer (Ports & Use Cases)**
  * (Tasks 4.1 - 4.8 remain unchanged from original plan)
  * [ ] 4.1. Create module file: src/application/mod.rs.
  * [ ] 4.2. Create module file: src/application/ports/mod.rs.
  * [ ] 4.3. Define TranscriptExtractorPort in src/application/ports/transcript_extractor_port.rs.
  * [ ] 4.4. Define TaskRepositoryPort in src/application/ports/task_repository_port.rs.
  * [ ] 4.5. Create module file: src/application/use_cases/mod.rs.
  * [ ] 4.6. Define ProcessTranscriptUseCase struct in src/application/use_cases/process_transcript.rs.
  * [ ] 4.7. Define ManageTaskUseCase struct in src/application/use_cases/manage_task.rs.
  * [ ] 4.8. Update src/lib.rs to declare and export all ports and use cases.
* [ ] 5. **Phase 4: Define Adapters Layer (Implementations)**
  * [ ] 5.1. Create module file: src/adapters/mod.rs.
  * [ ] 5.2. Define OllamaExtractorAdapter struct in src/adapters/ollama_adapter.rs. (Implements TranscriptExtractorPort using rig-core).
  * [ ] 5.3. Define CandleExtractorAdapter struct in src/adapters/candle_adapter.rs.
    * Implement TranscriptExtractorPort.
    * pub fn new() constructor will:
      * Use hf-hub to fetch the Phi-3.5 GGUF model and tokenizer.
      * Load the model and tokenizer into memory using candle.
    * extract_analysis method will:
      * Run the candle model pipeline with the transcript and a JSON-formatted prompt.
      * Receive raw text output from the model.
      * Attempt to parse the raw text into TranscriptAnalysis using serde_json::from_str.
      * Return the parsed struct or an error string if parsing fails.
  * [ ] 5.4. Define InMemoryTaskAdapter struct in src/adapters/in_memory_task_adapter.rs.
    * Implements TaskRepositoryPort using parking_lot::Mutex HashMaps for tasks and revisions.
  * [ ] 5.5. Update src/lib.rs to declare and export the adapters module.
* [ ] 6. **Phase 5: Model & Environment Setup (Agent Task)**
  * [ ] 6.1. **Ollama:** Agent must ensure microsoft/Phi-3-mini-instruct is available in the local Ollama service (e.g., ollama pull phi3:mini).
  * [ ] 6.2. **Candle:** Agent must ensure the GGUF-formatted Phi-3.5-mini-instruct model and its tokenizer are available for download via hf-hub.
* [ ] 7. **Phase 6: Define Infrastructure Layer (Wiring)**
  * [ ] 7.1. Create src/main.rs binary.
  * [ ] 7.2. Implement the main function to:
    * Instantiate adapters (InMemoryTaskAdapter and *one* of the Extractor adapters, e.g., OllamaExtractorAdapter).
    * Instantiate use cases with the Arcs.
    * Run the full demo pipeline (process, get, update, get history).
    * **Note:** The main.rs should be structured to easily swap the OllamaExtractorAdapter with the CandleExtractorAdapter to demonstrate the port/adapter pattern.
* [ ] 8. **Phase 7: Validation**
  * [ ] 8.1. Add doc tests to Domain structs.
  * [ ] 8.2. Add unit tests for use cases using mock implementations of the ports.
  * [ ] 8.3. Manually run cargo run and verify console output for *both* the Ollama and Candle adapter implementations (e.g., using feature flags).

## **Current Step**

* **Action:** Guard against CUDA build failures on macOS when running workspace-wide tests; provide platform-aware commands.
* **Details:**
  - Clarified build error cause and guidance in transcript_processor/build.rs (mentions `--all-features` on macOS and corrections).
  - Updated README Troubleshooting to explicitly warn against `cargo test --all-features` on macOS and provide per-package CPU/Metal/CUDA commands.
  - Retained existing GPU auto-detection, features (metal/cuda), and Cargo aliases in .cargo/config.toml.

Validation steps:
- macOS (CPU): cargo test -p transcript_processor -- --nocapture
- macOS (Metal): CANDLE_DEVICE=metal cargo test -p transcript_processor --features metal -- --nocapture
- Linux/Windows (CUDA): CANDLE_DEVICE=cuda cargo test -p transcript_processor --features cuda -- --nocapture
- Avoid: `cargo test --all-features` on macOS (will attempt CUDA and fail nvcc).

Notes:
- For workspace-wide runs on macOS, test each crate separately and enable features per-crate (use `-p transcript_processor`).

## **Blockers**

* None.

## **CRITICAL: Candle Adapter Model Selection**

**MANDATE: The Candle adapter MUST use Phi-3.5-mini-instruct (`microsoft/Phi-3.5-mini-instruct`) via the `phi3` model in candle-transformers 0.9.2-alpha.1.**

**Justification (Context7 MCP Verified):**
- The `candle_transformers::models::phi3` module is present in `candle-transformers@0.9.2-alpha.1` and implements the Phi-3 family (Config, Model, forward with `seqlen_offset`).
- `microsoft/Phi-3.5-mini-instruct` provides a two-shard safetensors model compatible with this module (config fields such as `rms_norm_eps`, `num_key_value_heads`, and `rope_scaling` match the `phi3::Config`).
- Prior deserialization errors were due to using the Phi-2 `phi` module against a Phi-3.5 config; switching to `phi3` resolves this structural mismatch.

**Model Details:**
- Model: `microsoft/Phi-3.5-mini-instruct`
- Size: ~7.6GB (2 sharded safetensors files)
- Files: `model-00001-of-00002.safetensors`, `model-00002-of-00002.safetensors`
- Tokenizer: `tokenizer.json`

**Operational Notes:**
- First run downloads and caches shards via `hf-hub`.
- Generation uses `phi3::Model::forward(&input_ids, seqlen_offset)` and greedy decoding by default.

**DO NOT:**
- Fall back to `microsoft/phi-2` in Candle unless reverting the adapter to use `models::phi` again.
- Mix `phi3` configs with `phi` module types.

**If Future Model Upgrade is Needed:**
1. Use Context7 MCP to review `candle-transformers` release notes for Phi-4/Phi-4-mini support.
2. Verify the target model's config fields against the matching module (phi3/phi4) in the crate.
3. Prototype loading and a minimal forward pass before changing tests and docs.

**Last Verified:** 2025-11-07T09:15:00Z
