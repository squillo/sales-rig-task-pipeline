//! Quick test client for Rigger gRPC server.
//!
//! Tests basic RPCs like AddTask, ListTasks, UpdateTask to verify server functionality.

use rigger_cli::commands::grpc_server::rigger::v1::*;
use rigger_cli::commands::grpc_server::rigger::v1::rigger_service_client::RiggerServiceClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Rigger gRPC Test Client");
    println!("{}", "=".repeat(50));
    println!();

    // Connect to server
    println!("📡 Connecting to Rigger gRPC server at http://[::1]:50051...");
    let mut client = RiggerServiceClient::connect("http://[::1]:50051").await?;
    println!("✅ Connected!");
    println!();

    // Test 1: Add a task
    println!("Test 1: AddTask");
    println!("{}", "-".repeat(50));
    let add_request = AddTaskRequest {
        title: "Test gRPC integration".to_string(),
        assignee: Some("TestBot".to_string()),
        due_date: Some("2025-12-31".to_string()),
        source_prd_id: None,
        parent_task_id: None,
    };

    let add_response = client.add_task(add_request).await?;
    let task = add_response.into_inner().task.expect("Task should be returned");
    println!("✅ Task created:");
    println!("   ID: {}", task.id);
    println!("   Title: {}", task.title);
    println!("   Status: {:?}", TaskStatus::try_from(task.status)?);
    println!("   Assignee: {}", task.assignee.as_ref().unwrap_or(&"None".to_string()));
    println!();

    let task_id = task.id.clone();

    // Test 2: List tasks
    println!("Test 2: ListTasks");
    println!("{}", "-".repeat(50));
    let list_request = ListTasksRequest {
        status: None,
        assignee: None,
        limit: None,
        offset: None,
    };

    let list_response = client.list_tasks(list_request).await?;
    let list_inner = list_response.into_inner();
    println!("✅ Found {} task(s)", list_inner.total_count);
    for (i, task) in list_inner.tasks.iter().enumerate() {
        println!("   {}. {}: {}", i + 1, task.id, task.title);
    }
    println!();

    // Test 3: Get specific task
    println!("Test 3: GetTask");
    println!("{}", "-".repeat(50));
    let get_request = GetTaskRequest {
        task_id: task_id.clone(),
    };

    let get_response = client.get_task(get_request).await?;
    let task = get_response.into_inner().task.expect("Task should be returned");
    println!("✅ Retrieved task:");
    println!("   ID: {}", task.id);
    println!("   Title: {}", task.title);
    println!("   Created: {}", task.created_at);
    println!();

    // Test 4: Update task status
    println!("Test 4: UpdateTask");
    println!("{}", "-".repeat(50));
    let update_request = UpdateTaskRequest {
        task_id: task_id.clone(),
        status: Some(TaskStatus::InProgress as i32),
        assignee: None,
        due_date: None,
    };

    let update_response = client.update_task(update_request).await?;
    let task = update_response.into_inner().task.expect("Task should be returned");
    println!("✅ Task updated:");
    println!("   ID: {}", task.id);
    println!("   New Status: {:?}", TaskStatus::try_from(task.status)?);
    println!();

    // Test 5: Parse PRD
    println!("Test 5: ParsePRD");
    println!("{}", "-".repeat(50));

    // Create a simple test PRD file
    let test_prd_path = "/tmp/test_prd.md";
    std::fs::write(test_prd_path, r#"# Test PRD for gRPC

## Objectives
- Verify gRPC server functionality
- Test ParsePRD RPC
- Validate protobuf schema

## Tech Stack
- Rust
- tonic
- gRPC

## Constraints
- Must compile without errors
- Must handle async operations correctly
"#)?;

    let parse_request = ParsePrdRequest {
        prd_file_path: test_prd_path.to_string(),
    };

    let parse_response = client.parse_prd(parse_request).await?;
    let prd = parse_response.into_inner();
    println!("✅ PRD parsed:");
    println!("   Title: {}", prd.prd_title);
    println!("   Objectives: {}", prd.objectives.len());
    println!("   Tech Stack: {}", prd.tech_stack.len());
    println!("   Constraints: {}", prd.constraints.len());
    println!();

    // Test 6: Delete task
    println!("Test 6: DeleteTask");
    println!("{}", "-".repeat(50));
    let delete_request = DeleteTaskRequest {
        task_id: task_id.clone(),
    };

    let delete_response = client.delete_task(delete_request).await?;
    let success = delete_response.into_inner().success;
    println!("✅ Task deleted: {}", success);
    println!();

    println!("{}", "=".repeat(50));
    println!("🎉 All tests passed!");

    Ok(())
}
