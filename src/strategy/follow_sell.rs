use super::*;
use crate::config::settings::Settings;

pub struct FollowSell;

impl Strategy for FollowSell {
    fn on_fill(&mut self, f: &ObservedFill, _settings: &Settings) -> Vec<TradePlan> {
        // Only process SELL events
        if f.side != Side::Sell {
            return Vec::new();
        }
        
        // Debug the percentage values
        
        let pct = if f.pct_of_balance >= 0.90 {
            println!("✅ [FOLLOW_SELL] Tracked wallet sold >90% → Bot will sell 100%");
            1.0
        } else {
            println!("✅ [FOLLOW_SELL] Tracked wallet sold {:.2}% → Bot will sell {:.2}%", 
                f.pct_of_balance * 100.0, f.pct_of_balance * 100.0);
            f.pct_of_balance
        };
        
        // Create appropriate TradePlan based on DEX type
        match f.dex {
            DexKind::Pumpfun => {
                vec![TradePlan::sell_pumpfun_percent(f.mint, pct)]
            }
            DexKind::PumpSwap => {
                println!("🔄 [FOLLOW_SELL] Creating PumpSwap sell plan for {:.2}%", pct * 100.0);
                vec![TradePlan::sell_pumpswap_percent(f.mint, pct)]
            }
            DexKind::Moonshot => {
                println!("🌙 [FOLLOW_SELL] Creating Moonshot sell plan for {:.2}%", pct * 100.0);
                vec![TradePlan::sell_moonshot_percent(f.mint, pct)]
            }
            DexKind::Raydium => {
                println!("⚡ [FOLLOW_SELL] Creating Raydium sell plan for {:.2}%", pct * 100.0);
                vec![TradePlan::sell_raydium_percent(f.mint, pct)]
            }
            DexKind::Meteora => {
                println!("⚡ [FOLLOW_SELL] Creating Meteora sell plan for {:.2}%", pct * 100.0);
                vec![TradePlan::sell_meteora_percent(f.mint, pct)]
            }
            DexKind::RaydiumLaunchpad => {
                println!("🚀 [FOLLOW_SELL] Creating Raydium Launchpad sell plan for {:.2}%", pct * 100.0);
                vec![TradePlan::sell_raydium_launchpad_percent(f.mint, pct)]
            }
        }
    }
}
