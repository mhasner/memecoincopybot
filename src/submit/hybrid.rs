//! Hybrid Submitter - Jito Bundle first, Helius Fast fallback
//! 
//! This module provides the primary submission strategy:
//! 1. Try Jito bundle submission for maximum speed and MEV protection
//! 2. Fall back to Helius Fast if Jito fails or times out
//! 3. Log performance metrics for both paths

