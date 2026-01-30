//! Streaming example - process databases one at a time without loading all into memory
//!
//! Run with:
//! ```
//! REDIS_CLOUD_API_KEY=xxx REDIS_CLOUD_API_SECRET=yyy cargo run --example stream_databases -- SUBSCRIPTION_ID
//! ```

use futures::StreamExt;
use redis_cloud::{CloudClient, CloudError};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("REDIS_CLOUD_API_KEY")?;
    let api_secret = std::env::var("REDIS_CLOUD_API_SECRET")?;

    let subscription_id: i32 = std::env::args()
        .nth(1)
        .expect("Usage: stream_databases SUBSCRIPTION_ID")
        .parse()
        .expect("SUBSCRIPTION_ID must be an integer");

    let client = CloudClient::builder()
        .api_key(api_key)
        .api_secret(api_secret)
        .build()?;

    println!("Streaming databases from subscription {subscription_id}...\n");

    // Get the database handler
    let db_handler = client.databases();

    // Stream databases - this fetches pages as needed instead of loading all at once
    let mut stream = std::pin::pin!(db_handler.stream_databases(subscription_id));

    // Process each database as it arrives
    let mut count = 0;
    while let Some(result) = stream.next().await {
        match result {
            Ok(db) => {
                count += 1;
                println!(
                    "[{}] {:?} - {:?} ({:?})",
                    count,
                    db.name.as_deref().unwrap_or("unnamed"),
                    db.status.as_deref().unwrap_or("unknown"),
                    db.public_endpoint.as_deref().unwrap_or("no endpoint")
                );
            }
            Err(CloudError::NotFound { .. }) => {
                println!("No databases found in subscription");
                break;
            }
            Err(e) => return Err(e.into()),
        }
    }

    println!("\nProcessed {count} databases");
    Ok(())
}
