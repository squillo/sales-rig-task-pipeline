//! Example gRPC sidecar client for subscribing to Rigger task events.
//!
//! This example demonstrates how a sidecar service can connect to the Rigger
//! gRPC server and subscribe to real-time task events (created, updated, deleted).
//!
//! # Usage
//!
//! 1. Start the Rigger gRPC server in one terminal:
//!    ```bash
//!    cargo run --bin rig -- grpc
//!    ```
//!
//! 2. Run this sidecar client example in another terminal:
//!    ```bash
//!    cargo run --example sidecar_client
//!    ```
//!
//! 3. In a third terminal, perform task operations to see events:
//!    ```bash
//!    # Using grpcurl (install with: brew install grpcurl)
//!    grpcurl -plaintext -d '{"title":"Test task"}' localhost:50051 rigger.v1.RiggerService/AddTask
//!    ```
//!
//! Revision History
//! - 2025-11-23T20:00:00Z @AI: Create sidecar client example for gRPC event streaming.

use rigger_cli::commands::grpc_server::rigger::v1::*;
use rigger_cli::commands::grpc_server::rigger::v1::rigger_service_client::RiggerServiceClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔌 Rigger Sidecar Client starting...");
    println!("   Connecting to Rigger gRPC server at http://[::1]:50051");
    println!();

    // Connect to Rigger gRPC server
    let mut client = RiggerServiceClient::connect("http://[::1]:50051").await?;
    println!("✅ Connected to Rigger gRPC server");

    // Subscribe to task events
    println!("📡 Subscribing to task events...");
    let request = tonic::Request::new(SubscribeToTaskEventsRequest {
        event_types: vec![], // Empty = subscribe to all event types
        assignee_filter: None,
    });

    let mut stream = client.subscribe_to_task_events(request).await?.into_inner();
    println!("✅ Subscribed to task event stream");
    println!();
    println!("Waiting for task events (Ctrl+C to exit)...");
    println!("---");

    // Process incoming task events
    while let Some(event) = stream.message().await? {
        print_task_event(&event);
    }

    println!("Stream closed");
    Ok(())
}

/// Prints a task event to stdout in a human-readable format.
fn print_task_event(event: &TaskEvent) {
    let event_type = match TaskEventType::try_from(event.event_type) {
        Ok(TaskEventType::Created) => "CREATED",
        Ok(TaskEventType::Updated) => "UPDATED",
        Ok(TaskEventType::Deleted) => "DELETED",
        Ok(TaskEventType::StatusChanged) => "STATUS_CHANGED",
        Ok(TaskEventType::Assigned) => "ASSIGNED",
        Ok(TaskEventType::Decomposed) => "DECOMPOSED",
        Ok(TaskEventType::Orchestrated) => "ORCHESTRATED",
        _ => "UNKNOWN",
    };

    println!("📬 Task Event Received:");
    println!("   Event ID: {}", event.event_id);
    println!("   Timestamp: {}", event.timestamp);
    println!("   Type: {}", event_type);

    if let Some(task) = &event.task {
        let status = match TaskStatus::try_from(task.status) {
            Ok(TaskStatus::Todo) => "Todo",
            Ok(TaskStatus::InProgress) => "InProgress",
            Ok(TaskStatus::Completed) => "Completed",
            Ok(TaskStatus::Archived) => "Archived",
            Ok(TaskStatus::PendingEnhancement) => "PendingEnhancement",
            Ok(TaskStatus::PendingComprehensionTest) => "PendingComprehensionTest",
            Ok(TaskStatus::PendingFollowOn) => "PendingFollowOn",
            Ok(TaskStatus::PendingDecomposition) => "PendingDecomposition",
            Ok(TaskStatus::Decomposed) => "Decomposed",
            Ok(TaskStatus::OrchestrationComplete) => "OrchestrationComplete",
            _ => "Unknown",
        };

        println!("   Task:");
        println!("     ID: {}", task.id);
        println!("     Title: {}", task.title);
        println!("     Status: {}", status);
        if let Some(ref assignee) = task.assignee {
            println!("     Assignee: {}", assignee);
        }
        if let Some(ref due_date) = task.due_date {
            println!("     Due Date: {}", due_date);
        }
    }

    if let Some(ref actor) = event.actor {
        println!("   Actor: {}", actor);
    }

    if !event.metadata.is_empty() {
        println!("   Metadata:");
        for (key, value) in &event.metadata {
            println!("     {}: {}", key, value);
        }
    }

    println!("---");
}
