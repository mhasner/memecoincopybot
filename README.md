# Copybot Ultimate v2

High-performance Solana trading bot designed for automated copy trading across multiple DEXs.
The system features advanced multi-wallet management allowing simultaneous tracking of multiple wallets with individual risk controls and position tracking. Its intelligent DEX router automatically detects and routes trades through the optimal exchange (PumpFun, Raydium CPMM, Raydium Launchpad, Meteora DLMM, Moonshot) based on real-time program ID analysis and liquidity conditions.

The bot exposes REST API endpoints that make it easily implementable into any frontend dashboard or trading interface, providing real-time access to trade execution metrics, position data, P\&L tracking, wallet balances, and performance analytics. Built with Rust for maximum performance, Geyser streaming for fast transaction detection, Jito bundles for MEV protection, and hybrid submission strategies combining Jito and RPC for optimal execution.

Modular architecture supporting custom strategy implementations. The system requires high-speed RPC infrastructure with private Geyser endpoints for optimal same-block execution and is designed for deployment on dedicated trading infrastructure by technical experts.

## Testingresults

Same block inclusion on 54% of executed Trades
Block after inclusion on 29% of exectued Trades
No Trade exectution on 17% of Trades detected

## Strategy


The strategy can be optimized for sure but I have found that copying wallets on very fresh mints is not profitable, hence the bot does not support it.
You will definetely need to find good wallets to track with high winrates and a good PnL.
Take Profit implementation is a placeholder because it would require constant RPC Polling and can be easily implemented. 


## 🚀 Features

- **Multi-DEX Support**: PumpFun, Raydium CPMM, Raydium Launchpad, Meteora, Moonshot
- **Smart DEX Routing**: Automatic DEX detection based on program IDs from transaction data
- **High-Speed Trading**: Jito bundle support and priority fee optimization
- **Real-time Data**: Geyser streaming for instant transaction detection
- **Position Management**: Automatic tracking of token positions and P&L
- **Advanced Transaction Building**: Deterministic pool derivation for maximum speed
- **Multiple Submission Methods**: Jito bundles, Helius fast lanes, hybrid submission
- **Comprehensive Logging**: Detailed transaction and trade logging

## 📋 Prerequisites

- **Rust**: Latest stable version (1.70+)
- **Solana CLI**: v1.16+ 
- **System**: Linux (Ubuntu 22.04+ recommended)
- **Memory**: 8GB+ RAM recommended
- **Network**: **High-speed RPC with private Geyser endpoint required** for optimal same-block execution times
- **Technical Expertise**: Requires a technical expert to implement and deploy on production servers

## 🛠 Installation

### 1. Clone the Repository

```bash
cd copybot-ultimate-v2
```

### 2. Install Dependencies

```bash
# Install Rust if not already installed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install Solana CLI
sh -c "$(curl -sSfL https://release.solana.com/v1.16.0/install)"

# Build the project
cargo build --release
```

### 3. Setup Configuration Files

```bash
# Copy example configuration
cp src/config/settings.json.example src/config/settings.json

# Create wallet configuration
cp src/config/bot_wallets.json.example src/config/bot_wallets.json
```

## ⚙️ Configuration

### Main Settings (`src/config/settings.json`)

```json
{
  "rpc_url": "https://your-rpc-endpoint.com",
  "ws_url": "wss://your-websocket-endpoint.com",
  "keypair_path": "/path/to/your/wallet.json",
  "jito": true,
  "buy_amount_sol": 0.01,
  "buy_slippage_percent": 5.0,
  "sell_slippage_percent": 5.0,
  "sell_min_sol_out": 0.005,
  "buy_bribe_sol": 0.001,
  "sell_bribe_sol": 0.001,
  "buy_priority_fee_sol": 0.0001,
  "sell_priority_fee_sol": 0.0001,
  "take_profit_percent": 100.0,
  "stop_loss_percent": -50.0
}
```

### Bot Wallets (`src/config/bot_wallets.json`)

```json
{
  "wallets": [
    {
      "address": "YourWalletAddress1...",
      "name": "Main Trading Wallet",
      "enabled": true,
      "sol_gate": 0.0005,
      "buy_amount_sol": 0.005
    }
  ]
}
```

### Environment Variables

Create a `.env` file in the project root:

```bash
# RPC Configuration
SOLANA_RPC_URL=https://your-mainnet-rpc.com
SOLANA_WS_URL=wss://your-websocket-endpoint.com

# Helius Configuration (if using Helius)
HELIUS_API_KEY=your-helius-api-key

# Jito Configuration (if using Jito)
JITO_BLOCK_ENGINE_URL=https://mainnet.block-engine.jito.wtf
JITO_RELAYER_URL=http://bundles-api-rest.jito.wtf

# Optional: Custom settings
RUST_LOG=info
```

