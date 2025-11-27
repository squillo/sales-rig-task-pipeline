//! Command modules for Rigger CLI.
//!
//! This module defines the CLI structure using clap derive API and organizes
//! subcommands into separate modules for maintainability.
//!
//! Revision History
//! - 2025-11-22T16:30:00Z @AI: Initial command structure for Rigger CLI.

pub mod init;
pub mod parse;
pub mod list;
pub mod do_task;
pub mod server;
pub mod grpc_server;
pub mod tui;

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
}
