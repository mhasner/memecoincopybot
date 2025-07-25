//! Fast ATA Creation Module - Zero RPC Calls for Maximum Frontrunning Speed
//! 
//! This module implements SDK-specific ATA creation patterns for each DEX:
//! - PumpFun: Always create with standard SPL Token program
//! - Raydium: Use token program from pool info (deterministic)
//! - Moonshot: Uses getAssociatedTokenAddress + ASSOCIATED_TOKEN_PROGRAM_ID
//! - Mercurial: Uses getOrCreateATAInstruction pattern
//! - All SDKs: ZERO RPC calls for existence checks