## 📁 Project Structure

```
copybot-ultimate-v2/
├── src/
│   ├── bin/                    # Executable binaries
│   │   ├── bot.rs             # Main trading bot executable
│   │   ├── list_chats.rs      # Utility to list available chats
│   │   └── test_*.rs          # Various testing utilities
│   ├── config/                # Configuration management
│   │   ├── mod.rs             # Configuration module exports
│   │   ├── settings.rs        # Settings struct and loading logic
│   │   ├── settings.json      # Main configuration file
│   │   └── bot_wallets.json   # Wallet configuration
│   ├── dex/                   # DEX integrations
│   │   ├── mod.rs             # DEX module exports
│   │   ├── router.rs          # Smart DEX routing logic
│   │   ├── pumpfun_simplified.rs  # PumpFun DEX integration
│   │   ├── pump_amm.rs        # PumpSwap AMM for migrated tokens
│   │   ├── raydium.rs         # Raydium CPMM integration
│   │   ├── raydium_launchpad.rs   # Raydium Launchpad integration
│   │   ├── meteora.rs         # Meteora DLMM and Mercurial integration
│   │   ├── moonshot.rs        # Moonshot DEX integration
│   │   └── types.rs           # Shared DEX types
│   ├── jito/                  # Jito bundle integration
│   │   ├── mod.rs             # Jito module exports
│   │   ├── wrapper.rs         # Jito transaction wrappers
│   │   ├── tip_accounts.rs    # Jito tip account management
│   │   └── bundle_builder.rs  # Jito bundle construction
│   ├── positions/             # Position management
│   │   ├── mod.rs             # Position tracking and P&L calculation
│   │   └── positions.json     # Stored position data
│   ├── rpc/                   # RPC and streaming data
│   │   ├── mod.rs             # RPC module exports
│   │   ├── geyser.rs          # Geyser streaming client
│   │   └── geyser_listener.rs # Geyser event processing
│   ├── state/                 # Application state management
│   │   └── mod.rs             # State module (re-exports positions)
│   ├── strategy/              # Trading strategies
│   │   ├── mod.rs             # Strategy types and exports
│   │   ├── engine.rs          # Main strategy execution engine
│   │   ├── follow_buy.rs      # Copy trading buy logic
│   │   ├── follow_sell.rs     # Copy trading sell logic
│   │   └── take_profit.rs     # Take profit strategy
│   ├── submit/                # Transaction submission
│   │   ├── mod.rs             # Submission module exports
│   │   ├── iface.rs           # Submitter interface
│   │   ├── jito_bundle.rs     # Jito bundle submission
│   │   ├── helius_fast.rs     # Helius fast lane submission
│   │   ├── helius_tips.rs     # Helius tip account management
│   │   └── hybrid.rs          # Hybrid submission (Jito + Helius)
│   ├── transactions/          # Transaction building utilities
│   │   ├── mod.rs             # Transaction module exports
│   │   ├── create_transaction.rs  # Generic transaction creation
│   │   ├── meteora_swap.rs    # Meteora-specific transactions
│   │   ├── raydium_swap.rs    # Raydium-specific transactions
│   │   └── utils.rs           # Transaction utilities
│   ├── tx/                    # Low-level transaction utilities
│   │   ├── mod.rs             # TX module exports
│   │   ├── ata.rs             # Associated Token Account utilities
│   │   ├── ata_fast.rs        # Fast ATA creation
│   │   ├── dedupe.rs          # Transaction deduplication
│   │   ├── factory.rs         # Transaction factory
│   │   └── wrapper.rs         # Transaction wrappers
│   ├── utils/                 # Utility modules
│   │   ├── mod.rs             # Utils module exports
│   │   ├── pool_tracker.rs    # Real-time pool state tracking
│   │   ├── token_tracker.rs   # Token balance tracking
│   │   ├── transaction_cache.rs   # Transaction caching
│   │   ├── live_trades.rs     # Live trade logging
│   │   ├── fees.rs            # Fee calculation utilities
│   │   ├── timing.rs          # Timing and performance utilities
│   │   └── multi_wallet.rs    # Multi-wallet management
│   └── lib.rs                 # Library root and exports
├── target/                    # Compiled binaries and build artifacts
├── positions/                 # Position data storage
│   └── positions.json         # Current positions (auto-generated)
├── jito-rs/                   # Jito Rust SDK (submodule)
├── protos/                    # Protocol buffer definitions
├── Cargo.toml                 # Rust project configuration
├── Cargo.lock                 # Dependency lock file
├── build.rs                   # Build script for protobuf compilation
├── live_trades.jsonl          # Live trade log (auto-generated)
└── README.md                  # This file
```

