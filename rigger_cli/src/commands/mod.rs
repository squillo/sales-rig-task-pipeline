//! Command modules for Rigger CLI.
//!
//! This module defines the CLI structure using clap derive API and organizes
//! subcommands into separate modules for maintainability.
//!
//! Revision History
//! - 2025-11-30T21:30:00Z @AI: Add artifacts generate command for Phase 5 artifact generator.
//! - 2025-11-28T23:00:00Z @AI: Add artifacts command for Phase 6 RAG CLI (Tasks 6.1, 6.2).
//! - 2025-11-22T16:30:00Z @AI: Initial command structure for Rigger CLI.

pub mod init;
pub mod parse;
pub mod list;
pub mod do_task;
pub mod server;
pub mod grpc_server;
pub mod tui;
pub mod artifacts;

/// Rig CLI - AI-driven project management for agents.
#[derive(clap::Parser)]
#[command(name = "rig")]
#[command(about = "Rig Task Pipeline - Project manager for AI agents", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

impl Cli {
    pub fn parse() -> Self {
        <Self as clap::Parser>::parse()
    }
}

#[derive(clap::Subcommand)]
pub enum Commands {
    /// Initialize .rigger directory in current working directory
    Init,

    /// Parse a PRD markdown file and generate tasks
    Parse {
        /// Path to the PRD markdown file
        prd_file: String,
    },

    /// List tasks with optional filters and sorting
    List {
        /// Filter by status (e.g., "pending", "in_progress", "completed")
        #[arg(long)]
        status: std::option::Option<String>,

        /// Filter by assignee name
        #[arg(long)]
        assignee: std::option::Option<String>,

        /// Sort by field (priority, created_at, due_date)
        #[arg(long, default_value = "created_at")]
        sort: String,

        /// Limit number of results
        #[arg(long)]
        limit: std::option::Option<String>,

        /// Offset for pagination
        #[arg(long)]
        offset: std::option::Option<String>,
    },

    /// Execute a task through the orchestration pipeline
    Do {
        /// Task ID to execute
        task_id: String,
    },

    /// Start MCP server mode (for IDE integration via stdio)
    Server,

    /// Start gRPC server mode (for distributed sidecar integration)
    Grpc {
        /// Port to listen on (default: 50051)
        #[arg(long, default_value = "50051")]
        port: u16,
    },

    /// Launch interactive TUI (Terminal User Interface)
    Tui,

    /// Manage knowledge artifacts (RAG knowledge base)
    Artifacts {
        #[command(subcommand)]
        command: ArtifactsCommands,
    },
}

/// Subcommands for artifacts management.
#[derive(clap::Subcommand)]
pub enum ArtifactsCommands {
    /// List all artifacts with optional filtering
    List {
        /// Filter by project ID
        #[arg(long)]
        project: std::option::Option<String>,

        /// Filter by source type (prd, file, web_research, user_input)
        #[arg(long)]
        source_type: std::option::Option<String>,

        /// Limit number of results (default: 20)
        #[arg(long)]
        limit: std::option::Option<String>,
    },

    /// Search artifacts using semantic similarity
    Search {
        /// Search query (natural language)
        query: String,

        /// Maximum number of results (default: 5)
        #[arg(long)]
        limit: std::option::Option<String>,

        /// Minimum similarity threshold 0.0-1.0 (default: 0.5)
        #[arg(long)]
        threshold: std::option::Option<String>,

        /// Filter by project ID
        #[arg(long)]
        project: std::option::Option<String>,
    },

    /// Generate artifacts from a directory or website
    Generate {
        /// Source path (directory) or URL (website) to generate artifacts from
        source: String,

        /// Project ID to associate artifacts with (default: directory name or domain)
        #[arg(long)]
        project: std::option::Option<String>,

        /// Maximum recursion depth for directories/crawling (default: 10)
        #[arg(long)]
        depth: std::option::Option<String>,

        /// Maximum number of files/pages to process (default: 1000)
        #[arg(long)]
        max_items: std::option::Option<String>,

        /// Chunking strategy: paragraph, sentence, fixed_size, whole_file (default: paragraph)
        #[arg(long)]
        chunk_strategy: std::option::Option<String>,

        /// Maximum chunk size in characters for fixed_size strategy (default: 1000)
        #[arg(long)]
        chunk_size: std::option::Option<String>,

        /// Additional glob patterns to exclude (comma-separated)
        #[arg(long)]
        exclude: std::option::Option<String>,
    },
}
