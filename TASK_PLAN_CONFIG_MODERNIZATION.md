---
task_id: CONFIG-MODERN-20251203
status: planning
created: 2025-12-03T07:30:00Z
priority: high
---

# Task: Modernize Configuration System & Editor

## Problem Statement

The current configuration system has **multiple incompatible structures**, lacks API key support, and the Config Editor UI doesn't reflect reality. We need a unified, extensible config system that supports:

- **Multiple LLM providers**: OpenAI, Anthropic (Claude), Ollama, Mistral, Groq, etc.
- **API key management**: Secure storage and masking in UI
- **Task tool slots**: main, research, fallback, embedding, vision, **chat_agent** (new!)
- **Provider-specific settings**: Base URLs, timeouts, retry policies
- **Future extensibility**: Easy to add new providers and features

## Current State Analysis

### Three Conflicting Structures

**1. Setup Wizard Output** (`rigger_cli/src/commands/tui.rs:6498`):
```json
{
  "provider": "ollama",
  "task_tools": {
    "main": { "provider": "ollama", "model": "llama3.2" },
    "research": { "provider": "ollama", "model": "llama3.2" },
    "fallback": { "provider": "ollama", "model": "llama3.2" },
    "embedding": { "provider": "ollama", "model": "nomic-embed-text" },
    "vision": { "provider": "ollama", "model": "llava:latest" }
  },
  "database_url": "sqlite:.rigger/tasks.db"
}
```

**2. OrchestratorConfig Struct** (`task_orchestrator/src/infrastructure/config.rs:24`):
```rust
pub struct OrchestratorConfig {
    pub model_roles: HashMap<String, String>,  // "router" -> "phi3"
    pub quantization: HashMap<String, String>,
    pub providers: ProviderConfig,
    pub performance: PerformanceConfig,
    pub tui: TuiConfig,
}
```

**3. Actual Runtime Config** (`.rigger/config.json`):
```json
{
  "database_url": "sqlite:.rigger/tasks.db",
  "model": {
    "fallback": "llama3.2",
    "main": "llama3.2",
    "research": "llama3.2"
  },
  "provider": "ollama"
}
```

### Missing Features
- ❌ No API key fields for OpenAI, Anthropic, Groq, etc.
- ❌ No chat_agent slot (currently uses env vars only)
- ❌ No provider base URLs (except Ollama)
- ❌ No timeout/retry configuration
- ❌ Config editor shows wrong structure
- ❌ No config migration strategy

---

## Plan

### Phase 1: Design Unified Config Schema (2-3 hours)

**1.1. Define Canonical Config Structure**
- [ ] Design `RiggerConfig` struct in new `rigger_core/src/config.rs`
- [ ] Include:
  - `providers: HashMap<String, ProviderConfig>` (openai, anthropic, ollama, mistral, groq)
  - `task_slots: TaskSlotConfig` (main, research, fallback, embedding, vision, **chat_agent**)
  - `database: DatabaseConfig`
  - `performance: PerformanceConfig`
  - `tui: TuiConfig`
  - `api_keys: ApiKeyConfig` (encrypted/secured)
- [ ] Document JSON schema with examples
- [ ] Design backwards compatibility layer

**1.2. Define Provider Config**
```rust
pub struct ProviderConfig {
    pub provider_type: ProviderType,  // OpenAI, Anthropic, Ollama, etc.
    pub base_url: Option<String>,
    pub api_key_env: Option<String>,  // Which env var to read (e.g., "OPENAI_API_KEY")
    pub timeout_seconds: u64,
    pub max_retries: usize,
    pub default_model: String,
}

pub enum ProviderType {
    OpenAI,
    Anthropic,  // Claude!
    Ollama,
    Mistral,
    Groq,
    Cohere,
    Custom(String),
}
```

**1.3. Define Task Slot Config**
```rust
pub struct TaskSlotConfig {
    pub main: TaskSlot,
    pub research: TaskSlot,
    pub fallback: TaskSlot,
    pub embedding: TaskSlot,
    pub vision: TaskSlot,
    pub chat_agent: TaskSlot,  // NEW!
}

pub struct TaskSlot {
    pub provider: String,  // Key into providers HashMap
    pub model: String,
    pub enabled: bool,
}
```

