//! Generic polling helpers shared by builders.

use anyhow::{Result, anyhow};
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use spl_token::state::Account as TokenAccount;
use solana_program::program_pack::Pack;

/// DEPRECATED: This function has been replaced with immediate token tracking
/// We no longer poll for ATA balances - instead we use known token amounts from BUY events
/// or fail fast if no known amount is available
pub async fn poll_until_nonzero_balance(_rpc: &RpcClient, _ata: &Pubkey) -> Result<u64> {
    Err(anyhow!("ATA polling has been disabled - use known token amounts instead"))
}

/// Fast ATA balance check without polling - returns immediately
pub async fn get_ata_balance_immediate(rpc: &RpcClient, ata: &Pubkey) -> Result<u64> {
    match rpc.get_account_data(ata) {
        Ok(ata_data) => {
            if let Ok(token_account) = TokenAccount::unpack(&ata_data) {
                Ok(token_account.amount)
            } else {
                Err(anyhow!("ATA exists but unpack failed"))
            }
        }
        Err(_) => {
            Err(anyhow!("ATA not found"))
        }
    }
}
