//! Test the new standard RPC implementation for live trades

use copybot_ultimate_v2::utils::live_trades;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    
    println!("🧪 Testing standard RPC implementation for live trades...");
    
    // Test with the same transaction signature we used before
    let test_mint = "test_mint".to_string();
    let test_signature = "test_signature".to_string();
    
    println!("📝 Processing test trade:");
    println!("   Mint: {}", test_mint);
    println!("   Signature: {}", test_signature);
    
    // Process the trade
    live_trades::process_geyser_trade(test_mint, test_signature).await;
    
    // Wait a bit for the background task to complete
    println!("⏳ Waiting for processing to complete...");
    tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
    
    println!("✅ Test completed! Check live_trades.jsonl for results.");
    
    Ok(())
}
