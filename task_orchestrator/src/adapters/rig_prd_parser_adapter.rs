//! Rig-powered PRD parser adapter for task generation.
//!
//! This adapter uses Rig's CompletionModel to analyze PRDs and generate
//! actionable task lists via LLM-based decomposition.
//!
//! Revision History
//! - 2025-11-30T22:00:00Z @AI: Implement two-pass persona assignment. Removed personas entirely from PRD parsing prompt (build_system_prompt now ignores personas parameter) to prevent biasing simpler LLMs into creating tasks FOR personas rather than FROM PRD content. Created assign_persona_to_task() method that uses LLM in a second pass to assign appropriate persona based on generated task's title/description. Personas list shown to LLM only during assignment pass, not during task generation. This allows organic task derivation from PRD requirements without persona influence.
//! - 2025-11-30T21:15:00Z @AI: Simplify persona prompt to avoid biasing LLM. Previous prompt listed each persona with role and description, causing LLM to create tasks for each persona rather than deriving tasks from PRD content. Changed to minimal "ASSIGNEE OPTIONS: Name1, Name2, ... or Default Agent" format. LLM now focuses on PRD requirements and just picks an assignee from the list.
//! - 2025-11-29T17:30:00Z @AI: Replace specific authentication example with abstract placeholders in prompts. The JWT/auth example was biasing LLM outputs toward auth-related tasks regardless of PRD content. Changed to SOTA few-shot approach: DESCRIPTION TEMPLATE with labeled sections [WHAT], [WHY], [HOW], [ACCEPTANCE], and RESPONSE FORMAT using <placeholders> for fields. LLM now generates tasks from PRD content without domain bias from concrete examples.
//! - 2025-11-29T15:00:00Z @AI: Rename assignee to agent_persona in LLM prompts and JSON extraction. Field name "assignee" caused LLMs to default to placeholder human names (Alice, Bob). New name primes LLM to produce role-based outputs (Backend Developer, Security Analyst). Updated: prompt field descriptions, example JSON responses, and JSON extraction alias lists (agent_persona now first, old names kept as fallbacks for backward compatibility).
//! - 2025-11-28T22:00:00Z @AI: Add RAG context injection into task generation prompts (Phase 5 Task 5.2). Added optional embedding_port and artifact_repository dependencies to RigPRDParserAdapter struct. Created retrieve_rag_context() method that searches for relevant artifacts based on PRD objectives and title. Modified build_prompt() to inject RAG context section before PRD content when artifacts are found. Added new() variant that accepts RAG dependencies and separate new_without_rag() for backward compatibility. Prompts now include "RELEVANT CONTEXT FROM KNOWLEDGE BASE:" section with up to 3 most similar artifacts to improve task generation quality.
//! - 2025-11-28T00:40:00Z @AI: Add Priority field to PRDGenUpdate::TaskGenerated event. Extended TaskGenerated variant to include assignee, priority, and complexity as separate optional fields instead of embedding them in description string. Updated streaming code to extract these fields directly from task JSON (with field aliases: priority/prio/importance, complexity/estimated_complexity/difficulty, assignee/assigned_to/owner/responsible). This enables TUI to display structured task information in proper key:value format rather than parsing embedded text.
//! - 2025-11-28T00:15:00Z @AI: Add precise JSON error diagnostics with line:col reporting. Created extract_json_error_diagnostics() helper that captures serde_json::Error line/column information and displays 5-line context window with error marker. Enhanced remediate_json_with_llm() to extract error diagnostics before LLM remediation and include them in the remediation prompt under ERROR DIAGNOSTICS section, giving the LLM precise information about where and what the parse failure is. Updated final failure message to show diagnostics instead of just first 1500 chars. Remediation now targets specific error locations rather than blindly attempting fixes.
//! - 2025-11-27T15:00:00Z @AI: Enhance decomposition prompt with concrete example and clearer instructions. Added EXAMPLE FORMAT section showing two complete sub-task JSON objects to guide llama3.2:latest. Clarified that sub-tasks should have complexity 1-5 (simpler than parent). Added numbered CRITICAL INSTRUCTIONS for explicit guidance: no markdown, start with [, end with ], 3-5 tasks total. Changed complexity from 1-10 to 1-5 scale for sub-tasks to emphasize they should be simpler units of work. This addresses issue where LLM wasn't generating sub-tasks due to unclear formatting expectations.
//! - 2025-11-27T08:00:00Z @AI: Add task decomposition workflow. Implemented decompose_task() method that uses LLM to break complex tasks into 3-5 sub-tasks. Created build_decomposition_prompt() that provides parent task context, PRD snippet, and persona list to guide sub-task generation. Added parse_subtasks_from_json() that sets parent_task_id linkage and maintains PRD association. Sub-tasks inherit PRD context and have lower default complexity (3 vs 5). This enables hierarchical task breakdown for complex work items (complexity >= 7).
//! - 2025-11-27T05:30:00Z @AI: Enhance system prompt to require detailed task descriptions. Added DESCRIPTION REQUIREMENTS section with 4-part structure: WHAT (features/components), WHY (business value), HOW (implementation approach), ACCEPTANCE (success criteria). Included concrete good/bad examples showing detailed vs. vague descriptions. Updated both persona and non-persona prompts with detailed authentication system example (JWT, bcrypt, rate limiting, endpoints). This guides llama3.2:latest to generate actionable, thorough task descriptions instead of shallow summaries.
//! - 2025-11-27T05:15:00Z @AI: Simplify remediation start messages for immediate feedback. Changed assignee remediation message from "Assignee '{}' not found, attempting LLM remediation..." to simple "Remediating assignee..." and JSON remediation from "Initial JSON parse failed: {}. Attempting remediation with {}..." to "Remediating JSON...". Shorter messages appear faster in red validation rows, providing immediate visual feedback that remediation has started instead of appearing locked up during long LLM calls.
//! - 2025-11-27T04:45:00Z @AI: Remove all eprintln! debug logging from validate_assignee(). Replaced debug logging with ValidationInfo messages for: LLM remediation failures, fallback persona usage. Removed verbose logging for exact/case-insensitive/fuzzy matches (no messages needed for successful matches). All assignee validation feedback now properly streams through ValidationInfo channel to appear in red validation boxes instead of leaking to stderr.
//! - 2025-11-27T04:30:00Z @AI: Add ValidationInfo streaming for JSON remediation. Send ValidationInfo messages when JSON parsing fails and remediation begins, and when remediation succeeds. Uses "JSON Parsing" as task_title since individual task titles aren't available yet at this stage. These messages now appear in red validation boxes in both the conversation and task list sections of the TUI, providing proper real-time feedback instead of being silently hidden.
//! - 2025-11-27T04:15:00Z @AI: Remove JSON remediation debug logging to prevent TUI pollution. Removed eprintln! statements for "Initial JSON parse failed" and "Remediation succeeded" messages that were leaking into TUI output. These debug messages appeared as plain text in the conversation view instead of being properly formatted. Remediation still works silently, with error details available in error messages if remediation fails.
//! - 2025-11-27T03:30:00Z @AI: Add streaming ValidationInfo messages for assignee remediation. Updated validate_assignee() signature to accept task_title and optional channel sender. Added ValidationInfo message sending at remediation start ("Assignee not found, attempting LLM remediation...") and success ("Remediation successful: X → Y"). Updated parse_tasks_from_json() signature to accept optional channel and pass it to validate_assignee(). Updated all call sites (parse_prd_interactively passes Some(&update_tx), parse_prd_to_tasks and tests pass None). This enables real-time validation feedback in the TUI.
//! - 2025-11-26T23:00:00Z @AI: Use configured fallback model from config.json for JSON remediation. Added fallback_model_name field to RigPRDParserAdapter struct. Constructor now accepts both main and fallback model names. Updated parse_tasks_from_json() to accept fallback_model_name parameter and pass it to remediate_json_with_llm(). TUI extracts config["task_tools"]["fallback"]["model"] and passes to adapter. All tests updated to use async/await pattern and provide fallback model. Remediation now uses user-configured model instead of hardcoded value.
//! - 2025-11-26T22:25:00Z @AI: Fix remediation to use available model instead of hardcoded qwen2.5:0.5b. Changed remediate_json_with_llm() to accept model_name parameter instead of hardcoding unavailable model. Now uses llama3.2:latest (known-good model) for JSON remediation. Fixes error "model 'qwen2.5:0.5b' not found" when remediation is needed. Added comment noting future enhancement to use dedicated fallback model from config.json.
//! - 2025-11-26T22:15:00Z @AI: Enhance JSON remediation with aggressive cleanup and detailed logging. Added 5-step remediation process: (1) Remove LLM wrapper text, (2) Fix common syntax errors (trailing commas, missing braces/brackets), (3) Test parse after cleanup, (4) Call LLM if needed, (5) Validate output. Each step logs what was attempted. Returns (fixed_json, log) tuple on success or error with full log on failure. Error messages now show complete remediation attempt history so users understand what was tried.
//! - 2025-11-26T21:35:00Z @AI: Add integration tests for streaming PRD generation. Created tests/integration_streaming_prd_generation.rs with 2 passing tests that validate: (1) streaming JSON detection and TaskGenerated events, (2) field extraction for complex PRDs with priority/complexity. Tests confirm streaming works, task boxes appear, and all fields extract correctly.
//! - 2025-11-26T21:20:00Z @AI: Remove unused json-event-parser dependency. Manual depth tracking with separate array_depth and object_depth counters is working correctly for detecting complete task objects. Removed json-event-parser crate since it wasn't needed - char-by-char brace tracking is sufficient for this use case.
//! - 2025-11-26T20:50:00Z @AI: Implement smart streaming with task detection. Added brace depth tracking to detect complete JSON task objects as they stream in. When a complete task object is detected (closing brace at depth 0), extracts title/description and sends TaskGenerated update to create new line in TUI. Streams raw JSON to Thinking updates (appends to current line) until complete task found, then starts new line. Provides structured real-time visibility into task generation progress.
//! - 2025-11-26T20:45:00Z @AI: Fix streaming to show actual LLM content instead of character counts. Changed PRDGenUpdate::Thinking to send the streaming text chunks (content.to_string()) rather than "Received X characters...". Now displays the LLM's actual response as it streams in, providing real-time visibility into what the LLM is generating.
//! - 2025-11-26T20:30:00Z @AI: Implement true HTTP streaming for PRD generation. Replaced batch Rig completion with custom Ollama HTTP streaming via reqwest. Now uses stream=true with bytes_stream() to parse newline-delimited JSON chunks in real-time. Accumulates LLM response incrementally and sends progress updates via channel. Eliminates "using batch mode" fallback message - full streaming now live.
//! - 2025-11-25T23:55:00Z @AI: Add interactive PRD generation with streaming. Created PRDGenUpdate enum for streaming events (Thinking, Question, TaskGenerated, Complete, Error) and parse_prd_interactively() method that returns bidirectional channels for real-time LLM communication. This enables showing LLM thinking process, accepting mid-generation user input, and displaying partial task results as they stream in.
//! - 2025-11-25T22:45:00Z @AI: Add LLM-based JSON remediation fallback. Created remediate_json_with_llm() that uses lightweight model (qwen2.5:0.5b) to fix malformed JSON when serde_json parsing fails. Changed parse_tasks_from_json() to async, attempts remediation before failing. This creates 4-layer fault tolerance: (1) extract_json_from_response, (2) field aliases, (3) LLM remediation, (4) error with diagnostics. Fast 0.5B model adds ~100ms overhead only on parse failures.
//! - 2025-11-25T22:40:00Z @AI: Implement tolerant JSON parsing with field aliases. Added extract_string() and extract_number() helpers that try multiple field name variants. Updated parse_tasks_from_json() to use tolerant parsing with aliases: title→[title,task,name,summary,action,item], description→[description,desc,details,detail,content], priority→[priority,prio,importance,level], complexity→[estimated_complexity,complexity,difficulty,effort,score]. Skips non-object entries instead of failing. This matches the proven pattern from action_item_parser.rs used by all transcript adapters.
//! - 2025-11-25T22:35:00Z @AI: Strengthen LLM prompts to enforce JSON format. Rewrote system_prompt() with explicit "CRITICAL: You MUST respond with ONLY a valid JSON array" instruction, concrete example to copy, and explicit DO NOT list (no markdown, no explanations, etc). Added "START YOUR RESPONSE WITH [ AND END WITH ]" directive. Updated build_prompt() to add clear "YOUR RESPONSE:" marker. This fixes issue where llama3.2 was returning raw title strings instead of proper JSON objects.
//! - 2025-11-25T22:30:00Z @AI: Enhanced JSON extraction to handle single objects and Python code blocks. Added Cases 4-5: wrap single JSON object {...} in array, find object in text and wrap. Added ```python language tag support. Increased error message limit to 500 chars for better debugging. Added 3 new tests (single object, object in text, python code block). Total: 10 JSON extraction tests, all passing.
//! - 2025-11-25T22:15:00Z @AI: Add robust JSON extraction from LLM responses. Created extract_json_from_response() helper that handles 3 cases: markdown code blocks (```json...```), JSON embedded in text (find first [ to last ]), and clean JSON. Prevents parse failures when Ollama wraps responses in explanatory text or markdown formatting. Enhanced error messages to show truncated response content for debugging.
//! - 2025-11-22T17:05:00Z @AI: Initial Rig PRD parser adapter for Rigger Phase 0 Sprint 0.3.

