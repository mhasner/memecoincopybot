//! Raydium DEX integration with deterministic pool derivation
//! 
//! Based on official Raydium SDK V2 - eliminates RPC calls for maximum frontrunning speed
//! Uses deterministic PDA derivation for instant pool address calculation

use anyhow::{anyhow, Result};
use log::{debug, info, warn};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Signer,
    system_program,
    sysvar,
    transaction::VersionedTransaction,
    account::Account,
};
use spl_token::ID as TOKEN_PROGRAM_ID;
use spl_associated_token_account::get_associated_token_address;
use std::str::FromStr;

use crate::{
    config::settings::Settings,
    utils::token_tracker,
};

// Raydium program IDs (CPMM - Concentrated Product Market Maker)
pub const RAYDIUM_CPMM_PROGRAM_ID: Pubkey = solana_sdk::pubkey!("CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C");

// Common token addresses
pub const WSOL_MINT: Pubkey = solana_sdk::pubkey!("So11111111111111111111111111111111111111112");

// Instruction discriminators from Raydium SDK
const SWAP_BASE_INPUT_DISCRIMINATOR: [u8; 8] = [143, 190, 90, 218, 196, 30, 51, 222];

// PDA seeds from official Raydium SDK
const AUTH_SEED: &[u8] = b"vault_and_lp_mint_auth_seed";
const AMM_CONFIG_SEED: &[u8] = b"amm_config";
const POOL_SEED: &[u8] = b"pool";
const POOL_LP_MINT_SEED: &[u8] = b"pool_lp_mint";
const POOL_VAULT_SEED: &[u8] = b"pool_vault";
const OBSERVATION_SEED: &[u8] = b"observation";

#[derive(Debug, Clone)]
pub struct RaydiumPoolInfo {
    pub pool_id: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub lp_mint: Pubkey,
    pub base_vault: Pubkey,
    pub quote_vault: Pubkey,
    pub authority: Pubkey,
    pub config_id: Pubkey,
    pub observation_id: Pubkey,
    pub base_decimals: u8,
    pub quote_decimals: u8,
    pub lp_decimals: u8,
    pub base_reserve: u64,
    pub quote_reserve: u64,
    pub lp_supply: u64,
    pub trade_fee_rate: u64,
}

pub struct RaydiumDex {
    program_id: Pubkey,
}

impl RaydiumDex {
    pub fn new() -> Result<Self> {
        Ok(Self {
            program_id: RAYDIUM_CPMM_PROGRAM_ID,
        })
    }

    /// Build a buy transaction for Raydium CPMM using deterministic derivation
    pub async fn build_buy_transaction(
        &self,
        settings: &Settings,
        mint: &Pubkey,
        lamports: u64,
    ) -> Result<VersionedTransaction> {
        
        // Use deterministic derivation - NO RPC CALLS!
        let pool_keys = self.derive_pool_keys_for_migrated_token(mint)?;
        
        // Apply slippage from settings for buy orders
        let slippage_bps = (settings.buy_slippage_percent * 100.0) as u64;
        let expected_tokens = lamports / 1000; // Conservative estimate
        let min_amount_out = (expected_tokens * (10_000 - slippage_bps)) / 10_000;
        
        
        
        let mut swap_instructions = self.build_swap_base_in_instruction(
            &settings.keypair.pubkey(),
            &pool_keys,
            lamports,
            min_amount_out,
            true, // is_buy
        ).await?;

        // Add tip instruction (choose tip account based on jito setting)
        let tip_to = if settings.jito {
            crate::jito::tip_accounts::next()
        } else {
            crate::submit::helius_tips::next()
        };
        let tip_lamports = (settings.buy_bribe_sol * solana_sdk::native_token::LAMPORTS_PER_SOL as f64) as u64;
        let tip_ix = solana_sdk::system_instruction::transfer(&settings.keypair.pubkey(), &tip_to, tip_lamports);

        // Combine all instructions: tip + ATA creation + swap + cleanup
        let mut all_instructions = vec![tip_ix];
        all_instructions.extend(swap_instructions);

        // Create transaction with all instructions
        let recent_blockhash = settings.rpc_client.get_latest_blockhash()?;
        let transaction = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &all_instructions,
            Some(&settings.keypair.pubkey()),
            &[&settings.keypair],
            recent_blockhash,
        );

        // Store estimated token amount in token tracker for future sells
        let me = settings.keypair.pubkey();
        let estimated_token_amount = min_amount_out; // Conservative estimate
        token_tracker::store_token_amount(&me, mint, estimated_token_amount).await;
        info!("💾 [RAYDIUM] Stored {} tokens for future operations", estimated_token_amount);

