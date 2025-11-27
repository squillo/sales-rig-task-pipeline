# transcript_extractor

Library crate that defines the domain and ports for extracting structured ActionItems from unstructured transcripts, along with reference adapters. Designed for clarity and testability using hexagonal architecture and the HEXSER framework.

## Knowledge Graph

- Crate: transcript_extractor
  - lib (crate root)
    - mod domain
      - action_item (Struct: ActionItem)
      - transcript_analysis (Struct: TranscriptAnalysis)
    - mod ports
      - transcript_extractor_port (Trait: TranscriptExtractorPort)
    - mod adapters
      - ollama_adapter (Struct: OllamaTranscriptExtractorAdapter; implements TranscriptExtractorPort)

## How it fits in

- The `transcript_processor` app depends on this crate for the extraction port and domain types.
- Adapters in `transcript_processor` offer additional implementations (Candle, Mistral.rs, Rig/OpenAI) that produce the same ActionItem schema used here.

## Usage (Library)

Typical usage in applications involves selecting an adapter that implements `TranscriptExtractorPort`, invoking it with raw transcript text, and obtaining a vector of `ActionItem` values.

Example pseudo-code (paths fully qualified per repository guidelines):

```rust
async fn extract(text: &str) -> std::result::Result<std::vec::Vec<transcript_extractor::domain::action_item::ActionItem>, std::string::String> {
    let adapter = transcript_extractor::adapters::ollama_adapter::OllamaTranscriptExtractorAdapter::new(
        std::string::String::from("llama3.2"),
    );
    adapter.extract_action_items(text).await
}
```

Note: Other adapters (Candle, Mistral, Rig) live in the `transcript_processor` crate under `src/adapters` and implement compatible extraction logic mapping to `ActionItem`.

## Documentation

- Tutorial: ../docs/TUTORIAL.md
- Architecture: ../docs/ARCHITECTURE.md
- Orchestration Flow: ../docs/FLOW.md

## Testing

- Run unit tests inside this crate directory:

```bash
cd transcript_extractor
cargo test
```

## Notes

- Follow the repository coding standards: fully qualified paths and no `use` statements inside crate modules where enforced.
- Keep adapters single-responsibility and avoid business logic in ports or domain types.
