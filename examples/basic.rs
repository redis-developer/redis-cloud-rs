//! Basic example showing how to connect and list subscriptions
//!
//! Run with:
//! ```
//! REDIS_CLOUD_API_KEY=xxx REDIS_CLOUD_API_SECRET=yyy cargo run --example basic
//! ```

use redis_cloud::{CloudClient, CloudError};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get credentials from environment
    let api_key = std::env::var("REDIS_CLOUD_API_KEY")
        .expect("REDIS_CLOUD_API_KEY environment variable required");
    let api_secret = std::env::var("REDIS_CLOUD_API_SECRET")
        .expect("REDIS_CLOUD_API_SECRET environment variable required");

    // Create client
    let client = CloudClient::builder()
        .api_key(api_key)
        .api_secret(api_secret)
        .build()?;

    // Get account information
    println!("Fetching account information...");
    let account = client.account().get_current_account().await?;
    if let Some(acc) = &account.account {
        println!("Account ID: {:?}", acc.id);
        println!("Account Name: {:?}", acc.name);
    }

    // List all Pro subscriptions
    println!("\nFetching Pro subscriptions...");
    match client.subscriptions().get_all_subscriptions().await {
        Ok(response) => {
            if let Some(subs) = response.subscriptions {
                println!("Found {} Pro subscriptions:", subs.len());
                for sub in subs {
                    println!(
                        "  - ID: {:?}, Name: {:?}, Status: {:?}",
                        sub.id, sub.name, sub.status
                    );
                }
            }
        }
        Err(CloudError::NotFound { .. }) => {
            println!("No Pro subscriptions found");
        }
        Err(e) => return Err(e.into()),
    }

    // List all Essentials subscriptions
    println!("\nFetching Essentials subscriptions...");
    match client.fixed_subscriptions().list().await {
        Ok(response) => {
            if let Some(subs) = response.subscriptions {
                println!("Found {} Essentials subscriptions:", subs.len());
                for sub in subs {
                    println!(
                        "  - ID: {:?}, Name: {:?}, Status: {:?}",
                        sub.id, sub.name, sub.status
                    );
                }
            }
        }
        Err(CloudError::NotFound { .. }) => {
            println!("No Essentials subscriptions found");
        }
        Err(e) => return Err(e.into()),
    }

    Ok(())
}
