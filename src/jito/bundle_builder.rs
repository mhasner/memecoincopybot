//! Jito Bundle Builder - Creates bundles with proper tips from settings
//! 
//! This module provides functions to build Jito bundles with transactions
//! that include proper tip amounts based on buy/sell settings.

use crate::config::settings::Settings;
use crate::strategy::Side;
use anyhow::Result;
use solana_sdk::{
    pubkey::Pubkey,
    signature::Signer,
    system_instruction,
    transaction::VersionedTransaction,
    message::VersionedMessage,
};
use std::str::FromStr;
use base64::Engine;

// Jito tip accounts (from official docs)
const TIP_ACCOUNTS: &[&str] = &[
    "96gYZGLnJYVFmbjzopPSU6QiEV5fGqZNyN9nmNhvrZU5",
    "HFqU5x63VTqvQss8hp11i4wVV8bD44PvwucfZ2bU7gRe", 
    "Cw8CFyM9FkoMi7K7Crf6HNQqf4uEMzpKw6QNghXLvLkY",
    "ADaUMid9yfUytqMBgopwjb2DTLSokTSzL1zt6iGPaS49",
    "DfXygSm4jCyNCybVYYK6DwvWqjKee8pbDmJGcLWNDXjh",
    "ADuUkR4vqLUMWXxW9gh6D6L8pMSawimctcNZ5pGwDcEt",
    "DttWaMuVvTiduZRnguLF7jNxTgiMBZ1hyAumKUiL2KRL",
    "3AVi9Tg9Uo68tJfuvoKvqKNWKkC5wPdSSdeBnizKZ6jT",
];

/// Get a random tip account for load balancing
pub fn get_tip_account() -> &'static str {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::SystemTime;
    
    let mut hasher = DefaultHasher::new();
    SystemTime::now().hash(&mut hasher);
    let index = (hasher.finish() as usize) % TIP_ACCOUNTS.len();
    TIP_ACCOUNTS[index]
}

/// Build a Jito bundle with main transaction + tip transaction
/// Uses proper tip amounts from settings based on buy/sell side
pub fn build_jito_bundle(
    main_tx: VersionedTransaction,
    settings: &Settings,
    side: Side,
) -> Result<String> {
    // Get tip amounts from settings based on buy/sell
    let (bribe_sol, priority_fee_sol) = match side {
        Side::Buy => (settings.buy_bribe_sol, settings.buy_priority_fee_sol),
        Side::Sell => (settings.sell_bribe_sol, settings.sell_priority_fee_sol),
    };

    // Calculate total tip in lamports (bribe + priority fee)
    let total_tip_lamports = settings.sol_to_lamports(bribe_sol + priority_fee_sol)?;
    
    // Ensure minimum tip of 1000 lamports as per Jito requirements
    let tip_lamports = total_tip_lamports.max(1000);

    // Get random tip account
    let tip_account = Pubkey::from_str(get_tip_account())?;

    // Build standalone tip transaction
    let tip_tx = crate::jito::wrapper::build_tip_only_tx(
        &settings.keypair,
        &tip_account,
        tip_lamports,
        &settings.rpc_client,
    )?;

    // Serialize both transactions to base64
    let main_tx_b64 = base64::engine::general_purpose::STANDARD.encode(
        bincode::serialize(&main_tx)?
    );
    let tip_tx_b64 = base64::engine::general_purpose::STANDARD.encode(
        bincode::serialize(&tip_tx)?
    );

    // Create bundle array - main transaction first, then tip
    let bundle = vec![main_tx_b64, tip_tx_b64];

    // Return as JSON string for the submitter
    Ok(serde_json::to_string(&bundle)?)
}

