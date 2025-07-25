//! Fan‑out engine that runs every strategy on each observed fill
//! and offers shared state ( PositionManager ) to them.

use crate::config::settings::Settings;
use once_cell::sync::OnceCell;
use std::sync::{Arc, Mutex};

use crate::{
    positions::PositionManager,
    strategy::{
        follow_buy::FollowBuy, follow_sell::FollowSell, take_profit::TakeProfit, ObservedFill,
        /* trait & helper types */
        Strategy, TradePlan, DexKind,
    },
};

/* ──────────────────────────────────────────────────────────────────── */
/*  Shared objects accessible from any strategy                        */
/* ──────────────────────────────────────────────────────────────────── */

/// One global handle so strategies such as `TakeProfit` can peek at the
/// current PositionManager without plumbing references through every call‑stack.
pub static STRATEGY_ENGINE: OnceCell<Arc<EngineShared>> = OnceCell::new();

/// Anything that needs to be visible across strategies belongs here.
pub struct EngineShared {
    pub positions: Mutex<PositionManager>,
}

impl EngineShared {
    pub fn new(pm: PositionManager) -> Self {
        Self {
            positions: Mutex::new(pm),
        }
    }
}

/* ──────────────────────────────────────────────────────────────────── */
/*  The engine proper                                                  */
/* ──────────────────────────────────────────────────────────────────── */

pub struct StrategyEngine {
    strategies: Vec<Box<dyn Strategy + Send>>,
    /// Keep an `Arc` around so the caller can still access the same
    /// PositionManager after constructing the engine.
    pub positions: Arc<EngineShared>,
}

impl StrategyEngine {
    /// Create a new engine and register every active strategy exactly once.
    pub fn new(shared: Arc<EngineShared>) -> Self {
        /* make the shared handle globally available */
        let _ = STRATEGY_ENGINE.set(shared.clone());

        /* -------- register strategies here -------- */
        let mut strategies: Vec<Box<dyn Strategy + Send>> = Vec::new();
        strategies.push(Box::new(FollowBuy)); // mirror tracked BUYs 1‑to‑1
        strategies.push(Box::new(FollowSell)); // mirror tracked SELLs with 90 %‑rule
        strategies.push(Box::new(TakeProfit)); // auto 50 % take‑profit at +120 % PnL
                                               /* ------------------------------------------ */

        Self {
            strategies,
            positions: shared,
        }
    }

    /// Run *every* strategy on the incoming fill and collect all plans.
    pub fn on_fill(&mut self, fill: &ObservedFill, settings: &Settings) -> Vec<TradePlan> {
        let mut out = Vec::new();
        for strat in &mut self.strategies {
            out.extend(strat.on_fill(fill, settings));
        }
        out
    }
}
