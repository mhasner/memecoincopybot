// ---------- File replaces: src/strategy/mod.rs ----------
//! Thin strategy layer – now with an engine dispatcher.

use solana_sdk::pubkey::Pubkey;

pub mod engine;
pub mod follow_buy;
pub mod follow_sell;
pub mod take_profit;

use crate::config::settings::Settings;

// pub mod take_profit;   // keep as soon as the file exists

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DexKind {
    Pumpfun,
    PumpSwap,  // Migrated PumpFun tokens using PumpSwap AMM
    Moonshot,
    Raydium,
    Meteora,   // Meteora DLMM
    RaydiumLaunchpad, // Raydium Launchpad (BONK launchpad)
}

/// Plan produced by a strategy and later converted into a signed
/// transaction by the DEX router.
#[derive(Clone, Debug)]
pub struct TradePlan {
    pub dex: DexKind,
    pub side: Side,
    pub mint: Pubkey,
    pub buy_lamports: u64,     // BUY only
    pub sell_pct: Option<f64>, // SELL only (0.0 – 1.0)
    pub known_token_amount: Option<u64>, // SELL only - skip ATA polling if provided
    pub calculated_token_amount: Option<u64>, // BUY only - actual min_out from calculation
}

impl TradePlan {
    /// Helper for Pumpfun BUY
    pub fn buy_pumpfun(mint: Pubkey, lamports: u64) -> Self {
        Self {
            dex: DexKind::Pumpfun,
            side: Side::Buy,
            mint,
            buy_lamports: lamports,
            sell_pct: None,
            known_token_amount: None,
            calculated_token_amount: None,
        }
    }

    /// Helper for Pumpfun SELL by %
    pub fn sell_pumpfun_percent(mint: Pubkey, pct: f64) -> Self {
        Self {
            dex: DexKind::Pumpfun,
            side: Side::Sell,
            mint,
            buy_lamports: 0,
            sell_pct: Some(pct),
            known_token_amount: None,
            calculated_token_amount: None,
        }
    }

    /// Helper for Pumpfun SELL by % with known token amount (skip ATA polling)
    pub fn sell_pumpfun_percent_with_amount(mint: Pubkey, pct: f64, token_amount: u64) -> Self {
        Self {
            dex: DexKind::Pumpfun,
            side: Side::Sell,
            mint,
            buy_lamports: 0,
            sell_pct: Some(pct),
            known_token_amount: Some(token_amount),
            calculated_token_amount: None,
        }
    }

    /// Helper for PumpSwap BUY
    pub fn buy_pumpswap(mint: Pubkey, lamports: u64) -> Self {
        Self {
            dex: DexKind::PumpSwap,
            side: Side::Buy,
            mint,
            buy_lamports: lamports,
            sell_pct: None,
            known_token_amount: None,
            calculated_token_amount: None,
        }
    }

    /// Helper for PumpSwap SELL by %
    pub fn sell_pumpswap_percent(mint: Pubkey, pct: f64) -> Self {
        Self {
            dex: DexKind::PumpSwap,
            side: Side::Sell,
            mint,
            buy_lamports: 0,
            sell_pct: Some(pct),
            known_token_amount: None,
            calculated_token_amount: None,
        }
    }

    /// Helper for PumpSwap SELL by % with known token amount (skip ATA polling)
    pub fn sell_pumpswap_percent_with_amount(mint: Pubkey, pct: f64, token_amount: u64) -> Self {
        Self {
            dex: DexKind::PumpSwap,
            side: Side::Sell,
            mint,
            buy_lamports: 0,
            sell_pct: Some(pct),
            known_token_amount: Some(token_amount),
            calculated_token_amount: None,
        }
    }

    /// Helper for Moonshot BUY
    pub fn buy_moonshot(mint: Pubkey, lamports: u64) -> Self {
        Self {
            dex: DexKind::Moonshot,
            side: Side::Buy,
            mint,
            buy_lamports: lamports,
            sell_pct: None,
            known_token_amount: None,
            calculated_token_amount: None,
        }
    }

    /// Helper for Moonshot SELL by %
    pub fn sell_moonshot_percent(mint: Pubkey, pct: f64) -> Self {
        Self {
            dex: DexKind::Moonshot,
            side: Side::Sell,
            mint,
            buy_lamports: 0,
            sell_pct: Some(pct),
            known_token_amount: None,
            calculated_token_amount: None,
        }
    }