**1.4. API Key Management**
- [ ] Define `ApiKeyConfig` with encrypted storage strategy
- [ ] Support env var fallback (`OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, etc.)
- [ ] Mask API keys in UI (show `sk-...abc123`)
- [ ] Never write API keys to config file (env vars only)

---

### Phase 2: Implement New Config System (3-4 hours)

**2.1. Create `rigger_core` Crate** (if doesn't exist)
- [ ] Add `rigger_core` to workspace
- [ ] Move shared config types here
- [ ] Add dependencies: `serde`, `serde_json`, `thiserror`

**2.2. Implement RiggerConfig**
- [ ] `rigger_core/src/config.rs` - main config struct
- [ ] `rigger_core/src/config/provider.rs` - provider configs
- [ ] `rigger_core/src/config/task_slots.rs` - task slot configs
- [ ] `rigger_core/src/config/migration.rs` - migration from old configs
- [ ] Implement `RiggerConfig::load(path)` with auto-migration
- [ ] Implement `RiggerConfig::save(path)`
- [ ] Implement `RiggerConfig::validate()` with helpful errors

**2.3. Migration Strategy**
```rust
impl RiggerConfig {
    pub fn load_with_migration(path: &Path) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)?;
        let raw: serde_json::Value = serde_json::from_str(&content)?;

        // Detect config version
        if raw.get("task_tools").is_some() {
            // V2 format (setup wizard output)
            Self::migrate_from_v2(raw)
        } else if raw.get("model_roles").is_some() {
            // V1 format (OrchestratorConfig)
            Self::migrate_from_v1(raw)
        } else {
            // V0 format (legacy simple format)
            Self::migrate_from_v0(raw)
        }
    }
}
```

---

### Phase 3: Update Config Editor UI (4-5 hours)

**3.1. Redesign Config Editor Layout**

New structure (hierarchical view):
```
⚙️  Configuration Editor

📦 PROVIDERS
  ▶ OpenAI
      Base URL: https://api.openai.com/v1
      API Key: [env: OPENAI_API_KEY] ✓ Set
      Default Model: gpt-4o-mini
      Timeout: 60s
  ▶ Anthropic (Claude)
      Base URL: https://api.anthropic.com/v1
      API Key: [env: ANTHROPIC_API_KEY] ✗ Not Set
      Default Model: claude-sonnet-4-5
      Timeout: 60s
  ▶ Ollama
      Base URL: http://localhost:11434
      Timeout: 120s

🔧 TASK SLOTS
  ▶ Main (Task Decomposition)
      Provider: ollama
      Model: llama3.2
      Enabled: ✓
  ▶ Research (Web/Artifact Search)
      Provider: ollama
      Model: llama3.2
      Enabled: ✓
  ▶ Chat Agent (Interactive)
      Provider: openai
      Model: gpt-4o-mini
      Enabled: ✓

💾 DATABASE
  URL: sqlite:.rigger/tasks.db

🎨 UI
  Theme: default
  Auto-refresh: 30s
