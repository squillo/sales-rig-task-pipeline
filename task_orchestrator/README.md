# task_orchestrator

Task Orchestrator crate: coordinates graph-based processing of tasks using a "Brain" (graph/state/nodes) and "Muscle" (ports/adapters). It integrates schema-enforced extraction via rig-core and now runs an assembled workflow on the rs-graph-llm runtime (graph-flow).

## Knowledge Graph

- Crate: task_orchestrator
  - lib (crate root)
    - fn crate_version() -> &'static str
  - Modules
    - ports (Port layer)
      - task_enhancement_port (Trait: generate_enhancement)
      - comprehension_test_port (Trait: generate_comprehension_test)
    - adapters (Adapter layer)
      - ollama_enhancement_adapter (Derives hexser::HexAdapter; implements TaskEnhancementPort)
      - ollama_comprehension_test_adapter (Derives hexser::HexAdapter; implements ComprehensionTestPort)
      - rig_prd_parser_adapter (Uses rig-core; implements PRDParserPort)
      - rig_task_decomposition_adapter (Uses rig-core; implements TaskDecompositionPort)
      - provider_factory (Factory for creating vendor-agnostic LLM adapters)
    - graph (Brain components)
      - state (GraphState struct: { task, routing_decision })
      - nodes
        - graph_node (temporary GraphNode trait abstraction)
        - semantic_router_node (routes to "enhance" or "decompose")
        - enhancement_node (calls TaskEnhancementPort, appends to state)
        - comprehension_test_node (calls ComprehensionTestPort, appends to state)
        - check_test_result_node (emits "pass"/"fail" and updates status)
      - orchestrator_graph (placeholder graph struct)
      - build_graph (placeholder -> OrchestratorGraph)
      - build_graph_flow (helper; returns GraphBuilder)
      - flow_integration (graph-flow integration skeleton)
      - flow_shims (Task shims delegating to nodes)
        - semantic_router_task_shim
        - enhancement_task_shim
        - comprehension_test_task_shim
        - check_test_result_task_shim
        - end_task (terminal)
      - assemble_orchestrator_flow (wires the graph: router → enhance → comprehend → check → end/loop)
    - use_cases (Application layer)
      - flow_runner (Legacy shim-based runner)
      - run_task_with_flow (Unified runtime helper using graph_flow::FlowRunner)
      - orchestrator (Facade for running flows)

## How it works

The assembled workflow is built with graph_flow::GraphBuilder and executed with graph_flow::FlowRunner. The state type is crate::graph::state::GraphState.

Flow:
- START → SemanticRouterTaskShim
  - If routing_decision == "enhance" → EnhancementTaskShim; else → EndTask
- EnhancementTaskShim → ComprehensionTestTaskShim → CheckTestResultTaskShim
  - If routing_decision == "pass" → EndTask; else → EnhancementTaskShim (loop)

Routing and updated Task are persisted in graph_flow::Context between tasks.

## Configuration

The task orchestrator supports multiple LLM providers through the ProviderFactory abstraction. You can configure providers using environment variables or programmatically.

### Environment Variables

| Variable | Description | Default | Example |
|----------|-------------|---------|---------|
| `TASK_ORCHESTRATOR_PROVIDER` | LLM provider to use | `ollama` | `ollama`, `openai`, `anthropic` |
| `OLLAMA_MODEL` | Ollama model name | `llama3.1` | `llama3.1`, `qwen2.5` |
| `OPENAI_MODEL` | OpenAI model name | `gpt-4` | `gpt-4`, `gpt-4-turbo` |
| `ANTHROPIC_MODEL` | Anthropic model name | `claude-3-5-sonnet-20241022` | `claude-3-5-sonnet-20241022` |
| `OPENAI_API_KEY` | OpenAI API key (required for OpenAI) | - | `sk-...` |
| `ANTHROPIC_API_KEY` | Anthropic API key (required for Anthropic) | - | `sk-ant-...` |
| `TEST_TYPE` | Comprehension test type | `short_answer` | `short_answer`, `multiple_choice`, `true_false` |

### Provider Status

- **Ollama**: ✅ Fully supported (no API key required)
- **OpenAI**: 🚧 Not yet implemented (placeholder returns error)
- **Anthropic**: 🚧 Not yet implemented (placeholder returns error)

