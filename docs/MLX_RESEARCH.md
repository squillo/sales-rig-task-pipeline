# MLX-LM Integration Research for Rigger

**Task**: Phase 5 Sprint 11 Task 5.7 - Research MLX-LM integration requirements

**Date**: 2025-11-23

**Status**: Research Complete ✅

---

## Executive Summary

**MLX** is Apple's machine learning framework optimized for Apple Silicon (M-series chips) with **30-50% faster inference** than Ollama for local LLM workloads. Integration via **mlx-rs** (Rust bindings) is **feasible but requires careful consideration** of trade-offs.

**Recommendation**: **Phase 1 - Subprocess Bridge** (Quick Win) → **Phase 2 - Native mlx-rs** (Long-term Goal)

---

## 1. What is MLX?

[MLX](https://github.com/ml-explore/mlx) is an array framework for machine learning on Apple Silicon, developed by Apple's machine learning research team.

### Key Advantages

1. **Unified Memory Architecture**: Direct CPU/GPU shared memory access (Apple Silicon advantage)
2. **Lazy Evaluation**: Arrays materialized only when needed (memory efficiency)
3. **Metal Acceleration**: Native GPU support via Apple's Metal framework
4. **30-50% Faster**: Benchmarks show significant speed improvements over Ollama on M-series chips ([Deep AI](https://deepai.tn/glossary/ollama/mlx-faster-than-ollama/), [Peddals Blog](https://blog.peddals.com/en/qwq32b-prev-mlx-lm-api-is-faster-than-ollama/))

### Performance Benchmarks

| Framework | Model | Chip | Tokens/Sec | Source |
|-----------|-------|------|------------|--------|
| MLX 0.15 | Llama 8B-4bit | M3 Max | ~65 t/s | [Markus Schall](https://www.markus-schall.de/en/2025/09/mlx-on-apple-silicon-as-local-ki-compared-with-ollama-co/) |
| Ollama | Llama 8B-4bit | M3 Max | ~45-50 t/s | [GitHub Discussion](https://github.com/ggml-org/llama.cpp/discussions/4167) |
| MLX | QwQ-32B | M4 | 30-50% faster | [Peddals Blog](https://blog.peddals.com/en/qwq32b-prev-mlx-lm-api-is-faster-than-ollama/) |

**Key Insight**: Performance gains are most significant on newer chips (M3/M4) and larger models.

---

## 2. mlx-rs: Rust Bindings

[mlx-rs](https://github.com/oxideai/mlx-rs) is an **unofficial** Rust binding for MLX, developed by [oxideai](https://github.com/oxideai).

### Current Status (November 2024)

- **Version**: 0.25.1 (July 2025 release)
- **Stars**: 208 on GitHub
- **Maturity**: **Active development** - usable but evolving
- **MSRV**: Rust 1.82.0+
- **Platform**: **macOS Apple Silicon only** (M1/M2/M3/M4)
- **Endorsement**: [Awni Hannun](https://x.com/awnihannun/status/1886846423905575330) (MLX team) acknowledged the project

### Crate Structure

```rust
mlx-rs        // Main bindings crate
mlx-sys       // Low-level C bindings (follows mlx-c versioning)
mlx-lm        // Language model utilities
mlx-macros    // Procedural macros
```

### Feature Flags

- `metal`: GPU acceleration via Apple Metal
- `accelerate`: Apple's Accelerate framework for optimized math

### API Examples

**Text Generation** ([Mistral Example](https://github.com/oxideai/mlx-rs/tree/main/examples/mistral)):
```rust
use mlx_rs::prelude::*;

// Load pre-trained Mistral model
let model = MistralModel::from_pretrained("mistralai/Mistral-7B-v0.1")?;

// Generate text
let prompt = "Explain quantum computing in simple terms:";
let output = model.generate(prompt, max_tokens=256)?;
println!("{}", output);
```

**Key Limitation** ⚠️: Closures capturing external arrays can **cause segfaults**. Workaround: Explicitly pass arrays as function parameters.

---

## 3. Integration Options for Rigger

### Option A: Python Subprocess Bridge (Recommended Phase 1)

**Approach**: Use Python `mlx-lm` package via subprocess, similar to how we call Ollama.

#### Pros ✅
- **Quick to implement**: ~2-3 days
- **Battle-tested**: MLX-LM Python package is mature and stable
- **Full feature set**: Access to all MLX-LM capabilities (fine-tuning, LoRA, etc.)
- **Platform detection**: Easy fallback to Ollama on non-macOS systems
- **No Rust complexity**: Avoid unsafe code and segfault risks

#### Cons ❌
- **Python dependency**: Requires `pip install mlx-lm`
- **IPC overhead**: Subprocess communication (though negligible for LLM latency)
- **Less idiomatic**: Not "pure Rust"

#### Implementation

```rust
// task_orchestrator/src/adapters/mlx_subprocess_adapter.rs
pub struct MlxSubprocessAdapter {
    model_name: String,
    python_path: String, // e.g., "/usr/local/bin/python3"
}

impl MlxSubprocessAdapter {
    pub async fn generate(&self, prompt: &str) -> Result<String, String> {
        let script = format!(
            r#"
import mlx_lm
model, tokenizer = mlx_lm.load("{}")
response = mlx_lm.generate(model, tokenizer, prompt="{}", max_tokens=256)
print(response)
            "#,
            self.model_name, prompt
        );

        let output = tokio::process::Command::new(&self.python_path)
            .arg("-c")
            .arg(&script)
            .output()
            .await?;

        Ok(String::from_utf8(output.stdout)?)
    }
}
```

**Environment Variable**: `INFERENCE_BACKEND=mlx`

---

### Option B: Native mlx-rs Integration (Phase 2 Goal)

**Approach**: Use `mlx-rs` crate directly for native Rust integration.

#### Pros ✅
- **Pure Rust**: No external dependencies
- **Maximum performance**: No subprocess overhead
- **Type safety**: Compile-time guarantees
- **Future-proof**: As mlx-rs matures, we benefit automatically

#### Cons ❌
- **Immature library**: Still in active development (0.25.1)
- **Segfault risks**: Known closure capture issues ([docs](https://oxideai.github.io/mlx-rs/mlx_rs/))
- **Complexity**: Requires deep understanding of mlx-rs API
- **Longer development**: ~1-2 weeks to implement safely
- **macOS-only**: No cross-platform support (must maintain Ollama fallback)

#### Implementation Sketch

```rust
// task_orchestrator/src/adapters/mlx_native_adapter.rs
use mlx_rs::prelude::*;

pub struct MlxNativeAdapter {
    model: Arc<MistralModel>,
}

impl MlxNativeAdapter {
    pub fn new(model_name: &str) -> Result<Self, String> {
        // Load model from HuggingFace or local path
        let model = MistralModel::from_pretrained(model_name)
            .map_err(|e| format!("Failed to load model: {}", e))?;

        Ok(Self {
            model: Arc::new(model),
        })
    }
}

#[async_trait::async_trait]
impl TaskEnhancementPort for MlxNativeAdapter {
    async fn generate_enhancement(&self, task: &Task) -> Result<Enhancement, String> {
        let prompt = format!("Enhance this task: {}", task.title);

        // IMPORTANT: Avoid closure capture to prevent segfaults
        let output = tokio::task::spawn_blocking({
            let model = self.model.clone();
            let prompt = prompt.clone();
            move || {
                model.generate(&prompt, GenerateOptions {
                    max_tokens: 256,
                    temperature: 0.7,
                    ..Default::default()
                })
            }
        })
        .await
        .map_err(|e| format!("Generation failed: {}", e))??;

        // Parse and return enhancement
        parse_enhancement(&output)
    }
}
```

#### Dependencies

```toml
[dependencies.mlx-rs]
version = "0.25"
features = ["metal", "accelerate"]
optional = true

[features]
mlx_backend = ["mlx-rs"]
```

---

## 4. Platform Detection Strategy

Rigger should **automatically select the best backend** based on platform:

```rust
// task_orchestrator/src/adapters/backend_detector.rs
pub enum InferenceBackend {
    Ollama,
    Mlx,
    Candle,
}

impl InferenceBackend {
    pub fn detect() -> Self {
        // Priority order:
        // 1. User override (INFERENCE_BACKEND env var)
        // 2. macOS M-series → MLX (if installed)
        // 3. Default → Ollama

        if let Ok(backend) = std::env::var("INFERENCE_BACKEND") {
            return match backend.as_str() {
                "mlx" => InferenceBackend::Mlx,
                "candle" => InferenceBackend::Candle,
                _ => InferenceBackend::Ollama,
            };
        }

        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        {
            // Check if MLX is available
            if MlxSubprocessAdapter::is_available() {
                return InferenceBackend::Mlx;
            }
        }

        InferenceBackend::Ollama
    }
}
```

---

## 5. Comparison Matrix

| Criterion | Ollama | MLX (Python) | MLX (Rust) |
|-----------|--------|--------------|------------|
| **Speed (M3)** | Baseline | +30-50% | +30-50% |
| **Cross-platform** | ✅ Yes | ❌ macOS only | ❌ macOS only |
| **Rust integration** | ✅ HTTP API | ⚠️ Subprocess | ✅ Native |
| **Maturity** | ✅ Stable | ✅ Stable | ⚠️ Beta |
| **Setup complexity** | ⚠️ Medium | ⚠️ Medium | ❌ High |
| **Development time** | N/A (done) | 2-3 days | 1-2 weeks |
| **Maintenance burden** | Low | Low | Medium |
| **Model compatibility** | ✅ Broad | ✅ Broad | ⚠️ Limited |

---

## 6. Recommended Implementation Plan

### Phase 1: MLX Subprocess Bridge (Week 1)

**Goal**: Get 30-50% speed improvement on macOS with minimal risk.

- [ ] **5.7**: Research complete ✅ (this document)
- [ ] **5.8**: Implement `MlxSubprocessAdapter`
  - Create adapter implementing `TaskEnhancementPort`
  - Create adapter implementing `TaskDecompositionPort`
  - Add platform detection (`is_mlx_available()`)
  - Environment variable: `INFERENCE_BACKEND=mlx`
- [ ] **5.9**: Documentation
  - `docs/MLX_SETUP.md`: Installation guide
  - Add to main README.md
  - Performance comparison table

**Success Criteria**:
- ✅ Phi-3, Orca-2, Mistral models run on MLX
- ✅ Automatic fallback to Ollama on non-macOS
- ✅ Benchmarks confirm speed improvement

### Phase 2: Native mlx-rs (Future - When Stable)

**Trigger**: mlx-rs reaches 1.0.0 or demonstrates production stability.

**Benefits**:
- Pure Rust (no Python dependency)
- Maximum performance
- Type safety

**Prerequisites**:
- mlx-rs closure capture segfault issue resolved
- Comprehensive test suite
- Community adoption validation

---

## 7. Alternative: mistral.rs

**Note**: [mistral.rs](https://github.com/EricLBuehler/mistral.rs) is a **separate project** - a high-performance LLM inference engine in Rust.

While confusingly named, it's **NOT** the same as mlx-rs and does NOT use MLX. However, it offers:
- Blazingly fast inference (quantization, paged attention)
- Cross-platform (not macOS-only)
- Native Rust

**Consideration**: If cross-platform speed is more important than macOS-specific optimization, `mistral.rs` might be a better choice than MLX.

---

## 8. Decision Tree

```
Is the user on macOS Apple Silicon?
├─ NO  → Use Ollama (cross-platform)
└─ YES → Is MLX-LM Python package installed?
         ├─ NO  → Use Ollama (graceful fallback)
         └─ YES → Use MLX (30-50% faster!)
```

---

## 9. Open Questions

1. **Model Format Compatibility**: Can MLX load GGUF models, or only HuggingFace formats?
   - **Answer**: MLX uses its own format (mlx-community models), not GGUF.
   - **Impact**: Separate model downloads required.

2. **Fine-tuning Support**: Do we need LoRA/fine-tuning for heterogeneous pipeline?
   - **Current**: Not required for Phase 5.
   - **Future**: MLX-LM excels at fine-tuning - valuable for Task 5.14+.

3. **Memory Usage**: How does MLX compare to Ollama in RAM consumption?
   - **Research needed**: Run benchmarks with `process-memory-stats`.

---

## 10. References

### MLX Framework
- [MLX GitHub](https://github.com/ml-explore/mlx)
- [MLX Documentation](https://ml-explore.github.io/mlx/)
- [MLX-LM Package](https://github.com/ml-explore/mlx-lm)

### mlx-rs Rust Bindings
- [mlx-rs GitHub](https://github.com/oxideai/mlx-rs)
- [mlx-rs crates.io](https://crates.io/crates/mlx-rs)
- [mlx-rs Documentation](https://oxideai.github.io/mlx-rs/mlx_rs/)

### Benchmarks & Comparisons
- [Deep AI: MLX vs Ollama Benchmark](https://deepai.tn/glossary/ollama/mlx-faster-than-ollama/)
- [Markus Schall: MLX on Apple Silicon Guide](https://www.markus-schall.de/en/2025/09/mlx-on-apple-silicon-as-local-ki-compared-with-ollama-co/)
- [Peddals Blog: MLX-LM API Performance](https://blog.peddals.com/en/qwq32b-prev-mlx-lm-api-is-faster-than-ollama/)
- [Rick's Blog: Benchmark LLM on Apple Silicon](https://ricklan.net/blog/2025/03/benchmark-llm-on-apple-silicon/)
- [GitHub: LLM Silicon Benchmarks Repository](https://github.com/ivanfioravanti/benchmarks_llm_silicon)

### Medium Articles
- [Apple MLX vs Llama.cpp vs Candle Comparison](https://medium.com/@zaiinn440/apple-mlx-vs-llama-cpp-vs-hugging-face-candle-rust-for-lightning-fast-llms-locally-5447f6e9255a)
- [Introduction to MLX Framework](https://medium.com/lolml/introduction-to-mlx-apples-machine-learning-framework-527b81f23fa5)

### Apple Resources
- [Apple MLX Research: M5 GPU Exploration](https://machinelearning.apple.com/research/exploring-llms-mlx-m5)
- [Apple Open Source MLX Project](https://opensource.apple.com/projects/mlx/)

---

## 11. Next Steps

**Immediate (This Sprint)**:
1. ✅ Complete research (this document)
2. ⏭️ Decide: Subprocess bridge vs Native mlx-rs
3. ⏭️ Implement chosen approach (Task 5.8)
4. ⏭️ Write setup documentation (Task 5.9)

**Future (Post-Sprint 11)**:
- Benchmark MLX vs Ollama with real Rigger workloads
- Evaluate mlx-rs stability for native integration
- Consider mistral.rs as cross-platform alternative

---

**Research Complete**: 2025-11-23T23:45:00Z

**Recommendation**: ✅ **Proceed with MLX Subprocess Bridge (Option A)** for immediate 30-50% speed gains on macOS with minimal risk.
