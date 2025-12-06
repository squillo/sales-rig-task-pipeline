//! Task slot configuration.
//!
//! Defines the six task slots (main, research, fallback, embedding, vision, chat_agent)
//! and their provider/model assignments.
//!
//! Revision History
//! - 2025-12-03T07:55:00Z @AI: Create TaskSlotConfig for rigger_core (Phase 2.2 of CONFIG-MODERN-20251203).

/// Configuration for all task slots.
///
/// Each slot represents a different capability in the Rigger system,
/// assigned to specific LLM providers and models.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct TaskSlotConfig {
    /// Primary task decomposition and generation
    #[serde(default = "default_main_slot")]
    pub main: TaskSlot,

    /// Web research and artifact search
    #[serde(default = "default_research_slot")]
    pub research: TaskSlot,

    /// Fallback when main provider fails
    #[serde(default = "default_fallback_slot")]
    pub fallback: TaskSlot,

    /// Generate embeddings for semantic search
    #[serde(default = "default_embedding_slot")]
    pub embedding: TaskSlot,

    /// Image analysis and description
    #[serde(default = "default_vision_slot")]
    pub vision: TaskSlot,

    /// Interactive chat agent with tool calling
    #[serde(default = "default_chat_agent_slot")]
    pub chat_agent: TaskSlot,
}

/// Configuration for a single task slot.
///
/// Links a specific capability to a provider and model.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct TaskSlot {
    /// Provider name (must exist in providers HashMap)
    pub provider: std::string::String,

    /// Model name for this slot
    pub model: std::string::String,

    /// Whether this slot is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Human-readable description
    pub description: std::string::String,

    /// Enable streaming responses (for chat_agent)
    #[serde(default, skip_serializing_if = "std::option::Option::is_none")]
    pub streaming: std::option::Option<bool>,
}

fn default_true() -> bool {
    true
}

fn default_main_slot() -> TaskSlot {
    TaskSlot {
        provider: std::string::String::from("ollama"),
        model: std::string::String::from("llama3.2"),
        enabled: true,
        description: std::string::String::from("Primary task decomposition and generation"),
        streaming: std::option::Option::None,
    }
}

fn default_research_slot() -> TaskSlot {
    TaskSlot {
        provider: std::string::String::from("ollama"),
        model: std::string::String::from("llama3.2"),
        enabled: true,
        description: std::string::String::from("Web research and artifact search"),
        streaming: std::option::Option::None,
    }
}

fn default_fallback_slot() -> TaskSlot {
    TaskSlot {
        provider: std::string::String::from("ollama"),
        model: std::string::String::from("llama3.2"),
        enabled: true,
        description: std::string::String::from("Fallback when main provider fails"),
        streaming: std::option::Option::None,
    }
}

fn default_embedding_slot() -> TaskSlot {
    TaskSlot {
        provider: std::string::String::from("ollama"),
        model: std::string::String::from("nomic-embed-text"),
        enabled: true,
        description: std::string::String::from("Generate embeddings for semantic search"),
        streaming: std::option::Option::None,
    }
}

fn default_vision_slot() -> TaskSlot {
    TaskSlot {
        provider: std::string::String::from("ollama"),
        model: std::string::String::from("llava:latest"),
        enabled: false,
        description: std::string::String::from("Image analysis and description"),
        streaming: std::option::Option::None,
    }
}

fn default_chat_agent_slot() -> TaskSlot {
    TaskSlot {
        provider: std::string::String::from("ollama"),
        model: std::string::String::from("llama3.2"),
        enabled: true,
        description: std::string::String::from("Interactive chat agent with tool calling"),
        streaming: std::option::Option::Some(true),
    }
}

impl Default for TaskSlotConfig {
    fn default() -> Self {
        Self {
            main: default_main_slot(),
            research: default_research_slot(),
            fallback: default_fallback_slot(),
            embedding: default_embedding_slot(),
            vision: default_vision_slot(),
            chat_agent: default_chat_agent_slot(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_task_slots() {
        // Test: Validates default task slot configuration.
        // Justification: Ensures sensible defaults for new installations.
        let config = TaskSlotConfig::default();

        std::assert_eq!(config.main.provider, "ollama");
        std::assert_eq!(config.main.model, "llama3.2");
        std::assert!(config.main.enabled);

        std::assert_eq!(config.embedding.model, "nomic-embed-text");
        std::assert!(config.embedding.enabled);

        std::assert!(!config.vision.enabled); // Vision disabled by default
        std::assert_eq!(config.chat_agent.streaming, std::option::Option::Some(true));
    }

    #[test]
    fn test_task_slot_serialization() {
        // Test: Validates TaskSlot JSON serialization.
        // Justification: Config must persist correctly to disk.
        let slot = TaskSlot {
            provider: std::string::String::from("anthropic"),
            model: std::string::String::from("claude-sonnet-4-5"),
            enabled: true,
            description: std::string::String::from("Chat agent"),
            streaming: std::option::Option::Some(true),
        };

        let json = serde_json::to_string(&slot).unwrap();
        let parsed: TaskSlot = serde_json::from_str(&json).unwrap();

        std::assert_eq!(parsed.provider, "anthropic");
        std::assert_eq!(parsed.streaming, std::option::Option::Some(true));
    }
}
