//! Build script for transcript_processor to guard against unsupported CUDA builds on macOS.
//!
//! This build script fails fast with a clear error message if the `cuda` feature
//! is enabled on a macOS target. On macOS, NVIDIA CUDA and `nvcc` are not available.
//! For GPU acceleration on Apple Silicon/macOS, use the `metal` feature instead.
//!
//! It also marks environment variables as inputs so Cargo knows when to rerun the script.
//!
//! Revision History
//! - 2025-11-08T13:45:30Z @AI: Clarify build error for `--all-features` on macOS; advise using per-crate Metal/CPU commands.
//! - 2025-11-08T13:25:00Z @AI: Add CUDA-on-macOS guard to prevent cudarc/nvcc build failures; guide to use `--features metal`.

fn main() {
    // Ensure the build script reruns if these env vars change.
    println!("cargo:rerun-if-env-changed=CARGO_CFG_TARGET_OS");
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_CUDA");

    // Detect platform and feature flags provided by Cargo.
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_else(|_| std::string::String::new());
    let cuda_feature_enabled = std::env::var("CARGO_FEATURE_CUDA").is_ok();

    // Fail fast on macOS when CUDA is enabled to avoid cudarc trying to invoke `nvcc`.
    if target_os == "macos" && cuda_feature_enabled {
        panic!(
            "CUDA feature is enabled on macOS. NVIDIA CUDA (nvcc) is unavailable on macOS. \
Use Metal instead: `cargo test -p transcript_processor --features metal -- --nocapture` \
Or run CPU: `cargo test -p transcript_processor -- --nocapture`."
        );
    }
}
