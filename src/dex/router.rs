//! DEX Router - Smart routing based on program ID detection from Geyser
//! 
//! This module provides instant DEX identification by comparing program IDs
//! from Geyser transaction data against known DEX program IDs.
//! 


use anyhow::Result;
use solana_sdk::pubkey::Pubkey;
use spl_associated_token_account;

use crate::{
    config::settings::Settings,
    strategy::{DexKind, Side},
    dex::{
        pumpfun_simplified,
        moonshot::MoonshotDex,
        raydium::RaydiumDex,
        raydium_launchpad,
    },
};

/// All known DEX program IDs collected from individual DEX modules
pub mod program_ids {
    use solana_sdk::pubkey::Pubkey;
    use std::str::FromStr;
    
    // PumpFun
    pub const PUMPFUN_PROGRAM_ID: &str = "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P";
    
    // PumpSwap AMM (for migrated PumpFun tokens)
    pub const PUMP_AMM_PROGRAM_ID: &str = "pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA";
    
    
    // Moonshot
    pub const MOONSHOT_PROGRAM_ID: &str = "MoonCVVNZFSYkqNXP6bxHLPL6QQJiMagDL3qcqUQTrG";
    
    // Raydium CPMM
    pub const RAYDIUM_CPMM_PROGRAM_ID: &str = "CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C";
    
    // Raydium Launchpad
    pub const RAYDIUM_LAUNCHPAD_PROGRAM_ID: &str = "LanMV9sAd7wArD4vJFi2qDdfnVhFxYSUg6eADduJ3uj";
    
    // Meteora DLMM
    pub const METEORA_DLMM_PROGRAM_ID: &str = "LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo";
    
    // Mercurial Dynamic AMM (part of Meteora ecosystem)
    pub const MERCURIAL_DYNAMIC_AMM_PROGRAM_ID: &str = "Eo7WjKq67rjJQSZxS6z3YkapzY3eMj6Xy8X5EQVn5UaB";
    
    /// Identify DEX type by program ID
    pub fn identify_dex_by_program_id(program_id: &Pubkey) -> Option<crate::strategy::DexKind> {
        let program_id_str = program_id.to_string();
        
        match program_id_str.as_str() {
            PUMPFUN_PROGRAM_ID => Some(crate::strategy::DexKind::Pumpfun),
            PUMP_AMM_PROGRAM_ID => Some(crate::strategy::DexKind::PumpSwap),
            MOONSHOT_PROGRAM_ID => Some(crate::strategy::DexKind::Moonshot),
            RAYDIUM_CPMM_PROGRAM_ID => Some(crate::strategy::DexKind::Raydium),
            RAYDIUM_LAUNCHPAD_PROGRAM_ID => Some(crate::strategy::DexKind::RaydiumLaunchpad),
            METEORA_DLMM_PROGRAM_ID => Some(crate::strategy::DexKind::Meteora),
            MERCURIAL_DYNAMIC_AMM_PROGRAM_ID => Some(crate::strategy::DexKind::Meteora),
            _ => None,
        }
    }
    
    /// Get all known program IDs as Pubkeys for validation
    pub fn get_all_program_ids() -> Vec<Pubkey> {
        vec![
            Pubkey::from_str(PUMPFUN_PROGRAM_ID).unwrap(),
            Pubkey::from_str(PUMP_AMM_PROGRAM_ID).unwrap(),
            Pubkey::from_str(MOONSHOT_PROGRAM_ID).unwrap(),
            Pubkey::from_str(RAYDIUM_CPMM_PROGRAM_ID).unwrap(),
            Pubkey::from_str(RAYDIUM_LAUNCHPAD_PROGRAM_ID).unwrap(),
            Pubkey::from_str(METEORA_DLMM_PROGRAM_ID).unwrap(),
            Pubkey::from_str(MERCURIAL_DYNAMIC_AMM_PROGRAM_ID).unwrap(),
        ]
    }
}

/// Smart DEX Router - routes transactions based on program ID detection
pub struct DexRouter;

