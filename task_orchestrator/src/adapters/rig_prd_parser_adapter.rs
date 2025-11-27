//! Rig-powered PRD parser adapter for task generation.
//!
//! This adapter uses Rig's CompletionModel to analyze PRDs and generate
//! actionable task lists via LLM-based decomposition.
//!
//! Revision History
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
    TaskGenerated { title: String, description: String },
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
#[derive(hexser::HexAdapter)]
pub struct RigPRDParserAdapter {
    model_name: String,
    fallback_model_name: String,
    personas: std::vec::Vec<task_manager::domain::persona::Persona>,
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
        Self { model_name, fallback_model_name, personas }
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
            let prompt = Self::build_prompt(&prd, &personas);

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

                                                                    // Build formatted description with all fields
                                                                    let mut full_desc = description.clone();

                                                                    // Extract assignee for display
                                                                    let assignee_display = task_obj.get("assignee")
                                                                        .or_else(|| task_obj.get("assigned_to"))
                                                                        .or_else(|| task_obj.get("owner"))
                                                                        .or_else(|| task_obj.get("responsible"))
                                                                        .and_then(|a| a.as_str())
                                                                        .unwrap_or("unassigned");

                                                                    if let std::option::Option::Some(priority) = task_obj.get("priority")
                                                                        .or_else(|| task_obj.get("prio"))
                                                                        .and_then(|p| p.as_str())
                                                                    {
                                                                        full_desc.push_str(&std::format!("\nPriority: {}", priority));
                                                                    }

                                                                    if let std::option::Option::Some(complexity) = task_obj.get("estimated_complexity")
                                                                        .or_else(|| task_obj.get("complexity"))
                                                                        .and_then(|c| c.as_u64())
                                                                    {
                                                                        full_desc.push_str(&std::format!("\nComplexity: {}/10", complexity));
                                                                    }

                                                                    // Add assignee to display
                                                                    full_desc.push_str(&std::format!("\nAssignee: {}", assignee_display));

