//! Runtime configuration loader and common helpers.

use std::{fmt, fs, path::Path, path::PathBuf, sync::Arc};

use anyhow::{Context, Result};
use bs58;
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    native_token::LAMPORTS_PER_SOL,
    signature::{Keypair, Signer},
};

/// ------------------------------------------------------------------
/// Wallet mappings
/// ------------------------------------------------------------------
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WalletConfig {
    pub label: String,
    pub address: String,
    pub enabled: bool,
    /// Per-wallet SOL gate - only follow if tracked wallet buys > this amount
    pub sol_gate: f64,
    /// Per-wallet buy amount - amount to buy when following this wallet
    pub buy_amount_sol: f64,
}

// Type alias for compatibility with API server
pub type TrackedWallet = WalletConfig;

#[derive(Debug, Deserialize)]
pub struct WalletKeypairEntry {
    pub name: String,
    pub address: String,
    pub private_key_base58: String,
}

/// ------------------------------------------------------------------
/// Fresh Mint Cache Configuration
/// ------------------------------------------------------------------
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FreshMintCacheConfig {
    pub enabled: bool,
    pub max_blocks_buffer: usize,
    pub max_cache_size: usize,
    pub cleanup_interval_seconds: u64,
    pub emergency_purge_threshold_mb: usize,
}

impl Default for FreshMintCacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_blocks_buffer: 5,
            max_cache_size: 10000,
            cleanup_interval_seconds: 30,
            emergency_purge_threshold_mb: 100,
        }
    }
}

/// ------------------------------------------------------------------
/// Serializable Settings for API responses
/// ------------------------------------------------------------------
#[derive(Debug, Serialize, Deserialize)]
pub struct SerializableSettings {
    pub workdir: PathBuf,
    pub rpc_url: String,
    pub fallback_rpc_url: String,
    pub relayer_url: String,
    pub geyser_url: String,
    pub geyser_token: Option<String>,
    pub wallets_file: String,
    pub active_wallet: String,
    pub jito: bool,
    pub tracked_wallets: Vec<WalletConfig>,
    pub buy_slippage_percent: f64,
    pub buy_bribe_sol: f64,
    pub buy_priority_fee_sol: f64,
    pub sell_amount_percent: f64,
    pub sell_min_sol_out: f64,
    pub sell_slippage_percent: f64,
    pub sell_bribe_sol: f64,
    pub sell_priority_fee_sol: f64,
    pub take_profit_percent: f64,
    pub take_profit_sell_fraction: f64,
    pub fresh_mint_cache: FreshMintCacheConfig,
}

/// ------------------------------------------------------------------
/// Main Settings object – *single definition only!*
/// ------------------------------------------------------------------
pub struct Settings {
    /* -------- infrastructure ------------------------ */
    pub workdir: PathBuf,
    pub rpc_url: String,
    pub fallback_rpc_url: String,
    pub relayer_url: String,
    pub geyser_url: String,
    pub geyser_token: Option<String>,

    /* -------- trading wallets ----------------------- */
    pub wallets_file: String,
    pub active_wallet: String,
    pub jito: bool,
    pub tracked_wallets: Vec<WalletConfig>,
    pub keypair: Arc<Keypair>,

    /* -------- BUY tuning ---------------------------- */
    pub buy_slippage_percent: f64,
    pub buy_bribe_sol: f64,
    pub buy_priority_fee_sol: f64,

    /* -------- SELL tuning --------------------------- */
    pub sell_amount_percent: f64,
    pub sell_min_sol_out: f64,
    pub sell_slippage_percent: f64,
    pub sell_bribe_sol: f64,
    pub sell_priority_fee_sol: f64,


    /* -------- fresh mint cache ---------------------- */
    pub fresh_mint_cache: FreshMintCacheConfig,

    /* -------- shared objects ------------------------ */
    pub rpc_client: Arc<RpcClient>,
    pub take_profit_percent: f64,
    pub take_profit_sell_fraction: f64,
}

impl Settings {
    /// --------------------------------------------------------------
    /// Read `settings.json` from disk.
    /// --------------------------------------------------------------
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let raw = fs::read_to_string(&path)
            .with_context(|| format!("reading settings file {:?}", path.as_ref()))?;
        let json: serde_json::Value = serde_json::from_str(&raw)?;

        /* -------- plain strings ---------------------------------- */
        let rpc_url = json["rpc_url"].as_str().unwrap_or_default().to_string();
        let fallback_rpc_url = json["fallback_rpc_url"]
            .as_str()
            .unwrap_or_default()
            .to_string();
        let relayer_url = json["relayer_url"].as_str().unwrap_or_default().to_string();
        let geyser_url = json["geyser_url"]
            .as_str()
            .unwrap_or("http://127.0.0.1:10000")
            .to_string();
        let geyser_token = json["geyser_token"].as_str().map(|s| s.to_string());