impl DexRouter {
    /// Identify DEX type from a list of program IDs (from Geyser transaction data)
    pub fn identify_dex_by_program_ids(program_ids: &[Pubkey]) -> Option<DexKind> {
        // Check each program ID against known DEX program IDs
        for program_id in program_ids {
            if let Some(dex_kind) = program_ids::identify_dex_by_program_id(program_id) {
                return Some(dex_kind);
            }
        }
        
        None
    }
    
    /// Route transaction to the appropriate DEX based on detected DEX type
    pub async fn route_transaction(
        settings: &Settings,
        mint: &Pubkey,
        side: Side,
        amount: u64,
        detected_dex: Option<DexKind>,
    ) -> Result<(solana_sdk::transaction::VersionedTransaction, u64)> {
        let dex_kind = detected_dex.unwrap_or_else(|| {
            println!("⚠️ [DEX_ROUTER] No DEX detected, falling back to PumpFun");
            DexKind::Pumpfun
        });
        
        
        match (dex_kind, side) {
            // PumpFun transactions
            (DexKind::Pumpfun, Side::Buy) => {
                pumpfun_simplified::fetch_pumpfun_swap_tx(settings, mint, amount).await
            }
            (DexKind::Pumpfun, Side::Sell) => {
                // FIXED: Use the sell function that accepts token amounts directly
                let tx = pumpfun_simplified::fetch_pumpfun_swap_tx_sell_with_amount(
                    settings, 
                    mint, 
                    1.0, // Always sell 100% of the specified amount
                    Some(amount) // Pass the exact token amount
                ).await?;
                Ok((tx, 0)) // Sell doesn't return token amount
            }
            
            // PumpSwap AMM transactions (migrated PumpFun tokens)
            (DexKind::PumpSwap, Side::Buy) => {
                // CRITICAL FIX: Use ONLY cached data - NO RPC calls during transaction building
                println!("🚀 [ROUTER] Building PumpSwap BUY transaction using cached data only");
                
                let (pool_pda, _) = crate::dex::pump_amm::derive_canonical_pump_pool(mint);
                let (creator, _) = crate::dex::pump_amm::derive_pump_pool_authority(mint);
                
                // CRITICAL FIX: Get coin_creator from cache ONLY - no RPC calls
                let coin_creator = if let Some(cached_creator) = crate::utils::pool_tracker::get_pump_swap_coin_creator(mint) {
                    cached_creator
                } else if let Some(constants) = crate::utils::pool_tracker::get_pumpfun_constants(mint) {
                    constants.creator
                } else {
                    println!("❌ [ROUTER] No cached coin_creator found for PumpSwap buy: {}", mint);
                    return Err(anyhow::anyhow!("No cached coin_creator found for PumpSwap buy: {}. This should be cached from Geyser.", mint));
                };
                
                let pool_data = crate::dex::pump_amm::Pool {
                    pool_bump: 255,
                    index: crate::dex::pump_amm::CANONICAL_POOL_INDEX,
                    creator,
                    base_mint: *mint,
                    quote_mint: crate::dex::pump_amm::WSOL_MINT,
                    lp_mint: Pubkey::default(),
                    pool_base_token_account: spl_associated_token_account::get_associated_token_address(&pool_pda, mint),
                    pool_quote_token_account: spl_associated_token_account::get_associated_token_address(&pool_pda, &crate::dex::pump_amm::WSOL_MINT),
                    lp_supply: 0,
                    coin_creator,
                };
                
                pumpfun_simplified::fetch_pump_amm_swap_tx(settings, mint, amount, &pool_pda, &creator, &pool_data).await
            }
            (DexKind::PumpSwap, Side::Sell) => {
                // CRITICAL FIX: Use ONLY cached data - NO RPC calls during transaction building
                println!("🚀 [ROUTER] Building PumpSwap SELL transaction using cached data only");
                
                let (pool_pda, _) = crate::dex::pump_amm::derive_canonical_pump_pool(mint);
                let (creator, _) = crate::dex::pump_amm::derive_pump_pool_authority(mint);
                
                // CRITICAL FIX: Get coin_creator from cache ONLY - no RPC calls
                let coin_creator = if let Some(cached_creator) = crate::utils::pool_tracker::get_pump_swap_coin_creator(mint) {
                    cached_creator
                } else if let Some(constants) = crate::utils::pool_tracker::get_pumpfun_constants(mint) {
                    constants.creator
                } else {
                    println!("❌ [ROUTER] No cached coin_creator found for PumpSwap sell: {}", mint);
                    return Err(anyhow::anyhow!("No cached coin_creator found for PumpSwap sell: {}. This should be cached from Geyser.", mint));
                };
                
                let pool_data = crate::dex::pump_amm::Pool {
                    pool_bump: 255,
                    index: crate::dex::pump_amm::CANONICAL_POOL_INDEX,
                    creator,
                    base_mint: *mint,
                    quote_mint: crate::dex::pump_amm::WSOL_MINT,
                    lp_mint: Pubkey::default(),
                    pool_base_token_account: spl_associated_token_account::get_associated_token_address(&pool_pda, mint),
                    pool_quote_token_account: spl_associated_token_account::get_associated_token_address(&pool_pda, &crate::dex::pump_amm::WSOL_MINT),
                    lp_supply: 0,
                    coin_creator,
                };
                
                let tx = pumpfun_simplified::fetch_pump_amm_sell_tx(
                    settings, 
                    mint, 
                    amount, // Pass amount directly like Moonshot
                    &pool_pda,
                    &creator,
                    &pool_data
                ).await?;
                Ok((tx, 0))
            }
            
            // Moonshot transactions
            (DexKind::Moonshot, Side::Buy) => {
                let moonshot_dex = MoonshotDex::new()?;
                let tx = moonshot_dex.build_buy_transaction(settings, mint, amount).await?;
                Ok((tx, amount)) // Return estimated amount
            }
            (DexKind::Moonshot, Side::Sell) => {
                let moonshot_dex = MoonshotDex::new()?;
                let tx = moonshot_dex.build_sell_transaction(settings, mint, amount).await?;
                Ok((tx, 0)) // Sell doesn't return token amount
            }
            
            // Raydium CPMM transactions
            (DexKind::Raydium, Side::Buy) => {
                let raydium_dex = RaydiumDex::new()?;
                let tx = raydium_dex.build_buy_transaction(settings, mint, amount).await?;
                Ok((tx, amount)) // Return estimated amount
            }
            (DexKind::Raydium, Side::Sell) => {
                let raydium_dex = RaydiumDex::new()?;
                let tx = raydium_dex.build_sell_transaction(settings, mint, amount).await?;
                Ok((tx, 0)) // Sell doesn't return token amount
            }
            
            // Raydium Launchpad transactions
            (DexKind::RaydiumLaunchpad, Side::Buy) => {
                // For Raydium Launchpad buys, amount is already in lamports
                let tx = raydium_launchpad::build_buy_transaction(settings, mint, amount).await?;
                Ok((tx, amount)) // Return estimated amount
            }
            (DexKind::RaydiumLaunchpad, Side::Sell) => {
                // For Raydium Launchpad sells, amount represents token amount to sell
                let tx = raydium_launchpad::build_sell_transaction(settings, mint, amount).await?;
                Ok((tx, 0)) // Sell doesn't return token amount
            }
            
            // Meteora transactions (Mercurial Dynamic AMM - detected by program ID)
            (DexKind::Meteora, Side::Buy) => {
                println!("🌊 [ROUTER] Meteora detected by program ID - routing to meteora.rs");
                let meteora_swap = crate::dex::meteora::MeteoraSwap::new_mercurial()?;
                let tx = meteora_swap.build_buy_transaction(settings, mint, amount).await?;
                Ok((tx, amount))
            }
            (DexKind::Meteora, Side::Sell) => {
                println!("🌊 [ROUTER] Meteora detected by program ID - routing to meteora.rs");
                let meteora_swap = crate::dex::meteora::MeteoraSwap::new_mercurial()?;
                let tx = meteora_swap.build_sell_transaction(settings, mint, amount).await?;
                Ok((tx, 0))
            }
        }
    }
    
