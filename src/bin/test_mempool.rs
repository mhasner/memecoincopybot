use std::time::Instant;
use std::sync::Arc;
use std::collections::HashMap;
use anyhow::Result;
use futures::stream::StreamExt;
use tokio::sync::Mutex;
use url::Url;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::Endpoint, Request};

use copybot_ultimate_v2::rpc::geyser::geyser::geyser_client::GeyserClient;
use copybot_ultimate_v2::rpc::geyser::geyser::{
    subscribe_update::UpdateOneof, SubscribeRequest,
    SubscribeRequestFilterAccounts, SubscribeRequestFilterTransactions,
};

// Hardcoded wallet to track
const TRACKED_WALLET: &str = "testwallet";

// Endpoints
const GRPC_URL: &str = "http://127.0.0.1:10000";
const RPC_WS_URL: &str = "ws://127.0.0.1:8900";

#[tokio::main]
async fn main() -> Result<()> {
    let start = Instant::now();
    println!("🚀 TRUE Mempool vs Processed Detection Test");
    println!("📡 Tracking wallet: {}", TRACKED_WALLET);
    println!("🔗 Yellowstone gRPC: {}", GRPC_URL);
    println!("🔗 Standard RPC WS: {}", RPC_WS_URL);
    println!("⏱️  Test started at t=0");
    println!("🔍 Waiting for transaction...\n");

    // Shared state to track detection times and transaction signatures
    let mempool_time = Arc::new(Mutex::new(None::<u128>));
    let processed_time = Arc::new(Mutex::new(None::<u128>));
    let detected_signatures = Arc::new(Mutex::new(HashMap::<String, bool>::new()));

    // Clone for tasks
    let mempool_time_clone = mempool_time.clone();
    let processed_time_clone = processed_time.clone();
    let signatures_clone1 = detected_signatures.clone();
    let signatures_clone2 = detected_signatures.clone();

    // Spawn TRUE mempool monitor using Yellowstone gRPC account updates
    let mempool_handle = tokio::spawn(async move {
        if let Err(e) = monitor_mempool_yellowstone_grpc(start, mempool_time_clone, signatures_clone1).await {
            println!("❌ Mempool monitor error: {}", e);
        }
    });

    // Spawn processed monitor using standard RPC WebSocket
    let processed_handle = tokio::spawn(async move {
        if let Err(e) = monitor_processed_rpc(start, processed_time_clone, signatures_clone2).await {
            println!("❌ Processed monitor error: {}", e);
        }
    });

    // Wait for both to complete or timeout after 60 seconds
    tokio::select! {
        _ = async {
            let _ = tokio::join!(mempool_handle, processed_handle);
        } => {},
        _ = tokio::time::sleep(tokio::time::Duration::from_secs(60)) => {
            println!("⏰ Test timed out after 60 seconds");
        }
    }

    // Print results
    let mempool_result = mempool_time.lock().await;
    let processed_result = processed_time.lock().await;

    println!("\n📊 RESULTS:");
    match (*mempool_result, *processed_result) {
        (Some(mempool_ms), Some(processed_ms)) => {
            println!("🚀 MEMPOOL (Yellowstone) detected at: {} ms", mempool_ms);
            println!("✅ PROCESSED (RPC) detected at: {} ms", processed_ms);
            if mempool_ms < processed_ms {
                println!("⚡ Mempool was {} ms faster!", processed_ms - mempool_ms);
                println!("💡 This is the TRUE advantage of mempool monitoring!");
            } else if processed_ms < mempool_ms {
                println!("🤔 Processed was {} ms faster (unusual)", mempool_ms - processed_ms);
            } else {
                println!("🟰 Both detected at the same time");
            }
        }
        (Some(mempool_ms), None) => {
            println!("🚀 MEMPOOL (Yellowstone) detected at: {} ms", mempool_ms);
            println!("❌ PROCESSED (RPC) detection failed or timed out");
            println!("💡 Mempool-only detection shows the speed advantage!");
        }
        (None, Some(processed_ms)) => {
            println!("❌ MEMPOOL (Yellowstone) detection failed or timed out");
            println!("✅ PROCESSED (RPC) detected at: {} ms", processed_ms);
            println!("⚠️  Check Yellowstone gRPC connection");
        }
        (None, None) => {
            println!("❌ Both detections failed or timed out");
            println!("🔧 Check both Yellowstone gRPC and RPC connections");
        }
    }

    Ok(())
}

