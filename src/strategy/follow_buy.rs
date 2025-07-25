use super::*;
use crate::config::settings::Settings;

pub struct FollowBuy;

impl Strategy for FollowBuy {
    fn on_fill(&mut self, f: &ObservedFill, settings: &Settings) -> Vec<TradePlan> {
        // Only follow buy transactions
        if f.side != Side::Buy {
            return Vec::new();
        }

        // Find the wallet configuration for this specific wallet
        // The wallet_label in ObservedFill contains the human-readable label from geyser_listener
        let wallet_config = settings.tracked_wallets
            .iter()
            .find(|w| w.label == f.wallet_label);

        let wallet_config = match wallet_config {
            Some(config) => config,
            None => {
                // If wallet not found in config, skip this trade
                return Vec::new();
            }
        };

        // Use per-wallet SOL gate - only copy buys > wallet's sol_gate
        let gate_lamports = settings
            .sol_to_lamports(wallet_config.sol_gate)
            .unwrap_or(u64::MAX);
            
        if f.cost_lamports < gate_lamports {
            return Vec::new();
        }

        // Use per-wallet buy amount
        let lamports = settings
            .sol_to_lamports(wallet_config.buy_amount_sol)
            .unwrap_or_else(|_| 0);

        // Create appropriate trade plan based on the DEX
        match f.dex {
            DexKind::Pumpfun => vec![TradePlan::buy_pumpfun(f.mint, lamports)],
            DexKind::PumpSwap => vec![TradePlan::buy_pumpswap(f.mint, lamports)],
            DexKind::Moonshot => vec![TradePlan::buy_moonshot(f.mint, lamports)],
            DexKind::Raydium => vec![TradePlan::buy_raydium(f.mint, lamports)],
            DexKind::Meteora => vec![TradePlan::buy_meteora(f.mint, lamports)],
            DexKind::RaydiumLaunchpad => vec![TradePlan::buy_raydium_launchpad(f.mint, lamports)],
        }
    }
}
