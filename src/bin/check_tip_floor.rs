//! Check current Jito tip floor recommendations
//! 
//! Usage: cargo run --bin check_tip_floor

use anyhow::Result;
use reqwest::Client;

const JITO_TIP_FLOOR_ENDPOINT: &str = "https://bundles.jito.wtf/api/v1/bundles/tip_floor";

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    println!("🔍 Checking current Jito tip floor recommendations...");
    
    let client = Client::new();
    
    let response = client
        .get(JITO_TIP_FLOOR_ENDPOINT)
        .send()
        .await?;

    let status = response.status();
    let response_text = response.text().await?;
    
    println!("📥 Response status: {}", status);
    
    if status.is_success() {
        let tip_data: serde_json::Value = serde_json::from_str(&response_text)?;
        println!("📥 Tip floor data: {}", serde_json::to_string_pretty(&tip_data)?);
        
        if let Some(tip_array) = tip_data.as_array() {
            if let Some(latest_tip) = tip_array.first() {
                println!("\n💰 Current Tip Recommendations:");
                
                if let Some(p25) = latest_tip.get("landed_tips_25th_percentile") {
                    let lamports = (p25.as_f64().unwrap_or(0.0) * 1_000_000_000.0) as u64;
                    println!("   25th percentile: {} SOL ({} lamports)", p25, lamports);
                }
                
                if let Some(p50) = latest_tip.get("landed_tips_50th_percentile") {
                    let lamports = (p50.as_f64().unwrap_or(0.0) * 1_000_000_000.0) as u64;
                    println!("   50th percentile: {} SOL ({} lamports)", p50, lamports);
                }
                
                if let Some(p75) = latest_tip.get("landed_tips_75th_percentile") {
                    let lamports = (p75.as_f64().unwrap_or(0.0) * 1_000_000_000.0) as u64;
                    println!("   75th percentile: {} SOL ({} lamports)", p75, lamports);
                }
                
                if let Some(p95) = latest_tip.get("landed_tips_95th_percentile") {
                    let lamports = (p95.as_f64().unwrap_or(0.0) * 1_000_000_000.0) as u64;
                    println!("   95th percentile: {} SOL ({} lamports)", p95, lamports);
                }
                
                println!("\n🎯 Your current tip: 0.001 SOL (1,000,000 lamports)");
                
                if let Some(p50) = latest_tip.get("landed_tips_50th_percentile") {
                    let recommended_lamports = (p50.as_f64().unwrap_or(0.0) * 1_000_000_000.0) as u64;
                    if recommended_lamports > 1_000_000 {
                        println!("⚠️  RECOMMENDATION: Consider increasing tip to at least {} lamports ({:.6} SOL) for better success rate", 
                                recommended_lamports, recommended_lamports as f64 / 1_000_000_000.0);
                    } else {
                        println!("✅ Your tip appears adequate for current conditions");
                    }
                }
            }
        }
    } else {
        println!("❌ Failed to get tip floor data: {}", response_text);
    }

    Ok(())
}