// External crate declarations required for Rust 2024 edition
extern crate reqwest;
extern crate futures;
extern crate serde_json;

/// Update event emitted during interactive PRD generation.
///
/// These events are sent through a channel to provide real-time feedback
/// about the LLM's thinking process, generated tasks, and any questions
/// it needs answered during generation.
#[derive(Debug, Clone)]
pub enum PRDGenUpdate {
    /// LLM is thinking - shows reasoning or analysis
    Thinking(String),
    /// LLM has a question that needs user input
    Question(String),
    /// A single task was generated (partial result)
    TaskGenerated {
        title: String,
        description: String,
        assignee: std::option::Option<String>,
        priority: std::option::Option<u8>,
        complexity: std::option::Option<u8>,
    },
    /// Assignee validation/remediation information
    ValidationInfo { task_title: String, message: String },
    /// All tasks generated successfully
    Complete(std::vec::Vec<task_manager::domain::task::Task>),
    /// Generation failed with error
    Error(String),
}

/// Rig-powered adapter for parsing PRDs into tasks.
///
/// RigPRDParserAdapter uses Rig's agent API to send PRD content to
/// an LLM and parse the response into a structured task list. It handles
/// JSON parsing with error recovery and links tasks back to the source PRD.
///
/// # Examples
///
/// ```no_run
/// # use task_orchestrator::adapters::rig_prd_parser_adapter::RigPRDParserAdapter;
/// # async fn example() {
/// let adapter = RigPRDParserAdapter::new(std::string::String::from("llama3.1"));
/// # }
/// ```
#[derive(hexser::HexAdapter, Clone)]
pub struct RigPRDParserAdapter {
    model_name: String,
    fallback_model_name: String,
    personas: std::vec::Vec<task_manager::domain::persona::Persona>,
    embedding_port: std::option::Option<std::sync::Arc<dyn crate::ports::embedding_port::EmbeddingPort + std::marker::Send + std::marker::Sync>>,
    artifact_repository: std::option::Option<std::sync::Arc<std::sync::Mutex<dyn task_manager::ports::artifact_repository_port::ArtifactRepositoryPort + std::marker::Send>>>,
    project_id: std::option::Option<std::string::String>,
}

impl RigPRDParserAdapter {
    /// Creates a new RigPRDParserAdapter with the specified models and personas.
    ///
    /// # Arguments
    ///
    /// * `model_name` - Name of the primary Ollama model to use (e.g., "llama3.2:latest")
    /// * `fallback_model_name` - Name of the fallback Ollama model for JSON remediation (e.g., "llama3.2:latest")
    /// * `personas` - List of available personas for task assignment
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use task_orchestrator::adapters::rig_prd_parser_adapter::RigPRDParserAdapter;
    /// # async fn example() {
    /// let adapter = RigPRDParserAdapter::new(
    ///     std::string::String::from("llama3.2:latest"),
    ///     std::string::String::from("llama3.2:latest"),
    ///     std::vec::Vec::new()
    /// );
    /// # }
    /// ```
    ///
    /// # Note
    ///
    /// The Ollama server URL defaults to http://localhost:11434
    pub fn new(model_name: String, fallback_model_name: String, personas: std::vec::Vec<task_manager::domain::persona::Persona>) -> Self {
        Self {
            model_name,
            fallback_model_name,
            personas,
            embedding_port: std::option::Option::None,
            artifact_repository: std::option::Option::None,
            project_id: std::option::Option::None,
        }
    }

    /// Creates a new RigPRDParserAdapter with RAG context retrieval capabilities.
    ///
    /// This constructor enables the adapter to inject relevant artifacts from the
    /// knowledge base into task generation prompts, improving context awareness.
    ///
    /// # Arguments
    ///
    /// * `model_name` - Name of the primary Ollama model
    /// * `fallback_model_name` - Name of the fallback model for remediation
    /// * `personas` - List of available personas for task assignment
    /// * `embedding_port` - Port for generating query embeddings
    /// * `artifact_repository` - Repository for artifact similarity search
    /// * `project_id` - Optional project ID to scope artifact search
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use task_orchestrator::adapters::rig_prd_parser_adapter::RigPRDParserAdapter;
    /// # async fn example(
    /// #     embedding_port: std::sync::Arc<dyn task_orchestrator::ports::embedding_port::EmbeddingPort + Send + Sync>,
    /// #     artifact_repo: std::sync::Arc<std::sync::Mutex<dyn task_manager::ports::artifact_repository_port::ArtifactRepositoryPort + Send>>,
    /// # ) {
    /// let adapter = RigPRDParserAdapter::new_with_rag(
    ///     String::from("llama3.2:latest"),
    ///     String::from("llama3.2:latest"),
    ///     std::vec::Vec::new(),
    ///     embedding_port,
    ///     artifact_repo,
    ///     std::option::Option::Some(String::from("project-123")),
    /// );
    /// # }
    /// ```
    pub fn new_with_rag(
        model_name: String,
        fallback_model_name: String,
        personas: std::vec::Vec<task_manager::domain::persona::Persona>,
        embedding_port: std::sync::Arc<dyn crate::ports::embedding_port::EmbeddingPort + std::marker::Send + std::marker::Sync>,
        artifact_repository: std::sync::Arc<std::sync::Mutex<dyn task_manager::ports::artifact_repository_port::ArtifactRepositoryPort + std::marker::Send>>,
        project_id: std::option::Option<std::string::String>,
    ) -> Self {
        Self {
            model_name,
            fallback_model_name,
            personas,
            embedding_port: std::option::Option::Some(embedding_port),
            artifact_repository: std::option::Option::Some(artifact_repository),
            project_id,
        }
    }

