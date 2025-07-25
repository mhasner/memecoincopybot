//! Generic helper types shared by the various DEX helpers.
//
//  At the moment we only need a very small `PoolItem` so that
//  `utils/pool_tracker.rs` compiles.  We will flesh this out later once
//  real pool‑tracking is hooked up.

use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PoolItem {
    /// Token mint of the pool.
    pub mint: Pubkey,
    /* Feel free to expand with more fields later, e.g.
     *   pub lp_token:    Pubkey,
     *   pub creator:     Pubkey,
     *   pub created_slot: u64,
     *   pub symbol:       String,
     */
}