The ProviderFactory validates provider names and API keys at creation time, making it easy to swap providers when OpenAI and Anthropic adapters are implemented.

## Usage: End-to-end examples

### Example 1: Using the Orchestrator facade (Recommended)

The Orchestrator provides a simple, high-level API for running tasks through the orchestration flow:

```rust
#[tokio::main]
async fn main() -> Result<(), String> {
    // Prepare a task (normally derived from a transcript ActionItem)
    let ai = transcript_extractor::domain::action_item::ActionItem {
        title: String::from("Write release notes for v0.1"),
        assignee: None,
        due_date: None,
    };
    let task = task_manager::domain::task::Task::from_action_item(&ai, None);

    // Create orchestrator from environment variables
    let orch = task_orchestrator::use_cases::orchestrator::Orchestrator::from_env()
        .map_err(|e| e.to_string())?;

    // Run the task through orchestration
    let updated = orch.run(task).await?;

    println!("Updated status: {:?}", updated.status);
    println!("Provider: {}, Model: {}", orch.provider(), orch.model());
    Ok(())
}
```

### Example 2: Explicit provider configuration

You can also create the Orchestrator with explicit provider and model:

```rust
#[tokio::main]
async fn main() -> Result<(), String> {
    let ai = transcript_extractor::domain::action_item::ActionItem {
        title: String::from("Implement user authentication"),
        assignee: None,
        due_date: None,
    };
    let task = task_manager::domain::task::Task::from_action_item(&ai, None);

    // Create orchestrator with explicit Ollama configuration
    let orch = task_orchestrator::use_cases::orchestrator::Orchestrator::new(
        "ollama",
        "qwen2.5",
        "multiple_choice",
    ).map_err(|e| e.to_string())?;

    let updated = orch.run(task).await?;

    println!("Task enhanced with {} model", orch.model());
    Ok(())
}
```

### Example 3: Using ProviderFactory directly

For advanced use cases, you can use the ProviderFactory and run_task_with_flow directly:

```rust
#[tokio::main]
async fn main() -> Result<(), String> {
    let ai = transcript_extractor::domain::action_item::ActionItem {
        title: String::from("Design database schema"),
        assignee: None,
        due_date: None,
    };
    let task = task_manager::domain::task::Task::from_action_item(&ai, None);

    // Create a ProviderFactory
    let factory = task_orchestrator::adapters::provider_factory::ProviderFactory::new(
        "ollama",
        "llama3.1",
    ).map_err(|e| e.to_string())?;

    // Run the orchestrator flow using the factory
    let updated = task_orchestrator::use_cases::run_task_with_flow::run_task_with_flow(
        &factory,
        "short_answer",
        task,
    ).await?;

    println!("Updated status: {:?}", updated.status);
    Ok(())
}
```

### Example 4: Swapping providers

When OpenAI and Anthropic adapters are implemented, you can easily swap providers:

```bash
# Use Ollama (default, no API key required)
export TASK_ORCHESTRATOR_PROVIDER=ollama
export OLLAMA_MODEL=llama3.1

# Or use OpenAI (once implemented)
export TASK_ORCHESTRATOR_PROVIDER=openai
export OPENAI_MODEL=gpt-4
export OPENAI_API_KEY=your-key-here

# Or use Anthropic (once implemented)
export TASK_ORCHESTRATOR_PROVIDER=anthropic
export ANTHROPIC_MODEL=claude-3-5-sonnet-20241022
export ANTHROPIC_API_KEY=your-key-here
```

Then use `Orchestrator::from_env()` to automatically pick up the configuration.

**Notes:**
- All examples avoid `use` statements; paths are fully qualified per repository guidelines.
- Choose a model installed in your Ollama instance for best results.
- The ProviderFactory validates provider names and ensures required API keys are set.

## Testing

Run crate tests from the crate directory:
- cd task_orchestrator && cargo test

Workspace-root tests: please run only after approval, as they can trigger optional heavy dependencies. We will request approval before running them in automation.

## Features (Cargo)

- graph_flow: Unified by default (no feature gate).
- sqlite_persistence: Placeholder feature for future persistent session storage integration.