    /// Validate that a program ID is from a known DEX
    pub fn is_known_dex_program_id(program_id: &Pubkey) -> bool {
        program_ids::identify_dex_by_program_id(program_id).is_some()
    }
    
    /// Get human-readable name for a program ID
    pub fn get_dex_name_by_program_id(program_id: &Pubkey) -> Option<&'static str> {
        let program_id_str = program_id.to_string();
        
        match program_id_str.as_str() {
            program_ids::PUMPFUN_PROGRAM_ID => Some("PumpFun"),
            program_ids::PUMP_AMM_PROGRAM_ID => Some("PumpSwap AMM"),
            program_ids::MOONSHOT_PROGRAM_ID => Some("Moonshot"),
            program_ids::RAYDIUM_CPMM_PROGRAM_ID => Some("Raydium CPMM"),
            program_ids::RAYDIUM_LAUNCHPAD_PROGRAM_ID => Some("Raydium Launchpad"),
            program_ids::METEORA_DLMM_PROGRAM_ID => Some("Meteora DLMM"),
            program_ids::MERCURIAL_DYNAMIC_AMM_PROGRAM_ID => Some("Mercurial Dynamic AMM"),
            _ => None,
        }
    }
}

/// Convert DexKind to human-readable string
fn dex_kind_to_string(dex_kind: &DexKind) -> &'static str {
    match dex_kind {
        DexKind::Pumpfun => "PumpFun",
        DexKind::PumpSwap => "PumpSwap AMM",
        DexKind::Moonshot => "Moonshot",
        DexKind::Raydium => "Raydium CPMM",
        DexKind::RaydiumLaunchpad => "Raydium Launchpad",
        DexKind::Meteora => "Meteora",
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_program_id_identification() {
        // Test PumpFun
        let pumpfun_id = Pubkey::from_str(program_ids::PUMPFUN_PROGRAM_ID).unwrap();
        assert_eq!(
            program_ids::identify_dex_by_program_id(&pumpfun_id),
            Some(DexKind::Pumpfun)
        );
        
        // Test Moonshot
        let moonshot_id = Pubkey::from_str(program_ids::MOONSHOT_PROGRAM_ID).unwrap();
        assert_eq!(
            program_ids::identify_dex_by_program_id(&moonshot_id),
            Some(DexKind::Moonshot)
        );
        
        // Test Raydium Launchpad
        let raydium_launchpad_id = Pubkey::from_str(program_ids::RAYDIUM_LAUNCHPAD_PROGRAM_ID).unwrap();
        assert_eq!(
            program_ids::identify_dex_by_program_id(&raydium_launchpad_id),
            Some(DexKind::RaydiumLaunchpad)
        );
        
        // Test unknown program ID
        let unknown_id = Pubkey::new_unique();
        assert_eq!(
            program_ids::identify_dex_by_program_id(&unknown_id),
            None
        );
    }
    
    #[test]
    fn test_router_identification() {
        let program_ids = vec![
            Pubkey::from_str(program_ids::RAYDIUM_LAUNCHPAD_PROGRAM_ID).unwrap(),
            Pubkey::new_unique(), // Unknown program ID
        ];
        
        let detected_dex = DexRouter::identify_dex_by_program_ids(&program_ids);
        assert_eq!(detected_dex, Some(DexKind::RaydiumLaunchpad));
    }
    
    #[test]
    fn test_no_matching_dex() {
        let program_ids = vec![
            Pubkey::new_unique(),
            Pubkey::new_unique(),
        ];
        
        let detected_dex = DexRouter::identify_dex_by_program_ids(&program_ids);
        assert_eq!(detected_dex, None);
    }
    
    #[test]
    fn test_get_all_program_ids() {
        let all_ids = program_ids::get_all_program_ids();
        assert_eq!(all_ids.len(), 7); // Should have 7 known DEX program IDs
        
        // Verify each ID is valid
        for id in all_ids {
            assert!(DexRouter::is_known_dex_program_id(&id));
        }
    }
    
    #[test]
    fn test_dex_name_lookup() {
        let pumpfun_id = Pubkey::from_str(program_ids::PUMPFUN_PROGRAM_ID).unwrap();
        assert_eq!(
            DexRouter::get_dex_name_by_program_id(&pumpfun_id),
            Some("PumpFun")
        );
        
        let unknown_id = Pubkey::new_unique();
        assert_eq!(
            DexRouter::get_dex_name_by_program_id(&unknown_id),
            None
        );
    }
}
