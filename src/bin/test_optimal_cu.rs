//! Test to find optimal compute unit budget for different DEX transactions
//! 
//! This tool simulates transactions for different DEXes to determine the actual
//! compute units consumed, helping optimize the CU budget and reduce overpayment.

use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::Instruction,
    message::{Message, VersionedMessage},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::VersionedTransaction,
    system_instruction,
    native_token::LAMPORTS_PER_SOL,
};
use std::str::FromStr;
use std::sync::Arc;
use spl_associated_token_account::get_associated_token_address;

// Hardcoded configuration values
const RPC_URL: &str = "https://api.mainnet-beta.solana.com";
const BUY_AMOUNT_SOL: f64 = 0.01;
const BUY_PRIORITY_FEE_SOL: f64 = 0.001;
const BUY_BRIBE_SOL: f64 = 0.001;
const SELL_PRIORITY_FEE_SOL: f64 = 0.001;
const SELL_BRIBE_SOL: f64 = 0.001;

// Test keypair (this is just for simulation, not real trading)
const TEST_PRIVATE_KEY: [u8; 64] = [
    174, 47, 154, 16, 202, 193, 206, 113, 199, 190, 53, 133, 169, 175, 31, 56,
    222, 53, 138, 189, 224, 216, 117, 173, 10, 149, 53, 45, 73, 251, 237, 246,
    15, 185, 186, 82, 177, 240, 148, 69, 241, 227, 167, 80, 141, 89, 240, 121,
    121, 35, 172, 247, 68, 251, 226, 218, 48, 63, 176, 109, 168, 89, 238, 135,
];

/// Simulate a transaction and return the compute units consumed
async fn simulate_transaction_cu(
    rpc_client: &RpcClient,
    instructions: &[Instruction],
    payer: &Keypair,
) -> Result<Option<u64>> {
    // Build transaction for simulation (without compute budget instructions)
    let blockhash = rpc_client.get_latest_blockhash()?;
    let message = Message::new_with_blockhash(instructions, Some(&payer.pubkey()), &blockhash);
    let signers: &[&dyn Signer] = &[payer];
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(message), signers)?;
    
    // Simulate the transaction
    match rpc_client.simulate_transaction(&tx) {
        Ok(response) => {
            if let Some(units_consumed) = response.value.units_consumed {
                Ok(Some(units_consumed))
            } else {
                println!("⚠️ No units_consumed in simulation response");
                Ok(None)
            }
        }
        Err(e) => {
            println!("❌ Simulation failed: {}", e);
            Ok(None)
        }
    }
}

/// Test compute units for a simple PumpFun-style transaction
async fn test_pumpfun_buy_cu(rpc_client: &RpcClient, payer: &Keypair) -> Result<Option<u64>> {
    println!("🧪 Testing PumpFun BUY transaction...");
    
    // Example mint (WSOL for testing)
    let mint = Pubkey::from_str("So11111111111111111111111111111111111111112")?;
    let amount_lamports = (BUY_AMOUNT_SOL * LAMPORTS_PER_SOL as f64) as u64;
    
    // Create a simple transfer instruction as a proxy for DEX swap
    let user_ata = get_associated_token_address(&payer.pubkey(), &mint);
    
    let instructions = vec![
        // ATA creation instruction (common in DEX swaps)
        spl_associated_token_account::instruction::create_associated_token_account(
            &payer.pubkey(),
            &payer.pubkey(),
            &mint,
            &spl_token::id(),
        ),
        // Simple transfer (proxy for swap)
        system_instruction::transfer(&payer.pubkey(), &user_ata, amount_lamports),
    ];
    
    simulate_transaction_cu(rpc_client, &instructions, payer).await
}

