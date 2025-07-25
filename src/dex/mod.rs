/* --------------------------------------------------------------------- */
/*  DEX execution router                                                 */
/* --------------------------------------------------------------------- */

pub mod moonshot;
pub mod pumpfun_simplified;
pub use pumpfun_simplified as pumpfun;
pub mod pumpfun_math;
pub mod pump_amm;
pub mod raydium;
pub mod raydium_launchpad;
pub mod meteora;
pub mod router; // <-- NEW: Smart DEX router
pub mod types; // <--  NEW  (exports `PoolItem` etc.)

use anyhow::{anyhow, Result};
use solana_sdk::transaction::VersionedTransaction;

use crate::{
    config::settings::Settings,
    strategy::{DexKind, Side, TradePlan},
};

/// Convert a high‑level [`TradePlan`] into a signed [`VersionedTransaction`].
/// For BUY operations, returns both the transaction and the calculated token amount.
/// Includes intelligent fallback for zero-RPC assumptions.
pub async fn build_tx_from_plan(
    settings: &Settings,
    plan: &TradePlan,
) -> Result<(VersionedTransaction, Option<u64>)> {
    match plan.dex {
        DexKind::Pumpfun => match plan.side {
            Side::Buy => {
                // Try PumpFun first (zero-RPC assumption)
                match pumpfun::fetch_pumpfun_swap_tx(settings, &plan.mint, plan.buy_lamports).await {
                    Ok((tx, token_amount)) => {
                        println!("✅ [FALLBACK] PumpFun assumption was correct for {}", plan.mint);
                        Ok((tx, Some(token_amount)))
                    }
                    Err(e) => {
                        println!("⚠️ [FALLBACK] PumpFun failed for {} ({}), detecting actual DEX...", plan.mint, e);
                        
                        // Fallback: Detect actual DEX and retry (only on failure)
                        let actual_dex = crate::utils::pool_tracker::detect_dex_type(&settings.rpc_client, &plan.mint).await;
                        
                        match actual_dex {
                            crate::utils::pool_tracker::DexType::Moonshot => {
                                println!("🌙 [FALLBACK] Retrying as Moonshot for {}", plan.mint);
                                let moonshot_dex = moonshot::MoonshotDex::new()?;
                                let tx = moonshot_dex.build_buy_transaction(settings, &plan.mint, plan.buy_lamports).await?;
                                Ok((tx, None))
                            }
                            _ => {
                                println!("❌ [FALLBACK] Token {} requires local RPC submission (migrated)", plan.mint);
                                Err(anyhow!("Token has migrated - use local RPC submitter instead"))
                            }
                        }
                    }
                }
            }
            Side::Sell => {
                let pct = plan
                    .sell_pct
                    .ok_or_else(|| anyhow!("TradePlan for SELL is missing `sell_pct`"))?;

                if pct <= 0.0 {
                    return Err(anyhow!("Sell percent must be > 0.0"));
                }

                // Try PumpFun first (zero-RPC assumption)
                match pumpfun::fetch_pumpfun_swap_tx_sell_with_amount(settings, &plan.mint, pct, plan.known_token_amount).await {
                    Ok(tx) => {
                        println!("✅ [FALLBACK] PumpFun assumption was correct for {}", plan.mint);
                        Ok((tx, None))
                    }
                    Err(e) => {
                        println!("⚠️ [FALLBACK] PumpFun failed for {} ({}), detecting actual DEX...", plan.mint, e);
                        
                        // Fallback: Detect actual DEX and retry (only on failure)
                        let actual_dex = crate::utils::pool_tracker::detect_dex_type(&settings.rpc_client, &plan.mint).await;
                        
                        match actual_dex {
                            crate::utils::pool_tracker::DexType::Moonshot => {
                                println!("🌙 [FALLBACK] Retrying as Moonshot for {}", plan.mint);
                                let moonshot_dex = moonshot::MoonshotDex::new()?;
                                
                                let token_amount = if let Some(known_amount) = plan.known_token_amount {
                                    (known_amount as f64 * pct) as u64
                                } else {
                                    return Err(anyhow!("Moonshot SELL requires known_token_amount"));
                                };
                                
                                let tx = moonshot_dex.build_sell_transaction(settings, &plan.mint, token_amount).await?;
                                Ok((tx, None))
                            }
                            _ => {
                                println!("❌ [FALLBACK] Token {} requires local RPC submission (migrated)", plan.mint);
                                Err(anyhow!("Token has migrated - use local RPC submitter instead"))
                            }
                        }
                    }
                }
            }
        },
        DexKind::Moonshot => {
            let moonshot_dex = moonshot::MoonshotDex::new()?;
            
            match plan.side {
                Side::Buy => {
                    let tx = moonshot_dex.build_buy_transaction(settings, &plan.mint, plan.buy_lamports).await?;
                    // For Moonshot, we don't have a reliable way to predict token amount beforehand
                    // The actual amount will be determined by the curve at execution time
                    Ok((tx, None))
                }
                Side::Sell => {
                    let pct = plan.sell_pct.ok_or_else(|| anyhow!("TradePlan for SELL is missing `sell_pct`"))?;
                    
                    if pct <= 0.0 {
                        return Err(anyhow!("Sell percent must be > 0.0"));
                    }
                    
                    // Calculate token amount to sell
                    let token_amount = if let Some(known_amount) = plan.known_token_amount {
                        (known_amount as f64 * pct) as u64
                    } else {
                        return Err(anyhow!("Moonshot SELL requires known_token_amount"));
                    };
                    
                    let tx = moonshot_dex.build_sell_transaction(settings, &plan.mint, token_amount).await?;
                    Ok((tx, None))
                }
            }
        },
        DexKind::PumpSwap => {
            // For PumpSwap (migrated PumpFun tokens), we should use local RPC submission
            // instead of trying to find pools via RPC calls
            return Err(anyhow!("PumpSwap transactions should be handled directly via local RPC submitter, not through build_tx_from_plan"));
        }
        DexKind::Raydium => {
            // For migrated PumpFun tokens on Raydium, we should use local RPC submission
            // instead of trying to find pools via RPC calls
            return Err(anyhow!("Raydium transactions should be handled directly via local RPC submitter, not through build_tx_from_plan"));
        }
        DexKind::Meteora => {
            // For Meteora DLMM, we should use local RPC submission
            // instead of trying to find pools via RPC calls
            return Err(anyhow!("Meteora transactions should be handled directly via local RPC submitter, not through build_tx_from_plan"));
        }
        DexKind::RaydiumLaunchpad => {
            // Raydium Launchpad uses build_tx_from_plan like PumpFun/Moonshot
            match plan.side {
                Side::Buy => {
                    let tx = raydium_launchpad::build_buy_transaction(settings, &plan.mint, plan.buy_lamports).await?;
                    Ok((tx, None))
                }
                Side::Sell => {
                    let pct = plan.sell_pct.ok_or_else(|| anyhow!("TradePlan for SELL is missing `sell_pct`"))?;
                    
                    if pct <= 0.0 {
                        return Err(anyhow!("Sell percent must be > 0.0"));
                    }
                    
                    let token_amount = if let Some(known_amount) = plan.known_token_amount {
                        (known_amount as f64 * pct) as u64
                    } else {
                        return Err(anyhow!("Raydium Launchpad SELL requires known_token_amount"));
                    };
                    
                    let tx = raydium_launchpad::build_sell_transaction(settings, &plan.mint, token_amount).await?;
                    Ok((tx, None))
                }
            }
        }
    }
}
