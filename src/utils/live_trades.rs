//! Live trades processing - receives data directly from Geyser, only fetches token symbol
//! 
//! Flow:
//! 1. Geyser sends: mint, signature, sol_amount, token_amount, timestamp (all data from Geyser)
//! 2. We calculate dollar amount from SOL amount
//! 3. We make metadata call to get token symbol (only RPC call needed)
//! 4. We write complete trade data to live_trades.jsonl

use std::fs::OpenOptions;
use std::io::Write;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use log::{info, error};
use reqwest::Client;
use tokio::time::{timeout, Duration};

const LIVE_TRADES_FILE: &str = "live_trades.jsonl";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveTrade {
    pub mint: String,
    pub signature: String,
    pub sol_amount: f64,
    pub usd_amount: f64,
    pub token_amount: u64,
    pub timestamp: DateTime<Utc>,
    pub symbol: String,
    pub side: String, // "buy" or "sell"
}

/// Process a trade from Geyser - all data provided directly from Geyser
pub async fn process_geyser_trade_with_data(
    mint: String,
    signature: String,
    sol_amount: f64,
    token_amount: u64,
    timestamp: chrono::DateTime<chrono::Utc>,
    side: String, // "buy" or "sell"
) {
    info!("🔄 [LIVE_TRADES] Processing trade with Geyser data - mint: {}, signature: {}, sol: {:.6}, tokens: {}", 
          mint, signature, sol_amount, token_amount);
    
    // Spawn background task to avoid blocking Geyser
    tokio::spawn(async move {
        if let Err(e) = enhance_and_write_trade_fast(mint, signature, sol_amount, token_amount, timestamp, side).await {
            error!("❌ [LIVE_TRADES] Failed to process trade: {}", e);
        }
    });
}

/// Legacy function for backward compatibility - will be removed
pub async fn process_geyser_trade(mint: String, signature: String) {
    info!("⚠️ [LIVE_TRADES] Using legacy process_geyser_trade - consider updating to process_geyser_trade_with_data");
    
    // For now, create dummy data - this should be updated in the caller
    let sol_amount = 0.0;
    let token_amount = 0;
    let timestamp = Utc::now();
    let side = "unknown".to_string();
    
    process_geyser_trade_with_data(mint, signature, sol_amount, token_amount, timestamp, side).await;
}

/// Fast trade processing - only fetches token symbol, no slow RPC calls
async fn enhance_and_write_trade_fast(
    mint: String, 
    signature: String, 
    sol_amount: f64, 
    token_amount: u64, 
    timestamp: DateTime<Utc>,
    side: String
) -> anyhow::Result<()> {
    // Step 1: Calculate USD amount from SOL amount
    let usd_amount = calculate_usd_amount(sol_amount).await;
    
    // Step 2: Get token symbol from metadata (only RPC call needed)
    let symbol = fetch_token_symbol(&mint).await.unwrap_or_else(|_| format!("{}...", &mint[..8]));
    
    // Step 3: Create complete trade
    let trade = LiveTrade {
        mint,
        signature,
        sol_amount,
        usd_amount,
        token_amount,
        timestamp,
        symbol,
        side,
    };
    
    // Step 4: Write to file
    write_trade_to_file(&trade).await?;
    
    info!("✅ [LIVE_TRADES] Enhanced and wrote trade - side: {}, sol: {:.6}, usd: ${:.2}, tokens: {}, symbol: {}", 
          trade.side, trade.sol_amount, trade.usd_amount, trade.token_amount, trade.symbol);
    
    Ok(())
}


/// Calculate USD amount from SOL amount using a simple price estimate
async fn calculate_usd_amount(sol_amount: f64) -> f64 {
    // For now, use a simple SOL price estimate
    // In production, you might want to fetch real-time SOL price from an API
    let sol_price_usd = 200.0; // Approximate SOL price in USD
    sol_amount * sol_price_usd
}

/// Fetch token symbol from Helius DAS API
async fn fetch_token_symbol(mint: &str) -> anyhow::Result<String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .build()?;
    
    let url = "your_rpc_url";
    
    let request_body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getAsset",
        "params": [mint]
    });

    let response = timeout(Duration::from_secs(5),
        client.post(url)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
    ).await??;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!("DAS API error: {}", response.status()));
    }

    let response_json: serde_json::Value = response.json().await?;
    
    let symbol = response_json
        .get("result")
        .and_then(|r| r.get("content"))
        .and_then(|c| c.get("metadata"))
        .and_then(|m| m.get("symbol"))
        .and_then(|s| s.as_str())
        .unwrap_or(&format!("{}...", &mint[..8]))
        .to_string();

    Ok(symbol)
}

/// Write trade to JSONL file
async fn write_trade_to_file(trade: &LiveTrade) -> anyhow::Result<()> {
    let json_line = serde_json::to_string(trade)?;
    
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(LIVE_TRADES_FILE)?;
    
    writeln!(file, "{}", json_line)?;
    file.flush()?;
    
    Ok(())
}