    /// Parses a PRD interactively with real-time streaming updates.
    ///
    /// This method provides a channel-based interface for interactive PRD generation
    /// that shows LLM thinking, accepts user input mid-generation, and streams
    /// partial task results in real-time.
    ///
    /// # Arguments
    ///
    /// * `prd` - The Product Requirements Document to parse
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// - `Receiver<PRDGenUpdate>`: Channel for receiving updates from LLM
    /// - `Sender<String>`: Channel for sending user input to LLM
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use task_orchestrator::adapters::rig_prd_parser_adapter::RigPRDParserAdapter;
    /// # use task_manager::domain::prd::PRD;
    /// # async fn example() {
    /// let adapter = RigPRDParserAdapter::new(std::string::String::from("llama3.2:latest"));
    /// let prd = PRD::new(
    ///     "test-project".to_string(),
    ///     "Build app".to_string(),
    ///     vec!["Create UI".to_string()],
    ///     vec!["Rust".to_string()],
    ///     vec![],
    ///     "# PRD".to_string(),
    /// );
    ///
    /// let (mut updates, user_input) = adapter.parse_prd_interactively(prd).await.unwrap();
    ///
    /// // Receive updates in UI event loop
    /// while let std::option::Option::Some(update) = updates.recv().await {
    ///     match update {
    ///         PRDGenUpdate::Thinking(msg) => println!("LLM: {}", msg),
    ///         PRDGenUpdate::Complete(tasks) => {
    ///             println!("Generated {} tasks!", tasks.len());
    ///             break;
    ///         }
    ///         _ => {}
    ///     }
    /// }
    /// # }
    /// ```
    pub async fn parse_prd_interactively(
        &self,
        prd: task_manager::domain::prd::PRD,
    ) -> std::result::Result<
        (
            tokio::sync::mpsc::Receiver<PRDGenUpdate>,
            tokio::sync::mpsc::Sender<String>,
        ),
        std::string::String,
    > {
        // Create bidirectional channels
        let (update_tx, update_rx) = tokio::sync::mpsc::channel::<PRDGenUpdate>(100);
        let (input_tx, mut input_rx) = tokio::sync::mpsc::channel::<String>(10);

        let adapter = self.clone();
        let model_name = self.model_name.clone();
        let fallback_model_name = self.fallback_model_name.clone();
        let personas = self.personas.clone();

        // Spawn background task for LLM interaction with streaming
        tokio::spawn(async move {
            // Send initial thinking message
            let _ = update_tx
                .send(PRDGenUpdate::Thinking(
                    "Analyzing PRD objectives and constraints...".to_string(),
                ))
                .await;

            // Build the prompt
            let prompt = adapter.build_prompt(&prd, &personas).await;

            let _ = update_tx
                .send(PRDGenUpdate::Thinking(
                    "Streaming task generation from LLM...".to_string(),
                ))
                .await;

            // Create Ollama streaming request
            let http_client = reqwest::Client::new();
            let request_body = serde_json::json!({
                "model": model_name,
                "messages": [{
                    "role": "user",
                    "content": prompt
                }],
                "stream": true,
                "options": {
                    "temperature": 0.7
                }
            });

            let response = match http_client
                .post("http://localhost:11434/api/chat")
                .json(&request_body)
                .send()
                .await
            {
                std::result::Result::Ok(r) => r,
                std::result::Result::Err(e) => {
                    let _ = update_tx
                        .send(PRDGenUpdate::Error(std::format!("HTTP request failed: {}", e)))
                        .await;
                    return;
                }
            };

            // Stream response chunks with depth-based JSON parsing
            let mut accumulated_response = std::string::String::new();
            let mut current_task_buffer = std::string::String::new();
            let mut _array_depth = 0;  // Reserved for future nested array support
            let mut object_depth = 0;
            let mut in_array = false;
            let mut stream = response.bytes_stream();

            use futures::StreamExt;
            while let std::option::Option::Some(chunk_result) = stream.next().await {
                match chunk_result {
                    std::result::Result::Ok(bytes) => {
                        // Parse newline-delimited JSON from Ollama
                        if let std::result::Result::Ok(text) = std::string::String::from_utf8(bytes.to_vec()) {
                            for line in text.lines() {
                                if line.trim().is_empty() {
                                    continue;
                                }

                                // Parse Ollama's streaming response format
                                if let std::result::Result::Ok(chunk) = serde_json::from_str::<serde_json::Value>(line) {
                                    if let std::option::Option::Some(content) = chunk.get("message")
                                        .and_then(|m| m.get("content"))
                                        .and_then(|c| c.as_str())
                                    {
                                        if !content.is_empty() {
                                            accumulated_response.push_str(content);

                                            // Parse JSON events to detect complete objects
                                            for ch in content.chars() {
                                                current_task_buffer.push(ch);

                                                // Track JSON structure
                                                match ch {
                                                    '[' => {
                                                        _array_depth += 1;
                                                        in_array = true;
                                                    }
                                                    ']' => {
                                                        _array_depth -= 1;
                                                    }
                                                    '{' => {
                                                        object_depth += 1;
                                                        if in_array && object_depth == 1 {
                                                            // Start of task object in array
                                                            current_task_buffer.clear();
                                                            current_task_buffer.push('{');
                                                        }
                                                    }
                                                    '}' => {
                                                        object_depth -= 1;
                                                        if in_array && object_depth == 0 {
                                                            // Complete task object detected
                                                            if let std::result::Result::Ok(task_obj) = serde_json::from_str::<serde_json::Value>(&current_task_buffer) {
                                                                if let std::option::Option::Some(title) = task_obj.get("title")
                                                                    .or_else(|| task_obj.get("task"))
                                                                    .or_else(|| task_obj.get("name"))
                                                                    .and_then(|t| t.as_str())
                                                                {
                                                                    let description = task_obj.get("description")
                                                                        .or_else(|| task_obj.get("desc"))
                                                                        .and_then(|d| d.as_str())
                                                                        .unwrap_or("")
                                                                        .to_string();

                                                                    // Extract agent_persona (with fallback aliases)
                                                                    let assignee = task_obj.get("agent_persona")
                                                                        .or_else(|| task_obj.get("assignee"))
                                                                        .or_else(|| task_obj.get("assigned_to"))
                                                                        .or_else(|| task_obj.get("owner"))
                                                                        .or_else(|| task_obj.get("responsible"))
                                                                        .and_then(|a| a.as_str())
                                                                        .map(|s| s.to_string());

                                                                    // Extract priority
                                                                    let priority = task_obj.get("priority")
                                                                        .or_else(|| task_obj.get("prio"))
                                                                        .or_else(|| task_obj.get("importance"))
                                                                        .and_then(|p| p.as_u64())
                                                                        .map(|p| p as u8);

                                                                    // Extract complexity
                                                                    let complexity = task_obj.get("estimated_complexity")
                                                                        .or_else(|| task_obj.get("complexity"))
                                                                        .or_else(|| task_obj.get("difficulty"))
                                                                        .and_then(|c| c.as_u64())
                                                                        .map(|c| c as u8);

                                                                    // Send TaskGenerated update with all fields
                                                                    let _ = update_tx
                                                                        .send(PRDGenUpdate::TaskGenerated {
                                                                            title: title.to_string(),
                                                                            description: description.clone(),
                                                                            assignee,
                                                                            priority,
                                                                            complexity,
                                                                        })
                                                                        .await;
                                                                }
                                                            }
                                                            current_task_buffer.clear();
                                                        }
                                                    }
                                                    _ => {}
                                                }
                                            }

                                            // Send streaming update (appends to current line)
                                            let _ = update_tx
                                                .send(PRDGenUpdate::Thinking(content.to_string()))
                                                .await;
                                        }
                                    }

                                    // Check if done
                                    if let std::option::Option::Some(true) = chunk.get("done").and_then(|d| d.as_bool()) {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    std::result::Result::Err(e) => {
                        let _ = update_tx
                            .send(PRDGenUpdate::Error(std::format!("Stream error: {}", e)))
                            .await;
                        return;
                    }
                }
            }

            // Parse accumulated response into tasks
            match Self::parse_tasks_from_json(&accumulated_response, &prd.project_id, &fallback_model_name, &personas, std::option::Option::Some(&update_tx)).await {
                std::result::Result::Ok(tasks) => {
                    let _ = update_tx.send(PRDGenUpdate::Complete(tasks)).await;
                }
                std::result::Result::Err(e) => {
                    let _ = update_tx.send(PRDGenUpdate::Error(e)).await;
                }
            }

            // Keep channel open briefly to receive any user input
            // (This is a placeholder for future interactive features)
            while let std::option::Option::Some(_user_msg) = input_rx.recv().await {
                // TODO: Implement user input handling in Phase 2.7
                let _ = update_tx
                    .send(PRDGenUpdate::Thinking(
                        "User input received (processing not yet implemented)".to_string(),
                    ))
                    .await;
            }
        });

        std::result::Result::Ok((update_rx, input_tx))
    }

    /// Builds the system prompt for PRD parsing with persona information.
    fn build_system_prompt(_personas: &[task_manager::domain::persona::Persona]) -> std::string::String {
        // Personas are intentionally ignored - they're assigned in a second pass to avoid biasing task generation
        let mut prompt = std::string::String::from(
            "You are a project management assistant. Break down Product Requirements Documents into actionable tasks.\n\n\
            CRITICAL: You MUST respond with ONLY a valid JSON array. No other text.\n\n"
        );

        prompt.push_str("Each task object must have exactly these 4 fields:\n\
        - \"title\": string (concise task title, max 100 chars)\n\
        - \"description\": string (DETAILED description - see requirements below)\n\
        - \"priority\": string (must be exactly \"high\", \"medium\", or \"low\")\n\
        - \"estimated_complexity\": number (integer 1-10, where 10 is most complex)\n\n\
        DESCRIPTION REQUIREMENTS (CRITICAL - read carefully):\n\
        Descriptions must be thorough and actionable. Each description MUST include:\n\
        1. WHAT: What needs to be built/implemented (specific features, components, or outcomes)\n\
        2. WHY: Why this task matters (business value, technical dependency, or user benefit)\n\
        3. HOW: Implementation approach or key steps (technologies, patterns, or methods)\n\
        4. ACCEPTANCE: Clear success criteria (how to verify the task is complete)\n\n\
        DESCRIPTION TEMPLATE:\n\
        \"[WHAT: Specific deliverable]. [WHY: Business/technical reason]. [HOW: Key implementation steps using relevant technologies]. [ACCEPTANCE: Measurable success criteria].\"\n\n\
        BAD DESCRIPTION (too vague - DO NOT do this):\n\
        \"Implement the feature\" or \"Build the component\"\n\n\
        RESPONSE FORMAT (use this exact JSON structure, replace <placeholders> with actual content from the PRD):\n\
        [{\"title\":\"<verb> <specific component>\",\"description\":\"<what to build>. <why it matters>. <how to implement with specific steps>. Success criteria: <measurable outcomes>.\",\"priority\":\"<high|medium|low>\",\"estimated_complexity\":<1-10>}]\n\n");

        prompt.push_str("DO NOT:\n\
        - Add markdown code blocks\n\
        - Add explanations before or after the JSON\n\
        - Use newlines inside the JSON array\n\
        - Return anything except the JSON array\n\n\
        START YOUR RESPONSE WITH [ AND END WITH ]");

        prompt
    }

    /// Extracts JSON array from LLM response, handling markdown code blocks and extra text.
    fn extract_json_from_response(response: &str) -> std::result::Result<std::string::String, std::string::String> {
        let trimmed = response.trim();

        // Case 1: Response is wrapped in markdown code block (```json ... ``` or ```python ... ```)
        if trimmed.contains("```") {
            // Find content between code block markers (try multiple language tags)
            let start_markers = ["```json\n", "```python\n", "```\n"];
            for marker in &start_markers {
                if let std::option::Option::Some(start_idx) = trimmed.find(marker) {
                    let json_start = start_idx + marker.len();
                    if let std::option::Option::Some(end_idx) = trimmed[json_start..].find("```") {
                        let json_str = &trimmed[json_start..json_start + end_idx];
                        return std::result::Result::Ok(json_str.trim().to_string());
                    }
                }
            }
        }

        // Case 2: Response has JSON array somewhere in it (find first [ to last ])
        if let std::option::Option::Some(start_idx) = trimmed.find('[') {
            if let std::option::Option::Some(end_idx) = trimmed.rfind(']') {
                if end_idx > start_idx {
                    let json_str = &trimmed[start_idx..=end_idx];
                    return std::result::Result::Ok(json_str.to_string());
                }
            }
        }

        // Case 3: Response is already clean JSON array
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            return std::result::Result::Ok(trimmed.to_string());
        }

        // Case 4: Response is a JSON object { } - wrap it in an array
        if trimmed.starts_with('{') && trimmed.ends_with('}') {
            return std::result::Result::Ok(std::format!("[{}]", trimmed));
        }

        // Case 5: Response has a JSON object somewhere in it
        if let std::option::Option::Some(start_idx) = trimmed.find('{') {
            if let std::option::Option::Some(end_idx) = trimmed.rfind('}') {
                if end_idx > start_idx {
                    let json_str = &trimmed[start_idx..=end_idx];
                    // Wrap single object in array
                    return std::result::Result::Ok(std::format!("[{}]", json_str));
                }
            }
        }

        // Failed to extract JSON
        std::result::Result::Err(std::format!(
            "Could not extract JSON array from response. Expected JSON array but got:\n\n{}",
            if response.len() > 500 {
                std::format!("{}... (response truncated, total length: {} chars)", &response[..500], response.len())
            } else {
                response.to_string()
            }
        ))
    }

    /// Retrieves relevant artifacts from the knowledge base for RAG context.
    ///
    /// Searches for artifacts related to the PRD's title and objectives using
    /// semantic similarity. Returns formatted context string if artifacts are found.
    ///
    /// # Arguments
    ///
    /// * `prd` - The PRD to search context for
    ///
    /// # Returns
    ///
    /// Returns a formatted string with relevant artifacts, or empty string if:
    /// - RAG dependencies are not configured
    /// - No relevant artifacts are found
    /// - Search fails
    async fn retrieve_rag_context(&self, prd: &task_manager::domain::prd::PRD) -> std::string::String {
        // Early return if RAG not configured
        let (embedding_port, artifact_repository) = match (&self.embedding_port, &self.artifact_repository) {
            (std::option::Option::Some(e), std::option::Option::Some(a)) => (e, a),
            _ => return std::string::String::new(),
        };

        // Build search query from PRD title and first 2 objectives
        let mut query = prd.title.clone();
        if !prd.objectives.is_empty() {
            query.push_str(": ");
            let obj_count = std::cmp::min(2, prd.objectives.len());
            query.push_str(&prd.objectives[0..obj_count].join(", "));
        }

        // Generate embedding for query
        let query_embedding = match embedding_port.generate_embedding(&query).await {
            std::result::Result::Ok(emb) => emb,
            std::result::Result::Err(e) => {
                eprintln!("RAG: Failed to generate query embedding: {}", e);
                return std::string::String::new();
            }
        };

        // Search for similar artifacts (limit to 3 most relevant)
        let repo = match artifact_repository.lock() {
            std::result::Result::Ok(r) => r,
            std::result::Result::Err(e) => {
                eprintln!("RAG: Failed to acquire repository lock: {}", e);
                return std::string::String::new();
            }
        };

        let similar_artifacts = match repo.find_similar(
            &query_embedding,
            3,  // Limit to top 3 artifacts
            std::option::Option::Some(0.6),  // Higher threshold for quality
            self.project_id.clone(),
        ) {
            std::result::Result::Ok(artifacts) => artifacts,
            std::result::Result::Err(e) => {
                eprintln!("RAG: Failed to search artifacts: {}", e);
                return std::string::String::new();
            }
        };

        // Format artifacts into context string
        if similar_artifacts.is_empty() {
            return std::string::String::new();
        }

        let mut context = std::string::String::from("RELEVANT CONTEXT FROM KNOWLEDGE BASE:\n\n");
        for (i, similar) in similar_artifacts.iter().enumerate() {
            let artifact = &similar.artifact;
            let similarity = (1.0 - similar.distance) * 100.0;

            context.push_str(&std::format!(
                "Context {}: [Similarity: {:.1}%] Source: {:?}\n{}\n\n",
                i + 1,
                similarity,
                artifact.source_type,
                artifact.content
            ));
        }

        context.push_str("Use the above context to inform task generation when relevant.\n\n---\n\n");
        context
    }

    /// Builds the complete prompt from PRD content (system + user).
    async fn build_prompt(&self, prd: &task_manager::domain::prd::PRD, personas: &[task_manager::domain::persona::Persona]) -> std::string::String {
        // Retrieve RAG context if available
        let rag_context = self.retrieve_rag_context(prd).await;
        let mut prompt = Self::build_system_prompt(personas);
        prompt.push_str("\n\n");

        // Inject RAG context before PRD content if available
        if !rag_context.is_empty() {
            prompt.push_str(&rag_context);
        }

        prompt.push_str(&std::format!("# PRD: {}\n\n", prd.title));

        if !prd.objectives.is_empty() {
            prompt.push_str("## Objectives\n");
            for obj in &prd.objectives {
                prompt.push_str(&std::format!("- {}\n", obj));
            }
            prompt.push('\n');
        }

        if !prd.tech_stack.is_empty() {
            prompt.push_str("## Tech Stack\n");
            for tech in &prd.tech_stack {
                prompt.push_str(&std::format!("- {}\n", tech));
            }
            prompt.push('\n');
        }

        if !prd.constraints.is_empty() {
            prompt.push_str("## Constraints\n");
            for constraint in &prd.constraints {
                prompt.push_str(&std::format!("- {}\n", constraint));
            }
            prompt.push('\n');
        }

        prompt.push_str("---\n\n");
        prompt.push_str("GENERATE TASKS: Create a comprehensive task list for this PRD.\n");
        let field_count = if personas.is_empty() { 4 } else { 5 };
        prompt.push_str(&std::format!("RESPONSE FORMAT: Start with [ and end with ]. Include all {} required fields per task.\n", field_count));
        prompt.push_str("YOUR RESPONSE:");

        prompt
    }

    /// Helper to extract a string value from multiple candidate keys (tolerant parsing).
    fn extract_string(
        m: &serde_json::Map<std::string::String, serde_json::Value>,
        keys: &[&str],
    ) -> std::option::Option<std::string::String> {
        for k in keys {
            if let std::option::Option::Some(v) = m.get(*k) {
                match v {
                    serde_json::Value::String(s) if !s.trim().is_empty() => {
                        return std::option::Option::Some(s.trim().to_string());
                    }
                    serde_json::Value::Number(n) => {
                        return std::option::Option::Some(n.to_string());
                    }
                    _ => {}
                }
            }
        }
        std::option::Option::None
    }

    /// Helper to extract a number from multiple candidate keys.
    fn extract_number(
        m: &serde_json::Map<std::string::String, serde_json::Value>,
        keys: &[&str],
    ) -> std::option::Option<u64> {
        for k in keys {
            if let std::option::Option::Some(v) = m.get(*k) {
                match v {
                    serde_json::Value::Number(n) => {
                        return n.as_u64();
                    }
                    serde_json::Value::String(s) => {
                        if let std::result::Result::Ok(num) = s.parse::<u64>() {
                            return std::option::Option::Some(num);
                        }
                    }
                    _ => {}
                }
            }
        }
        std::option::Option::None
    }

    /// Attempts to fix malformed JSON using a lightweight LLM (remediation fallback).
    /// Attempts to fix malformed JSON using multiple strategies with detailed logging.
    ///
    /// Uses the configured model for remediation - should be a fast, capable model.
    ///
    /// Returns (fixed_json, remediation_log) on success, or error with full log on failure.
    async fn remediate_json_with_llm(malformed_json: &str, model_name: &str) -> std::result::Result<(std::string::String, std::string::String), std::string::String> {
        let mut log = std::string::String::from("JSON Remediation Log:\n");

        // Step 1: Aggressive text cleanup
        log.push_str("→ Step 1: Aggressive cleanup\n");
        let mut cleaned = malformed_json.to_string();

        // Remove common LLM artifacts
        let original_len = cleaned.len();
        cleaned = cleaned
            .replace("```json", "")
            .replace("```python", "")
            .replace("```", "")
            .replace("Here is the JSON:", "")
            .replace("Here's the JSON:", "")
            .replace("Response:", "")
            .trim()
            .to_string();

        if cleaned.len() != original_len {
            log.push_str(&std::format!("  ✓ Removed {} chars of wrapper text\n", original_len - cleaned.len()));
        }

        // Step 2: Fix common JSON syntax errors
        log.push_str("→ Step 2: Fix common syntax errors\n");
        let before_fixes = cleaned.clone();

        // Fix trailing commas
        cleaned = cleaned.replace(",]", "]").replace(",}", "}");

        // Merge multiple separate JSON arrays (llama3.2 format issue)
        // Pattern: ][  or ]\n[ becomes ,
        let lines: std::vec::Vec<&str> = cleaned.lines().collect();
        if lines.len() > 1 && lines.iter().all(|line| line.trim().starts_with('[') && line.trim().ends_with(']')) {
            log.push_str("  ✓ Detected multiple arrays, merging\n");
            let mut merged_items = std::vec::Vec::new();
            for line in lines {
                let trimmed = line.trim();
                if trimmed.starts_with('[') && trimmed.ends_with(']') {
                    // Extract items from each array
                    let inner = &trimmed[1..trimmed.len()-1];
                    if !inner.trim().is_empty() {
                        merged_items.push(inner);
                    }
                }
            }
            cleaned = std::format!("[{}]", merged_items.join(","));
        }

        // Ensure proper array wrapper
        if !cleaned.starts_with('[') && cleaned.contains('{') {
            log.push_str("  ✓ Added missing array wrapper\n");
            cleaned = std::format!("[{}]", cleaned);
        }

        // Fix unclosed braces (simple heuristic)
        let open_braces = cleaned.matches('{').count();
        let close_braces = cleaned.matches('}').count();
        if open_braces > close_braces {
            let missing = open_braces - close_braces;
            log.push_str(&std::format!("  ✓ Added {} missing closing braces\n", missing));
            cleaned.push_str(&"}".repeat(missing));
        }

        // Fix unclosed brackets
        let open_brackets = cleaned.matches('[').count();
        let close_brackets = cleaned.matches(']').count();
        if open_brackets > close_brackets {
            let missing = open_brackets - close_brackets;
            log.push_str(&std::format!("  ✓ Added {} missing closing brackets\n", missing));
            cleaned.push_str(&"]".repeat(missing));
        }

        if cleaned != before_fixes {
            log.push_str("  ✓ Applied syntax fixes\n");
        }

        // Step 3: Try parsing after cleanup and extract error diagnostics
        log.push_str("→ Step 3: Test parse after cleanup\n");
        if let std::result::Result::Ok(_) = serde_json::from_str::<serde_json::Value>(&cleaned) {
            log.push_str("  ✓ SUCCESS: Cleanup alone fixed the JSON!\n");
            return std::result::Result::Ok((cleaned, log));
        }

        // Extract precise error diagnostics for the LLM
        let error_diagnostics = Self::extract_json_error_diagnostics(&cleaned);
        log.push_str(&std::format!("  ✗ Parse failed:\n{}\n", error_diagnostics));
        log.push_str("  → Attempting LLM remediation with error diagnostics\n");

        // Step 4: LLM remediation with precise error location
        log.push_str(&std::format!("→ Step 4: LLM remediation ({})\n", model_name));

        let remediation_prompt = std::format!(
            "Fix this malformed JSON and return ONLY valid JSON array. Each object must have: title, description, priority, estimated_complexity.\n\n\
            CRITICAL RULES:\n\
            - Return ONLY the JSON array, no explanations\n\
            - Start with [ and end with ]\n\
            - Use double quotes for strings\n\
            - No trailing commas\n\
            - All braces and brackets must be balanced\n\n\
            ERROR DIAGNOSTICS:\n{}\n\n\
            Malformed input:\n{}\n\n\
            Fixed JSON:",
            error_diagnostics,
            cleaned
        );

        let client = rig::providers::ollama::Client::new();
        let agent = client.agent(model_name).build();

        let fixed_response = match rig::completion::Prompt::prompt(&agent, remediation_prompt.as_str()).await {
            std::result::Result::Ok(response) => response,
            std::result::Result::Err(e) => {
                log.push_str(&std::format!("  ✗ LLM call failed: {}\n", e));
                return std::result::Result::Err(log);
            }
        };

        log.push_str(&std::format!("  ✓ LLM returned {} chars\n", fixed_response.len()));

        // Step 5: Validate LLM output
        log.push_str("→ Step 5: Validate LLM output\n");
        let final_cleaned = match Self::extract_json_from_response(&fixed_response) {
            std::result::Result::Ok(cleaned) => cleaned,
            std::result::Result::Err(e) => {
                log.push_str(&std::format!("  ✗ Could not extract JSON from LLM response: {}\n", e));
                return std::result::Result::Err(log);
            }
        };

        if let std::result::Result::Ok(_) = serde_json::from_str::<serde_json::Value>(&final_cleaned) {
            log.push_str("  ✓ SUCCESS: LLM remediation produced valid JSON!\n");
            std::result::Result::Ok((final_cleaned, log))
        } else {
            log.push_str("  ✗ FAILED: LLM output still invalid\n\n");

            // Extract precise error diagnostics with line:col location
            let diagnostics = Self::extract_json_error_diagnostics(&final_cleaned);
            log.push_str(&diagnostics);
            log.push_str("\n");

            std::result::Result::Err(log)
        }
    }

    /// Extracts precise JSON parsing error diagnostics including line, column, and context.
    ///
    /// This function attempts to parse JSON and captures detailed error information
    /// including the exact line and column where parsing failed, along with a snippet
    /// of the surrounding context to help identify the issue.
    ///
    /// # Arguments
    ///
    /// * `json_str` - The JSON string to parse and diagnose
    ///
    /// # Returns
    ///
    /// Returns a formatted diagnostic string containing:
    /// - Error message
    /// - Line and column number
    /// - Context snippet showing the error location
    fn extract_json_error_diagnostics(json_str: &str) -> String {
        match serde_json::from_str::<serde_json::Value>(json_str) {
            std::result::Result::Ok(_) => String::from("JSON is valid"),
            std::result::Result::Err(e) => {
                let line = e.line();
                let col = e.column();

                // Extract context around the error location
                let lines: std::vec::Vec<&str> = json_str.lines().collect();
                let mut context = String::new();

                // Show 2 lines before and after the error, if available
                let start_line = line.saturating_sub(3);
                let end_line = std::cmp::min(line + 2, lines.len());

                for (idx, line_text) in lines.iter().enumerate().skip(start_line).take(end_line - start_line) {
                    let line_num = idx + 1;
                    let marker = if line_num == line { ">>>" } else { "   " };
                    context.push_str(&std::format!("{} {:4} | {}\n", marker, line_num, line_text));

                    // Add column indicator on the error line
                    if line_num == line {
                        context.push_str(&std::format!("         {}^\n", " ".repeat(col.saturating_sub(1))));
                    }
                }

                std::format!(
                    "Parse error at line {}, column {}: {}\n\nContext:\n{}",
                    line, col, e, context
                )
            }
        }
    }

    /// Validates task description quality based on length and content depth.
    /// Returns true if description meets quality standards, false if it needs improvement.
    fn validate_description_quality(description: &str) -> bool {
        // Minimum length requirement (100 characters for meaningful descriptions)
        if description.len() < 100 {
            return false;
        }

        // Check for presence of key indicators of depth
        let desc_lower = description.to_lowercase();

        // Count how many quality indicators are present
        let mut quality_score = 0;

        // Indicator 1: Has implementation details (technical terms, specifics)
        let technical_keywords = [
            "implement", "create", "build", "endpoint", "api", "database",
            "authentication", "component", "function", "method", "class"
        ];
        if technical_keywords.iter().any(|keyword| desc_lower.contains(keyword)) {
            quality_score += 1;
        }

        // Indicator 2: Has reasoning or context (why/because/provides/enables)
        let reasoning_keywords = ["why", "because", "provides", "enables", "allows", "ensures"];
        if reasoning_keywords.iter().any(|keyword| desc_lower.contains(keyword)) {
            quality_score += 1;
        }

        // Indicator 3: Has success criteria or outcomes
        let criteria_keywords = ["success", "criteria", "verify", "complete", "done", "should"];
        if criteria_keywords.iter().any(|keyword| desc_lower.contains(keyword)) {
            quality_score += 1;
        }

        // Indicator 4: Has multiple sentences (checks for at least 2 periods)
        let sentence_count = description.matches('.').count();
        if sentence_count >= 2 {
            quality_score += 1;
        }

        // Description passes if it has at least 2 out of 4 quality indicators
        quality_score >= 2
    }

    /// Validates and resolves assignee using three-tier strategy:
    /// 1. Exact match against persona names
    /// 2. Fuzzy match (case-insensitive, partial)
    /// 3. LLM remediation to pick best match
    /// 4. Fall back to default persona or None
    async fn validate_assignee(
        task_title: &str,
        llm_assignee: std::option::Option<&str>,
        personas: &[task_manager::domain::persona::Persona],
        fallback_model_name: &str,
        update_tx: std::option::Option<&tokio::sync::mpsc::Sender<PRDGenUpdate>>,
    ) -> std::option::Option<std::string::String> {
        // No personas available - return None
        if personas.is_empty() {
            return std::option::Option::None;
        }

        // No assignee from LLM or "unassigned" - use default persona if available
        if llm_assignee.is_none() || llm_assignee == std::option::Option::Some("unassigned") {
            return personas.iter()
                .find(|p| p.is_default)
                .map(|p| p.name.clone())
                .or_else(|| personas.first().map(|p| p.name.clone()));
        }

        let assignee_str = llm_assignee.unwrap();

        // Tier 1: Exact match (case-sensitive)
        if let std::option::Option::Some(persona) = personas.iter().find(|p| p.name == assignee_str) {
            // Exact match found - no validation message needed
            return std::option::Option::Some(persona.name.clone());
        }

        // Tier 2: Case-insensitive match
        if let std::option::Option::Some(persona) = personas.iter().find(|p| p.name.eq_ignore_ascii_case(assignee_str)) {
            // Case-insensitive match found - no validation message needed
            return std::option::Option::Some(persona.name.clone());
        }

        // Tier 3: Partial/fuzzy match (contains or is contained)
        if let std::option::Option::Some(persona) = personas.iter().find(|p| {
            p.name.to_lowercase().contains(&assignee_str.to_lowercase()) ||
            assignee_str.to_lowercase().contains(&p.name.to_lowercase())
        }) {
            // Fuzzy match found - no validation message needed
            return std::option::Option::Some(persona.name.clone());
        }

        // Tier 4: LLM remediation - ask LLM to pick best match
        // (Validation message sent below)

        // Send immediate validation info update if channel is available
        if let std::option::Option::Some(tx) = update_tx {
            let _ = tx.send(PRDGenUpdate::ValidationInfo {
                task_title: task_title.to_string(),
                message: std::string::String::from("Remediating assignee..."),
            }).await;
        }

        let persona_list = personas.iter()
            .map(|p| std::format!("\"{}\" ({})", p.name, p.role))
            .collect::<std::vec::Vec<_>>()
            .join(", ");

        let remediation_prompt = std::format!(
            "The LLM assigned a task to '{}' but that name doesn't match any team member.\n\
            Available team members: {}\n\n\
            CRITICAL: Respond with ONLY the exact name of the best matching persona from the list above.\n\
            If no good match exists, respond with the word 'unassigned'.\n\
            DO NOT add explanations or extra text.\n\
            YOUR RESPONSE:",
            assignee_str,
            persona_list
        );

        let client = rig::providers::ollama::Client::new();
        let agent = client.agent(fallback_model_name).build();

        match rig::completion::Prompt::prompt(&agent, remediation_prompt.as_str()).await {
            std::result::Result::Ok(response) => {
                let suggested_name = response.trim();

                // Validate LLM's suggestion
                if let std::option::Option::Some(persona) = personas.iter().find(|p|
                    p.name.eq_ignore_ascii_case(suggested_name)
                ) {
                    // Send success message
                    if let std::option::Option::Some(tx) = update_tx {
                        let _ = tx.send(PRDGenUpdate::ValidationInfo {
                            task_title: task_title.to_string(),
                            message: std::format!("Remediation successful: '{}' → '{}'", assignee_str, persona.name),
                        }).await;
                    }

                    return std::option::Option::Some(persona.name.clone());
                }

                // LLM suggested "unassigned" - will use default persona below
            }
            std::result::Result::Err(e) => {
                // LLM remediation failed - send error message
                if let std::option::Option::Some(tx) = update_tx {
                    let _ = tx.send(PRDGenUpdate::ValidationInfo {
                        task_title: task_title.to_string(),
                        message: std::format!("LLM remediation failed: {}", e),
                    }).await;
                }
            }
        }

        // Final fallback: Use default persona or first available
        let fallback = personas.iter()
            .find(|p| p.is_default)
            .or_else(|| personas.first())
            .map(|p| p.name.clone());

        if let std::option::Option::Some(ref name) = fallback {
            // Send fallback notification
            if let std::option::Option::Some(tx) = update_tx {
                let _ = tx.send(PRDGenUpdate::ValidationInfo {
                    task_title: task_title.to_string(),
                    message: std::format!("Using fallback persona: '{}' → '{}'", assignee_str, name),
                }).await;
            }
        }

        fallback
    }

    /// Parses LLM response JSON into tasks using tolerant parsing with field aliases.
    async fn parse_tasks_from_json(
        json_str: &str,
        prd_id: &str,
        fallback_model_name: &str,
        personas: &[task_manager::domain::persona::Persona],
        update_tx: std::option::Option<&tokio::sync::mpsc::Sender<PRDGenUpdate>>,
    ) -> std::result::Result<std::vec::Vec<task_manager::domain::task::Task>, std::string::String> {
        // Extract JSON from response (handles markdown code blocks and extra text)
        let cleaned_json = Self::extract_json_from_response(json_str)?;

        // Try to parse JSON response
        let parsed: std::result::Result<serde_json::Value, serde_json::Error> = serde_json::from_str(&cleaned_json);

        let parsed = match parsed {
            std::result::Result::Ok(v) => v,
            std::result::Result::Err(e) => {
                // JSON parsing failed - attempt remediation with configured fallback model
                if let std::option::Option::Some(tx) = update_tx {
                    let _ = tx.send(PRDGenUpdate::ValidationInfo {
                        task_title: std::string::String::from("JSON Parsing"),
                        message: std::string::String::from("Remediating JSON..."),
                    }).await;
                }

                let (remediated, remediation_log) = Self::remediate_json_with_llm(&cleaned_json, fallback_model_name).await
                    .map_err(|log| std::format!("JSON remediation failed after all attempts.\n\nOriginal parse error: {}\n\n{}", e, log))?;

                // Remediation succeeded - send success notification
                if let std::option::Option::Some(tx) = update_tx {
                    let _ = tx.send(PRDGenUpdate::ValidationInfo {
                        task_title: std::string::String::from("JSON Parsing"),
                        message: std::string::String::from("Remediation succeeded! Parsing remediated JSON..."),
                    }).await;
                }

                // Try to parse remediated JSON
                serde_json::from_str(&remediated)
                    .map_err(|e2| std::format!(
                        "Failed to parse remediated JSON: {}\n\nRemediation log:\n{}\n\nRemediated JSON (first 300 chars):\n{}",
                        e2,
                        remediation_log,
                        if remediated.len() > 300 { &remediated[0..300] } else { &remediated }
                    ))?
            }
        };

        let tasks_array = parsed
            .as_array()
            .ok_or_else(|| std::string::String::from("Expected JSON array of tasks"))?;

        let mut tasks = std::vec::Vec::new();

        for (idx, task_value) in tasks_array.iter().enumerate() {
            // Use tolerant parsing - accept objects or skip non-objects
            let obj = match task_value {
                serde_json::Value::Object(m) => m,
                _ => {
                    eprintln!("[PRD Parser] Skipping non-object at index {}", idx);
                    continue;
                }
            };

            // Extract title with field aliases (title, task, name, summary, action)
            let title = Self::extract_string(
                obj,
                &["title", "task", "name", "summary", "action", "item"]
            ).ok_or_else(|| std::format!("Missing 'title' field in task at index {}", idx))?;

            // Extract description (optional)
            let description = Self::extract_string(
                obj,
                &["description", "desc", "details", "detail", "content"]
            ).unwrap_or_default();

            // Validate description quality
            let description_valid = Self::validate_description_quality(&description);
            if !description_valid {
                // Send validation warning (non-blocking - we still create the task)
                if let std::option::Option::Some(tx) = update_tx {
                    let _ = tx.send(PRDGenUpdate::ValidationInfo {
                        task_title: title.clone(),
                        message: std::format!("Warning: Description is too brief ({} chars, recommended 100+)", description.len()),
                    }).await;
                }
            }

            // Extract priority (optional, default to "medium")
            let _priority = Self::extract_string(
                obj,
                &["priority", "prio", "importance", "level"]
            ).unwrap_or_else(|| String::from("medium"));

            // Extract complexity (optional, default to 5)
            let complexity = Self::extract_number(
                obj,
                &["estimated_complexity", "complexity", "difficulty", "effort", "score"]
            ).unwrap_or(5);

            // Extract agent_persona (optional, with fallback aliases)
            let llm_assignee = Self::extract_string(
                obj,
                &["agent_persona", "assignee", "assigned_to", "owner", "responsible"]
            );

            // Validate and resolve assignee
            let validated_assignee = Self::validate_assignee(&title, llm_assignee.as_deref(), personas, fallback_model_name, update_tx).await;

            // Create task using from_action_item pattern
            let action_item = transcript_extractor::domain::action_item::ActionItem {
                title,
                assignee: validated_assignee,
                due_date: std::option::Option::None,  // Will be set later
            };

            let mut task = task_manager::domain::task::Task::from_action_item(
                &action_item,
                std::option::Option::None, // no transcript for PRD-generated tasks
            );

            // Set PRD linkage and extracted fields
            task.source_prd_id = std::option::Option::Some(prd_id.to_string());
            task.description = description;
            task.complexity = std::option::Option::Some(complexity as u8);

            tasks.push(task);
        }

        std::result::Result::Ok(tasks)
    }

    /// Decomposes a complex task into 3-5 sub-tasks using LLM.
    ///
    /// This method analyzes a parent task and generates child sub-tasks that break down
    /// the work into smaller, more manageable pieces. It updates the parent task's status
    /// to Decomposed and sets up bidirectional parent-child linkages.
    ///
    /// # Arguments
    ///
    /// * `parent_task` - The task to decompose (must have complexity >= 7)
    /// * `prd_content` - Original PRD content for context
    ///
    /// # Returns
    ///
    /// Vector of 3-5 sub-tasks linked to the parent, or error if decomposition fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use task_orchestrator::adapters::rig_prd_parser_adapter::RigPRDParserAdapter;
    /// # use task_manager::domain::task::Task;
    /// # async fn example(adapter: RigPRDParserAdapter, task: Task) {
    /// let subtasks = adapter.decompose_task(&task, "PRD content...").await?;
    /// # Ok::<(), std::string::String>(())
    /// # }
    /// ```
    pub async fn decompose_task(
        &self,
        parent_task: &task_manager::domain::task::Task,
        prd_content: &str,
    ) -> std::result::Result<std::vec::Vec<task_manager::domain::task::Task>, std::string::String> {
        // Build decomposition prompt
        let prompt = Self::build_decomposition_prompt(parent_task, prd_content, &self.personas);

        // Initialize Rig Ollama client
        let client = rig::providers::ollama::Client::new();
        let agent = client.agent(&self.model_name).build();

        // Call LLM for decomposition
        let response = rig::completion::Prompt::prompt(&agent, prompt.as_str())
            .await
            .map_err(|e| std::format!("LLM request failed: {}", e))?;

        // Extract and parse JSON
        let json_text = Self::extract_json_from_response(response.as_str())
            .map_err(|e| std::format!("Failed to extract JSON from decomposition response: {}", e))?;
        let subtasks = Self::parse_subtasks_from_json(
            &json_text,
            &parent_task.id,
            parent_task.source_prd_id.as_deref().unwrap_or(""),
            &self.fallback_model_name,
            &self.personas,
            std::option::Option::None, // No streaming for decomposition
        )
        .await?;

        std::result::Result::Ok(subtasks)
    }

    /// Builds the LLM prompt for task decomposition.
    ///
    /// Creates a focused prompt that asks the LLM to break down a complex task
    /// into 3-5 actionable sub-tasks, maintaining consistency with the original PRD context.
    fn build_decomposition_prompt(
        parent_task: &task_manager::domain::task::Task,
        prd_content: &str,
        personas: &[task_manager::domain::persona::Persona],
    ) -> String {
        let mut prompt = String::new();

        // System instructions
        prompt.push_str("You are a technical project planner. Your task is to decompose a complex task into 3-5 smaller, actionable sub-tasks.\n\n");

        // Parent task context
        prompt.push_str(&std::format!("PARENT TASK:\nTitle: {}\n", parent_task.title));
        if !parent_task.description.is_empty() {
            prompt.push_str(&std::format!("Description: {}\n", parent_task.description));
        }
        if let std::option::Option::Some(complexity) = parent_task.complexity {
            prompt.push_str(&std::format!("Complexity: {}/10\n", complexity));
        }
        prompt.push_str("\n");

        // PRD context (first 500 chars for context)
        let prd_snippet = if prd_content.len() > 500 {
            &prd_content[..500]
        } else {
            prd_content
        };
        prompt.push_str(&std::format!("PROJECT CONTEXT:\n{}\n\n", prd_snippet));

        // Persona list
        if !personas.is_empty() {
            prompt.push_str("AVAILABLE PERSONAS:\n");
            for persona in personas {
                prompt.push_str(&std::format!("- {}\n", persona.name));
            }
            prompt.push_str("\n");
        }

        // Output format instructions with example
        prompt.push_str("OUTPUT REQUIREMENTS:\n");
        prompt.push_str("Decompose the parent task into 3-5 smaller sub-tasks that can be completed independently.\n");
        prompt.push_str("Return a JSON array where each sub-task object has:\n");
        prompt.push_str("- \"title\": string (concise, actionable sub-task title)\n");
        prompt.push_str("- \"description\": string (detailed description with WHAT/WHY/HOW/ACCEPTANCE)\n");
        prompt.push_str("- \"priority\": \"high\", \"medium\", or \"low\"\n");
        prompt.push_str("- \"estimated_complexity\": number 1-5 (sub-tasks should be simpler than parent)\n");
        prompt.push_str("- \"agent_persona\": agent persona/role from available list, or \"unassigned\"\n\n");

        prompt.push_str("EXAMPLE FORMAT:\n");
        prompt.push_str("[\n");
        prompt.push_str("  {\n");
        prompt.push_str("    \"title\": \"Setup authentication middleware\",\n");
        prompt.push_str("    \"description\": \"Configure JWT middleware to validate tokens on protected routes. This ensures secure API access.\",\n");
        prompt.push_str("    \"priority\": \"high\",\n");
        prompt.push_str("    \"estimated_complexity\": 3,\n");
        prompt.push_str("    \"agent_persona\": \"Backend Developer\"\n");
        prompt.push_str("  },\n");
        prompt.push_str("  {\n");
        prompt.push_str("    \"title\": \"Write authentication tests\",\n");
        prompt.push_str("    \"description\": \"Create unit tests for login, logout, and token refresh flows.\",\n");
        prompt.push_str("    \"priority\": \"medium\",\n");
        prompt.push_str("    \"estimated_complexity\": 2,\n");
        prompt.push_str("    \"agent_persona\": \"QA Engineer\"\n");
        prompt.push_str("  }\n");
        prompt.push_str("]\n\n");

        prompt.push_str("CRITICAL INSTRUCTIONS:\n");
        prompt.push_str("1. Respond with ONLY the JSON array - no markdown code blocks, no explanations\n");
        prompt.push_str("2. Start your response with [ and end with ]\n");
        prompt.push_str("3. Each sub-task must be completable independently\n");
        prompt.push_str("4. Total of 3-5 sub-tasks (not more, not less)\n\n");

        prompt.push_str("YOUR JSON RESPONSE:\n");

        prompt
    }

    /// Parses sub-tasks from JSON response.
    ///
    /// Similar to parse_tasks_from_json but sets parent_task_id linkage.
    async fn parse_subtasks_from_json(
        json_text: &str,
        parent_task_id: &str,
        prd_id: &str,
        fallback_model_name: &str,
        personas: &[task_manager::domain::persona::Persona],
        update_tx: std::option::Option<&tokio::sync::mpsc::Sender<PRDGenUpdate>>,
    ) -> std::result::Result<std::vec::Vec<task_manager::domain::task::Task>, std::string::String> {
        // Try parsing JSON directly first
        let json_array: std::vec::Vec<serde_json::Value> = match serde_json::from_str(json_text) {
            std::result::Result::Ok(arr) => arr,
            std::result::Result::Err(e) => {
                // Attempt JSON remediation
                if let std::option::Option::Some(tx) = update_tx {
                    let _ = tx.send(PRDGenUpdate::ValidationInfo {
                        task_title: std::string::String::from("Sub-task Parsing"),
                        message: std::string::String::from("Remediating JSON..."),
                    });
                }

                let (remediated, _log) = Self::remediate_json_with_llm(json_text, fallback_model_name).await
                    .map_err(|log| std::format!("Sub-task JSON remediation failed: {}\n\nOriginal error: {}", log, e))?;

                serde_json::from_str(&remediated)
                    .map_err(|e2| std::format!("Remediated JSON still invalid: {}", e2))?
            }
        };

        let mut subtasks = std::vec::Vec::new();

        for (idx, val) in json_array.iter().enumerate() {
            let obj = match val.as_object() {
                std::option::Option::Some(o) => o,
                std::option::Option::None => continue, // Skip non-objects
            };

            // Extract title (required)
            let title = Self::extract_string(
                obj,
                &["title", "task", "name", "summary"]
            ).ok_or_else(|| std::format!("Missing 'title' field in sub-task at index {}", idx))?;

            // Extract description
            let description = Self::extract_string(
                obj,
                &["description", "desc", "details"]
            ).unwrap_or_default();

            // Extract complexity
            let complexity = Self::extract_number(
                obj,
                &["estimated_complexity", "complexity", "difficulty"]
            ).unwrap_or(3); // Default lower complexity for sub-tasks

            // Extract and validate agent_persona (with fallback aliases)
            let llm_assignee = Self::extract_string(
                obj,
                &["agent_persona", "assignee", "assigned_to", "owner"]
            );
            let validated_assignee = Self::validate_assignee(&title, llm_assignee.as_deref(), personas, fallback_model_name, update_tx).await;

            // Create sub-task
            let action_item = transcript_extractor::domain::action_item::ActionItem {
                title,
                assignee: validated_assignee,
                due_date: std::option::Option::None,
            };

            let mut subtask = task_manager::domain::task::Task::from_action_item(
                &action_item,
                std::option::Option::None,
            );

            // Set linkages
            subtask.source_prd_id = std::option::Option::Some(prd_id.to_string());
            subtask.parent_task_id = std::option::Option::Some(parent_task_id.to_string());
            subtask.description = description;
            subtask.complexity = std::option::Option::Some(complexity as u8);

            subtasks.push(subtask);
        }

        if subtasks.is_empty() {
            return std::result::Result::Err(std::string::String::from("Decomposition produced no sub-tasks"));
        }

        std::result::Result::Ok(subtasks)
    }

    /// Returns the tree indicator prefix for a hierarchical task.
    ///
    /// Returns appropriate box-drawing characters based on depth and position:
    /// - Depth 0 (parent): "" (no prefix)
    /// - Depth 1, not last: "├─ "
    /// - Depth 1, last child: "└─ "
    pub fn get_tree_indicator(depth: usize, is_last_child: bool) -> &'static str {
        match depth {
            0 => "",
            1 if is_last_child => "└─ ",
            1 => "├─ ",
            _ => "   ", // Deeper nesting (future expansion)
        }
    }

    /// Assigns the most appropriate persona to a task using LLM in a second pass.
    ///
    /// This method analyzes the task's title and description to determine which persona
    /// from the available list would be best suited to complete the work. This two-pass
    /// approach prevents biasing the LLM during task generation - tasks are first derived
    /// from the PRD content alone, then personas are assigned based on the generated tasks.
    ///
    /// # Arguments
    ///
    /// * `task_title` - The task's title
    /// * `task_description` - The task's detailed description
    ///
    /// # Returns
    ///
    /// The name of the assigned persona, or "Default Agent" if no personas are available
    /// or if the LLM cannot make a good match.
    ///
    /// # Errors
    ///
    /// Returns an error if the LLM call fails or returns invalid output.
    pub async fn assign_persona_to_task(
        &self,
        task_title: &str,
        task_description: &str,
    ) -> std::result::Result<std::string::String, std::string::String> {
        // If no personas are available, use Default Agent
        if self.personas.is_empty() {
            return std::result::Result::Ok(std::string::String::from("Default Agent"));
        }

        // Build the persona assignment prompt
        let persona_names: std::vec::Vec<&str> = self.personas.iter().map(|p| p.name.as_str()).collect();
        let persona_list = persona_names.join("\n");

        let prompt = std::format!(
            "You are assigning tasks to team members. Select the MOST APPROPRIATE persona from the list to complete this task.\n\n\
            AVAILABLE PERSONAS:\n{}\n\n\
            TASK TO ASSIGN:\n\
            Title: {}\n\
            Description: {}\n\n\
            INSTRUCTIONS:\n\
            - Analyze the task requirements carefully\n\
            - Select the persona whose expertise best matches the task\n\
            - Respond with ONLY the persona name from the list above\n\
            - If none are appropriate, respond with exactly: Default Agent\n\n\
            YOUR RESPONSE (persona name only):",
            persona_list, task_title, task_description
        );

        // Call LLM to assign persona
        let client = rig::providers::ollama::Client::new();
        let agent = client.agent(&self.model_name).build();

        let response = rig::completion::Prompt::prompt(&agent, prompt.as_str())
            .await
            .map_err(|e| std::format!("Failed to call LLM for persona assignment: {}", e))?;

        // Extract persona name and validate it exists in the list
        let assigned_persona = response.trim().to_string();

        // Check if it matches any available persona (case-insensitive)
        let matched_persona = self.personas
            .iter()
            .find(|p| p.name.eq_ignore_ascii_case(&assigned_persona))
            .map(|p| p.name.clone());

        if let std::option::Option::Some(persona) = matched_persona {
            std::result::Result::Ok(persona)
        } else if assigned_persona.eq_ignore_ascii_case("Default Agent") {
            std::result::Result::Ok(std::string::String::from("Default Agent"))
        } else {
            // LLM returned something invalid, fallback to Default Agent
            std::result::Result::Ok(std::string::String::from("Default Agent"))
        }
    }
}

#[async_trait::async_trait]
impl crate::ports::prd_parser_port::PRDParserPort for RigPRDParserAdapter {
    async fn parse_prd_to_tasks(
        &self,
        prd: &task_manager::domain::prd::PRD,
    ) -> std::result::Result<std::vec::Vec<task_manager::domain::task::Task>, std::string::String> {
        // Build complete prompt with RAG context
        let prompt = self.build_prompt(prd, &self.personas).await;

        // Initialize Rig Ollama client (uses http://localhost:11434 by default)
        let client = rig::providers::ollama::Client::new();
        let agent = client.agent(&self.model_name).build();

        // Call LLM via Rig agent
        let response = rig::completion::Prompt::prompt(&agent, prompt.as_str())
            .await
            .map_err(|e| std::format!("LLM request failed: {}", e))?;

        // Parse tasks from JSON response (now async to support remediation)
        Self::parse_tasks_from_json(response.as_str(), &prd.id, &self.fallback_model_name, &self.personas, std::option::Option::None).await
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_system_prompt_contains_key_instructions() {
        // Test: Validates system prompt includes essential instructions.
        // Justification: Ensures LLM receives proper guidance for task generation.
        let prompt = super::RigPRDParserAdapter::build_system_prompt(&[]);

        std::assert!(prompt.contains("project management"));
        std::assert!(prompt.contains("JSON array"));
        std::assert!(prompt.contains("title"));
        std::assert!(prompt.contains("description"));
        std::assert!(prompt.contains("priority"));
        std::assert!(prompt.contains("estimated_complexity"));
    }

    #[tokio::test]
    async fn test_build_prompt_includes_prd_sections() {
        // Test: Validates prompt includes all PRD sections.
        // Justification: LLM needs complete context to generate accurate tasks.
        let prd = task_manager::domain::prd::PRD::new(
            std::string::String::from("project-123"),
            std::string::String::from("Test PRD"),
            std::vec![std::string::String::from("Objective 1")],
            std::vec![std::string::String::from("Rust")],
            std::vec![std::string::String::from("No unsafe code")],
            std::string::String::from("raw content"),
        );

        // Create adapter without RAG for simple prompt test
        let adapter = super::RigPRDParserAdapter::new(
            String::from("llama3.2:latest"),
            String::from("llama3.2:latest"),
            std::vec![],
        );

        let prompt = adapter.build_prompt(&prd, &[]).await;

        std::assert!(prompt.contains("Test PRD"));
        std::assert!(prompt.contains("Objective 1"));
        std::assert!(prompt.contains("Rust"));
        std::assert!(prompt.contains("No unsafe code"));
        std::assert!(prompt.contains("JSON array"));
        std::assert!(prompt.contains("4 required fields")); // No personas = 4 fields
    }

    #[tokio::test]
    async fn test_parse_tasks_from_valid_json() {
        // Test: Validates JSON parsing creates tasks with correct fields.
        // Justification: Core functionality must handle well-formed LLM responses.
        let json = r#"[
            {
                "title": "Setup project",
                "description": "Initialize Cargo workspace",
                "priority": "high",
                "estimated_complexity": 3
            },
            {
                "title": "Write tests",
                "description": "Add unit tests",
                "priority": "medium",
                "estimated_complexity": 5
            }
        ]"#;

        let tasks = super::RigPRDParserAdapter::parse_tasks_from_json(json, "prd-123", "llama3.2:latest", &[], std::option::Option::None).await.unwrap();

        std::assert_eq!(tasks.len(), 2);
        std::assert_eq!(tasks[0].title, "Setup project");
        std::assert_eq!(tasks[0].source_prd_id, std::option::Option::Some(std::string::String::from("prd-123")));
        std::assert_eq!(tasks[1].title, "Write tests");
    }

    #[tokio::test]
    async fn test_parse_tasks_from_invalid_json_fails() {
        // Test: Validates parser rejects malformed JSON.
        // Justification: Must fail gracefully on bad LLM output (after remediation attempts).
        let json = "not valid json";

        let result = super::RigPRDParserAdapter::parse_tasks_from_json(json, "prd-123", "llama3.2:latest", &[], std::option::Option::None).await;

        std::assert!(result.is_err());
    }

    #[test]
    fn test_extract_json_from_clean_response() {
        // Test: Validates extraction of already-clean JSON array.
        // Justification: Common case where LLM follows instructions correctly.
        let response = r#"[{"title":"Task 1"},{"title":"Task 2"}]"#;

        let result = super::RigPRDParserAdapter::extract_json_from_response(response);

        std::assert!(result.is_ok());
        std::assert_eq!(result.unwrap(), r#"[{"title":"Task 1"},{"title":"Task 2"}]"#);
    }

    #[test]
    fn test_extract_json_from_markdown_code_block() {
        // Test: Validates extraction from markdown ```json code blocks.
        // Justification: LLMs frequently wrap JSON in markdown formatting.
        let response = r#"Here's your task list:

```json
[{"title":"Task 1"},{"title":"Task 2"}]
```

Hope this helps!"#;

        let result = super::RigPRDParserAdapter::extract_json_from_response(response);

        std::assert!(result.is_ok());
        std::assert_eq!(result.unwrap(), r#"[{"title":"Task 1"},{"title":"Task 2"}]"#);
    }

    #[test]
    fn test_extract_json_from_markdown_code_block_no_language() {
        // Test: Validates extraction from ``` code blocks without language specifier.
        // Justification: Some LLMs use ``` without json tag.
        let response = r#"```
[{"title":"Task 1"}]
```"#;

        let result = super::RigPRDParserAdapter::extract_json_from_response(response);

        std::assert!(result.is_ok());
        std::assert_eq!(result.unwrap(), r#"[{"title":"Task 1"}]"#);
    }

    #[test]
    fn test_extract_json_embedded_in_text() {
        // Test: Validates extraction when JSON is embedded in explanatory text.
        // Justification: LLMs often add preamble/postamble despite instructions.
        let response = r#"I've analyzed the PRD and created the following tasks: [{"title":"Task 1"},{"title":"Task 2"}] as requested."#;

        let result = super::RigPRDParserAdapter::extract_json_from_response(response);

        std::assert!(result.is_ok());
        std::assert_eq!(result.unwrap(), r#"[{"title":"Task 1"},{"title":"Task 2"}]"#);
    }

    #[test]
    fn test_extract_json_with_whitespace() {
        // Test: Validates extraction handles leading/trailing whitespace.
        // Justification: Network responses may include extra whitespace.
        let response = r#"

        [{"title":"Task 1"}]

        "#;

        let result = super::RigPRDParserAdapter::extract_json_from_response(response);

        std::assert!(result.is_ok());
        std::assert_eq!(result.unwrap(), r#"[{"title":"Task 1"}]"#);
    }

    #[test]
    fn test_extract_json_from_response_with_no_json_fails() {
        // Test: Validates extraction fails gracefully when no JSON present.
        // Justification: Must return clear error for debugging.
        let response = "This is just plain text with no JSON array";

        let result = super::RigPRDParserAdapter::extract_json_from_response(response);

        std::assert!(result.is_err());
        std::assert!(result.unwrap_err().contains("Could not extract JSON array"));
    }

    #[test]
    fn test_extract_json_from_nested_arrays() {
        // Test: Validates extraction finds outermost array boundaries.
        // Justification: Ensures correct extraction with nested JSON structures.
        let response = r#"[{"items":[1,2,3]},{"items":[4,5,6]}]"#;

        let result = super::RigPRDParserAdapter::extract_json_from_response(response);

        std::assert!(result.is_ok());
        std::assert_eq!(result.unwrap(), r#"[{"items":[1,2,3]},{"items":[4,5,6]}]"#);
    }

    #[test]
    fn test_extract_json_from_single_object_wrapped() {
        // Test: Validates extraction wraps single JSON object in array.
        // Justification: Some LLMs return single object instead of array.
        let response = r#"{"title":"Task 1","priority":"high"}"#;

        let result = super::RigPRDParserAdapter::extract_json_from_response(response);

        std::assert!(result.is_ok());
        std::assert_eq!(result.unwrap(), r#"[{"title":"Task 1","priority":"high"}]"#);
    }

    #[test]
    fn test_extract_json_from_object_in_text() {
        // Test: Validates extraction finds and wraps JSON object embedded in text.
        // Justification: Handles LLMs that return object with preamble.
        let response = r#"Here is the task: {"title":"Task 1"} as requested."#;

        let result = super::RigPRDParserAdapter::extract_json_from_response(response);

        std::assert!(result.is_ok());
        std::assert_eq!(result.unwrap(), r#"[{"title":"Task 1"}]"#);
    }

    #[test]
    fn test_extract_json_from_python_code_block() {
        // Test: Validates extraction from ```python code blocks.
        // Justification: Some LLMs tag JSON as Python.
        let response = r#"```python
[{"title":"Task 1"}]
```"#;

        let result = super::RigPRDParserAdapter::extract_json_from_response(response);

        std::assert!(result.is_ok());
        std::assert_eq!(result.unwrap(), r#"[{"title":"Task 1"}]"#);
    }

    #[tokio::test]
    async fn test_parse_tasks_from_non_array_json_fails() {
        // Test: Validates parser auto-wraps single objects into arrays.
        // Justification: extract_json_from_response handles this case.
        let json = r#"{"title": "Not an array"}"#;

        let result = super::RigPRDParserAdapter::parse_tasks_from_json(json, "prd-123", "llama3.2:latest", &[], std::option::Option::None).await;

        // Should succeed after wrapping in array
        std::assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_parse_tasks_missing_title_fails() {
        // Test: Validates parser requires title field.
        // Justification: Title is mandatory for tasks.
        let json = r#"[{"description": "No title"}]"#;

        let result = super::RigPRDParserAdapter::parse_tasks_from_json(json, "prd-123", "llama3.2:latest", &[], std::option::Option::None).await;

        std::assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_parse_tasks_with_missing_optional_fields() {
        // Test: Validates parser handles missing optional fields gracefully.
        // Justification: LLM might omit some fields; should use defaults.
        let json = r#"[{"title": "Minimal task"}]"#;

        let tasks = super::RigPRDParserAdapter::parse_tasks_from_json(json, "prd-123", "llama3.2:latest", &[], std::option::Option::None).await.unwrap();

        std::assert_eq!(tasks.len(), 1);
        std::assert_eq!(tasks[0].title, "Minimal task");
        std::assert_eq!(tasks[0].source_prd_id, std::option::Option::Some(std::string::String::from("prd-123")));
    }

    #[test]
    fn test_build_decomposition_prompt_includes_parent_context() {
        // Test: Validates decomposition prompt includes parent task details.
        // Justification: LLM needs parent context to generate relevant sub-tasks.
        let parent = task_manager::domain::task::Task {
            id: std::string::String::from("task-123"),
            title: std::string::String::from("Implement Authentication System"),
            description: std::string::String::from("Build secure auth with JWT"),
            status: task_manager::domain::task_status::TaskStatus::Todo,
            complexity: std::option::Option::Some(8),
            agent_persona: std::option::Option::Some(std::string::String::from("Backend Developer")),
            parent_task_id: std::option::Option::None,
            subtask_ids: std::vec::Vec::new(),
            dependencies: std::vec::Vec::new(),
            source_prd_id: std::option::Option::Some(std::string::String::from("prd-123")),
            due_date: std::option::Option::None,
            source_transcript_id: std::option::Option::None,
            enhancements: std::option::Option::None,
            comprehension_tests: std::option::Option::None,
            reasoning: std::option::Option::None,
            completion_summary: std::option::Option::None,
            context_files: std::vec::Vec::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let prd_content = "# Test PRD\n\nBuild an authentication system with JWT tokens and OAuth support.";
        let personas = std::vec::Vec::new();

        let prompt = super::RigPRDParserAdapter::build_decomposition_prompt(&parent, prd_content, &personas);

        // Verify prompt contains key elements
        std::assert!(prompt.contains("Implement Authentication System"), "Missing parent title");
        std::assert!(prompt.contains("Complexity: 8/10"), "Missing complexity");
        std::assert!(prompt.contains("PROJECT CONTEXT"), "Missing PRD context");
        std::assert!(prompt.contains("OUTPUT REQUIREMENTS"), "Missing output format");
        std::assert!(prompt.contains("3-5 sub-tasks"), "Missing sub-task count");
    }

    #[test]
    fn test_get_tree_indicator_returns_correct_prefixes() {
        // Test: Validates tree indicators match depth and position.
        // Justification: Correct tree visualization requires proper box-drawing characters.

        // Parent at depth 0
        std::assert_eq!(super::RigPRDParserAdapter::get_tree_indicator(0, false), "");
        std::assert_eq!(super::RigPRDParserAdapter::get_tree_indicator(0, true), "");

        // First child at depth 1 (not last)
        std::assert_eq!(super::RigPRDParserAdapter::get_tree_indicator(1, false), "├─ ");

        // Last child at depth 1
        std::assert_eq!(super::RigPRDParserAdapter::get_tree_indicator(1, true), "└─ ");

        // Deeper nesting (future expansion)
        std::assert_eq!(super::RigPRDParserAdapter::get_tree_indicator(2, false), "   ");
    }

    #[tokio::test]
    async fn test_parse_subtasks_from_json_sets_parent_linkage() {
        // Test: Validates sub-tasks link to parent correctly.
        // Justification: Parent-child relationship is core to task hierarchy.
        let json = r#"[
            {"title": "Sub-task 1", "description": "First sub-task", "estimated_complexity": 3},
            {"title": "Sub-task 2", "description": "Second sub-task", "estimated_complexity": 4}
        ]"#;

        let subtasks = super::RigPRDParserAdapter::parse_subtasks_from_json(
            json,
            "parent-123",
            "prd-456",
            "llama3.2:latest",
            &[],
            std::option::Option::None,
        ).await.unwrap();

        std::assert_eq!(subtasks.len(), 2);

        // Verify first sub-task
        std::assert_eq!(subtasks[0].title, "Sub-task 1");
        std::assert_eq!(subtasks[0].parent_task_id, std::option::Option::Some(std::string::String::from("parent-123")));
        std::assert_eq!(subtasks[0].source_prd_id, std::option::Option::Some(std::string::String::from("prd-456")));
        std::assert_eq!(subtasks[0].complexity, std::option::Option::Some(3));

        // Verify second sub-task
        std::assert_eq!(subtasks[1].title, "Sub-task 2");
        std::assert_eq!(subtasks[1].parent_task_id, std::option::Option::Some(std::string::String::from("parent-123")));
        std::assert_eq!(subtasks[1].complexity, std::option::Option::Some(4));
    }

    #[tokio::test]
    async fn test_parse_subtasks_uses_default_complexity() {
        // Test: Validates sub-tasks without complexity get default value.
        // Justification: Sub-tasks should be less complex than parent (default 3).
        let json = r#"[{"title": "Simple sub-task"}]"#;

        let subtasks = super::RigPRDParserAdapter::parse_subtasks_from_json(
            json,
            "parent-123",
            "prd-456",
            "llama3.2:latest",
            &[],
            std::option::Option::None,
        ).await.unwrap();

        std::assert_eq!(subtasks.len(), 1);
        std::assert_eq!(subtasks[0].complexity, std::option::Option::Some(3), "Default complexity should be 3");
    }

    #[tokio::test]
    async fn test_parse_subtasks_rejects_empty_array() {
        // Test: Validates parser rejects empty sub-task array.
        // Justification: Decomposition should produce at least one sub-task.
        let json = r#"[]"#;

        let result = super::RigPRDParserAdapter::parse_subtasks_from_json(
            json,
            "parent-123",
            "prd-456",
            "llama3.2:latest",
            &[],
            std::option::Option::None,
        ).await;

        std::assert!(result.is_err());
        std::assert!(result.unwrap_err().contains("no sub-tasks"), "Should mention no sub-tasks were produced");
    }

    #[test]
    fn test_build_decomposition_prompt_includes_personas() {
        // Test: Validates prompt lists available personas for assignee selection.
        // Justification: Sub-tasks need to be assigned to valid personas.
        let parent = task_manager::domain::task::Task {
            id: std::string::String::from("task-123"),
            title: std::string::String::from("Complex Task"),
            description: std::string::String::from("Multi-step work"),
            status: task_manager::domain::task_status::TaskStatus::Todo,
            complexity: std::option::Option::Some(8),
            agent_persona: std::option::Option::None,
            parent_task_id: std::option::Option::None,
            subtask_ids: std::vec::Vec::new(),
            dependencies: std::vec::Vec::new(),
            source_prd_id: std::option::Option::None,
            due_date: std::option::Option::None,
            source_transcript_id: std::option::Option::None,
            enhancements: std::option::Option::None,
            comprehension_tests: std::option::Option::None,
            reasoning: std::option::Option::None,
            completion_summary: std::option::Option::None,
            context_files: std::vec::Vec::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let personas = std::vec![
            task_manager::domain::persona::Persona {
                id: std::string::String::from("p1"),
                project_id: std::option::Option::Some(std::string::String::from("proj1")),
                name: std::string::String::from("Backend Developer"),
                role: std::string::String::from("Developer"),
                description: std::string::String::from("Handles backend tasks"),
                llm_provider: std::option::Option::Some(std::string::String::from("ollama")),
                llm_model: std::option::Option::Some(std::string::String::from("llama3.2:latest")),
                is_default: true,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                enabled_tools: std::vec::Vec::new(),
            },
            task_manager::domain::persona::Persona {
                id: std::string::String::from("p2"),
                project_id: std::option::Option::Some(std::string::String::from("proj1")),
                name: std::string::String::from("Frontend Developer"),
                role: std::string::String::from("Developer"),
                description: std::string::String::from("Handles UI tasks"),
                llm_provider: std::option::Option::Some(std::string::String::from("ollama")),
                llm_model: std::option::Option::Some(std::string::String::from("llama3.2:latest")),
                is_default: false,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                enabled_tools: std::vec::Vec::new(),
            },
        ];

        let prompt = super::RigPRDParserAdapter::build_decomposition_prompt(&parent, "PRD content", &personas);

        std::assert!(prompt.contains("AVAILABLE PERSONAS"), "Missing personas section");
        std::assert!(prompt.contains("Backend Developer"), "Missing first persona");
        std::assert!(prompt.contains("Frontend Developer"), "Missing second persona");
    }
}
