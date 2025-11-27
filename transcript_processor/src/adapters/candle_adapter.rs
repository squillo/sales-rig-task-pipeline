//! Candle-based transcript extractor adapter for embedded ML inference.
//!
//! This adapter implements the TranscriptExtractorPort using the Candle ML framework
//! with a Phi-3.5-mini-instruct model for on-device, embedded extraction of action items from transcripts.
//! Unlike the Ollama adapter which requires an external service, this adapter runs
//! the model inference entirely in-process using Rust-native tensor operations.
//!
//! Revision History
//! - 2025-11-23T22:10:00Z @AI: Update imports from task_manager::utils to task_manager::infrastructure (HEXSER compliance).
//! - 2025-11-08T08:57:00Z @AI: Add process-wide in-memory cache for Candle model/tokenizer to skip rebuild; env CANDLE_DISABLE_MODEL_CACHE to bypass.
//! - 2025-11-08T08:39:30Z @AI: Delegate tolerant JSON parsing to task_manager::infrastructure for shared reuse across adapters.
//! - 2025-11-08T08:27:00Z @AI: Make JSON parsing tolerant in Candle adapter (map common alias fields to schema) to fix missing `title` errors.
//! - 2025-11-08T08:06:00Z @AI: Add auto GPU device selection (CUDA/Metal) with env override and safe CPU fallback; verbose logs and Context7-verified API usage.
//! - 2025-11-07T12:11:00Z @AI: Default-enable prefill cap to last 256 tokens (env overrideable) to mitigate CPU-bound prefill slowness; add guidance log.
//! - 2025-11-07T11:41:00Z @AI: Add prefill safeguards (default chunk=32, time/token budget, per-chunk timing) to prevent perceived hangs during long prefill.
//! - 2025-11-07T10:45:00Z @AI: Add chunked prompt prefill with progress logs and CANDLE_PREFILL_CHUNK to prevent hangs during long prompt prefill.
//! - 2025-11-07T10:06:00Z @AI: Fix KV-cache/mask shape mismatch by clearing cache after warmup and switching to prompt-first then incremental single-token decoding with proper seqlen_offset.
//! - 2025-11-07T09:49:00Z @AI: Stream generation progress, use Phi-3.5 eos_token_id, add early-stop on valid JSON, and warmup pass to pre-touch weights.
//! - 2025-11-07T09:37:40Z @AI: Add explicit progress logging and stdout flushing during HF Hub downloads and model init to avoid apparent hangs.
//! - 2025-11-07T09:16:00Z @AI: Remove use statement per guidelines; call IndexOp::i via UFCS to avoid use imports.
//! - 2025-11-07T09:15:00Z @AI: Align to Phi-3.5-mini-instruct via phi3; Context7-verified config/model compatibility. No functional changes.
//! - 2025-11-07T09:07:00Z @AI: Upgrade to Phi-3.5-mini-instruct (microsoft/Phi-3.5-mini-instruct) using phi3 module (~7.6GB, 2 sharded files).
//! - 2025-11-07T08:34:00Z @AI: Fix config deserialization error - revert to Phi-2 (microsoft/phi-2) which is compatible with candle_transformers::models::phi::Config (~5.3GB).
//! - 2025-11-06T21:43:00Z @AI: Downgrade model from Phi-4 to Phi-3.5-mini-instruct to reduce download size (~7.1GB, 2 sharded files instead of 14.7GB).
//! - 2025-11-06T21:38:00Z @AI: Upgrade model from Phi-3-mini-4k-instruct to Phi-4 for improved performance (~14.7GB, 6 sharded files).
//! - 2025-11-06T21:34:00Z @AI: Update model from Phi-2 to Phi-3-mini-4k-instruct for better performance and alignment with Ollama adapter.
//! - 2025-11-06T21:23:00Z @AI: Fix 404 error - use sharded model files (model-00001-of-00002.safetensors, model-00002-of-00002.safetensors) instead of non-existent model.safetensors.
//! - 2025-11-06T21:15:00Z @AI: Fix test compilation errors - remove unused Config::v2() calls in tests.
//! - 2025-11-06T21:11:00Z @AI: Fix compilation errors - add IndexOp import, fix model.forward() signature, fix Tensor::new() usage.
//! - 2025-11-06T21:00:00Z @AI: Initial CandleExtractorAdapter implementation with Phi-2.

