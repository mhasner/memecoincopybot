//! DEX‑agnostic helpers for composing transactions.

use solana_sdk::instruction::Instruction;
use solana_sdk::compute_budget::ComputeBudgetInstruction;

/// Append compute budget instructions for `fee_sol` with proper CU limits
/// This ensures predictable fees and better transaction prioritization
pub fn push_compute_budget_ix(ixs: &mut Vec<Instruction>, fee_sol: f64) {
    if fee_sol > 0.0 {
        // CRITICAL FIX: Set compute unit limit first for predictable fees
        // Updated CU limit to 180k for better transaction success rate
        ixs.push(ComputeBudgetInstruction::set_compute_unit_limit(250_000));
        
        let p = crate::utils::fees::tip_to_cu_price(fee_sol);
        if p > 0 {
            ixs.push(ComputeBudgetInstruction::set_compute_unit_price(p));
        }
    }
}
