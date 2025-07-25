use lazy_static::lazy_static;
use solana_sdk::pubkey::Pubkey;
use std::collections::{HashMap, HashSet};
use tokio::sync::RwLock;

/// Per-wallet we only care whether we *currently* hold a given mint,
/// so a HashSet<mint> is enough (no quantities yet).
pub type MintBalances = HashSet<Pubkey>;

lazy_static! {
    /// wallet → set of mints we’re holding
    pub static ref WALLET_STATE: RwLock<HashMap<Pubkey, MintBalances>> =
        RwLock::new(HashMap::new());
}