        let wallets_file = json["wallets_file"]
            .as_str()
            .unwrap_or("./wallets.json")
            .to_string();
        let active_wallet = json["active_wallet"]
            .as_str()
            .unwrap_or("TradingBot")
            .to_string();
        let jito = json["jito"].as_bool().unwrap_or(true); // Default to true for backward compatibility

        /* -------- numeric parameters ----------------------------- */
        let buy_slippage_percent = json["buy_slippage_percent"].as_f64().unwrap_or(0.5);
        let buy_bribe_sol = json["buy_bribe_sol"].as_f64().unwrap_or(0.0001);
        let buy_priority_fee_sol = json["buy_priority_fee_sol"].as_f64().unwrap_or(0.0001);

        let sell_amount_percent = json["sell_amount_percent"].as_f64().unwrap_or(100.0);
        let sell_min_sol_out = json["sell_min_sol_out"].as_f64().unwrap_or(0.01);
        let sell_slippage_percent = json["sell_slippage_percent"].as_f64().unwrap_or(0.5);
        let sell_bribe_sol = json["sell_bribe_sol"].as_f64().unwrap_or(0.0001);
        let sell_priority_fee_sol = json["sell_priority_fee_sol"].as_f64().unwrap_or(0.0001);
        let take_profit_percent = json["take_profit_percent"].as_f64().unwrap_or(120.0);
        let take_profit_sell_fraction = json["take_profit_sell_fraction"].as_f64().unwrap_or(0.5);


        /* -------- fresh mint cache configuration ----------------- */
        let fresh_mint_cache = if let Some(cache_config) = json.get("fresh_mint_cache") {
            serde_json::from_value(cache_config.clone())
                .unwrap_or_else(|_| FreshMintCacheConfig::default())
        } else {
            FreshMintCacheConfig::default()
        };

        /* -------- tracked wallets & main keypair ----------------- */
        let mut tracked_wallets: Vec<WalletConfig> = Vec::new();
        if let Some(wallets_array) = json["tracked_wallets"].as_array() {
            for wallet_value in wallets_array {
                let label = wallet_value["label"].as_str().unwrap_or("Unknown").to_string();
                let address = wallet_value["address"].as_str().unwrap_or("").to_string();
                // Default to enabled=true for backward compatibility
                let enabled = wallet_value["enabled"].as_bool().unwrap_or(true);
                // Per-wallet SOL gate and buy amount - required fields
                let sol_gate = wallet_value["sol_gate"].as_f64().unwrap_or(0.001);
                let buy_amount_sol = wallet_value["buy_amount_sol"].as_f64().unwrap_or(0.003);
                
                tracked_wallets.push(WalletConfig {
                    label,
                    address,
                    enabled,
                    sol_gate,
                    buy_amount_sol,
                });
            }
        }

        let wallet_map_raw = fs::read_to_string(&wallets_file)
            .with_context(|| format!("reading wallets file {}", wallets_file))?;
        let wallet_list: Vec<WalletKeypairEntry> =
            serde_json::from_str(&wallet_map_raw).context("parsing wallets file")?;

        let active_wallet_entry = wallet_list
            .iter()
            .find(|w| w.name == active_wallet)
            .ok_or_else(|| anyhow::anyhow!("active wallet `{active_wallet}` not found"))?;

        let private_key_bytes = bs58::decode(&active_wallet_entry.private_key_base58)
            .into_vec()
            .context("decoding base58 key")?;
        let keypair = Arc::new(Keypair::from_bytes(&private_key_bytes)?);

