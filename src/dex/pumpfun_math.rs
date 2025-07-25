use anyhow::{Result, anyhow};
use borsh::BorshDeserialize;
use solana_sdk::{pubkey::Pubkey};
use solana_client::rpc_client::RpcClient;

/// Struct to decode the bonding curve account state via Borsh
#[derive(BorshDeserialize, Debug)]
pub struct BCHeader {
    _disc: u64,
    _vtr: u64,
    _vsr: u64,
    _rtr: u64,
    _rsr: u64,
    _sup: u64,
    _complete: bool,
    _pad: [u8; 7],
    pub creator: Pubkey,
}

/// Load the bonding header from the on-chain bonding curve PDA
pub fn load_bonding_header(rpc: &RpcClient, bonding_curve: &Pubkey) -> Result<BCHeader> {
    let data = rpc
        .get_account_data(bonding_curve)
        .map_err(|e| anyhow!("Failed to fetch bonding curve account: {}", e))?;

    if data.len() < 88 {
        return Err(anyhow!("Bonding curve account too short: {} bytes", data.len()));
    }

    let header = BCHeader::try_from_slice(&data[0..88])?;
    Ok(header)
}

/// Estimate the minimum tokens out given a bonding curve state and SOL input
pub fn min_tokens_out(bc_data: &[u8], lamports: u64) -> u64 {
    let mut vsr = u64::from_le_bytes(bc_data[16..24].try_into().unwrap()) as u128;
    let mut vtr = u64::from_le_bytes(bc_data[8..16].try_into().unwrap()) as u128;
    let mut sol = lamports as u128;
    let mut out = 0u128;

    while sol > 0 {
        let price = (vsr * 1_000_000) / vtr;
        let cost = price / 1_000_000;
        if cost == 0 || cost > sol {
            break;
        }
        sol -= cost;
        vsr += cost;
        vtr -= 1;
        out += 1;

        if out > 10_000 {
            break;
        }
    }

    out as u64
}