        Ok(VersionedTransaction::from(transaction))
    }

    /// Build a sell transaction for Raydium CPMM using deterministic derivation
    pub async fn build_sell_transaction(
        &self,
        settings: &Settings,
        mint: &Pubkey,
        token_amount: u64,
    ) -> Result<VersionedTransaction> {
        
        // Use deterministic derivation - NO RPC CALLS!
        let pool_keys = self.derive_pool_keys_for_migrated_token(mint)?;
        
        // Apply slippage from settings for sell orders
        let slippage_bps = (settings.sell_slippage_percent * 100.0) as u64;
        let base_min_sol = settings.sell_min_sol_out * solana_sdk::native_token::LAMPORTS_PER_SOL as f64;
        let min_amount_out = ((base_min_sol * (10_000 - slippage_bps) as f64) / 10_000.0) as u64;
        
        
        
        let mut swap_instructions = self.build_swap_base_in_instruction(
            &settings.keypair.pubkey(),
            &pool_keys,
            token_amount,
            min_amount_out,
            false, // is_sell
        ).await?;

        // Add tip instruction (choose tip account based on jito setting)
        let tip_to = if settings.jito {
            crate::jito::tip_accounts::next()
        } else {
            crate::submit::helius_tips::next()
        };
        let tip_lamports = (settings.sell_bribe_sol * solana_sdk::native_token::LAMPORTS_PER_SOL as f64) as u64;
        let tip_ix = solana_sdk::system_instruction::transfer(&settings.keypair.pubkey(), &tip_to, tip_lamports);

        // Combine all instructions: tip + ATA creation + swap + cleanup
        let mut all_instructions = vec![tip_ix];
        all_instructions.extend(swap_instructions);

        // Create transaction with all instructions
        let recent_blockhash = settings.rpc_client.get_latest_blockhash()?;
        let transaction = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &all_instructions,
            Some(&settings.keypair.pubkey()),
            &[&settings.keypair],
            recent_blockhash,
        );

        // Update token tracker after sell
        let me = settings.keypair.pubkey();
        if let Some(current_amount) = token_tracker::get_token_amount(&me, mint).await {
            if token_amount >= current_amount {
                // Selling all tokens - clear tracker
                token_tracker::clear_token_amount(&me, mint).await;
                info!("🗑️ [RAYDIUM] Cleared all tokens after 100% sell");
            } else {
                // Partial sell - update remaining amount
                let remaining = current_amount.saturating_sub(token_amount);
                token_tracker::update_token_amount(&me, mint, remaining).await;
                info!("🔄 [RAYDIUM] Updated: {} -> {} tokens after sell", current_amount, remaining);
            }
        }

        Ok(VersionedTransaction::from(transaction))
    }

    /// Try to derive pool keys for migrated PumpFun token using multiple config indices
    /// This is INSTANT - no RPC calls needed!
    pub fn derive_pool_keys_for_migrated_token(&self, mint: &Pubkey) -> Result<RaydiumPoolInfo> {
        let wsol = WSOL_MINT;
        let program_id = self.program_id;
        
        // Try multiple config indices (0-7 are common for migrated tokens)
        for config_index in 0..8u16 {
            let config_id = self.derive_config_id(&program_id, config_index);
            
            // Determine mint order (Raydium requires mintA < mintB)
            let (mint_a, mint_b) = if mint.to_bytes() < wsol.to_bytes() {
                (*mint, wsol)
            } else {
                (wsol, *mint)
            };
            
            // Derive all addresses using official SDK patterns
            let pool_id = self.derive_pool_id(&program_id, &config_id, &mint_a, &mint_b);
            let authority = self.derive_pool_authority(&program_id);
            let lp_mint = self.derive_lp_mint(&program_id, &pool_id);
            let vault_a = self.derive_vault(&program_id, &pool_id, &mint_a);
            let vault_b = self.derive_vault(&program_id, &pool_id, &mint_b);
            let observation_id = self.derive_observation_id(&program_id, &pool_id);
            
            
            // For now, return the first attempt (config 0)
            // In production, you might want to check which pool actually exists
            if config_index == 0 {
                
                return Ok(RaydiumPoolInfo {
                    pool_id,
                    base_mint: mint_a,
                    quote_mint: mint_b,
                    lp_mint,
                    base_vault: vault_a,
                    quote_vault: vault_b,
                    authority,
                    config_id,
                    observation_id,
                    base_decimals: if mint_a == *mint { 6 } else { 9 }, // Token vs SOL
                    quote_decimals: if mint_b == *mint { 6 } else { 9 },
                    lp_decimals: 6,
                    base_reserve: 1000000000, // Placeholder - real calculation on-chain
                    quote_reserve: 1000000000,
                    lp_supply: 1000000000,
                    trade_fee_rate: 2500, // 0.25%
                });
            }
        }
        
        Err(anyhow!("Could not derive valid pool keys for migrated token"))
    }

    /// Derive config ID using SDK pattern: ["amm_config", u16_to_bytes(index)]
    /// CRITICAL: SDK uses BIG-ENDIAN, not little-endian!
    fn derive_config_id(&self, program_id: &Pubkey, index: u16) -> Pubkey {
        let index_bytes = index.to_be_bytes(); // BIG-ENDIAN like SDK!
        let (config_id, _bump) = Pubkey::find_program_address(
            &[AMM_CONFIG_SEED, &index_bytes],
            program_id,
        );
        config_id
    }

    /// Derive pool ID using SDK pattern: ["pool", config_id, mint_a, mint_b]
    fn derive_pool_id(&self, program_id: &Pubkey, config_id: &Pubkey, mint_a: &Pubkey, mint_b: &Pubkey) -> Pubkey {
        let (pool_id, _bump) = Pubkey::find_program_address(
            &[POOL_SEED, config_id.as_ref(), mint_a.as_ref(), mint_b.as_ref()],
            program_id,
        );
        pool_id
    }

    /// Derive pool authority using SDK pattern: ["vault_and_lp_mint_auth_seed"]
    fn derive_pool_authority(&self, program_id: &Pubkey) -> Pubkey {
        let (authority, _bump) = Pubkey::find_program_address(
            &[AUTH_SEED],
            program_id,
        );
        authority
    }

    /// Derive vault using SDK pattern: ["pool_vault", pool_id, mint]
    fn derive_vault(&self, program_id: &Pubkey, pool_id: &Pubkey, mint: &Pubkey) -> Pubkey {
        let (vault, _bump) = Pubkey::find_program_address(
            &[POOL_VAULT_SEED, pool_id.as_ref(), mint.as_ref()],
            program_id,
        );
        vault
    }

    /// Derive LP mint using SDK pattern: ["pool_lp_mint", pool_id]
    fn derive_lp_mint(&self, program_id: &Pubkey, pool_id: &Pubkey) -> Pubkey {
        let (lp_mint, _bump) = Pubkey::find_program_address(
            &[POOL_LP_MINT_SEED, pool_id.as_ref()],
            program_id,
        );
        lp_mint
    }

    /// Derive observation ID using SDK pattern: ["observation", pool_id]
    fn derive_observation_id(&self, program_id: &Pubkey, pool_id: &Pubkey) -> Pubkey {
        let (observation_id, _bump) = Pubkey::find_program_address(
            &[OBSERVATION_SEED, pool_id.as_ref()],
            program_id,
        );
        observation_id
    }

    /// Build swap base in instruction with ATA creation using official SDK structure
    async fn build_swap_base_in_instruction(
        &self,
        user: &Pubkey,
        pool_info: &RaydiumPoolInfo,
        amount_in: u64,
        min_amount_out: u64,
        is_buy: bool,
    ) -> Result<Vec<Instruction>> {

        // Determine input/output mints and accounts based on mint ordering
        let (input_mint, output_mint, input_vault, output_vault) = if is_buy {
            // Buying token with SOL
            if pool_info.base_mint == WSOL_MINT {
                // SOL is base mint
                (pool_info.base_mint, pool_info.quote_mint, pool_info.base_vault, pool_info.quote_vault)
            } else {
                // SOL is quote mint
                (pool_info.quote_mint, pool_info.base_mint, pool_info.quote_vault, pool_info.base_vault)
            }
        } else {
            // Selling token for SOL
            if pool_info.base_mint == WSOL_MINT {
                // SOL is base mint
                (pool_info.quote_mint, pool_info.base_mint, pool_info.quote_vault, pool_info.base_vault)
            } else {
                // SOL is quote mint
                (pool_info.base_mint, pool_info.quote_mint, pool_info.base_vault, pool_info.quote_vault)
            }
        };

        // Get user token accounts
        let user_input_account = get_associated_token_address(user, &input_mint);
        let user_output_account = get_associated_token_address(user, &output_mint);

        let mut instructions = Vec::new();

        // CRITICAL FIX: Add ATA creation instructions based on official Raydium SDK pattern
        // This is what was missing and causing the "AccountNotInitialized" error
        
        // Create input ATA if needed - use IDEMPOTENT creation (won't fail if exists)
        if input_mint != WSOL_MINT {
            // For regular tokens, create ATA using idempotent instruction
            let create_ata_ix = spl_associated_token_account::instruction::create_associated_token_account_idempotent(
                user,  // fee payer
                user,  // owner
                &input_mint, // mint
                &TOKEN_PROGRAM_ID, // token program
            );
            instructions.push(create_ata_ix);
        }

        // Create output ATA if needed - THIS WAS THE CRITICAL MISSING PIECE!
        if output_mint != WSOL_MINT {
            // For regular tokens, create ATA using idempotent instruction
            let create_ata_ix = spl_associated_token_account::instruction::create_associated_token_account_idempotent(
                user,  // fee payer
                user,  // owner
                &output_mint, // mint
                &TOKEN_PROGRAM_ID, // token program
            );
            instructions.push(create_ata_ix);
        }

        // Handle WSOL accounts specially (following official SDK pattern)
        if input_mint == WSOL_MINT && amount_in > 0 {
            // For SOL input, we need to fund the WSOL ATA
            // First ensure WSOL ATA exists
            let create_wsol_ata_ix = spl_associated_token_account::instruction::create_associated_token_account_idempotent(
                user,  // fee payer
                user,  // owner
                &WSOL_MINT, // WSOL mint
                &TOKEN_PROGRAM_ID, // token program
            );
            instructions.push(create_wsol_ata_ix);
            
            // Transfer SOL to WSOL ATA to fund it
            let transfer_ix = solana_sdk::system_instruction::transfer(
                user,
                &user_input_account,
                amount_in,
            );
            instructions.push(transfer_ix);
            
            // Sync native (convert SOL to WSOL tokens)
            let sync_native_ix = spl_token::instruction::sync_native(&TOKEN_PROGRAM_ID, &user_input_account)?;
            instructions.push(sync_native_ix);
            
        }

        if output_mint == WSOL_MINT {
            // For SOL output, ensure WSOL ATA exists (will be unwrapped later)
            let create_wsol_ata_ix = spl_associated_token_account::instruction::create_associated_token_account_idempotent(
                user,  // fee payer
                user,  // owner
                &WSOL_MINT, // WSOL mint
                &TOKEN_PROGRAM_ID, // token program
            );
            instructions.push(create_wsol_ata_ix);
        }

        // Build swap instruction data: discriminator + amount_in + min_amount_out
        let mut instruction_data = Vec::new();
        instruction_data.extend_from_slice(&SWAP_BASE_INPUT_DISCRIMINATOR);
        instruction_data.extend_from_slice(&amount_in.to_le_bytes());
        instruction_data.extend_from_slice(&min_amount_out.to_le_bytes());

        // Build accounts based on official Raydium SDK makeSwapCpmmBaseInInstruction
        let accounts = vec![
            AccountMeta::new(*user, true),                              // payer (signer)
            AccountMeta::new_readonly(pool_info.authority, false),      // authority
            AccountMeta::new_readonly(pool_info.config_id, false),      // configId
            AccountMeta::new(pool_info.pool_id, false),                 // poolId
            AccountMeta::new(user_input_account, false),                // userInputAccount
            AccountMeta::new(user_output_account, false),               // userOutputAccount
            AccountMeta::new(input_vault, false),                       // inputVault
            AccountMeta::new(output_vault, false),                      // outputVault
            AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),         // inputTokenProgram
            AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),         // outputTokenProgram
            AccountMeta::new_readonly(input_mint, false),               // inputMint
            AccountMeta::new_readonly(output_mint, false),              // outputMint
            AccountMeta::new(pool_info.observation_id, false),          // observationId
        ];

        // Add the main swap instruction
        let swap_instruction = Instruction {
            program_id: self.program_id,
            accounts,
            data: instruction_data,
        };
        instructions.push(swap_instruction);

        // Add cleanup instructions for WSOL accounts if needed
        if output_mint == WSOL_MINT {
            // Close WSOL account and transfer SOL back to user
            let close_wsol_ix = spl_token::instruction::close_account(
                &spl_token::id(),
                &user_output_account,
                user, // destination for remaining lamports
                user, // owner
                &[],  // no multisig
            )?;
            instructions.push(close_wsol_ix);
        }


        Ok(instructions)
    }

    /// Check if a mint has a Raydium pool - ALWAYS FALSE for frontrunning speed
    pub async fn has_pool(&self, _rpc_client: &RpcClient, _mint: &Pubkey) -> bool {
        // For frontrunning, we NEVER make API calls
        // We use deterministic derivation and let the transaction fail if pool doesn't exist
        false
    }

    /// Calculate swap amounts using Raydium's constant product formula with fees
    pub fn calculate_swap_amount(
        &self,
        reserve_in: u64,
        reserve_out: u64,
        amount_in: u64,
        _is_buy: bool,
    ) -> Result<u64> {
        if reserve_in == 0 || reserve_out == 0 {
            return Err(anyhow!("Invalid pool reserves"));
        }
        
        // Raydium CPMM uses constant product formula: x * y = k
        // With fees: output = (amount_in * (1000000 - fee_rate) * reserve_out) / (reserve_in * 1000000 + amount_in * (1000000 - fee_rate))
        // Default fee rate is 0.25% = 2500 out of 1000000
        
        let fee_rate = 2500u64; // 0.25%
        let fee_denominator = 1000000u64;
        
        let amount_in_with_fee = amount_in
            .checked_mul(fee_denominator - fee_rate)
            .ok_or_else(|| anyhow!("Overflow in fee calculation"))?;
            
        let numerator = amount_in_with_fee
            .checked_mul(reserve_out)
            .ok_or_else(|| anyhow!("Overflow in numerator calculation"))?;
            
        let denominator = reserve_in
            .checked_mul(fee_denominator)
            .ok_or_else(|| anyhow!("Overflow in denominator calculation"))?
            .checked_add(amount_in_with_fee)
            .ok_or_else(|| anyhow!("Overflow in denominator addition"))?;
            
        let output_amount = numerator
            .checked_div(denominator)
            .ok_or_else(|| anyhow!("Division by zero in swap calculation"))?;
            
        debug!("💱 [RAYDIUM] Swap calculation: {} -> {} (reserves: {} -> {})", 
                amount_in, output_amount, reserve_in, reserve_out);
        
        Ok(output_amount)
    }
}

