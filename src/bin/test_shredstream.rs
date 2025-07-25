use std::time::Instant;
use anyhow::Result;
use tonic::{transport::Endpoint, Request};
use tokio_stream::StreamExt;

use copybot_ultimate_v2::generated::shredstream::{
    shredstream_proxy_client::ShredstreamProxyClient,
    SubscribeEntriesRequest,
};

// Your shred receiver endpoint
const SHRED_ENDPOINT: &str = "http://145.40.93.84:1002";

#[tokio::main]
async fn main() -> Result<()> {
    let start = Instant::now();
    println!("🚀 ShredStream Connection Test");
    println!("📡 Connecting to: {}", SHRED_ENDPOINT);
    println!("⏱️  Test started at t=0");
    println!("🔍 Attempting to connect and stream entries...\n");

    // Test basic connectivity first
    println!("🔌 Testing basic connectivity...");
    match test_connectivity().await {
        Ok(_) => println!("✅ Basic connectivity test passed"),
        Err(e) => {
            println!("❌ Basic connectivity failed: {}", e);
            println!("💡 This might be normal if authentication is required");
        }
    }

    // Try to connect to ShredStream Proxy
    println!("\n🔌 Connecting to ShredStream Proxy...");
    match connect_shredstream_proxy().await {
        Ok(_) => println!("✅ ShredStream connection successful!"),
        Err(e) => {
            println!("❌ ShredStream connection failed: {}", e);
            println!("💡 Possible reasons:");
            println!("   - Authentication required");
            println!("   - Service not running on this endpoint");
            println!("   - Different protocol/port needed");
        }
    }

    // Try alternative connection methods
    println!("\n🔍 Testing alternative connection methods...");
    test_alternative_connections().await;

    println!("\n📊 Test completed in {} ms", start.elapsed().as_millis());
    Ok(())
}

async fn test_connectivity() -> Result<()> {
    // Simple TCP connection test
    use tokio::net::TcpStream;
    use std::time::Duration;
    
    let timeout = Duration::from_secs(5);
    let addr = "145.40.93.84:1002";
    
    match tokio::time::timeout(timeout, TcpStream::connect(addr)).await {
        Ok(Ok(_stream)) => {
            println!("✅ TCP connection to {} successful", addr);
            Ok(())
        }
        Ok(Err(e)) => {
            println!("❌ TCP connection failed: {}", e);
            Err(e.into())
        }
        Err(_) => {
            println!("❌ TCP connection timed out");
            Err(anyhow::anyhow!("Connection timeout"))
        }
    }
}

async fn connect_shredstream_proxy() -> Result<()> {
    // Try to connect to ShredStream Proxy service
    let channel = Endpoint::from_shared(SHRED_ENDPOINT)?
        .timeout(std::time::Duration::from_secs(10))
        .connect()
        .await?;
    
    let mut client = ShredstreamProxyClient::new(channel);
    
    println!("📡 Connected to ShredStream Proxy, subscribing to entries...");
    
    let request = Request::new(SubscribeEntriesRequest {});
    let mut stream = client.subscribe_entries(request).await?.into_inner();
    
    println!("🎯 Listening for shred entries...");
    
    let mut entry_count = 0;
    let start = Instant::now();
    
    // Listen for entries for 30 seconds or until we get 10 entries
    while let Some(entry_result) = tokio::time::timeout(
        std::time::Duration::from_secs(30),
        stream.next()
    ).await? {
        match entry_result {
            Ok(entry) => {
                entry_count += 1;
                let elapsed = start.elapsed().as_millis();
                
                println!("🚀 Entry #{} received at t={} ms:", entry_count, elapsed);
                println!("   Slot: {}", entry.slot);
                println!("   Entries data length: {} bytes", entry.entries.len());
                
                // Try to parse the entries data
                if !entry.entries.is_empty() {
                    analyze_entry_data(&entry.entries, entry.slot);
                }
                
                if entry_count >= 10 {
                    println!("✅ Received {} entries, stopping test", entry_count);
                    break;
                }
            }
            Err(e) => {
                println!("❌ Error receiving entry: {}", e);
                break;
            }
        }
    }
    
    if entry_count == 0 {
        println!("⚠️  No entries received within 30 seconds");
    } else {
        println!("✅ Successfully received {} entries", entry_count);
    }
    
    Ok(())
}

fn analyze_entry_data(data: &[u8], slot: u64) {
    println!("🔍 Analyzing entry data for slot {}:", slot);
    println!("   Raw data length: {} bytes", data.len());
    
    if data.len() >= 8 {
        // Try to parse as Vec<Entry> length prefix
        let len_bytes = &data[0..8];
        let vec_len = u64::from_le_bytes(len_bytes.try_into().unwrap_or([0; 8]));
        println!("   Potential Vec length: {}", vec_len);
    }
    
    // Show first 32 bytes as hex for analysis
    let preview_len = std::cmp::min(32, data.len());
    let hex_preview: String = data[0..preview_len]
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join(" ");
    println!("   First {} bytes (hex): {}", preview_len, hex_preview);
    
    // Look for potential transaction signatures (64 bytes)
    if data.len() >= 64 {
        println!("   Potential signature data found");
    }
}

async fn test_alternative_connections() {
    // Test different protocols and ports
    let alternatives = vec![
        "http://145.40.93.84:1001",
        "http://145.40.93.84:1003", 
        "https://145.40.93.84:1002",
        "grpc://145.40.93.84:1002",
    ];
    
    for endpoint in alternatives {
        println!("🔍 Testing alternative endpoint: {}", endpoint);
        match test_endpoint(endpoint).await {
            Ok(_) => println!("✅ {} - Connection successful", endpoint),
            Err(e) => println!("❌ {} - Failed: {}", endpoint, e),
        }
    }
}

async fn test_endpoint(endpoint: &str) -> Result<()> {
    let endpoint_string = endpoint.to_string();
    let channel = Endpoint::from_shared(endpoint_string)?
        .timeout(std::time::Duration::from_secs(5))
        .connect()
        .await?;
    
    let mut client = ShredstreamProxyClient::new(channel);
    let request = Request::new(SubscribeEntriesRequest {});
    
    // Just try to establish the stream, don't wait for data
    let _stream = client.subscribe_entries(request).await?;
    Ok(())
}
