//! Database operations example
//!
//! Run with:
//! ```
//! REDIS_CLOUD_API_KEY=xxx REDIS_CLOUD_API_SECRET=yyy cargo run --example databases
//! ```

use redis_cloud::CloudClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("REDIS_CLOUD_API_KEY")?;
    let api_secret = std::env::var("REDIS_CLOUD_API_SECRET")?;

    let client = CloudClient::builder()
        .api_key(api_key)
        .api_secret(api_secret)
        .build()?;

    // First, get the subscription ID from command line or use first available
    let subscription_id = std::env::args().nth(1).and_then(|s| s.parse::<i32>().ok());

    let subscription_id = match subscription_id {
        Some(id) => id,
        None => {
            // Get first Pro subscription
            let subs = client.subscriptions().get_all_subscriptions().await?;
            subs.subscriptions
                .and_then(|s| s.first().and_then(|sub| sub.id))
                .expect("No subscriptions found. Pass subscription ID as argument.")
        }
    };

    println!("Using subscription ID: {subscription_id}");

    // List databases in subscription
    println!("\nFetching databases...");
    let response = client
        .databases()
        .get_subscription_databases(subscription_id, None, None)
        .await?;

    for sub_info in &response.subscription {
        let dbs = &sub_info.databases;
        println!("Found {} databases:", dbs.len());
        for db in dbs {
            println!(
                "  - ID: {:?}, Name: {:?}, Status: {:?}",
                db.database_id, db.name, db.status
            );
            if let Some(endpoint) = &db.public_endpoint {
                println!("    Endpoint: {endpoint}");
            }
            if let Some(memory) = db.memory_limit_in_gb {
                println!("    Memory: {memory} GB");
            }
        }
    }

    // Get all databases using pagination helper
    println!("\nFetching all databases (with pagination)...");
    let all_dbs = client
        .databases()
        .get_all_databases(subscription_id)
        .await?;
    println!("Total databases: {}", all_dbs.len());

    Ok(())
}
