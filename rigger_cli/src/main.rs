//! Rigger CLI - Command-line interface for AI-driven project management.
//!
//! This binary provides a command-line interface for Rigger, enabling users
//! to initialize projects, parse PRDs, list tasks, and execute tasks through
//! the orchestration pipeline.
//!
//! Revision History
//! - 2025-11-30T21:45:00Z @AI: Add artifacts generate command for Phase 5 artifact generator CLI.
//! - 2025-11-28T23:00:00Z @AI: Add artifacts command handling for Phase 6 RAG CLI (Tasks 6.1, 6.2).
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
        commands::Commands::Artifacts { command } => {
            match command {
                commands::ArtifactsCommands::List { project, source_type, limit } => {
                    let parsed_limit = limit.as_ref().and_then(|s| s.parse::<usize>().ok());
                    commands::artifacts::list(
                        project.as_deref(),
                        source_type.as_deref(),
                        parsed_limit,
                    ).await?;
                }
                commands::ArtifactsCommands::Search { query, limit, threshold, project } => {
                    let parsed_limit = limit.as_ref().and_then(|s| s.parse::<usize>().ok());
                    let parsed_threshold = threshold.as_ref().and_then(|s| s.parse::<f32>().ok());
                    commands::artifacts::search(
                        &query,
                        parsed_limit,
                        parsed_threshold,
                        project.as_deref(),
                    ).await?;
                }
                commands::ArtifactsCommands::Generate {
                    source,
                    project,
                    depth,
                    max_items,
                    chunk_strategy,
                    chunk_size,
                    exclude,
                } => {
                    let parsed_depth = depth.as_ref().and_then(|s| s.parse::<usize>().ok());
                    let parsed_max_items = max_items.as_ref().and_then(|s| s.parse::<usize>().ok());
                    let parsed_chunk_size = chunk_size.as_ref().and_then(|s| s.parse::<usize>().ok());
                    commands::artifacts::generate(
                        &source,
                        project.as_deref(),
                        parsed_depth,
                        parsed_max_items,
                        chunk_strategy.as_deref(),
                        parsed_chunk_size,
                        exclude.as_deref(),
                    ).await?;
                }
            }
        }
    }

    std::result::Result::Ok(())
}
