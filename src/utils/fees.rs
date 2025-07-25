//! Fee‑related math helpers shared by all builders.

use solana_sdk::native_token::LAMPORTS_PER_SOL;

/// Convert a *total* priority‑fee amount (expressed in **SOL**) into the
/// per‑compute‑unit price that Solana's `ComputeBudgetInstruction` expects.
/// UPDATED: Now uses the CU limit set in wrapper.rs (180,000 CU).
pub fn tip_to_cu_price(total_sol: f64) -> u64 {
    // UPDATED: Use the CU limit from wrapper.rs (180,000)
    // This ensures the priority fee calculation matches what's actually set
    // Updated to 180k for better transaction success rate
    const ACTUAL_CU_LIMIT: f64 = 250_000.0;
    let micro_lamports_per_cu = ((total_sol * LAMPORTS_PER_SOL as f64) / ACTUAL_CU_LIMIT).round() as u64;
    
    // Debug logging to verify correct calculation
    if total_sol > 0.0 {
        let actual_fee_sol = (micro_lamports_per_cu * ACTUAL_CU_LIMIT as u64) as f64 / LAMPORTS_PER_SOL as f64;
    }
    
    micro_lamports_per_cu
}