```

**3.2. Implement New Editor State**
- [ ] Replace flat `config_editor_items: Vec<(String, String)>` with tree structure
- [ ] Add `ConfigEditorNode` enum: `Section`, `Provider`, `TaskSlot`, `Field`
- [ ] Add expand/collapse state tracking
- [ ] Add field validation (e.g., valid URLs, model names)

**3.3. Implement Editor Navigation**
- [ ] Up/Down: Navigate within current section
- [ ] Right/Enter: Expand section or edit field
- [ ] Left/Esc: Collapse section or cancel edit
- [ ] Tab: Next field
- [ ] Shift+Tab: Previous field

**3.4. Implement Special Field Types**
- [ ] Text input (model names, URLs)
- [ ] Dropdown (provider selection)
- [ ] Toggle (enabled/disabled)
- [ ] API Key display (masked `sk-...abc123`, link to env var)
- [ ] Validation feedback (red X or green checkmark)

**3.5. Add Provider Quick Actions**
- [ ] 'a' - Add new provider
- [ ] 'd' - Delete provider (with confirmation)
- [ ] 't' - Test provider connection (ping API)
- [ ] 's' - Save config
- [ ] 'r' - Reload from disk

---

### Phase 4: Integrate with Existing Systems (2-3 hours)

**4.1. Update Setup Wizard**
- [ ] Modify wizard to generate new config format
- [ ] Add API key prompt step (optional)
- [ ] Add Anthropic/Claude as provider choice
- [ ] Update confirmation screen with new structure

**4.2. Update LLM Chat Agent**
- [ ] Read `chat_agent` slot from config
- [ ] Support provider switching (OpenAI, Anthropic, Ollama)
- [ ] Fall back to env vars if not configured
- [ ] Show current provider/model in chat dialog

**4.3. Update Task Orchestrator**
- [ ] Modify adapters to read from new config
- [ ] Support provider switching per task slot
- [ ] Update `ProviderFactory` to use new `ProviderConfig`

**4.4. Update CLI Commands**
- [ ] `rig config show` - Display current config (masked API keys)
- [ ] `rig config validate` - Check config validity
- [ ] `rig config migrate` - Explicitly migrate old config
- [ ] `rig config test <provider>` - Test provider connectivity

---

### Phase 5: Add Anthropic/Claude Support (2-3 hours)

**5.1. Create AnthropicAdapter**
- [ ] `task_orchestrator/src/adapters/anthropic_adapter.rs`
- [ ] Implement using `rig` crate (already has Anthropic support!)
- [ ] Support streaming responses
- [ ] Implement tool calling

**5.2. Add Anthropic to ProviderFactory**
- [ ] Update factory to create Anthropic adapters
- [ ] Map config to adapter construction
- [ ] Test with Claude models

**5.3. Add Claude Models to UI**
- [ ] Add model list: `claude-opus-4`, `claude-sonnet-4-5`, `claude-haiku-4`
- [ ] Update setup wizard provider selection
- [ ] Add to config editor dropdowns

---

### Phase 6: Testing & Documentation (2-3 hours)

**6.1. Unit Tests**
- [ ] Config serialization/deserialization
- [ ] Migration from V0, V1, V2 formats
- [ ] Config validation
- [ ] Provider config parsing

**6.2. Integration Tests**
- [ ] Setup wizard → new config format
- [ ] Config editor → save → reload
- [ ] Provider switching in chat agent
- [ ] Multi-provider task execution

**6.3. Documentation**
- [ ] Update README with new config structure
- [ ] Document provider setup (API keys, base URLs)
- [ ] Add migration guide
- [ ] Add troubleshooting section

**6.4. User-Facing Changes**
- [ ] Add config migration notification on first run
- [ ] Show config validation errors with helpful messages
- [ ] Add "Getting Started with Claude" guide

---

## Success Criteria

- ✅ Single, canonical config structure
- ✅ Support for OpenAI, Anthropic/Claude, Ollama, Mistral, Groq
- ✅ API key management via env vars (never in config file)
- ✅ Config editor reflects actual structure
- ✅ Automatic migration from old configs
- ✅ Chat agent can use Claude models
- ✅ All tests pass
- ✅ Documentation complete

---

## Future Extensions (Phase 7+)

**Provider Support**
- [ ] Groq (fast inference)
- [ ] Cohere (embeddings)
- [ ] Google Gemini
- [ ] Azure OpenAI
- [ ] AWS Bedrock
- [ ] Local LLaMA.cpp server
- [ ] vLLM server
- [ ] Text Generation Inference (TGI)

**Advanced Features**
- [ ] Per-slot temperature/top_p settings
- [ ] Token usage tracking per provider
- [ ] Cost estimation
- [ ] Rate limiting per provider
- [ ] Fallback chains (if OpenAI fails → try Claude → try Ollama)
- [ ] Model aliases (map "gpt4" → "gpt-4o-mini")
- [ ] Prompt templates per slot
- [ ] Config presets ("cost-optimized", "quality-focused", "local-only")

**Security**
- [ ] Encrypted API key storage (OS keychain)
- [ ] Config file encryption option
- [ ] Audit log for API key access
- [ ] Per-user configs (multi-tenant support)

**UI Enhancements**
- [ ] Config diff viewer (show changes before save)
- [ ] Provider health dashboard (latency, success rate)
- [ ] Model switching hot key (Ctrl+P?)
- [ ] Import/export config profiles
- [ ] Cloud config sync (optional)

---

## Notes

- **Backwards Compatibility**: Auto-migration ensures existing users aren't disrupted
- **API Key Security**: Never write API keys to config.json, always use env vars
- **Extensibility**: `ProviderType::Custom(String)` allows user-defined providers
- **Validation**: Catch misconfigurations early with helpful error messages
- **Testing**: Comprehensive tests prevent regressions during migration

---

## Estimated Timeline

- **Phase 1** (Design): 2-3 hours
- **Phase 2** (Implementation): 3-4 hours
- **Phase 3** (UI): 4-5 hours
- **Phase 4** (Integration): 2-3 hours
- **Phase 5** (Claude Support): 2-3 hours
- **Phase 6** (Testing/Docs): 2-3 hours

**Total**: 15-21 hours (2-3 days of focused work)

---

## Current Step

**Status**: Planning complete, ready for Phase 1 implementation

**Next Action**: Create design document for `RiggerConfig` structure with full JSON schema

**Blockers**: None