/// Test compute units for a simple sell transaction
async fn test_pumpfun_sell_cu(rpc_client: &RpcClient, payer: &Keypair) -> Result<Option<u64>> {
    println!("🧪 Testing PumpFun SELL transaction...");
    
    // Example mint (WSOL for testing)
    let mint = Pubkey::from_str("So11111111111111111111111111111111111111112")?;
    let user_ata = get_associated_token_address(&payer.pubkey(), &mint);
    
    let instructions = vec![
        // Token transfer instruction (proxy for sell)
        spl_token::instruction::transfer(
            &spl_token::id(),
            &user_ata,
            &payer.pubkey(),
            &payer.pubkey(),
            &[],
            1_000_000, // 1 token
        )?,
    ];
    
    simulate_transaction_cu(rpc_client, &instructions, payer).await
}

/// Test compute units for a more complex transaction with multiple instructions
async fn test_complex_transaction_cu(rpc_client: &RpcClient, payer: &Keypair) -> Result<Option<u64>> {
    println!("🧪 Testing Complex multi-instruction transaction...");
    
    let mint = Pubkey::from_str("So11111111111111111111111111111111111111112")?;
    let user_ata = get_associated_token_address(&payer.pubkey(), &mint);
    let amount_lamports = (BUY_AMOUNT_SOL * LAMPORTS_PER_SOL as f64) as u64;
    
    let instructions = vec![
        // Multiple instructions to simulate a complex DEX transaction
        spl_associated_token_account::instruction::create_associated_token_account(
            &payer.pubkey(),
            &payer.pubkey(),
            &mint,
            &spl_token::id(),
        ),
        system_instruction::transfer(&payer.pubkey(), &user_ata, amount_lamports),
        spl_token::instruction::sync_native(&spl_token::id(), &user_ata)?,
        spl_token::instruction::close_account(
            &spl_token::id(),
            &user_ata,
            &payer.pubkey(),
            &payer.pubkey(),
            &[],
        )?,
    ];
    
    simulate_transaction_cu(rpc_client, &instructions, payer).await
}