// TRUE mempool detection using Yellowstone gRPC account updates
async fn monitor_mempool_yellowstone_grpc(
    start: Instant, 
    detection_time: Arc<Mutex<Option<u128>>>,
    signatures: Arc<Mutex<HashMap<String, bool>>>
) -> Result<()> {
    println!("🔌 Connecting to Yellowstone gRPC for TRUE mempool monitoring...");
    
    let mut accounts_map = HashMap::new();
    let mut transactions_map = HashMap::new();
    
    // Subscribe to account updates for the tracked wallet (TRUE mempool)
    // This will detect balance changes immediately when transactions enter the mempool
    accounts_map.insert(
        "tracked_wallet_accounts".into(),
        SubscribeRequestFilterAccounts {
            account: vec![TRACKED_WALLET.to_string()],
            owner: vec![],
            filters: vec![],
            nonempty_txn_signature: None,
        },
    );
    
    // Also subscribe to transactions for signature extraction
    transactions_map.insert(
        "tracked_wallet_transactions".into(),
        SubscribeRequestFilterTransactions {
            account_include: vec![TRACKED_WALLET.to_string()],
            account_exclude: vec![],
            account_required: vec![],
            signature: None,
            vote: None,
            failed: None,
        },
    );

    let (req_tx, req_rx) = tokio::sync::mpsc::channel(8);
    req_tx
        .send(SubscribeRequest {
            accounts: accounts_map,
            transactions: transactions_map,
            commitment: None, // No commitment = mempool level (fastest)
            ..Default::default()
        })
        .await?;

    let channel = Endpoint::from_shared(GRPC_URL)?
        .connect()
        .await?;
    let mut client = GeyserClient::new(channel);

    let request = Request::new(ReceiverStream::new(req_rx));
    let mut stream = client.subscribe(request).await?.into_inner();
    println!("📡 Yellowstone gRPC TRUE mempool subscription active...");

    // Main event processing loop
    while let Some(update) = stream.message().await? {
        match update.update_oneof {
            Some(UpdateOneof::Account(account_update)) => {
                // Account balance changed in mempool!
                let elapsed = start.elapsed().as_millis();
                let account_key = bs58::encode(&account_update.account.unwrap().pubkey).into_string();
                
                if account_key == TRACKED_WALLET {
                    println!("🚀 MEMPOOL detected account change for {} at t={} ms", TRACKED_WALLET, elapsed);
                    
                    // Store detection time
                    *detection_time.lock().await = Some(elapsed);
                    signatures.lock().await.insert(format!("account_change_{}", elapsed), true);
                    
                    return Ok(());
                }
            }
            Some(UpdateOneof::Transaction(tx_update)) => {
                // Transaction in mempool
                let txn = tx_update.transaction.unwrap();
                
                // Extract transaction signature
                if let Some(signature_bytes) = txn.transaction
                    .as_ref()
                    .and_then(|t| t.signatures.first()) {
                    
                    let signature = bs58::encode(signature_bytes).into_string();
                    let elapsed = start.elapsed().as_millis();
                    
                    println!("🚀 MEMPOOL detected transaction {} at t={} ms", signature, elapsed);
                    
                    // Store detection time
                    *detection_time.lock().await = Some(elapsed);
                    signatures.lock().await.insert(signature, true);
                    
                    return Ok(());
                }
            }
            _ => {}
        }
    }

    Ok(())
}

// Processed detection using standard RPC WebSocket
async fn monitor_processed_rpc(
    start: Instant, 
    detection_time: Arc<Mutex<Option<u128>>>,
    signatures: Arc<Mutex<HashMap<String, bool>>>
) -> Result<()> {
    println!("🔌 Connecting to RPC WebSocket for processed monitoring...");
    
    let url = Url::parse(RPC_WS_URL)?;
    let (ws_stream, _) = tokio_tungstenite::connect_async(url).await?;
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    // Subscribe to logs for the tracked wallet with processed commitment
    let subscribe_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "logsSubscribe",
        "params": [
            {
                "mentions": [TRACKED_WALLET]
            },
            {
                "commitment": "processed"
            }
        ]
    });

    use tokio_tungstenite::tungstenite::Message;
    use futures::SinkExt;
    
    ws_sender.send(Message::Text(subscribe_request.to_string())).await?;
    println!("📡 RPC WebSocket processed subscription active...");

    while let Some(msg) = ws_receiver.next().await {
        match msg? {
            Message::Text(text) => {
                // Parse the JSON response
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                    // Check if this is a log notification (not subscription confirmation)
                    if json.get("method").and_then(|m| m.as_str()) == Some("logsNotification") {
                        if let Some(params) = json.get("params") {
                            if let Some(result) = params.get("result") {
                                if let Some(value) = result.get("value") {
                                    if let Some(signature) = value.get("signature").and_then(|s| s.as_str()) {
                                        let elapsed = start.elapsed().as_millis();
                                        println!("✅ PROCESSED detected tx {} at t={} ms", signature, elapsed);
                                        
                                        // Store detection time
                                        *detection_time.lock().await = Some(elapsed);
                                        signatures.lock().await.insert(signature.to_string(), true);
                                        
                                        return Ok(());
                                    }
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    Ok(())
}