    /// Helper for Moonshot SELL by % with known token amount (skip ATA polling)
    pub fn sell_moonshot_percent_with_amount(mint: Pubkey, pct: f64, token_amount: u64) -> Self {
        Self {
            dex: DexKind::Moonshot,
            side: Side::Sell,
            mint,
            buy_lamports: 0,
            sell_pct: Some(pct),
            known_token_amount: Some(token_amount),
            calculated_token_amount: None,
        }
    }

    /// Helper for Raydium BUY
    pub fn buy_raydium(mint: Pubkey, lamports: u64) -> Self {
        Self {
            dex: DexKind::Raydium,
            side: Side::Buy,
            mint,
            buy_lamports: lamports,
            sell_pct: None,
            known_token_amount: None,
            calculated_token_amount: None,
        }
    }

    /// Helper for Raydium SELL by %
    pub fn sell_raydium_percent(mint: Pubkey, pct: f64) -> Self {
        Self {
            dex: DexKind::Raydium,
            side: Side::Sell,
            mint,
            buy_lamports: 0,
            sell_pct: Some(pct),
            known_token_amount: None,
            calculated_token_amount: None,
        }
    }

    /// Helper for Raydium SELL by % with known token amount (skip ATA polling)
    pub fn sell_raydium_percent_with_amount(mint: Pubkey, pct: f64, token_amount: u64) -> Self {
        Self {
            dex: DexKind::Raydium,
            side: Side::Sell,
            mint,
            buy_lamports: 0,
            sell_pct: Some(pct),
            known_token_amount: Some(token_amount),
            calculated_token_amount: None,
        }
    }

    /// Helper for Meteora BUY
    pub fn buy_meteora(mint: Pubkey, lamports: u64) -> Self {
        Self {
            dex: DexKind::Meteora,
            side: Side::Buy,
            mint,
            buy_lamports: lamports,
            sell_pct: None,
            known_token_amount: None,
            calculated_token_amount: None,
        }
    }

    /// Helper for Meteora SELL by %
    pub fn sell_meteora_percent(mint: Pubkey, pct: f64) -> Self {
        Self {
            dex: DexKind::Meteora,
            side: Side::Sell,
            mint,
            buy_lamports: 0,
            sell_pct: Some(pct),
            known_token_amount: None,
            calculated_token_amount: None,
        }
    }

    /// Helper for Meteora SELL by % with known token amount (skip ATA polling)
    pub fn sell_meteora_percent_with_amount(mint: Pubkey, pct: f64, token_amount: u64) -> Self {
        Self {
            dex: DexKind::Meteora,
            side: Side::Sell,
            mint,
            buy_lamports: 0,
            sell_pct: Some(pct),
            known_token_amount: Some(token_amount),
            calculated_token_amount: None,
        }
    }

    /// Helper for Raydium Launchpad BUY
    pub fn buy_raydium_launchpad(mint: Pubkey, lamports: u64) -> Self {
        Self {
            dex: DexKind::RaydiumLaunchpad,
            side: Side::Buy,
            mint,
            buy_lamports: lamports,
            sell_pct: None,
            known_token_amount: None,
            calculated_token_amount: None,
        }
    }

    /// Helper for Raydium Launchpad SELL by %
    pub fn sell_raydium_launchpad_percent(mint: Pubkey, pct: f64) -> Self {
        Self {
            dex: DexKind::RaydiumLaunchpad,
            side: Side::Sell,
            mint,
            buy_lamports: 0,
            sell_pct: Some(pct),
            known_token_amount: None,
            calculated_token_amount: None,
        }
    }

    /// Helper for Raydium Launchpad SELL by % with known token amount (skip ATA polling)
    pub fn sell_raydium_launchpad_percent_with_amount(mint: Pubkey, pct: f64, token_amount: u64) -> Self {
        Self {
            dex: DexKind::RaydiumLaunchpad,
            side: Side::Sell,
            mint,
            buy_lamports: 0,
            sell_pct: Some(pct),
            known_token_amount: Some(token_amount),
            calculated_token_amount: None,
        }
    }
}

/// What we observe on‑chain and feed into [`Strategy::on_fill`].
#[derive(Clone, Debug)]
pub struct ObservedFill {
    pub mint: Pubkey,
    pub side: Side,
    pub cost_lamports: u64,
    pub pct_of_balance: f64,
    pub dex: DexKind,
    pub wallet_label: String, // Human-readable wallet label
}

pub trait Strategy: Send {
    fn on_fill(&mut self, fill: &ObservedFill, settings: &Settings) -> Vec<TradePlan>;
}
