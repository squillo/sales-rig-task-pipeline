# MLX Backend Setup Guide

This guide covers how to set up and use the MLX inference backend for 30-50% faster LLM inference on macOS Apple Silicon.

## Overview

MLX is Apple's machine learning framework optimized for Apple Silicon chips (M1/M2/M3/M4). It leverages the unified memory architecture and Metal GPU acceleration to provide significantly faster inference than traditional backends like Ollama.

**Performance Gains**: 30-50% faster inference on M3/M4 chips compared to Ollama with the same models.

## Prerequisites

### Hardware Requirements

- **macOS Apple Silicon** (M1, M2, M3, or M4 chip)
- Not supported on Intel Macs or other operating systems

### Software Requirements

1. **Python 3.9+**
   ```bash
   python3 --version
   ```

2. **pip** (Python package manager)
   ```bash
   pip3 --version
   ```

## Installation

### Step 1: Install mlx-lm Python Package

```bash
pip3 install mlx-lm
```

This installs the MLX language model library with GPU acceleration support.

### Step 2: Verify Installation

```bash
python3 -c "import mlx_lm; print('MLX-LM installed successfully')"
```

Expected output: `MLX-LM installed successfully`

### Step 3: Download MLX-Optimized Models

MLX uses models from the `mlx-community` namespace on HuggingFace. The first time you use a model, it will be downloaded automatically.

**Recommended models**:
- `mlx-community/Phi-3-mini-4k-instruct` (default, 3.8B parameters)
- `mlx-community/orca-2-7b` (7B parameters, excellent reasoning)
- `mlx-community/Mistral-7B-Instruct-v0.2` (7B parameters)

To pre-download a model:
```bash
python3 -c "import mlx_lm; mlx_lm.load('mlx-community/Phi-3-mini-4k-instruct')"
```

## Configuration

### Automatic Backend Selection

By default, `ProviderFactory::from_env()` automatically detects if MLX is available on your system. If you're on macOS Apple Silicon with mlx-lm installed, it will use MLX automatically.

### Manual Backend Selection

To explicitly use MLX or override auto-detection:

```bash
export TASK_ORCHESTRATOR_PROVIDER=mlx
```

To force Ollama even on macOS:
```bash
export TASK_ORCHESTRATOR_PROVIDER=ollama
```

### Model Selection

Specify which MLX model to use:

```bash
export MLX_MODEL="mlx-community/Phi-3-mini-4k-instruct"
```

**Available models**:
- `mlx-community/Phi-3-mini-4k-instruct` (default, fast routing)
- `mlx-community/orca-2-7b` (complex reasoning)
- `mlx-community/Mistral-7B-Instruct-v0.2` (general purpose)
- `mlx-community/Meta-Llama-3.1-8B-Instruct` (large model)

### Python Path Override

If your Python installation is not in the default path:

```bash
export PYTHON_PATH=/path/to/python3
```

## Usage

### Basic Usage

```rust
use task_orchestrator::adapters::provider_factory::ProviderFactory;

// Auto-detect and use MLX if available
let factory = ProviderFactory::from_env()?;

// Create enhancement adapter (uses MLX on macOS)
let enhancer = factory.create_enhancement_adapter()?;
```

### Explicit MLX Backend

```rust
use task_orchestrator::adapters::provider_factory::ProviderFactory;

// Force MLX backend
let factory = ProviderFactory::new("mlx", "mlx-community/Phi-3-mini-4k-instruct")?;
let enhancer = factory.create_enhancement_adapter()?;
```

### Heterogeneous Pipeline with MLX

The MLX backend supports role-based model selection for optimal performance:

```rust
use task_orchestrator::adapters::provider_factory::ProviderFactory;
use task_orchestrator::domain::model_role::ModelRole;

let factory = ProviderFactory::from_env()?;

// Router uses fast Phi-3 model
let router = factory.create_enhancement_adapter_for_role(ModelRole::Router)?;

// Decomposer uses reasoning-optimized Orca-2
let decomposer = factory.create_task_decomposition_adapter_for_role(ModelRole::Decomposer)?;
```

## Performance Benchmarks

### Inference Speed Comparison (M3 Max)

| Framework | Model | Tokens/Sec | Notes |
|-----------|-------|------------|-------|
| MLX 0.15 | Llama 8B-4bit | ~65 t/s | Unified memory + Metal |
| Ollama | Llama 8B-4bit | ~45-50 t/s | CPU-optimized |
| MLX | Phi-3-mini | ~45 t/s | Fast routing model |
| Ollama | Phi-3-mini | ~30 t/s | Default backend |

**Speed Improvement**: 30-50% faster on average for the same models.

### Memory Efficiency

MLX uses unified memory architecture, allowing efficient CPU/GPU memory sharing:
- **Phi-3-mini**: ~2.5GB RAM
- **Orca-2-7B**: ~4.5GB RAM
- **Llama 8B**: ~5.5GB RAM

## Platform Detection

The MLX adapter automatically detects if it's available:

```rust
use task_orchestrator::adapters::mlx_subprocess_adapter::MlxSubprocessAdapter;

if MlxSubprocessAdapter::is_available() {
    println!("MLX backend available - using optimized inference");
} else {
    println!("MLX not available - falling back to Ollama");
}
```

**Detection checks**:
1. Running on macOS (`target_os = "macos"`)
2. Apple Silicon architecture (`target_arch = "aarch64"`)
3. Python is installed
4. `mlx-lm` package is available

## Troubleshooting

### Error: "MLX_NOT_AVAILABLE"

**Cause**: MLX backend is not available on this system.

**Solutions**:
1. Verify you're on macOS Apple Silicon:
   ```bash
   uname -m  # Should show "arm64"
   ```

2. Verify mlx-lm is installed:
   ```bash
   python3 -c "import mlx_lm; print('OK')"
   ```

3. Install mlx-lm if missing:
   ```bash
   pip3 install mlx-lm
   ```

### Error: "Failed to execute Python"

**Cause**: Python executable not found.

**Solutions**:
1. Install Python 3.9+:
   ```bash
   brew install python@3.11
   ```

2. Set explicit Python path:
   ```bash
   export PYTHON_PATH=$(which python3)
   ```

### Error: "Failed to load model"

**Cause**: Model not downloaded or incorrect model name.

**Solutions**:
1. Verify model exists on HuggingFace: https://huggingface.co/mlx-community

2. Pre-download the model:
   ```bash
   python3 -c "import mlx_lm; mlx_lm.load('mlx-community/Phi-3-mini-4k-instruct')"
   ```

3. Check disk space (models are 2-8GB each)

### Slow First Run

**Cause**: Model is being downloaded on first use.

**Solution**: Pre-download models (see Installation Step 3).

### Error: "ERROR: ..."

**Cause**: Python script execution failed.

**Solutions**:
1. Run with verbose error output:
   ```bash
   RUST_LOG=debug cargo run
   ```

2. Test MLX directly:
   ```bash
   python3 -c "
   import mlx_lm
   model, tokenizer = mlx_lm.load('mlx-community/Phi-3-mini-4k-instruct')
   response = mlx_lm.generate(model, tokenizer, prompt='Hello', max_tokens=10)
   print(response)
   "
   ```

## Comparison with Other Backends

### MLX vs Ollama

| Feature | MLX | Ollama |
|---------|-----|--------|
| Speed | 30-50% faster | Baseline |
| Platform | macOS only | Cross-platform |
| Setup | pip install | Binary download |
| Model format | MLX-optimized | GGUF |
| GPU acceleration | Metal | CPU-only |

**When to use MLX**: macOS Apple Silicon development/production for maximum performance.

**When to use Ollama**: Cross-platform compatibility, Linux/Windows deployment, simpler setup.

### MLX vs Candle

| Feature | MLX | Candle |
|---------|-----|--------|
| Speed | ~45 t/s | ~25-35 t/s |
| Platform | macOS only | Cross-platform |
| Binary size | External Python | Embedded Rust |
| Dependencies | Python + mlx-lm | None (pure Rust) |
| Maturity | Stable | Experimental |

**When to use MLX**: macOS with Python already installed.

**When to use Candle**: Embedded deployments, no external dependencies required.

## Environment Variables Reference

| Variable | Default | Description |
|----------|---------|-------------|
| `TASK_ORCHESTRATOR_PROVIDER` | Auto-detect | Provider name: "mlx", "ollama", "openai", "anthropic" |
| `MLX_MODEL` | `mlx-community/Phi-3-mini-4k-instruct` | MLX model identifier |
| `PYTHON_PATH` | Auto-detect | Path to Python 3.9+ executable |

## Advanced Configuration

### Custom Model Registry

To use custom MLX models from your own repository:

```bash
export MLX_MODEL="your-username/your-custom-model"
```

### Performance Tuning

MLX automatically optimizes for your hardware. No tuning needed in most cases.

For extreme memory constraints, use smaller models:
- `mlx-community/Phi-3-mini-4k-instruct` (2.5GB)
- `mlx-community/TinyLlama-1.1B` (1GB)

### Integration with Other Tools

MLX subprocess adapter is compatible with:
- **Task Orchestrator**: Full support for all ports
- **Heterogeneous Pipeline**: Role-based model selection
- **Benchmarking Tools**: Performance metrics collection

## References

- [MLX Framework](https://github.com/ml-explore/mlx) - Official Apple MLX repository
- [MLX-LM Documentation](https://github.com/ml-explore/mlx-lm) - Language model utilities
- [MLX Community Models](https://huggingface.co/mlx-community) - Pre-optimized models
- [MLX Research Document](./MLX_RESEARCH.md) - Detailed research and benchmarks

## Next Steps

1. Install mlx-lm: `pip3 install mlx-lm`
2. Run Rigger: It will auto-detect and use MLX
3. Compare performance with Ollama using benchmarking tools
4. Report any issues at: https://github.com/squillo/rig-task-pipeline/issues

---

**Last Updated**: 2025-11-24 (Phase 5 Sprint 11 Task 5.9)