/// Test with tip instructions included
async fn test_transaction_with_tip_cu(rpc_client: &RpcClient, payer: &Keypair) -> Result<Option<u64>> {
    println!("🧪 Testing transaction with tip instruction...");
    
    let mint = Pubkey::from_str("So11111111111111111111111111111111111111112")?;
    let user_ata = get_associated_token_address(&payer.pubkey(), &mint);
    let amount_lamports = (BUY_AMOUNT_SOL * LAMPORTS_PER_SOL as f64) as u64;
    let tip_lamports = (BUY_BRIBE_SOL * LAMPORTS_PER_SOL as f64) as u64;
    
    // Helius tip account (example)
    let tip_account = Pubkey::from_str("4ACfpUFoaSD9bfPdeu6DBt89gB6ENTeHBXCAi87NhDEE")?;
    
    let instructions = vec![
        // Tip instruction (common in MEV protection)
        system_instruction::transfer(&payer.pubkey(), &tip_account, tip_lamports),
        // Main transaction
        spl_associated_token_account::instruction::create_associated_token_account(
            &payer.pubkey(),
            &payer.pubkey(),
            &mint,
            &spl_token::id(),
        ),
        system_instruction::transfer(&payer.pubkey(), &user_ata, amount_lamports),
        spl_token::instruction::sync_native(&spl_token::id(), &user_ata)?,
    ];
    
    simulate_transaction_cu(rpc_client, &instructions, payer).await
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Starting Compute Unit Optimization Test");
    println!("==========================================");
    
    // Create RPC client
    let rpc_client = RpcClient::new(RPC_URL.to_string());
    println!("✅ Connected to RPC: {}", RPC_URL);
    
    // Create test keypair
    let keypair = Keypair::from_bytes(&TEST_PRIVATE_KEY)?;
    println!("✅ Test keypair loaded: {}", keypair.pubkey());
    println!("📊 Current CU limit: 400,000 (hardcoded in wrapper.rs)");
    println!();
    
    let mut results = Vec::new();
    
    // Test different transaction types
    println!("📈 Testing different transaction patterns...");
    println!("============================================");
    
    // Test 1: Simple buy transaction
    if let Some(cu) = test_pumpfun_buy_cu(&rpc_client, &keypair).await? {
        println!("✅ PumpFun BUY CU usage: {}", cu);
        results.push(("PumpFun BUY", cu));
    }
    
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    
    // Test 2: Simple sell transaction
    if let Some(cu) = test_pumpfun_sell_cu(&rpc_client, &keypair).await? {
        println!("✅ PumpFun SELL CU usage: {}", cu);
        results.push(("PumpFun SELL", cu));
    }
    
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    
    // Test 3: Complex multi-instruction transaction
    if let Some(cu) = test_complex_transaction_cu(&rpc_client, &keypair).await? {
        println!("✅ Complex transaction CU usage: {}", cu);
        results.push(("Complex Multi-IX", cu));
    }
    
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    
    // Test 4: Transaction with tip
    if let Some(cu) = test_transaction_with_tip_cu(&rpc_client, &keypair).await? {
        println!("✅ Transaction with tip CU usage: {}", cu);
        results.push(("With Tip", cu));
    }
    
    // Analyze results
    println!("\n📊 COMPUTE UNIT ANALYSIS");
    println!("========================");
    
    if results.is_empty() {
        println!("❌ No successful simulations. Check your RPC connection.");
        return Ok(());
    }
    
    let mut max_cu = 0u64;
    let mut min_cu = u64::MAX;
    let mut total_cu = 0u64;
    
    for (test_name, cu) in &results {
        println!("{}: {} CU", test_name, cu);
        
        max_cu = max_cu.max(*cu);
        min_cu = min_cu.min(*cu);
        total_cu += cu;
    }
    
    let avg_cu = total_cu / results.len() as u64;
    
    println!("\n📈 STATISTICS");
    println!("=============");
    println!("Minimum CU usage: {}", min_cu);
    println!("Maximum CU usage: {}", max_cu);
    println!("Average CU usage: {}", avg_cu);
    println!("Current setting:  400,000 CU");
    
    // Calculate overpayment
    let current_cu = 400_000u64;
    if max_cu > 0 {
        let overpay_percentage = ((current_cu as f64 - max_cu as f64) / current_cu as f64) * 100.0;
        println!("Overpayment:      {:.1}%", overpay_percentage);
    }
    
    // Recommendations
    println!("\n💡 RECOMMENDATIONS");
    println!("==================");
    
    // Add 20% margin to max CU usage for safety
    let recommended_cu = (max_cu as f64 * 1.2) as u64;
    println!("Recommended CU limit: {} (max usage + 20% margin)", recommended_cu);
    
    // Show potential savings
    if recommended_cu < current_cu {
        let savings_percentage = ((current_cu - recommended_cu) as f64 / current_cu as f64) * 100.0;
        println!("Potential savings: {:.1}%", savings_percentage);
        
        println!("\n🔧 To apply this optimization, update src/tx/wrapper.rs:");
        println!("Change line: ixs.push(ComputeBudgetInstruction::set_compute_unit_limit(400_000));");
        println!("To:          ixs.push(ComputeBudgetInstruction::set_compute_unit_limit({}));", recommended_cu);
        
        println!("\nAnd update src/utils/fees.rs:");
        println!("Change: const ACTUAL_CU_LIMIT: f64 = 400_000.0;");
        println!("To:     const ACTUAL_CU_LIMIT: f64 = {}.0;", recommended_cu);
    } else {
        println!("✅ Current CU limit is already optimal or close to optimal");
    }
    
    println!("\n📋 TYPICAL CU USAGE BY TRANSACTION TYPE");
    println!("=======================================");
    println!("• Simple transfers: 300-1,000 CU");
    println!("• Token swaps: 30,000-80,000 CU");
    println!("• Complex DEX operations: 100,000-200,000 CU");
    println!("• Your current setting: 400,000 CU");
    
    println!("\n⚠️  NOTE: Test this with your actual DEX transactions!");
    println!("This test uses simplified proxy transactions. Real DEX swaps may use different amounts.");
    println!("CU usage can vary based on network conditions and account states.");
    
    Ok(())
}
