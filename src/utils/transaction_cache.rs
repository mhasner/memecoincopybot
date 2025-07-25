//! Transaction cache for fresh mint frontrunning
//! 
//! Caches pre-built transactions with fee recipient fetched once during caching.
//! During frontrun execution: ZERO RPC calls, just use cached transaction.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use solana_sdk::{pubkey::Pubkey, transaction::VersionedTransaction};
use anyhow::Result;
use log::info;

use crate::config::settings::Settings;
use crate::dex::pumpfun_simplified::fetch_pumpfun_swap_tx;

/// Cached transaction data for a fresh mint
#[derive(Debug, Clone)]
pub struct CachedTransaction {
    pub transaction: VersionedTransaction,
    pub min_tokens_out: u64,
    pub fee_recipient: Pubkey,
    pub cached_at: std::time::Instant,
}

/// Transaction cache manager
pub struct TransactionCache {
    fresh_mints: Arc<RwLock<HashMap<String, CachedTransaction>>>,
    general_cache: Arc<RwLock<HashMap<String, CachedTransaction>>>,
}

impl TransactionCache {
    pub fn new() -> Self {
        Self {
            fresh_mints: Arc::new(RwLock::new(HashMap::new())),
            general_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Cache a fresh mint transaction with fee recipient fetched once
    pub async fn cache_fresh_mint(
        &self,
        mint: &Pubkey,
        settings: &Settings,
        buy_amount_sol: f64,
    ) -> Result<()> {
        let mint_str = mint.to_string();
        
        // Check if already cached
        {
            let fresh_cache = self.fresh_mints.read().await;
            if fresh_cache.contains_key(&mint_str) {
                info!("💾 [CACHE] Fresh mint {} already cached", mint_str);
                return Ok(());
            }
        }

        // Get fee recipient from Global account (ONE RPC call)
        // Use pool_tracker to derive complete account set
        let derived = crate::utils::pool_tracker::derive_complete_pumpfun_accounts(&settings.rpc_client, &mint).await?;
        let fee_recipient = derived.fee_recipient;
        
        // Build transaction with all accounts resolved
        let lamports_limit = (buy_amount_sol * solana_sdk::native_token::LAMPORTS_PER_SOL as f64) as u64;
        let (tx, min_tokens_out) = fetch_pumpfun_swap_tx(settings, mint, lamports_limit).await?;
        
        // Cache the transaction
        let cached_tx = CachedTransaction {
            transaction: tx,
            min_tokens_out,
            fee_recipient,
            cached_at: std::time::Instant::now(),
        };
        
        {
            let mut fresh_cache = self.fresh_mints.write().await;
            fresh_cache.insert(mint_str.clone(), cached_tx);
        }
        
        info!("💾 [FRESH_MINT] Cached fresh mint: {}", mint_str);
        Ok(())
    }

    /// Get cached transaction for fresh mint frontrunning
    pub async fn get_fresh_mint_transaction(&self, mint: &Pubkey) -> Option<CachedTransaction> {
        let mint_str = mint.to_string();
        let fresh_cache = self.fresh_mints.read().await;
        let cache_size = fresh_cache.len();
        
        info!("🔍 [CACHE] Checking fresh mint cache for {}, cache size: {}", mint_str, cache_size);
        
        if let Some(cached) = fresh_cache.get(&mint_str) {
            // Check if cache is still fresh (within 30 seconds)
            if cached.cached_at.elapsed().as_secs() < 30 {
                info!("✅ [CACHE] Found fresh mint cache entry for {}", mint_str);
                return Some(cached.clone());
            } else {
                info!("⏰ [CACHE] Fresh mint cache entry expired for {}", mint_str);
            }
        } else {
            info!("❌ [CACHE] No fresh mint entry found for {}", mint_str);
        }
        
        None
    }

    /// Get cached transaction from general cache
    pub async fn get_general_transaction(&self, mint: &Pubkey) -> Option<CachedTransaction> {
        let mint_str = mint.to_string();
        let general_cache = self.general_cache.read().await;
        let cache_size = general_cache.len();
        
        info!("🔍 [CACHE] Checking general cache for {}, cache size: {}", mint_str, cache_size);
        
        if let Some(cached) = general_cache.get(&mint_str) {
            // Check if cache is still fresh (within 60 seconds)
            if cached.cached_at.elapsed().as_secs() < 60 {
                info!("✅ [CACHE] Found general cache entry for {}", mint_str);
                return Some(cached.clone());
            } else {
                info!("⏰ [CACHE] General cache entry expired for {}", mint_str);
            }
        } else {
            info!("❌ [CACHE] No general cache entry found for {}", mint_str);
        }
        
        None
    }

    /// Cache a transaction in general cache
    pub async fn cache_general_transaction(
        &self,
        mint: &Pubkey,
        transaction: VersionedTransaction,
        min_tokens_out: u64,
        fee_recipient: Pubkey,
    ) {
        let mint_str = mint.to_string();
        let cached_tx = CachedTransaction {
            transaction,
            min_tokens_out,
            fee_recipient,
            cached_at: std::time::Instant::now(),
        };
        
        {
            let mut general_cache = self.general_cache.write().await;
            general_cache.insert(mint_str.clone(), cached_tx);
        }
        
        info!("💾 [GENERAL] Cached transaction for mint: {}", mint_str);
    }

    /// Get cache statistics
    pub async fn get_stats(&self) -> (usize, usize) {
        let fresh_cache = self.fresh_mints.read().await;
        let general_cache = self.general_cache.read().await;
        (fresh_cache.len(), general_cache.len())
    }

    /// Clear expired entries
    pub async fn cleanup_expired(&self) {
        let now = std::time::Instant::now();
        
        // Clean fresh mint cache (30 second expiry)
        {
            let mut fresh_cache = self.fresh_mints.write().await;
            let before_count = fresh_cache.len();
            fresh_cache.retain(|_, cached| now.duration_since(cached.cached_at).as_secs() < 30);
            let after_count = fresh_cache.len();
            if before_count != after_count {
                info!("🧹 [CACHE] Cleaned {} expired fresh mint entries", before_count - after_count);
            }
        }
        
        // Clean general cache (60 second expiry)
        {
            let mut general_cache = self.general_cache.write().await;
            let before_count = general_cache.len();
            general_cache.retain(|_, cached| now.duration_since(cached.cached_at).as_secs() < 60);
            let after_count = general_cache.len();
            if before_count != after_count {
                info!("🧹 [CACHE] Cleaned {} expired general entries", before_count - after_count);
            }
        }
    }

    /// Check if we have a valid pre-signed transaction for the mint
    pub async fn has_valid_transaction(&self, mint: &Pubkey) -> bool {
        // Check fresh mint cache first
        if self.get_fresh_mint_transaction(mint).await.is_some() {
            return true;
        }
        
        // Check general cache
        if self.get_general_transaction(mint).await.is_some() {
            return true;
        }
        
        info!("❌ [CACHE] No valid pre-signed tx for mint: {}", mint);
        false
    }
}

/// Global transaction cache instance
lazy_static::lazy_static! {
    pub static ref TRANSACTION_CACHE: TransactionCache = TransactionCache::new();
}
