//! Helius /fast endpoint submitter — JSON-RPC compliant

use crate::submit::iface::Submitter;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use log::{info, warn};
use std::time::Instant;

#[derive(Clone)]
pub struct HeliusFast {
    url: String,
    client: Client,
}

impl HeliusFast {
    pub fn new(url: String) -> Self {
        info!("🚀 [HELIUS] Fast submitter initialized: {}", url);
        Self {
            url,
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(5))
                .build()
                .expect("reqwest build failed"),
        }
    }

    /// Ping the Helius endpoint to keep connection warm
    pub async fn ping(&self) -> Result<()> {
        // Extract base URL and construct ping endpoint
        let ping_url = if self.url.contains("helius-rpc.com") {
            // For Helius endpoints, use the ping endpoint
            "your_connection_warming_endpoint"
        } else {
            // For other endpoints, try appending /ping
            return Err(anyhow!("Ping not supported for non-Helius endpoints"));
        };

        let start_time = Instant::now();
        
        let res = self.client
            .get(ping_url)
            .send()
            .await?;

        let ping_time = start_time.elapsed();
        
        if res.status().is_success() {
            info!("🏓 [HELIUS] Connection warmed in {:.2}ms", ping_time.as_millis());
            Ok(())
        } else {
            warn!("⚠️ [HELIUS] Ping failed with status: {}", res.status());
            Err(anyhow!("Ping failed with status: {}", res.status()))
        }
    }
}

#[async_trait]
impl Submitter for HeliusFast {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    async fn submit(&self, tx_b64: String, _skip: bool) -> Result<String> {
        let start_time = Instant::now();
        
        let body = json!({
            "jsonrpc": "2.0",
            "id": "copybot",
            "method": "sendTransaction",
            "params": [
                tx_b64,
                {
                    "encoding": "base64",
                    "skipPreflight": true,
                    "maxRetries": 0
                }
            ]
        });

        let res = self.client
            .post(&self.url)
            .json(&body)
            .send()
            .await?;

        let status = res.status();
        if !status.is_success() {
            let err_text = res.text().await.unwrap_or_default();
            return Err(anyhow!("Helius FAST HTTP {}: {}", status, err_text));
        }

        let resp: serde_json::Value = res.json().await?;
        let sig = resp["result"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing 'result' in response: {:?}", resp))?;

        let submit_time = start_time.elapsed();
        info!("⚡ [HELIUS] Fast submission in {:.2}ms: {}", 
              submit_time.as_millis(), sig);

        Ok(sig.to_string())
    }
}
