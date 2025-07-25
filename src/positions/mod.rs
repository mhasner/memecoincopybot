#![allow(clippy::derived_hash_with_manual_eq)]

use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::ops::Sub;
use std::{
    collections::HashMap,
    fs,
    io::{self, Write},
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

/* --------------------------------------------------------------------- */
/*  On‑disk location                                                     */
/* --------------------------------------------------------------------- */
const STORAGE_PATH: &str = "src/positions/positions.json";

/* --------------------------------------------------------------------- */
/*  A single open position                                               */
/* --------------------------------------------------------------------- */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub mint: Pubkey,
    pub balance: u128,      // base‑unit tokens
    pub cost_lamports: u64, // total cost basis
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_price: Option<f64>,
    pub updated_at: u64,
}

impl Position {
    pub fn avg_cost(&self) -> f64 {
        if self.balance == 0 {
            0.0
        } else {
            self.cost_lamports as f64 / self.balance as f64
        }
    }

    pub fn unrealised_pnl_pct(&self) -> Option<f64> {
        self.last_price
            .map(|p| ((p / self.avg_cost()) - 1.0) * 100.0)
    }
}

/* --------------------------------------------------------------------- */
/*  Manager                                                              */
/* --------------------------------------------------------------------- */
#[derive(Debug, Default)]
pub struct PositionManager {
    positions: HashMap<Pubkey, Position>,
}

impl PositionManager {
    pub fn load() -> io::Result<Self> {
        let path = Path::new(STORAGE_PATH);
        if !path.exists() {
            return Ok(Self::default());
        }
        let bytes = fs::read(path)?;
        let map: HashMap<Pubkey, Position> = serde_json::from_slice(&bytes)?;
        Ok(Self { positions: map })
    }

    fn persist(&self) -> io::Result<()> {
        if let Some(parent) = Path::new(STORAGE_PATH).parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_vec_pretty(&self.positions)?;
        let mut file = fs::File::create(STORAGE_PATH)?;
        file.write_all(&json)?;
        Ok(())
    }

    /* ------------------------------ trade recording ------------------ */
    pub fn record_buy(
        &mut self,
        mint: Pubkey,
        qty_base_units: u128,
        cost_lamports: u64,
    ) -> io::Result<()> {
        let now = unix_timestamp();
        let entry = self.positions.entry(mint).or_insert(Position {
            mint,
            balance: 0,
            cost_lamports: 0,
            last_price: None,
            updated_at: now,
        });

        entry.balance += qty_base_units;
        entry.cost_lamports += cost_lamports;
        entry.updated_at = now;
        self.persist()
    }

    pub fn record_sell(
        &mut self,
        mint: Pubkey,
        qty_base_units: u128,
        _received_lamports: u64, // not used yet, kept for completeness
    ) -> io::Result<()> {
        if let Some(pos) = self.positions.get_mut(&mint) {
            if qty_base_units >= pos.balance {
                self.positions.remove(&mint);
            } else {
                let pct = qty_base_units as f64 / pos.balance as f64;
                let reduce_cost = (pos.cost_lamports as f64 * pct).round() as u64;

                pos.balance -= qty_base_units;
                pos.cost_lamports -= reduce_cost;
                pos.updated_at = unix_timestamp();
            }
            self.persist()?;
        }
        Ok(())
    }

    /* ------------------------------ aux helpers ---------------------- */
    pub fn update_price(&mut self, mint: Pubkey, price_lamports: f64) -> io::Result<()> {
        if let Some(pos) = self.positions.get_mut(&mint) {
            pos.last_price = Some(price_lamports);
            pos.updated_at = unix_timestamp();
            self.persist()?;
        }
        Ok(())
    }

    pub fn unrealised_pct(&self, mint: Pubkey) -> Option<f64> {
        self.positions
            .get(&mint)
            .and_then(|p| p.unrealised_pnl_pct())
    }

    pub fn balance(&self, mint: Pubkey) -> u128 {
        self.positions.get(&mint).map(|p| p.balance).unwrap_or(0)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Position> {
        self.positions.values()
    }
}

fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