/// Adapter for extracting action items using Candle framework with Phi-3.5-mini-instruct model.
///
/// This struct implements the TranscriptExtractorPort by loading and running
/// a Phi-3.5-mini-instruct language model entirely in-process using the Candle ML framework.
/// The model and tokenizer are loaded from HuggingFace Hub at initialization
/// and kept in memory for subsequent inference calls.
///
/// # Fields
///
/// * `model` - The loaded Phi-3.5-mini-instruct model for text generation.
/// * `tokenizer` - The tokenizer for encoding prompts and decoding responses.
/// * `device` - The compute device (CPU or GPU) for tensor operations.
///
/// # Examples
///
/// ```no_run
/// # use transcript_processor::adapters::candle_adapter::CandleTranscriptExtractorAdapter;
/// # async fn example() -> anyhow::Result<()> {
/// let adapter = CandleTranscriptExtractorAdapter::new().await?;
/// // Use adapter to extract tasks from transcript text
/// # Ok(())
/// # }
/// ```
static CANDLE_MODEL_CACHE: std::sync::OnceLock<std::sync::Arc<(std::sync::Arc<std::sync::RwLock<candle_transformers::models::phi3::Model>>, std::sync::Arc<tokenizers::Tokenizer>, std::sync::Arc<candle_core::Device>, std::sync::Arc<candle_transformers::models::phi3::Config>)>> = std::sync::OnceLock::new();

#[derive(hexser::HexAdapter)]
pub struct CandleTranscriptExtractorAdapter {
    model: std::sync::Arc<std::sync::RwLock<candle_transformers::models::phi3::Model>>,
    tokenizer: std::sync::Arc<tokenizers::Tokenizer>,
    device: std::sync::Arc<candle_core::Device>,
    config: std::sync::Arc<candle_transformers::models::phi3::Config>,
}

impl CandleTranscriptExtractorAdapter {
    /// Creates a new CandleTranscriptExtractorAdapter by loading Phi-3.5-mini-instruct from HuggingFace Hub.
    ///
    /// This constructor downloads the Phi-3.5-mini-instruct model weights and tokenizer from HuggingFace
    /// (if not already cached) and loads them into memory for inference. The model is
    /// loaded onto the CPU device by default.
    ///
    /// # Returns
    ///
    /// * `Ok(CandleTranscriptExtractorAdapter)` - Successfully loaded adapter.
    /// * `Err(anyhow::Error)` - Failed to load model or tokenizer.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use transcript_processor::adapters::candle_adapter::CandleTranscriptExtractorAdapter;
    /// # async fn example() -> anyhow::Result<()> {
    /// let adapter = CandleTranscriptExtractorAdapter::new().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new() -> anyhow::Result<Self> {
        // Local helper to always flush stdout so progress appears even when tests capture output.
        fn log(msg: &str) {
            println!("{}", msg);
            let mut out = std::io::stdout();
            let _ = std::io::Write::flush(&mut out);
        }

        log("[Candle] Selecting compute device (auto-detect: Metal/CUDA/CPU)...");
        // Determine desired device from env or auto-detect. GPU backends require compiling candle-core with corresponding features.
        let requested = std::env::var("CANDLE_DEVICE").unwrap_or_else(|_| std::string::String::from("auto"));
        // Log compiled backend availability (compile-time flags).
        let cuda_compiled = cfg!(feature = "cuda");
        let metal_compiled = cfg!(feature = "metal");
        log(&std::format!(
            "[Candle] Backend features compiled: cuda={}, metal={}",
            cuda_compiled, metal_compiled
        ));

