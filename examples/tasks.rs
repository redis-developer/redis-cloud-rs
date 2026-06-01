//! Poll a Redis Cloud async task by ID and print its state.
//!
//! Most write operations (create subscription, create database, delete
//! database, ...) return a `taskId`. Use this example to look the task up
//! later and watch its state transition through `processing` → `completed`
//! / `failed`.
//!
//! Run with:
//! ```bash
//! REDIS_CLOUD_API_KEY=xxx REDIS_CLOUD_API_SECRET=yyy \
//!   cargo run --example tasks -- TASK_ID
//! ```
//!
//! Pass `--watch` after the task id to poll every 5 seconds until the task
//! reaches a terminal state.

use redis_cloud::types::TaskStatus;
use redis_cloud::{CloudClient, CloudError};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let task_id = args.next().ok_or("usage: tasks TASK_ID [--watch]")?;
    let watch = args.next().as_deref() == Some("--watch");

    let api_key = std::env::var("REDIS_CLOUD_API_KEY")?;
    let api_secret = std::env::var("REDIS_CLOUD_API_SECRET")?;

    let client = CloudClient::builder()
        .api_key(api_key)
        .api_secret(api_secret)
        .build()?;

    loop {
        match client.tasks().get_task_by_id(task_id.clone()).await {
            Ok(task) => {
                println!(
                    "task {}: status={:?} command={:?} progress={:?}",
                    task.task_id.as_deref().unwrap_or(&task_id),
                    task.status,
                    task.command_type,
                    task.progress,
                );
                if let Some(desc) = task.description.as_deref() {
                    println!("  {desc}");
                }
                let terminal = matches!(
                    task.status,
                    Some(TaskStatus::ProcessingCompleted | TaskStatus::ProcessingError)
                );
                if !watch || terminal {
                    break;
                }
            }
            Err(CloudError::NotFound { .. }) => {
                eprintln!("task {task_id} not found");
                std::process::exit(1);
            }
            Err(e) => return Err(e.into()),
        }
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }

    Ok(())
}