## 🎯 Key Components Explained

### DEX Router (`src/dex/router.rs`)
The heart of the multi-DEX system. Automatically detects which DEX to use based on program IDs found in transaction data:

- **PumpFun**: `6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P`
- **PumpSwap AMM**: `pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA`
- **Raydium CPMM**: `CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C`
- **Meteora DLMM**: `LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo`
- **Moonshot**: `MoonCVVNZFSYkqNXP6bxHLPL6QQJiMagDL3qcqUQTrG`

### Strategy Engine (`src/strategy/engine.rs`)
Coordinates all trading activities:
- Processes incoming transaction data from Geyser
- Determines appropriate trading actions
- Manages position sizing and risk
- Handles take profit and stop loss logic

### Pool Tracker (`src/utils/pool_tracker.rs`)
Maintains real-time state of all tracked pools:
- Caches pool data for instant access
- Tracks token migrations between DEXs
- Stores bonding curve data for PumpFun tokens
- Manages creator information for fee calculations

### Transaction Submission (`src/submit/`)
Multiple submission strategies for optimal execution:
- **Jito Bundles**: For MEV protection and guaranteed execution
- **Helius Fast**: Direct submission to Helius fast lanes
- **Hybrid**: Tries Jito first, falls back to Helius

## 🚀 Usage

### Running the Main Bot

```bash
# Run in development mode with logging
RUST_LOG=info cargo run --bin bot

# Run optimized release version
cargo run --release --bin bot

# Run with specific configuration
RUST_LOG=debug cargo run --bin bot -- --config custom_settings.json
```

### Available Binaries

```bash
# Main trading bot
cargo run --bin bot

# Test various components
cargo run --bin test_mempool
cargo run --bin test_shredstream
cargo run --bin check_bundle_status
```

### Testing Individual Components

```bash
# Test PumpFun integration
cargo test pumpfun

# Test DEX router
cargo test dex_router

# Test transaction building
cargo test transactions

# Run all tests
cargo test
```

## 🔧 Configuration Details

### Trading Parameters

| Parameter | Description | Default | Range |
|-----------|-------------|---------|-------|
| `buy_amount_sol` | SOL amount per buy trade | 0.01 | 0.001-10.0 |
| `buy_slippage_percent` | Buy slippage tolerance | 5.0 | 0.1-50.0 |
| `sell_slippage_percent` | Sell slippage tolerance | 5.0 | 0.1-50.0 |
| `buy_bribe_sol` | Tip amount for buy trades | 0.001 | 0.0001-0.1 |
| `sell_bribe_sol` | Tip amount for sell trades | 0.001 | 0.0001-0.1 |
| `take_profit_percent` | Take profit threshold | 100.0 | 10.0-1000.0 |
| `stop_loss_percent` | Stop loss threshold | -50.0 | -90.0 to -5.0 |

### RPC Configuration

For optimal performance, use a high-quality RPC provider:

**Recommended Providers:**
- **Helius**: Fast, reliable, good for production
- **QuickNode**: High performance, good uptime
- **Triton**: Specialized for trading applications
- **GenesysGo**: Good balance of speed and cost

**RPC Requirements:**
- WebSocket support for real-time data
- High rate limits (1000+ requests/second)
- Low latency (<10ms to Solana validators)
- **Private Geyser endpoint required** for optimal same-block execution
- **Self-hosted RPC infrastructure recommended** for maximum performance

### Wallet Setup

1. **Generate a new wallet** (recommended for security):
```bash
solana-keygen new --outfile ~/trading-wallet.json
```

2. **Fund the wallet** with SOL for trading and fees:
```bash
# Check balance
solana balance ~/trading-wallet.json

# Airdrop on devnet (for testing)
solana airdrop 2 ~/trading-wallet.json --url devnet
```

3. **Update configuration** with wallet path:
```json
{
  "keypair_path": "/home/user/trading-wallet.json"
}
```

## 📊 Monitoring and Logging

### Log Files

- **Application Logs**: Console output with configurable levels
- **Trade Logs**: `live_trades.jsonl` - JSONL format for easy parsing
- **Position Data**: `positions/positions.json` - Current positions and P&L

### Log Levels

```bash
# Minimal logging
RUST_LOG=warn cargo run --bin bot

# Standard logging
RUST_LOG=info cargo run --bin bot

# Detailed logging
RUST_LOG=debug cargo run --bin bot

# Maximum logging
RUST_LOG=trace cargo run --bin bot
```

### Performance Monitoring

The bot includes built-in performance metrics endpoints that can be integrated into any frontend:
- Transaction build times
- Submission latencies
- Success/failure rates
- Pool detection accuracy

