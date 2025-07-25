pub mod iface;
pub mod helius_fast;
pub mod helius_tips;
pub mod jito_bundle;
pub mod hybrid;

use iface::Submitter;
use std::sync::Arc;
use solana_client::rpc_client::RpcClient;
use solana_sdk::signature::Keypair;

/// Returns the appropriate submitter based on jito setting
/// When jito=true: hybrid submitter (Jito first, Helius fallback) for maximum speed and reliability
/// When jito=false: Helius-only submitter for direct submission
pub fn default(relayer_url: &str, rpc_client: RpcClient, keypair: Keypair, jito_enabled: bool) -> Arc<dyn Submitter> {
    if jito_enabled {
        Arc::new(hybrid::HybridSubmitter::new(relayer_url.to_string(), rpc_client, keypair))
    } else {
        Arc::new(helius_fast::HeliusFast::new(relayer_url.to_string()))
    }
}

/// Returns the Jito-only submitter for testing
pub fn jito_only() -> Arc<dyn Submitter> {
    Arc::new(jito_bundle::JitoBundle::new())
}

/// Returns the Helius-only submitter for fallback scenarios
pub fn helius_only(relayer_url: &str) -> Arc<dyn Submitter> {
    Arc::new(helius_fast::HeliusFast::new(relayer_url.to_string()))
}

/// Ping the Helius connection for the given submitter to keep it warm
/// Works with both HybridSubmitter and HeliusFast submitters
pub async fn ping_connection(submitter: &Arc<dyn Submitter>) -> anyhow::Result<()> {
    // Try to downcast to HybridSubmitter first
    if let Some(hybrid) = submitter.as_any().downcast_ref::<hybrid::HybridSubmitter>() {
        return hybrid.ping().await;
    }
    
    // Try to downcast to HeliusFast
    if let Some(helius) = submitter.as_any().downcast_ref::<helius_fast::HeliusFast>() {
        return helius.ping().await;
    }
    
    // For other submitters (like JitoBundle), ping is not supported
    Err(anyhow::anyhow!("Ping not supported for this submitter type"))
}
