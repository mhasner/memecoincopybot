//! Copy trading timing metrics - measures latency from tracked wallet detection to our confirmation

use std::collections::HashMap;
use std::time::Instant;
use tokio::sync::RwLock;
use solana_sdk::pubkey::Pubkey;
use once_cell::sync::Lazy;

/// Global timing tracker for copy trading performance metrics
static TIMING_TRACKER: Lazy<RwLock<HashMap<String, Instant>>> = 
    Lazy::new(|| RwLock::new(HashMap::new()));

/// Record the start time when we detect a tracked wallet event
pub async fn start_timing(mint: &Pubkey, event_type: &str) {
    let key = format!("{}_{}", mint, event_type);
    let mut tracker = TIMING_TRACKER.write().await;
    tracker.insert(key, Instant::now());
    
}

/// Calculate and log the elapsed time from detection to our confirmation
pub async fn end_timing(mint: &Pubkey, event_type: &str) -> Option<u128> {
    let key = format!("{}_{}", mint, event_type);
    let mut tracker = TIMING_TRACKER.write().await;
    
    if let Some(start_time) = tracker.remove(&key) {
        let elapsed = start_time.elapsed();
        let elapsed_ms = elapsed.as_millis();
        
        println!("🎯 [COPY_LATENCY] {} {} -> Our confirmation: {:.2}ms", 
                 event_type, mint, elapsed_ms);
        
        // Log performance categories
        match elapsed_ms {
            0..=100 => println!("🚀 [PERFORMANCE] EXCELLENT: <100ms copy latency"),
            101..=250 => println!("✅ [PERFORMANCE] GOOD: 100-250ms copy latency"),
            251..=500 => println!("⚠️ [PERFORMANCE] FAIR: 250-500ms copy latency"),
            501..=1000 => println!("🐌 [PERFORMANCE] SLOW: 500ms-1s copy latency"),
            _ => println!("🚨 [PERFORMANCE] VERY SLOW: >1s copy latency"),
        }
        
        Some(elapsed_ms)
    } else {
        println!("⚠️ [TIMING_END] No start time found for {} {}", event_type, mint);
        None
    }
}

/// Clean up old timing entries (prevent memory leaks)
pub async fn cleanup_old_timings() {
    let mut tracker = TIMING_TRACKER.write().await;
    let now = Instant::now();
    
    // Remove entries older than 30 seconds (likely failed transactions)
    tracker.retain(|key, start_time| {
        let age = now.duration_since(*start_time);
        if age.as_secs() > 30 {
            println!("🧹 [TIMING_CLEANUP] Removed stale timing entry: {}", key);
            false
        } else {
            true
        }
    });
}

/// Get current number of pending timings (for debugging)
pub async fn get_pending_count() -> usize {
    let tracker = TIMING_TRACKER.read().await;
    tracker.len()
}
