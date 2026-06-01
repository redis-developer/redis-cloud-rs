//! Generate and download a Redis Cloud cost report in FOCUS format.
//!
//! The cost-report endpoints are a two-step async flow:
//!
//! 1. `POST /cost-report` kicks off generation and returns a `taskId`.
//! 2. Poll `tasks().get_task_by_id(...)` until the task completes; the
//!    response payload carries the `costReportId`.
//! 3. `GET /cost-report/{costReportId}` returns the report file.
//!
//! This example wires those steps together.
//!
//! Run with:
//! ```bash
//! REDIS_CLOUD_API_KEY=xxx REDIS_CLOUD_API_SECRET=yyy \
//!   cargo run --example cost_report -- 2026-04-01 2026-04-30
//! ```
//!
//! The maximum date range supported by the live API is 40 days.

use redis_cloud::CloudClient;
use redis_cloud::cost_report::CostReportCreateRequest;
use redis_cloud::types::TaskStatus;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let start = args
        .next()
        .ok_or("usage: cost_report START_DATE END_DATE (dates in YYYY-MM-DD)")?;
    let end = args
        .next()
        .ok_or("usage: cost_report START_DATE END_DATE (dates in YYYY-MM-DD)")?;

    let api_key = std::env::var("REDIS_CLOUD_API_KEY")?;
    let api_secret = std::env::var("REDIS_CLOUD_API_SECRET")?;

    let client = CloudClient::builder()
        .api_key(api_key)
        .api_secret(api_secret)
        .build()?;

    // Step 1 — kick off generation.
    let request = CostReportCreateRequest::new(&start, &end);
    let task = client.cost_reports().generate_cost_report(request).await?;
    let task_id = task
        .task_id
        .clone()
        .ok_or("server did not return a task_id")?;
    println!("kicked off cost-report generation: task {task_id}");

    // Step 2 — poll the task until it reaches a terminal state.
    println!("polling task...");
    let report_id = loop {
        tokio::time::sleep(Duration::from_secs(5)).await;
        let state = client.tasks().get_task_by_id(task_id.clone()).await?;
        println!("  status={:?} progress={:?}", state.status, state.progress);
        match state.status {
            Some(TaskStatus::ProcessingCompleted) => {
                // ProcessorResponse shape varies; raw JSON is the safest read.
                let raw = serde_json::to_value(&state.response)?;
                break raw
                    .get("resourceId")
                    .and_then(|v| v.as_str())
                    .map(str::to_string)
                    .or_else(|| {
                        raw.get("costReportId")
                            .and_then(|v| v.as_str())
                            .map(str::to_string)
                    })
                    .ok_or("task completed but did not return a costReportId")?;
            }
            Some(TaskStatus::ProcessingError) => {
                return Err(format!(
                    "task ended with status {:?}: {}",
                    state.status,
                    serde_json::to_string(&state.response)?
                )
                .into());
            }
            _ => {}
        }
    };

    // Step 3 — download the report bytes.
    println!("downloading report {report_id}");
    let bytes = client
        .cost_reports()
        .download_cost_report(&report_id)
        .await?;

    let out_path = format!("cost-report-{report_id}.json");
    std::fs::write(&out_path, &bytes)?;
    println!("wrote {} bytes to {out_path}", bytes.len());

    Ok(())
}
