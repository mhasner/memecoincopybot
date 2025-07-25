//! utils/pool_tracker.rs
//! -------------------------------------------------------------
//! Keeps a live in‑process registry of on‑chain liquidity pools,
//! and maps each Pump.fun mint to the BUY / SELL creator‑vault PDAs
//! we've observed in tracked transactions.