/// Enhanced version that adds tip instruction to main transaction + standalone tip
/// This provides better MEV protection by including tip in the main transaction
pub fn build_enhanced_jito_bundle(
    mut main_tx: VersionedTransaction,
    settings: &Settings,
    side: Side,
) -> Result<String> {
    // Get tip amounts from settings based on buy/sell
    let (bribe_sol, priority_fee_sol) = match side {
        Side::Buy => (settings.buy_bribe_sol, settings.buy_priority_fee_sol),
        Side::Sell => (settings.sell_bribe_sol, settings.sell_priority_fee_sol),
    };

    // Calculate total tip in lamports (bribe + priority fee)
    let total_tip_lamports = settings.sol_to_lamports(bribe_sol + priority_fee_sol)?;
    
    // Ensure minimum tip of 1000 lamports as per Jito requirements
    let tip_lamports = total_tip_lamports.max(1000);

    // Get random tip account
    let tip_account = Pubkey::from_str(get_tip_account())?;

    // Add tip instruction to main transaction
    let main_tx_with_tip = add_tip_instruction_to_transaction(
        main_tx,
        &settings.keypair,
        &tip_account,
        tip_lamports,
        &settings.rpc_client,
    )?;

    // Build standalone tip transaction
    let tip_tx = crate::jito::wrapper::build_tip_only_tx(
        &settings.keypair,
        &tip_account,
        tip_lamports,
        &settings.rpc_client,
    )?;

    // Serialize both transactions to base64
    let main_tx_b64 = base64::engine::general_purpose::STANDARD.encode(
        bincode::serialize(&main_tx_with_tip)?
    );
    let tip_tx_b64 = base64::engine::general_purpose::STANDARD.encode(
        bincode::serialize(&tip_tx)?
    );

    // Create bundle array - main transaction with tip first, then standalone tip
    let bundle = vec![main_tx_b64, tip_tx_b64];

    // Return as JSON string for the submitter
    Ok(serde_json::to_string(&bundle)?)
}

/// Add tip instruction to an existing transaction
fn add_tip_instruction_to_transaction(
    main_tx: VersionedTransaction,
    payer: &solana_sdk::signature::Keypair,
    tip_account: &Pubkey,
    tip_lamports: u64,
    rpc_client: &solana_client::rpc_client::RpcClient,
) -> Result<VersionedTransaction> {
    // Extract the message from the main transaction
    let mut instructions = match &main_tx.message {
        VersionedMessage::Legacy(msg) => msg.instructions.clone(),
        VersionedMessage::V0(_) => {
            return Err(anyhow::anyhow!("V0 messages not supported yet for bundle building"));
        }
    };
    
    let mut account_keys = match &main_tx.message {
        VersionedMessage::Legacy(msg) => msg.account_keys.clone(),
        VersionedMessage::V0(_) => {
            return Err(anyhow::anyhow!("V0 messages not supported yet for bundle building"));
        }
    };
    
    // Add tip account if not already present
    if !account_keys.contains(tip_account) {
        account_keys.push(*tip_account);
    }
    
    // Add system program if not already present
    if !account_keys.contains(&solana_sdk::system_program::id()) {
        account_keys.push(solana_sdk::system_program::id());
    }
    
    // Create tip instruction
    let tip_instruction = system_instruction::transfer(&payer.pubkey(), tip_account, tip_lamports);
    
    // Convert tip instruction to use account indices
    let tip_instruction_indexed = solana_sdk::instruction::CompiledInstruction {
        program_id_index: account_keys.iter().position(|&key| key == solana_sdk::system_program::id())
            .ok_or_else(|| anyhow::anyhow!("System program not found in account keys"))? as u8,
        accounts: vec![
            account_keys.iter().position(|&key| key == payer.pubkey())
                .ok_or_else(|| anyhow::anyhow!("Payer not found in account keys"))? as u8,
            account_keys.iter().position(|&key| key == *tip_account)
                .ok_or_else(|| anyhow::anyhow!("Tip account not found in account keys"))? as u8,
        ],
        data: tip_instruction.data,
    };
    
    instructions.push(tip_instruction_indexed);
    
    // Rebuild the main transaction with tip instruction
    let blockhash = rpc_client.get_latest_blockhash()?;
    let main_message = solana_sdk::message::Message {
        header: match &main_tx.message {
            VersionedMessage::Legacy(msg) => msg.header.clone(),
            VersionedMessage::V0(_) => {
                return Err(anyhow::anyhow!("V0 messages not supported yet for bundle building"));
            }
        },
        account_keys,
        recent_blockhash: blockhash,
        instructions,
    };
    
    let main_tx_with_tip = VersionedTransaction::try_new(
        VersionedMessage::Legacy(main_message),
        &[payer]
    )?;

    Ok(main_tx_with_tip)
}