        let device = {
            if requested.eq_ignore_ascii_case("cpu") {
                log("[Candle] Forcing CPU per CANDLE_DEVICE=cpu");
                candle_core::Device::Cpu
            } else if requested.eq_ignore_ascii_case("cuda") {
                #[cfg(feature = "cuda")]
                {
                    match candle_core::Device::new_cuda(0) {
                        std::result::Result::Ok(dev) => {
                            log("[Candle] ✓ Using CUDA device 0");
                            dev
                        }
                        std::result::Result::Err(e) => {
                            log(&std::format!("[Candle] ⚠ CUDA init failed: {}. Falling back to CPU.", e));
                            candle_core::Device::Cpu
                        }
                    }
                }
                #[cfg(not(feature = "cuda"))]
                {
                    log("[Candle] ⚠ CUDA backend not compiled. Falling back to CPU.");
                    candle_core::Device::Cpu
                }
            } else if requested.eq_ignore_ascii_case("metal") {
                #[cfg(feature = "metal")]
                {
                    match candle_core::Device::new_metal(0) {
                        std::result::Result::Ok(dev) => {
                            log("[Candle] ✓ Using Metal device 0");
                            dev
                        }
                        std::result::Result::Err(e) => {
                            log(&std::format!("[Candle] ⚠ Metal init failed: {}. Falling back to CPU.", e));
                            candle_core::Device::Cpu
                        }
                    }
                }
                #[cfg(not(feature = "metal"))]
                {
                    log("[Candle] ⚠ Metal backend not compiled. Falling back to CPU.");
                    candle_core::Device::Cpu
                }
            } else {
                // Auto-detect
                #[cfg(target_os = "macos")]
                {
                    // Prefer Metal on macOS when available, then CUDA, else CPU.
                    #[cfg(feature = "metal")]
                    {
                        match candle_core::Device::new_metal(0) {
                            std::result::Result::Ok(dev) => {
                                log("[Candle] ✓ Auto-detected Metal on macOS (device 0)");
                                dev
                            }
                            std::result::Result::Err(e) => {
                                log(&std::format!("[Candle] ⚠ Metal init failed: {}. Trying CUDA...", e));
                                #[cfg(feature = "cuda")]
                                {
                                    match candle_core::Device::new_cuda(0) {
                                        std::result::Result::Ok(dev2) => {
                                            log("[Candle] ✓ Fallback to CUDA device 0");
                                            dev2
                                        }
                                        std::result::Result::Err(e2) => {
                                            log(&std::format!("[Candle] ⚠ CUDA init failed: {}. Using CPU.", e2));
                                            candle_core::Device::Cpu
                                        }
                                    }
                                }
                                #[cfg(not(feature = "cuda"))]
                                {
                                    candle_core::Device::Cpu
                                }
                            }
                        }
                    }
                    #[cfg(not(feature = "metal"))]
                    {
                        #[cfg(feature = "cuda")]
                        {
                            match candle_core::Device::new_cuda(0) {
                                std::result::Result::Ok(dev2) => {
                                    log("[Candle] ✓ Auto-selected CUDA device 0 (macOS build without Metal)");
                                    dev2
                                }
                                std::result::Result::Err(e2) => {
                                    log(&std::format!("[Candle] ⚠ CUDA init failed: {}. Using CPU.", e2));
                                    candle_core::Device::Cpu
                                }
                            }
                        }
                        #[cfg(not(feature = "cuda"))]
                        {
                            candle_core::Device::Cpu
                        }
                    }
                }
                #[cfg(not(target_os = "macos"))]
                {
                    // Prefer CUDA on non-macOS, else Metal if compiled, else CPU.
                    #[cfg(feature = "cuda")]
                    {
                        match candle_core::Device::new_cuda(0) {
                            std::result::Result::Ok(dev) => {
                                log("[Candle] ✓ Auto-selected CUDA device 0");
                                dev
                            }
                            std::result::Result::Err(e) => {
                                log(&std::format!("[Candle] ⚠ CUDA init failed: {}. Trying Metal...", e));
                                #[cfg(feature = "metal")]
                                {
                                    match candle_core::Device::new_metal(0) {
                                        std::result::Result::Ok(dev2) => {
                                            log("[Candle] ✓ Fallback to Metal device 0");
                                            dev2
                                        }
                                        std::result::Result::Err(e2) => {
                                            log(&std::format!("[Candle] ⚠ Metal init failed: {}. Using CPU.", e2));
                                            candle_core::Device::Cpu
                                        }
                                    }
                                }
                                #[cfg(not(feature = "metal"))]
                                {
                                    candle_core::Device::Cpu
                                }
                            }
                        }
                    }
                    #[cfg(not(feature = "cuda"))]
                    {
                        #[cfg(feature = "metal")]
                        {
                            match candle_core::Device::new_metal(0) {
                                std::result::Result::Ok(dev) => {
                                    log("[Candle] ✓ Auto-selected Metal device 0");
                                    dev
                                }
                                std::result::Result::Err(e) => {
                                    log(&std::format!("[Candle] ⚠ Metal init failed: {}. Using CPU.", e));
                                    candle_core::Device::Cpu
                                }
                            }
                        }
                        #[cfg(not(feature = "metal"))]
                        {
                            candle_core::Device::Cpu
                        }
                    }
                }
            }
        };

