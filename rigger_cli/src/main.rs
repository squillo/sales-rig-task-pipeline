//! Rigger CLI - Command-line interface for AI-driven project management.
//!
//! This binary provides a command-line interface for Rigger, enabling users
//! to initialize projects, parse PRDs, list tasks, and execute tasks through
//! the orchestration pipeline.
//!
//! Revision History
//! - 2025-11-24T01:05:00Z @AI: Add hexagonal architecture modules (ports, adapters, services) for clipboard integration.
//! - 2025-11-22T16:30:00Z @AI: Initial CLI structure with clap subcommands for Rigger Phase 0 Sprint 0.2.

mod commands;
mod display;
mod ports;
mod adapters;
mod services;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = commands::Cli::parse();

    match cli.command {
        commands::Commands::Init => {
            commands::init::execute().await?;
        }
        commands::Commands::Parse { prd_file } => {
            commands::parse::execute(&prd_file).await?;
        }
        commands::Commands::List { status, assignee, sort, limit, offset } => {
            commands::list::execute(status.as_deref(), assignee.as_deref(), &sort, limit.as_deref(), offset.as_deref()).await?;
        }
        commands::Commands::Do { task_id } => {
            commands::do_task::execute(&task_id).await?;
        }
        commands::Commands::Server => {
            commands::server::execute().await?;
        }
        commands::Commands::Grpc { port: _ } => {
            commands::grpc_server::execute().await?;
        }
        commands::Commands::Tui => {
            commands::tui::execute().await?;
        }
    }

    std::result::Result::Ok(())
}
