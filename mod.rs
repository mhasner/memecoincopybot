pub mod config;
pub mod rpc;
pub mod transactions;
pub mod utils;

// ✅ This replaces 'mod generated'
pub mod generated {
    include!(concat!(env!("OUT_DIR"), "/jito/searcher.rs"));
    include!(concat!(env!("OUT_DIR"), "/jito/shared.rs"));
    include!(concat!(env!("OUT_DIR"), "/jito/bundle.rs"));
    include!(concat!(env!("OUT_DIR"), "/jito/relayer.rs"));
    include!(concat!(env!("OUT_DIR"), "/jito/packet.rs"));
}

// ✅ For geyser + solana-storage
pub mod solana_storage {
    include!(concat!(env!("OUT_DIR"), "/jito/solana.storage.confirmed_block.rs"));
}

pub mod geyser {
    include!(concat!(env!("OUT_DIR"), "/jito/geyser.rs"));
}