/// Detect if a mint is traded on Raydium - ALWAYS FALSE for frontrunning
pub async fn detect_raydium_pool(_rpc_client: &RpcClient, _mint: &Pubkey) -> bool {
    // For frontrunning, we never do detection calls
    // We rely on the migration detection in pool_tracker
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_deterministic_derivation() {
        let raydium_dex = RaydiumDex::new().unwrap();
        let test_mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap(); // USDC
        
        // Test deterministic derivation
        let pool_keys = raydium_dex.derive_pool_keys_for_migrated_token(&test_mint);
        assert!(pool_keys.is_ok());
        
        let keys = pool_keys.unwrap();
        assert_ne!(keys.pool_id, Pubkey::default());
        assert_ne!(keys.authority, Pubkey::default());
        assert_ne!(keys.base_vault, Pubkey::default());
        assert_ne!(keys.quote_vault, Pubkey::default());
        
    }

    #[test]
    fn test_swap_calculation() {
        let raydium_dex = RaydiumDex::new().unwrap();
        
        // Test swap calculation with realistic values
        let reserve_in = 1_000_000_000; // 1000 SOL
        let reserve_out = 1_000_000_000_000; // 1M tokens
        let amount_in = 1_000_000_000; // 1 SOL
        
        let result = raydium_dex.calculate_swap_amount(reserve_in, reserve_out, amount_in, true);
        assert!(result.is_ok());
        
        let output = result.unwrap();
        assert!(output > 0);
        assert!(output < reserve_out); // Should be less than total reserve
        
    }

    #[test]
    fn test_mint_ordering() {
        let raydium_dex = RaydiumDex::new().unwrap();
        let wsol = WSOL_MINT;
        
        // Test with a mint that should be ordered before WSOL
        let mint_before = Pubkey::from_str("11111111111111111111111111111111").unwrap();
        let keys_before = raydium_dex.derive_pool_keys_for_migrated_token(&mint_before).unwrap();
        assert_eq!(keys_before.base_mint, mint_before);
        assert_eq!(keys_before.quote_mint, wsol);
        
        // Test with a mint that should be ordered after WSOL
        let mint_after = Pubkey::from_str("zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz").unwrap();
        let keys_after = raydium_dex.derive_pool_keys_for_migrated_token(&mint_after).unwrap();
        assert_eq!(keys_after.base_mint, wsol);
        assert_eq!(keys_after.quote_mint, mint_after);
        
    }
}