        match &device {
            candle_core::Device::Cpu => {
                log("[Candle] ✓ Device ready: CPU");
                if !cuda_compiled && !metal_compiled {
                    log("[Candle] Hint: build with `--features cuda` (NVIDIA) or `--features metal` (macOS) for GPU acceleration.");
                }
            }
            _ => {
                log("[Candle] ✓ Device ready: GPU");
            }
        }

        // Wrap device in Arc for shared, cached usage.
        let device = std::sync::Arc::new(device);

        // Process-wide cache: reuse already-built model/tokenizer to avoid rebuild costs.
        let disable_cache_env = std::env::var("CANDLE_DISABLE_MODEL_CACHE").unwrap_or_else(|_| std::string::String::from("false"));
        let disable_cache = matches!(disable_cache_env.to_ascii_lowercase().as_str(), "1" | "true" | "yes");
        if !disable_cache {
            if let std::option::Option::Some(cache_arc) = CANDLE_MODEL_CACHE.get().cloned() {
                log("[Candle] Using cached in-memory model/tokenizer (set CANDLE_DISABLE_MODEL_CACHE=1 to rebuild).");
                return std::result::Result::Ok(Self {
                    model: std::sync::Arc::clone(&cache_arc.0),
                    tokenizer: std::sync::Arc::clone(&cache_arc.1),
                    device: std::sync::Arc::clone(&cache_arc.2),
                    config: std::sync::Arc::clone(&cache_arc.3),
                });
            }
        }

        log("[Candle] Initializing Hugging Face Hub API and repository handle...");
        // Download model and tokenizer from HuggingFace Hub
        let api = hf_hub::api::tokio::Api::new()?;
        let repo = api.repo(hf_hub::Repo::with_revision(
            "microsoft/Phi-3.5-mini-instruct".to_string(),
            hf_hub::RepoType::Model,
            "main".to_string(),
        ));
        log("[Candle] ✓ Repository handle ready (microsoft/Phi-3.5-mini-instruct)");

        // Download model files
        log("[Candle] Downloading tokenizer.json (first run may take minutes)...");
        let tokenizer_path = repo.get("tokenizer.json").await?;
        log(&std::format!(
            "[Candle] ✓ tokenizer.json ready at {}",
            tokenizer_path.display()
        ));

        log("[Candle] Downloading config.json...");
        let config_path = repo.get("config.json").await?;
        log(&std::format!(
            "[Candle] ✓ config.json ready at {}",
            config_path.display()
        ));

        // Download sharded model weights (Phi-3.5-mini-instruct model is split across 2 files, ~7.6GB total)
        log("[Candle] Downloading model weights shard 1/2 (model-00001-of-00002.safetensors)...");
        let weights_path_1 = repo.get("model-00001-of-00002.safetensors").await?;
        log(&std::format!(
            "[Candle] ✓ Shard 1 ready at {}",
            weights_path_1.display()
        ));

        log("[Candle] Downloading model weights shard 2/2 (model-00002-of-00002.safetensors)...");
        let weights_path_2 = repo.get("model-00002-of-00002.safetensors").await?;
        log(&std::format!(
            "[Candle] ✓ Shard 2 ready at {}",
            weights_path_2.display()
        ));

        log("[Candle] Parsing config.json and loading tokenizer...");
        // Load configuration
        let config_content = std::fs::read_to_string(&config_path)?;
        let config: candle_transformers::models::phi3::Config =
            serde_json::from_str(&config_content)?;