        /* -------- misc ------------------------------------------- */
        let rpc_client = Arc::new(RpcClient::new(rpc_url.clone()));
        let workdir = json["workdir"]
            .as_str()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));

        Ok(Self {
            workdir,
            rpc_url,
            fallback_rpc_url,
            relayer_url,
            geyser_url,
            geyser_token,
            wallets_file,
            active_wallet,
            jito,
            tracked_wallets,
            keypair,
            buy_slippage_percent,
            buy_bribe_sol,
            buy_priority_fee_sol,
            sell_amount_percent,
            sell_min_sol_out,
            sell_slippage_percent,
            sell_bribe_sol,
            sell_priority_fee_sol,
            fresh_mint_cache,
            rpc_client,
            take_profit_percent,
            take_profit_sell_fraction,
        })
    }

    /// --------------------------------------------------------------
    /// Helper: convert SOL → lamports and round to nearest integer.
    /// --------------------------------------------------------------
    pub fn sol_to_lamports(&self, sol: f64) -> Result<u64> {
        Ok((sol * LAMPORTS_PER_SOL as f64).round() as u64)
    }

    /// --------------------------------------------------------------
    /// Load settings from default config/settings.json file.
    /// --------------------------------------------------------------
    pub fn load() -> Result<Self> {
        Self::load_from_file("config/settings.json")
    }

    /// --------------------------------------------------------------
    /// Save settings back to config/settings.json file.
    /// --------------------------------------------------------------
    pub fn save(&self) -> Result<()> {
        self.save_to_file("config/settings.json")
    }

    /// --------------------------------------------------------------
    /// Save settings to a specific file path.
    /// --------------------------------------------------------------
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let settings_json = serde_json::json!({
            "rpc_url": self.rpc_url,
            "fallback_rpc_url": self.fallback_rpc_url,
            "relayer_url": self.relayer_url,
            "geyser_url": self.geyser_url,
            "geyser_token": self.geyser_token,
            "wallets_file": self.wallets_file,
            "active_wallet": self.active_wallet,
            "jito": self.jito,
            "tracked_wallets": self.tracked_wallets,
            "buy_slippage_percent": self.buy_slippage_percent,
            "buy_bribe_sol": self.buy_bribe_sol,
            "buy_priority_fee_sol": self.buy_priority_fee_sol,
            "sell_amount_percent": self.sell_amount_percent,
            "sell_min_sol_out": self.sell_min_sol_out,
            "sell_slippage_percent": self.sell_slippage_percent,
            "sell_bribe_sol": self.sell_bribe_sol,
            "sell_priority_fee_sol": self.sell_priority_fee_sol,
            "take_profit_percent": self.take_profit_percent,
            "take_profit_sell_fraction": self.take_profit_sell_fraction
        });

        let json_string = serde_json::to_string_pretty(&settings_json)?;
        fs::write(&path, json_string)
            .with_context(|| format!("writing settings to {:?}", path.as_ref()))?;
        
        Ok(())
    }

    /// --------------------------------------------------------------
    /// Convert to serializable format for API responses.
    /// --------------------------------------------------------------
    pub fn to_serializable(&self) -> SerializableSettings {
        SerializableSettings {
            workdir: self.workdir.clone(),
            rpc_url: self.rpc_url.clone(),
            fallback_rpc_url: self.fallback_rpc_url.clone(),
            relayer_url: self.relayer_url.clone(),
            geyser_url: self.geyser_url.clone(),
            geyser_token: self.geyser_token.clone(),
            wallets_file: self.wallets_file.clone(),
            active_wallet: self.active_wallet.clone(),
            jito: self.jito,
            tracked_wallets: self.tracked_wallets.clone(),
            buy_slippage_percent: self.buy_slippage_percent,
            buy_bribe_sol: self.buy_bribe_sol,
            buy_priority_fee_sol: self.buy_priority_fee_sol,
            sell_amount_percent: self.sell_amount_percent,
            sell_min_sol_out: self.sell_min_sol_out,
            sell_slippage_percent: self.sell_slippage_percent,
            sell_bribe_sol: self.sell_bribe_sol,
            sell_priority_fee_sol: self.sell_priority_fee_sol,
            take_profit_percent: self.take_profit_percent,
            take_profit_sell_fraction: self.take_profit_sell_fraction,
            fresh_mint_cache: self.fresh_mint_cache.clone(),
        }
    }

    /// --------------------------------------------------------------
    /// Helper: get only enabled wallets for tracking.
    /// --------------------------------------------------------------
    pub fn enabled_wallets(&self) -> Vec<&WalletConfig> {
        self.tracked_wallets.iter().filter(|w| w.enabled).collect()
    }
}

/* ------------------------------------------------------------------ */
/*  Manual Clone & Debug implementations (RpcClient isn’t Clone/Debug) */
/* ------------------------------------------------------------------ */
impl Clone for Settings {
    fn clone(&self) -> Self {
        Self {
            workdir: self.workdir.clone(),
            rpc_url: self.rpc_url.clone(),
            fallback_rpc_url: self.fallback_rpc_url.clone(),
            relayer_url: self.relayer_url.clone(),
            geyser_url: self.geyser_url.clone(),
            geyser_token: self.geyser_token.clone(),
            wallets_file: self.wallets_file.clone(),
            active_wallet: self.active_wallet.clone(),
            jito: self.jito,
            tracked_wallets: self.tracked_wallets.clone(),
            keypair: Arc::clone(&self.keypair),
            buy_slippage_percent: self.buy_slippage_percent,
            buy_bribe_sol: self.buy_bribe_sol,
            buy_priority_fee_sol: self.buy_priority_fee_sol,
            sell_amount_percent: self.sell_amount_percent,
            sell_min_sol_out: self.sell_min_sol_out,
            sell_slippage_percent: self.sell_slippage_percent,
            sell_bribe_sol: self.sell_bribe_sol,
            sell_priority_fee_sol: self.sell_priority_fee_sol,
            fresh_mint_cache: self.fresh_mint_cache.clone(),
            rpc_client: Arc::clone(&self.rpc_client),
            take_profit_percent: self.take_profit_percent,
            take_profit_sell_fraction: self.take_profit_sell_fraction,
        }
    }
}

impl fmt::Debug for Settings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Settings")
            .field("workdir", &self.workdir)
            .field("rpc_url", &self.rpc_url)
            .field("relayer_url", &self.relayer_url)
            .field("active_wallet", &self.active_wallet)
            .field("tracked_wallets", &self.tracked_wallets)
            .finish_non_exhaustive()
    }
}