*Note: These are API endpoints that can be consumed by custom dashboards or monitoring frontends.*

## 🔍 Troubleshooting

### Common Issues

**1. RPC Connection Errors**
```
Error: RPC request failed
```
- Check RPC URL is correct and accessible
- Verify API key if required
- Try a different RPC provider
- Check network connectivity

**2. Insufficient SOL Balance**
```
Error: Insufficient funds for transaction
```
- Check wallet balance: `solana balance`
- Ensure enough SOL for trades + fees
- Reduce trade amounts or increase wallet balance

**3. Transaction Failures**
```
Error: Transaction simulation failed
```
- Check slippage settings (increase if needed)
- Verify pool exists and has liquidity
- Check if token is still tradeable
- Review transaction logs for specific errors

**4. Pool Detection Issues**
```
Warning: No DEX detected, falling back to PumpFun
```
- Check if token has migrated to different DEX
- Verify program IDs in router configuration
- Check Geyser connection for real-time data

**5. Build Errors**
```
Error: failed to compile
```
- Update Rust: `rustup update`
- Clean build: `cargo clean && cargo build`
- Check dependencies: `cargo check`

### Debug Mode

Enable debug mode for detailed troubleshooting:

```bash
# Enable debug logging
RUST_LOG=debug cargo run --bin bot

# Enable trace logging (very verbose)
RUST_LOG=trace cargo run --bin bot

# Debug specific modules
RUST_LOG=copybot_ultimate_v2::dex=debug cargo run --bin bot
```

### Performance Optimization

**1. System Optimization**
```bash
# Increase file descriptor limits
ulimit -n 65536

# Set CPU governor to performance
echo performance | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor
```

**2. Network Optimization**
- Use geographically close RPC providers
- Consider dedicated servers in data centers
- Monitor network latency regularly

**3. Configuration Tuning**
- Adjust retry counts based on network conditions
- Optimize tip amounts for current network congestion
- Fine-tune slippage based on token volatility

## 🔐 Security Considerations

### Wallet Security
- **Never share private keys** or wallet files
- Use **dedicated trading wallets** with limited funds
- Consider **hardware wallets** for key storage
- Regularly **rotate wallet keys** for production use

### RPC Security
- Use **authenticated RPC endpoints** when possible
- **Rotate API keys** regularly
- **Monitor usage** to detect unauthorized access
- Consider **IP whitelisting** for production

### Code Security
- **Review all configuration** before running
- **Test on devnet** before mainnet deployment
- **Monitor transactions** for unexpected behavior
- **Keep dependencies updated** for security patches

## 🚀 Advanced Usage

### Custom DEX Integration

To add support for a new DEX:

1. **Create DEX module** in `src/dex/your_dex.rs`
2. **Add program ID** to `src/dex/router.rs`
3. **Implement transaction building** functions
4. **Add to DEX router** logic
5. **Test thoroughly** on devnet

### Custom Trading Strategies

To implement custom strategies:

1. **Create strategy module** in `src/strategy/your_strategy.rs`
2. **Implement strategy logic** following existing patterns
3. **Add to strategy engine** in `src/strategy/engine.rs`
4. **Configure parameters** in settings
5. **Test with small amounts** first

### Multi-Wallet Trading

The bot supports multiple wallets for diversification:

1. **Configure wallets** in `bot_wallets.json`
2. **Set individual limits** per wallet
3. **Monitor balances** across all wallets
4. **Implement wallet rotation** logic

## 📈 Performance Metrics

The bot tracks various performance metrics:

- **Trade Success Rate**: Percentage of successful trades
- **Average Execution Time**: Time from signal to execution
- **Slippage Analysis**: Actual vs expected slippage
- **Profit/Loss Tracking**: Real-time P&L calculation
- **DEX Performance**: Success rates per DEX

## 🤝 Contributing

When contributing to the project:

1. **Follow Rust conventions** and use `cargo fmt`
2. **Add tests** for new functionality
3. **Update documentation** for changes
4. **Test thoroughly** on devnet first
5. **Consider security implications** of changes

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## ⚠️ Disclaimer

This software is for educational and research purposes only. Trading cryptocurrencies involves substantial risk of loss. The authors are not responsible for any financial losses incurred through the use of this software. Always test thoroughly on devnet before using real funds.

**IMPORTANT NOTICES:**
- **No Guarantees**: There are no guarantees of performance, profitability, or functionality
- **Technical Expertise Required**: This system requires a technical expert to properly implement and deploy on production servers
- **Self-Hosted Infrastructure**: Optimal performance requires self-hosted RPC infrastructure with private Geyser endpoints

## 📞 Support

For questions about the bot, contact "mhasner" on Discord.

---

**Happy Trading! 🚀**