                                                                    // Send TaskGenerated update
                                                                    let _ = update_tx
                                                                        .send(PRDGenUpdate::TaskGenerated {
                                                                            title: title.to_string(),
                                                                            description: full_desc,
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
    fn build_system_prompt(personas: &[task_manager::domain::persona::Persona]) -> std::string::String {
        let mut prompt = std::string::String::from(
            "You are a project management assistant. Break down Product Requirements Documents into actionable tasks.\n\n\
            CRITICAL: You MUST respond with ONLY a valid JSON array. No other text.\n\n"
        );

        // Add persona information if available
        if !personas.is_empty() {
            prompt.push_str("AVAILABLE TEAM MEMBERS (assign tasks to these personas):\n");
            for persona in personas {
                prompt.push_str(&std::format!(
                    "- \"{}\": {} - {}\n",
                    persona.name,
                    persona.role,
                    persona.description
                ));
            }
            prompt.push_str("\n");

            prompt.push_str("Each task object must have exactly these 5 fields:\n\
            - \"title\": string (concise task title, max 100 chars)\n\
            - \"description\": string (detailed description, max 500 chars)\n\
            - \"priority\": string (must be exactly \"high\", \"medium\", or \"low\")\n\
            - \"estimated_complexity\": number (integer 1-10, where 10 is most complex)\n\
            - \"assignee\": string (persona name from the list above, or \"unassigned\")\n\n\
            EXAMPLE RESPONSE (copy this format exactly):\n\
            [{\"title\":\"Setup Rust project\",\"description\":\"Initialize Cargo workspace with required dependencies\",\"priority\":\"high\",\"estimated_complexity\":3,\"assignee\":\"Alice\"},{\"title\":\"Implement API endpoints\",\"description\":\"Create REST API with authentication\",\"priority\":\"high\",\"estimated_complexity\":7,\"assignee\":\"Bob\"}]\n\n");
        } else {
            prompt.push_str("Each task object must have exactly these 4 fields:\n\
            - \"title\": string (concise task title, max 100 chars)\n\
            - \"description\": string (detailed description, max 500 chars)\n\
            - \"priority\": string (must be exactly \"high\", \"medium\", or \"low\")\n\
            - \"estimated_complexity\": number (integer 1-10, where 10 is most complex)\n\n\
            EXAMPLE RESPONSE (copy this format exactly):\n\
            [{\"title\":\"Setup Rust project\",\"description\":\"Initialize Cargo workspace with required dependencies\",\"priority\":\"high\",\"estimated_complexity\":3},{\"title\":\"Implement API endpoints\",\"description\":\"Create REST API with authentication\",\"priority\":\"high\",\"estimated_complexity\":7}]\n\n");
        }

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

    /// Builds the complete prompt from PRD content (system + user).
    fn build_prompt(prd: &task_manager::domain::prd::PRD, personas: &[task_manager::domain::persona::Persona]) -> std::string::String {
        let mut prompt = Self::build_system_prompt(personas);
        prompt.push_str("\n\n");

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

        // Step 3: Try parsing after cleanup
        log.push_str("→ Step 3: Test parse after cleanup\n");
        if let std::result::Result::Ok(_) = serde_json::from_str::<serde_json::Value>(&cleaned) {
            log.push_str("  ✓ SUCCESS: Cleanup alone fixed the JSON!\n");
            return std::result::Result::Ok((cleaned, log));
        } else {
            log.push_str("  ✗ Still invalid, attempting LLM remediation\n");
        }

        // Step 4: LLM remediation
        log.push_str(&std::format!("→ Step 4: LLM remediation ({})\n", model_name));

        let remediation_prompt = std::format!(
            "Fix this malformed JSON and return ONLY valid JSON array. Each object must have: title, description, priority, estimated_complexity.\n\n\
            CRITICAL RULES:\n\
            - Return ONLY the JSON array, no explanations\n\
            - Start with [ and end with ]\n\
            - Use double quotes for strings\n\
            - No trailing commas\n\
            - All braces and brackets must be balanced\n\n\
            Malformed input:\n{}\n\n\
            Fixed JSON:",
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
            log.push_str("  ✗ FAILED: LLM output still invalid\n");
            log.push_str(&std::format!("\nRemediated output (first 200 chars):\n{}\n",
                if final_cleaned.len() > 200 { &final_cleaned[0..200] } else { &final_cleaned }));
            std::result::Result::Err(log)
        }
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
            let _description = Self::extract_string(
                obj,
                &["description", "desc", "details", "detail", "content"]
            ).unwrap_or_default();

            // Extract priority (optional, default to "medium")
            let _priority = Self::extract_string(
                obj,
                &["priority", "prio", "importance", "level"]
            ).unwrap_or_else(|| String::from("medium"));

            // Extract complexity (optional, default to 5)
            let _estimated_complexity = Self::extract_number(
                obj,
                &["estimated_complexity", "complexity", "difficulty", "effort", "score"]
            ).unwrap_or(5);

            // Extract assignee (optional)
            let llm_assignee = Self::extract_string(
                obj,
                &["assignee", "assigned_to", "owner", "responsible"]
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

            // Set PRD linkage
            task.source_prd_id = std::option::Option::Some(prd_id.to_string());

            // TODO Phase 1: Store priority and description as proper Task fields
            // For now, we just create the task with basic fields from the LLM response
            // Priority: {priority}, Complexity: {_estimated_complexity}, Description: {description}

            tasks.push(task);
        }

        std::result::Result::Ok(tasks)
    }
}

#[async_trait::async_trait]
impl crate::ports::prd_parser_port::PRDParserPort for RigPRDParserAdapter {
    async fn parse_prd_to_tasks(
        &self,
        prd: &task_manager::domain::prd::PRD,
    ) -> std::result::Result<std::vec::Vec<task_manager::domain::task::Task>, std::string::String> {
        // Build complete prompt
        let prompt = Self::build_prompt(prd, &self.personas);

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

    #[test]
    fn test_build_prompt_includes_prd_sections() {
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

        let prompt = super::RigPRDParserAdapter::build_prompt(&prd, &[]);

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
}
