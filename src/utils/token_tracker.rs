//! Token Amount Tracking
//! 
//! Tracks exact token amounts from our BUY transactions to enable
//! immediate SELLs without ATA polling delays.

use std::collections::HashMap;
use tokio::sync::RwLock;
use solana_sdk::pubkey::Pubkey;
use once_cell::sync::Lazy;

/// Global token amount storage
/// Maps (wallet_pubkey, mint) -> token_amount
static TOKEN_AMOUNTS: Lazy<RwLock<HashMap<(Pubkey, Pubkey), u64>>> = 
    Lazy::new(|| RwLock::new(HashMap::new()));

/// Store the exact token amount we received from a BUY transaction
pub async fn store_token_amount(wallet: &Pubkey, mint: &Pubkey, amount: u64) {
    let mut amounts = TOKEN_AMOUNTS.write().await;
    amounts.insert((*wallet, *mint), amount);
}

/// Get the stored token amount for a wallet+mint combination
pub async fn get_token_amount(wallet: &Pubkey, mint: &Pubkey) -> Option<u64> {
    let amounts = TOKEN_AMOUNTS.read().await;
    let amount = amounts.get(&(*wallet, *mint)).copied();
    if let Some(amt) = amount {
        println!("📖 [TOKEN_TRACKER] Retrieved {} tokens for wallet {} mint {}", amt, wallet, mint);
    } else {
    }
    amount
}

/// Remove token amount (after selling all tokens)
pub async fn clear_token_amount(wallet: &Pubkey, mint: &Pubkey) {
    let mut amounts = TOKEN_AMOUNTS.write().await;
    if let Some(amount) = amounts.remove(&(*wallet, *mint)) {
    }
}

/// Update token amount after partial sell
pub async fn update_token_amount(wallet: &Pubkey, mint: &Pubkey, new_amount: u64) {
    let mut amounts = TOKEN_AMOUNTS.write().await;
    amounts.insert((*wallet, *mint), new_amount);
    println!("🔄 [TOKEN_TRACKER] Updated to {} tokens for wallet {} mint {}", new_amount, wallet, mint);
}

/// Calculate sell amount based on percentage of our holdings
pub async fn calculate_sell_amount(wallet: &Pubkey, mint: &Pubkey, percentage: f64) -> Option<u64> {
    let amount = get_token_amount(wallet, mint).await?;
    let sell_amount = (percentage * amount as f64).round() as u64;
    println!("🧮 [TOKEN_TRACKER] Calculated sell amount: {:.2}% of {} = {} tokens", 
             percentage * 100.0, amount, sell_amount);
    Some(sell_amount)
}
