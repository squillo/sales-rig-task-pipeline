#!/usr/bin/env bash

# setup-ollama.sh
# Automated Ollama setup script for RigTask Pipeline project.
#
# This script installs Ollama natively on your machine, starts the service,
# pulls the required llama3.2 model, and verifies the installation.
#
# Supported Platforms:
# - macOS (via Homebrew)
# - Linux (via official install script)
#
# Usage:
#   ./setup-ollama.sh
#
# Revision History
# - 2025-11-06T20:19:00Z @AI: Fix Apple Silicon Rosetta 2 compatibility by forcing ARM64 execution.
# - 2025-11-06T20:14:00Z @AI: Initial script for native Ollama installation.

set -e  # Exit on error
set -u  # Exit on undefined variable

# Color output for better readability
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
OLLAMA_MODEL="llama3.2"
OLLAMA_HOST="${OLLAMA_HOST:-http://localhost:11434}"

# Helper functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Detect OS
detect_os() {
    if [[ "$OSTYPE" == "darwin"* ]]; then
        echo "macos"
    elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
        echo "linux"
    else
        echo "unknown"
    fi
}

# Check if Ollama is already installed
check_ollama_installed() {
    if command -v ollama &> /dev/null; then
        return 0
    else
        return 1
    fi
}

# Check if Ollama service is running
check_ollama_running() {
    if curl -s "$OLLAMA_HOST/api/tags" &> /dev/null; then
        return 0
    else
        return 1
    fi
}

# Install Ollama on macOS
install_ollama_macos() {
    log_info "Detected macOS. Installing Ollama via Homebrew..."

    # Check if Homebrew is installed
    if ! command -v brew &> /dev/null; then
        log_error "Homebrew is not installed. Please install it first:"
        echo "  /bin/bash -c \"\$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\""
        exit 1
    fi

    # Detect Apple Silicon (ARM64) and force ARM execution
    # This fixes "Cannot install under Rosetta 2 in ARM default prefix" error
    local machine_arch=$(uname -m)
    if [[ "$machine_arch" == "arm64" ]]; then
        log_info "Detected Apple Silicon (ARM64). Forcing ARM execution..."
        log_info "Running: arch -arm64 brew install ollama"
        arch -arm64 brew install ollama
    else
        log_info "Detected Intel Mac. Running standard installation..."
        log_info "Running: brew install ollama"
        brew install ollama
    fi

    log_success "Ollama installed successfully via Homebrew"
}

# Install Ollama on Linux
install_ollama_linux() {
    log_info "Detected Linux. Installing Ollama via official install script..."

    # Download and run official install script
    log_info "Running: curl -fsSL https://ollama.com/install.sh | sh"
    curl -fsSL https://ollama.com/install.sh | sh

    log_success "Ollama installed successfully via official installer"
}

# Start Ollama service
start_ollama_service() {
    local os=$1

    log_info "Starting Ollama service..."

    if [[ "$os" == "macos" ]]; then
        # On macOS, start as background process
        log_info "Starting Ollama service in background..."
        if ! check_ollama_running; then
            # Start ollama serve in background
            nohup ollama serve > /tmp/ollama.log 2>&1 &
            log_info "Ollama service started (PID: $!)"
            log_info "Logs: tail -f /tmp/ollama.log"
        fi
    elif [[ "$os" == "linux" ]]; then
        # On Linux, use systemd if available
        if command -v systemctl &> /dev/null; then
            log_info "Starting Ollama service via systemd..."
            sudo systemctl start ollama
            sudo systemctl enable ollama
        else
            # Fallback to background process
            log_info "Starting Ollama service in background..."
            nohup ollama serve > /tmp/ollama.log 2>&1 &
            log_info "Ollama service started (PID: $!)"
        fi
    fi

    # Wait for service to be ready
    log_info "Waiting for Ollama service to be ready..."
    local max_attempts=30
    local attempt=0
    while ! check_ollama_running; do
        attempt=$((attempt + 1))
        if [ $attempt -ge $max_attempts ]; then
            log_error "Ollama service failed to start after ${max_attempts} attempts"
            exit 1
        fi
        sleep 1
        echo -n "."
    done
    echo ""

    log_success "Ollama service is running"
}

# Pull required model
pull_model() {
    log_info "Pulling model: $OLLAMA_MODEL"
    log_info "This may take a few minutes depending on your connection..."

    if ollama pull "$OLLAMA_MODEL"; then
        log_success "Model $OLLAMA_MODEL pulled successfully"
    else
        log_error "Failed to pull model $OLLAMA_MODEL"
        exit 1
    fi
}

# Verify installation
verify_installation() {
    log_info "Verifying installation..."

    # Check if ollama command exists
    if ! check_ollama_installed; then
        log_error "Ollama command not found in PATH"
        exit 1
    fi

    # Check if service is running
    if ! check_ollama_running; then
        log_error "Ollama service is not responding at $OLLAMA_HOST"
        exit 1
    fi

    # Check if model is available
    if ollama list | grep -q "$OLLAMA_MODEL"; then
        log_success "Model $OLLAMA_MODEL is available"
    else
        log_error "Model $OLLAMA_MODEL not found in local models"
        exit 1
    fi

    # Test model with simple prompt
    log_info "Testing model with a simple prompt..."
    if echo "Hello" | ollama run "$OLLAMA_MODEL" > /dev/null 2>&1; then
        log_success "Model test completed successfully"
    else
        log_warning "Model test failed, but installation appears correct"
    fi
}

# Main installation flow
main() {
    echo "======================================"
    echo "  Ollama Setup for RigTask Pipeline"
    echo "======================================"
    echo ""

    # Detect OS
    local os=$(detect_os)
    if [[ "$os" == "unknown" ]]; then
        log_error "Unsupported operating system: $OSTYPE"
        log_info "Please install Ollama manually: https://ollama.com/download"
        exit 1
    fi

    log_info "Detected OS: $os"
    echo ""

    # Check if already installed
    if check_ollama_installed; then
        log_success "Ollama is already installed"
        log_info "Version: $(ollama --version)"
    else
        # Install based on OS
        if [[ "$os" == "macos" ]]; then
            install_ollama_macos
        elif [[ "$os" == "linux" ]]; then
            install_ollama_linux
        fi
    fi

    echo ""

    # Start service
    start_ollama_service "$os"

    echo ""

    # Pull model
    pull_model

    echo ""

    # Verify everything works
    verify_installation

    echo ""
    echo "======================================"
    log_success "Ollama setup complete!"
    echo "======================================"
    echo ""
    log_info "Next steps:"
    echo "  1. Run the tests: cd transcript_processor && cargo test"
    echo "  2. Run the application: cd transcript_processor && cargo run"
    echo ""
    log_info "Useful commands:"
    echo "  - List models: ollama list"
    echo "  - Stop service: pkill ollama"
    echo "  - View logs: tail -f /tmp/ollama.log"
    echo ""
}

# Run main function
main "$@"
