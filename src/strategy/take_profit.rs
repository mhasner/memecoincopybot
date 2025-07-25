use super::*;
use crate::config::settings::Settings;
use crate::strategy::engine::STRATEGY_ENGINE;

pub struct TakeProfit;

impl Strategy for TakeProfit {
    fn on_fill(&mut self, f: &ObservedFill, settings: &Settings) -> Vec<TradePlan> {
        // CRITICAL FIX: Only check take-profit on OTHER people's fills, not our own BUYs
        // Take-profit should trigger when we see market activity (other traders), 
        // not immediately after our own BUY
        
        // For now, only trigger take-profit on tracked wallet SELL events
        // This prevents immediate selling after our own BUY
        if f.side != Side::Sell {
            return Vec::new();
        }

        // Update price based on market activity and check for take-profit
        if let Some(engine) = STRATEGY_ENGINE.get() {
            let pm = engine.positions.lock().unwrap();
            
            // Check if we have a position in this mint
            let current_balance = pm.balance(f.mint);
            if current_balance == 0 {
                return Vec::new(); // No position, no take-profit
            }

            // Check unrealized PnL
            if let Some(pnl) = pm.unrealised_pct(f.mint) {
                println!("🎯 [TAKE_PROFIT] Checking PnL for {}: {:.2}% (threshold: {:.2}%)", 
                    f.mint, pnl, settings.take_profit_percent);
                
                if pnl >= settings.take_profit_percent {
                    println!("💰 [TAKE_PROFIT] Triggering take-profit: {:.2}% profit >= {:.2}% threshold", 
                        pnl, settings.take_profit_percent);
                    
                    // Create appropriate sell plan based on the DEX where we saw activity
                    let sell_plan = match f.dex {
                        DexKind::Pumpfun => TradePlan::sell_pumpfun_percent(
                            f.mint,
                            settings.take_profit_sell_fraction,
                        ),
                        DexKind::PumpSwap => TradePlan::sell_pumpswap_percent(
                            f.mint,
                            settings.take_profit_sell_fraction,
                        ),
                        DexKind::Moonshot => TradePlan::sell_moonshot_percent(
                            f.mint,
                            settings.take_profit_sell_fraction,
                        ),
                        DexKind::Raydium => TradePlan::sell_raydium_percent(
                            f.mint,
                            settings.take_profit_sell_fraction,
                        ),
                        DexKind::Meteora => TradePlan::sell_meteora_percent(
                            f.mint,
                            settings.take_profit_sell_fraction,
                        ),
                        DexKind::RaydiumLaunchpad => TradePlan::sell_raydium_launchpad_percent(
                            f.mint,
                            settings.take_profit_percent,
                        ),
                    };
                    
                    return vec![sell_plan];
                }
            }
        }
        Vec::new()
    }
}