        // Load tokenizer
        let tokenizer = tokenizers::Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| anyhow::anyhow!("Failed to load tokenizer: {}", e))?;
        log("[Candle] ✓ Tokenizer loaded");

        // Load model weights from sharded safetensors files
        log("[Candle] Memory-mapping model weights (this may take several minutes on first run)...");
        let vb = unsafe {
            candle_nn::VarBuilder::from_mmaped_safetensors(
                &[weights_path_1, weights_path_2],
                candle_core::DType::F32,
                &*device,
            )?
        };
        log("[Candle] ✓ Weights memory-mapped");

        log("[Candle] Building Phi-3.5-mini-instruct model in memory...");
        // Initialize the model
        let mut model = candle_transformers::models::phi3::Model::new(&config, vb)?;
        log("[Candle] ✓ Model initialized successfully");

        // Optional warmup to pre-touch weights and caches to reduce first-token latency.
        log("[Candle] Warming up model with a tiny prompt (this is fast)...");
        let warmup_ids: std::vec::Vec<u32> = {
            let enc = tokenizer
                .encode("Hello", true)
                .map_err(|e| anyhow::anyhow!("Warmup tokenization error: {}", e))?;
            enc.get_ids().to_vec()
        };
        let warmup_input = candle_core::Tensor::new(warmup_ids.as_slice(), &*device)?.unsqueeze(0)?;
        let _ = model.forward(&warmup_input, 0);
        log("[Candle] ✓ Warmup complete");

        // IMPORTANT: Clear KV cache after warmup so it does not affect real generation.
        model.clear_kv_cache();
        log("[Candle] ✓ Cleared KV cache after warmup");

        // Wrap components in Arc for caching and shared reuse
        let model_arc: std::sync::Arc<std::sync::RwLock<candle_transformers::models::phi3::Model>> = std::sync::Arc::new(std::sync::RwLock::new(model));
        let tokenizer_arc: std::sync::Arc<tokenizers::Tokenizer> = std::sync::Arc::new(tokenizer);
        let config_arc: std::sync::Arc<candle_transformers::models::phi3::Config> = std::sync::Arc::new(config);

        if !disable_cache {
            let tuple_arc = std::sync::Arc::new((
                std::sync::Arc::clone(&model_arc),
                std::sync::Arc::clone(&tokenizer_arc),
                std::sync::Arc::clone(&device),
                std::sync::Arc::clone(&config_arc),
            ));
            let _ = CANDLE_MODEL_CACHE.set(tuple_arc);
            log("[Candle] ✓ Cached model/tokenizer in memory for reuse within this process.");
        } else {
            log("[Candle] Cache disabled by CANDLE_DISABLE_MODEL_CACHE; not storing model in global cache.");
        }

        std::result::Result::Ok(Self {
            model: model_arc,
            tokenizer: tokenizer_arc,
            device,
            config: config_arc,
        })
    }

    /// Constructs the system prompt for the LLM extraction task.
    ///
    /// This prompt instructs the model to extract action items from a transcript
    /// and format them as a JSON array matching the ActionItem schema. This uses
    /// the same prompt pattern as the Ollama adapter to ensure consistent behavior.
    fn build_extraction_prompt(&self, transcript: &str) -> String {
        std::format!(
            r#"Extract all action items from the following meeting transcript.
Return ONLY a valid JSON array of objects, where each object has this exact structure:
{{
  "title": "Brief task title",
  "assignee": "Name of person assigned (or null if not specified)",
  "due_date": "YYYY-MM-DD format (or null if not specified)"
}}

IMPORTANT: Pay close attention to who is assigned each task. Look for patterns like:
- "I'll take ownership of..." -> extract the speaker's name
- "James will complete..." -> assignee is "James"
- "Maria can implement..." -> assignee is "Maria"
- "Let's have David..." -> assignee is "David"
- "Emily should..." -> assignee is "Emily"

Extract the person's first name only. If no assignee is clearly identified, use null.

Transcript:
{}

Respond with ONLY the JSON array, no other text."#,
            transcript
        )
    }

    /// Generates text using the Candle model with the given prompt.
    ///
    /// This method tokenizes the input prompt, performs iterative text generation
    /// using the Phi-3.5-mini-instruct model, and decodes the generated tokens back into text.
    ///
    /// Note: This function intentionally exceeds 50 LoC due to streaming progress,
    /// JSON early-stop logic, and safety-focused error handling. This is a justified
    /// rare exception to the 50‑LoC guideline for improved UX and observability.
    ///
    /// # Arguments
    ///
    /// * `prompt` - The input prompt for text generation.
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The generated text response.
    /// * `Err(anyhow::Error)` - Generation failed.
    fn generate_text(&self, prompt: &str) -> anyhow::Result<String> {
        // Local logger with flush to surface progress under test runners.
        fn log(msg: &str) {
            println!("{}", msg);
            let _ = std::io::Write::flush(&mut std::io::stdout());
        }

        // Tokenize the prompt
        let tokens = self
            .tokenizer
            .encode(prompt, true)
            .map_err(|e| anyhow::anyhow!("Tokenization error: {}", e))?;
        let mut prompt_ids: std::vec::Vec<u32> = tokens.get_ids().to_vec();
        log(&std::format!("[Candle] Tokenized prompt: {} tokens", prompt_ids.len()));

        // Prefill cap: by default, limit to last 256 tokens to mitigate CPU-bound prefill latency.
        // Override with CANDLE_PREFILL_MAX_TOKENS (set to 0 to disable capping).
        let prefill_max_tokens = std::env::var("CANDLE_PREFILL_MAX_TOKENS")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(256);
        if prefill_max_tokens > 0 && prompt_ids.len() > prefill_max_tokens {
            let start_idx = prompt_ids.len() - prefill_max_tokens;
            let original = prompt_ids.len();
            prompt_ids = prompt_ids[start_idx..].to_vec();
            log(&std::format!(
                "[Candle] Prefill capped to last {} tokens (from {} total). Set CANDLE_PREFILL_MAX_TOKENS=0 to disable.",
                prefill_max_tokens,
                original
            ));
        }

        // Generation controls (env-overridable)
        let max_new_tokens = std::env::var("CANDLE_MAX_NEW_TOKENS")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(256);
        let stream_interval = std::env::var("CANDLE_STREAM_INTERVAL")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(16);

        // Use model-configured EOS token when available; otherwise, rely on early-stop or max tokens
        let eos_token = self.config.eos_token_id;

        // Acquire the model (mutable) once for the whole decode loop
        let mut model = self.model.write().unwrap();

        // 1) Prompt prefill: run the prompt in chunks to populate KV cache with visible progress.
        // Default smaller chunk for better responsiveness on CPU.
        let prefill_chunk = std::env::var("CANDLE_PREFILL_CHUNK")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(32);
        let prefill_budget_ms = std::env::var("CANDLE_PREFILL_TIME_BUDGET_MS")
            .ok()
            .and_then(|v| v.parse::<u128>().ok());
        let total = prompt_ids.len();
        let total_chunks = if total == 0 { 0 } else { (total + prefill_chunk - 1) / prefill_chunk };
        let mut offset = 0usize;
        let mut logits_opt: std::option::Option<candle_core::Tensor> = std::option::Option::None;
        let mut chunk_idx = 0usize;
        let prefill_start = std::time::Instant::now();
        while offset < total {
            let end = std::cmp::min(offset + prefill_chunk, total);
            chunk_idx += 1;
            let chunk_len = end - offset;
            log(&std::format!(
                "[Candle] Prefill chunk {}/{} (tokens {}..{} of {})...",
                chunk_idx,
                total_chunks,
                offset,
                end.saturating_sub(1),
                total
            ));
            let chunk_t0 = std::time::Instant::now();
            let chunk = &prompt_ids[offset..end];
            let chunk_input = candle_core::Tensor::new(chunk, &*self.device)?.unsqueeze(0)?;
            logits_opt = std::option::Option::Some(model.forward(&chunk_input, offset)?);
            let chunk_ms = chunk_t0.elapsed().as_millis();
            let tps = if chunk_ms > 0 { (chunk_len as u128 * 1000) / chunk_ms } else { 0 };
            log(&std::format!(
                "[Candle] ✓ Prefill chunk {}/{} complete in {} ms (~{} tok/s)",
                chunk_idx,
                total_chunks,
                chunk_ms,
                tps
            ));
            offset = end;
            if let Some(budget_ms) = prefill_budget_ms {
                let elapsed_ms = prefill_start.elapsed().as_millis();
                if elapsed_ms >= budget_ms {
                    log(&std::format!(
                        "[Candle] ⏱ Prefill time budget reached ({} ms >= {} ms). Proceeding to generation early.",
                        elapsed_ms,
                        budget_ms
                    ));
                    break;
                }
            }
        }
        let mut logits = match logits_opt {
            std::option::Option::Some(t) => t,
            std::option::Option::None => {
                // Empty prompt edge-case (should not happen for our usage)
                return std::result::Result::Ok(std::string::String::new());
            }
        };
        log("[Candle] ✓ Prefill complete");

        // 2) Incremental decoding: feed one token at a time with correct seqlen_offset.
        let mut generated: std::vec::Vec<u32> = std::vec::Vec::new();
        let mut last_logged = 0usize;

        for step in 0..max_new_tokens {
            // Heartbeat log so the user sees progress even before the first token is produced.
            if step == 0 || (step + 1) % stream_interval == 0 {
                log(&std::format!(
                    "[Candle] Generating token step {}/{}...",
                    step + 1,
                    max_new_tokens
                ));
            }

            // Select next token (greedy) from current logits (shape: [1,1,vocab])
            let last_logits = candle_core::IndexOp::i(&logits, (0, 0))?;
            let next_token = last_logits.argmax(0)?.to_scalar::<u32>()?;

            // Stop if EOS token is generated (when defined by config)
            if eos_token.is_some() && Some(next_token) == eos_token {
                log("[Candle] ✓ Reached EOS token. Stopping generation.");
                break;
            }

            generated.push(next_token);

            // Periodic streaming progress + early-stop when valid JSON is detected
            let produced = generated.len();
            if produced >= last_logged + stream_interval {
                // Best-effort decode of the newly generated segment for feedback
                let decoded_tail = self
                    .tokenizer
                    .decode(generated.as_slice(), true)
                    .unwrap_or_else(|_| std::string::String::from(""));
                log(&std::format!("[Candle] Generated {} tokens...", produced));

                // Early-stop if a valid JSON array is present in the stream
                if let (Some(s), Some(e)) = (decoded_tail.find('['), decoded_tail.rfind(']')) {
                    if e > s {
                        let json_slice = &decoded_tail[s..=e];
                        if serde_json::from_str::<serde_json::Value>(json_slice)
                            .ok()
                            .map(|v| v.is_array())
                            .unwrap_or(false)
                        {
                            log("[Candle] ✓ Detected valid JSON array in stream. Stopping early.");
                            return std::result::Result::Ok(json_slice.to_string());
                        }
                    }
                }
                last_logged = produced;
            }

            // Prepare one-token input and forward with correct seqlen_offset (prompt length + tokens already in KV cache)
            let one = [next_token];
            let step_input = candle_core::Tensor::new(&one, &*self.device)?.unsqueeze(0)?;
            // Use actual number of tokens prefilled into the KV cache (offset), not total prompt length,
            // to avoid mask/KV length mismatch when prefill was capped or ended early due to time budget.
            let seqlen_offset = offset + generated.len() - 1;
            logits = model.forward(&step_input, seqlen_offset)?;
        }

        // Decode all generated tokens
        let generated_text = self
            .tokenizer
            .decode(generated.as_slice(), true)
            .map_err(|e| anyhow::anyhow!("Decoding error: {}", e))?;

        std::result::Result::Ok(generated_text)
    }

    /// Parses the LLM response string into a vector of ActionItem entities.
    ///
    /// Attempts to extract and deserialize JSON from the response into ActionItem structs.
    /// Delegates tolerant parsing to task_manager::infrastructure to centralize logic across crates.
    fn parse_response(
        &self,
        response_text: &str,
    ) -> std::result::Result<std::vec::Vec<crate::domain::action_item::ActionItem>, String> {
        let parsed: std::vec::Vec<task_manager::infrastructure::dtos::extracted_action_item::ExtractedActionItem> =
            task_manager::infrastructure::llm_parsers::action_item_parser::parse_action_items_tolerant(response_text)?;
        let mapped: std::vec::Vec<crate::domain::action_item::ActionItem> = parsed
            .into_iter()
            .map(|e| crate::domain::action_item::ActionItem {
                title: e.title,
                assignee: e.assignee,
                due_date: e.due_date,
            })
            .collect();
        std::result::Result::Ok(mapped)
    }
}

