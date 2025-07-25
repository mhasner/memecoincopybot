//! Global "already bought this mint" guard with optimistic marking
//! 
//! Supports both immediate optimistic marking (for double-prevention) and
//! confirmed marking (after Geyser confirmation) for accurate state management.
//! 
//! Features automatic timeout cleanup for pending transactions that never confirm.

use std::collections::{HashSet, HashMap};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;
use once_cell::sync::Lazy;
use solana_sdk::pubkey::Pubkey;

/// HashSet key = `"wallet_pubkey:mint_pubkey"`
/// Final confirmed buys - only set after Geyser confirmation
pub static CONFIRMED_BUYS: Lazy<Mutex<HashSet<String>>> =
    Lazy::new(|| Mutex::new(HashSet::new()));

/// Optimistic pending buys with timestamps for timeout tracking
/// HashMap key = `"wallet_pubkey:mint_pubkey"`, value = timestamp_ms
pub static PENDING_BUYS: Lazy<Mutex<HashMap<String, u64>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Timeout for pending transactions in milliseconds (1 second)
const PENDING_TIMEOUT_MS: u64 = 1000;

/// Get current timestamp in milliseconds
fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Check if we should allow a buy (only check pending, not confirmed)
/// This allows multiple legitimate buys of the same token over time
/// Only prevents rapid-fire duplicates within the timeout window
pub async fn should_allow_buy(wallet: &Pubkey, mint: &Pubkey) -> bool {
    let key = format!("{wallet}:{mint}");
    
    // FIXED: Only check pending buys, not confirmed buys
    // This allows legitimate new buy signals after previous buys are confirmed
    // The purpose of dedupe is to prevent same-block/rapid-fire duplicates, not to prevent all future buys
    
    // Check pending with timeout cleanup
    let mut pending = PENDING_BUYS.lock().await;
    if let Some(&timestamp) = pending.get(&key) {
        let now = current_timestamp_ms();
        let age_ms = now.saturating_sub(timestamp);
        if age_ms > PENDING_TIMEOUT_MS {
            return true;
        } else {
            return false;
        }
    }

    true
}

/// Mark buy as pending (optimistic) - call immediately on transaction submission
pub async fn mark_pending_buy(wallet: &Pubkey, mint: &Pubkey) {
    let key = format!("{wallet}:{mint}");
    let timestamp = current_timestamp_ms();
    PENDING_BUYS.lock().await.insert(key, timestamp);
}

/// Confirm buy (move from pending to confirmed) - call after Geyser confirmation
pub async fn confirm_buy(wallet: &Pubkey, mint: &Pubkey) {
    let key = format!("{wallet}:{mint}");
    
    // Remove from pending
    PENDING_BUYS.lock().await.remove(&key);
    
    // Add to confirmed
    CONFIRMED_BUYS.lock().await.insert(key);
    
    println!("✅ [DEDUPE] Confirmed buy for {}", mint);
}

/// Rollback pending buy (if transaction fails) - removes from pending
pub async fn rollback_pending_buy(wallet: &Pubkey, mint: &Pubkey) {
    let key = format!("{wallet}:{mint}");
    if PENDING_BUYS.lock().await.remove(&key).is_some() {
    }
}

/// Legacy function - mark as confirmed directly (for backward compatibility)
pub async fn mark_bought(wallet: &Pubkey, mint: &Pubkey) {
    confirm_buy(wallet, mint).await;
}

/// Clear confirmed buy after 100% sell - call after Geyser sell confirmation
pub async fn clear(wallet: &Pubkey, mint: &Pubkey) {
    let key = format!("{wallet}:{mint}");
    
    // Remove from both pending and confirmed (in case of edge cases)
    PENDING_BUYS.lock().await.remove(&key);
    if CONFIRMED_BUYS.lock().await.remove(&key) {
    }
}

/// Check if we have a confirmed buy (for sell operations)
pub async fn has_confirmed_buy(wallet: &Pubkey, mint: &Pubkey) -> bool {
    let key = format!("{wallet}:{mint}");
    CONFIRMED_BUYS.lock().await.contains(&key)
}

/// Get all confirmed buys (for debugging/monitoring)
pub async fn get_confirmed_buys() -> Vec<String> {
    CONFIRMED_BUYS.lock().await.iter().cloned().collect()
}

/// Get all pending buys (for debugging/monitoring)
pub async fn get_pending_buys() -> Vec<String> {
    PENDING_BUYS.lock().await.keys().cloned().collect()
}

/// Clean up old pending buys (call periodically to prevent memory leaks)
/// Removes all pending transactions older than the timeout threshold
pub async fn cleanup_old_pending() {
    let now = current_timestamp_ms();
    let mut pending = PENDING_BUYS.lock().await;
    let initial_count = pending.len();
    
    // Remove expired pending transactions
    pending.retain(|_key, timestamp| {
        let age_ms = now.saturating_sub(*timestamp);
        age_ms <= PENDING_TIMEOUT_MS
    });
}

/// Start background cleanup task that runs every 500ms
/// Call this once during bot initialization
pub async fn start_cleanup_task() {
    tokio::spawn(async {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(500));
        loop {
            interval.tick().await;
            cleanup_old_pending().await;
        }
    });
    println!("🧹 [DEDUPE] Started background cleanup task (500ms interval)");
}
