//! Defines the LLMAgentPort output port for conversational AI agents with tool calling.
//!
//! This port represents the interface for interacting with LLM-based agents that can
//! use tools to answer questions and perform tasks. It follows the HEXSER framework's
//! port pattern to enable swapping between different LLM providers (OpenAI, Ollama, etc.)
//! while maintaining the same interface.
//!
//! Revision History
//! - 2025-12-03T00:00:00Z @AI: Initial LLMAgentPort trait definition for chain-of-thought chat agent.

/// Represents a message in the conversation.
#[derive(Debug, Clone)]
pub struct AgentMessage {
    /// Role of the message sender (system, user, assistant)
    pub role: AgentRole,

    /// Content of the message
    pub content: std::string::String,
}

/// Role of a message sender in the conversation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentRole {
    /// System message providing context or instructions
    System,

    /// Message from the user
    User,

    /// Message from the AI assistant
    Assistant,
}

/// Represents a tool call made by the agent during inference.
#[derive(Debug, Clone)]
pub struct ToolCallInfo {
    /// Name of the tool that was called
    pub tool_name: std::string::String,

    /// JSON-formatted arguments passed to the tool
    pub args_json: std::string::String,

    /// Result returned by the tool (if completed)
    pub result: std::option::Option<std::string::String>,

    /// Status of the tool call
    pub status: ToolCallStatus,
}

/// Status of a tool call during agent execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolCallStatus {
    /// Tool call is pending execution
    Pending,

    /// Tool is currently executing
    Running,

    /// Tool completed successfully
    Success,

    /// Tool execution failed
    Failed,
}

/// Represents a streaming token from the agent response.
#[derive(Debug, Clone)]
pub enum StreamToken {
    /// A text token to append to the response
    Content(std::string::String),

    /// Agent started executing a tool
    ToolCallStart(ToolCallInfo),

    /// Agent completed a tool execution
    ToolCallEnd {
        /// Name of the tool
        tool_name: std::string::String,

        /// Result from the tool
        result: std::string::String,
    },

    /// Stream has completed
    Done,

    /// An error occurred during streaming
    Error(std::string::String),
}

/// Port (interface) for conversational AI agents with tool calling capabilities.
///
/// LLMAgentPort defines the contract for chat-based agents that can use tools
/// to answer questions and perform tasks. Implementations can use different
/// LLM providers (OpenAI, Ollama, Anthropic) while adhering to this interface.
///
/// # Examples
///
/// ```no_run
/// # use task_orchestrator::ports::llm_agent_port::{LLMAgentPort, AgentMessage, AgentRole, StreamToken};
/// # async fn example<A: LLMAgentPort>(agent: &A) {
/// let messages = vec![
///     AgentMessage {
///         role: AgentRole::System,
///         content: "You are a helpful assistant.".to_string(),
///     },
///     AgentMessage {
///         role: AgentRole::User,
///         content: "What tasks are in progress?".to_string(),
///     },
/// ];
///
/// let mut stream = agent.chat_with_tools(messages).await.unwrap();
/// while let Some(token) = stream.recv().await {
///     match token {
///         StreamToken::Content(text) => print!("{}", text),
///         StreamToken::Done => break,
///         _ => {}
///     }
/// }
/// # }
/// ```
#[async_trait::async_trait]
pub trait LLMAgentPort: std::marker::Send + std::marker::Sync {
    /// Initiates a conversation with the agent and returns a streaming response.
    ///
    /// This method sends a list of messages to the agent and receives a stream
    /// of tokens back. The agent may call tools during inference, which will be
    /// indicated via ToolCallStart and ToolCallEnd tokens in the stream.
    ///
    /// # Arguments
    ///
    /// * `messages` - Conversation history including system, user, and assistant messages
    ///
    /// # Returns
    ///
    /// A receiver channel that yields streaming tokens as the agent responds.
    ///
    /// # Errors
    ///
    /// Returns error string if the agent fails to initialize or communicate with the LLM.
    async fn chat_with_tools(
        &self,
        messages: std::vec::Vec<AgentMessage>,
    ) -> std::result::Result<
        tokio::sync::mpsc::Receiver<StreamToken>,
        std::string::String,
    >;

    /// Cancels an ongoing streaming response.
    ///
    /// This method attempts to stop a currently running chat stream. It's useful
    /// for implementing user-initiated cancellation (e.g., Esc key during streaming).
    ///
    /// # Returns
    ///
    /// Ok if cancellation succeeded, Err with description if it failed.
    async fn cancel_stream(&self) -> std::result::Result<(), std::string::String>;
}
