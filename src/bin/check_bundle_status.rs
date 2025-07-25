//! Quick test to check Jito bundle status
//! 
//! Usage: cargo run --bin check_bundle_status

use anyhow::Result;
use reqwest::Client;
use serde_json::json;

const JITO_BUNDLE_STATUS_ENDPOINT: &str = "your_endpoint";

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    // Bundle ID from the logs
    let bundle_id = "testbundle";
    
    println!("🔍 Checking bundle status for: {}", bundle_id);
    
    let client = Client::new();
    
    let payload = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getBundleStatuses",
        "params": [
            [bundle_id]
        ]
    });

    println!("📤 Sending request to: {}", JITO_BUNDLE_STATUS_ENDPOINT);
    println!("📤 Payload: {}", serde_json::to_string_pretty(&payload)?);

    let response = client
        .post(JITO_BUNDLE_STATUS_ENDPOINT)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await?;

    let status = response.status();
    let response_text = response.text().await?;
    
    println!("📥 Response status: {}", status);
    println!("📥 Response body: {}", response_text);

    if status.is_success() {
        let response_json: serde_json::Value = serde_json::from_str(&response_text)?;
        println!("📥 Parsed response: {}", serde_json::to_string_pretty(&response_json)?);
        
        if let Some(result) = response_json.get("result") {
            if let Some(value) = result.get("value") {
                if let Some(bundles) = value.as_array() {
                    if let Some(bundle_info) = bundles.first() {
                        if bundle_info.is_null() {
                            println!("❌ Bundle not found or not landed");
                        } else {
                            println!("✅ Bundle info found:");
                            println!("   Bundle ID: {}", bundle_info.get("bundle_id").unwrap_or(&json!("N/A")));
                            println!("   Transactions: {}", bundle_info.get("transactions").unwrap_or(&json!("N/A")));
                            println!("   Slot: {}", bundle_info.get("slot").unwrap_or(&json!("N/A")));
                            println!("   Confirmation Status: {}", bundle_info.get("confirmation_status").unwrap_or(&json!("N/A")));
                            println!("   Error: {}", bundle_info.get("err").unwrap_or(&json!("N/A")));
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