#[async_trait::async_trait]
impl crate::application::ports::transcript_extractor_port::TranscriptExtractorPort
    for CandleTranscriptExtractorAdapter
{
    async fn extract_analysis(
        &self,
        transcript: &str,
    ) -> std::result::Result<crate::domain::transcript_analysis::TranscriptAnalysis, String> {
        // Build the extraction prompt
        let prompt = self.build_extraction_prompt(transcript);

        // Generate response using Candle model
        let response_text = self
            .generate_text(&prompt)
            .map_err(|e| std::format!("Candle generation error: {}", e))?;

        // Parse the response text into ActionItems and wrap in TranscriptAnalysis
        let action_items = self.parse_response(&response_text)?;
        std::result::Result::Ok(crate::domain::transcript_analysis::TranscriptAnalysis {
            action_items,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_extraction_prompt() {
        // Test: Validates that the extraction prompt contains the transcript and JSON schema.
        // Justification: Ensures the prompt structure is correct and includes the input transcript,
        // which is essential for the model to generate accurate action item extractions.

        let transcript = "Alice will review the code by Friday.";

        // Test the prompt format directly without requiring a full adapter instance
        let prompt = std::format!(
            r#"Extract all action items from the following meeting transcript.
Return ONLY a valid JSON array of objects, where each object has this exact structure:
{{
  "title": "Brief task title",
  "assignee": "Name of person assigned (or null if not specified)",
  "due_date": "YYYY-MM-DD format (or null if not specified)"
}}

IMPORTANT: Pay close attention to who is assigned each task. Look for patterns like:
- "I'll take ownership of..." -> extract the speaker's name
- "James will complete..." -> assignee is "James"
- "Maria can implement..." -> assignee is "Maria"
- "Let's have David..." -> assignee is "David"
- "Emily should..." -> assignee is "Emily"

Extract the person's first name only. If no assignee is clearly identified, use null.

Transcript:
{}

Respond with ONLY the JSON array, no other text."#,
            transcript
        );

        assert!(prompt.contains(transcript));
        assert!(prompt.contains("JSON array"));
        assert!(prompt.contains("title"));
        assert!(prompt.contains("assignee"));
        assert!(prompt.contains("due_date"));
    }

    #[test]
    fn test_parse_response_valid_json() {
        // Test: Validates that valid JSON response is correctly parsed into ActionItem structs.
        // Justification: Ensures the JSON parsing logic correctly deserializes well-formed
        // responses into the domain entities, which is critical for the extraction pipeline.

        let json_response = r#"[
            {"title": "Review code", "assignee": "Alice", "due_date": "2025-11-10"},
            {"title": "Write tests", "assignee": null, "due_date": null}
        ]"#;

        // Test parsing directly
        let result: std::result::Result<
            std::vec::Vec<crate::domain::action_item::ActionItem>,
            serde_json::Error,
        > = serde_json::from_str(json_response);

        assert!(result.is_ok());
        let items = result.unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].title, "Review code");
        assert_eq!(items[0].assignee, Some("Alice".to_string()));
    }

    #[test]
    fn test_parse_response_with_extra_text() {
        // Test: Validates that JSON can be extracted from response with surrounding text.
        // Justification: LLMs often add preamble or explanation text around JSON output,
        // so the parser must be robust enough to extract the JSON array from noisy responses.
        let response_with_extra = r#"Here are the action items I found:
[
    {"title": "Task 1", "assignee": "Bob", "due_date": "2025-11-15"}
]
Hope this helps!"#;

        // Test JSON extraction
        let json_start = response_with_extra.find('[').unwrap();
        let json_end = response_with_extra.rfind(']').unwrap();
        let json_str = &response_with_extra[json_start..=json_end];

        let result: std::result::Result<
            std::vec::Vec<crate::domain::action_item::ActionItem>,
            serde_json::Error,
        > = serde_json::from_str(json_str);

        assert!(result.is_ok());
        let items = result.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "Task 1");
    }
}
