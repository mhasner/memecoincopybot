// App-specific modules
pub mod config;
pub mod dex;
pub mod jito;
pub mod positions;
pub mod rpc;
pub mod state;
pub mod strategy;
pub mod submit;
pub mod transactions;
pub mod tx;
pub mod utils;

// Explicitly structured generated submodules
pub mod generated {
    pub mod searcher {
        include!(concat!(env!("OUT_DIR"), "/jito/searcher.rs"));
    }

    pub mod shared {
        include!(concat!(env!("OUT_DIR"), "/jito/shared.rs"));
    }

    pub mod bundle {
        include!(concat!(env!("OUT_DIR"), "/jito/bundle.rs"));
    }

    pub mod relayer {
        include!(concat!(env!("OUT_DIR"), "/jito/relayer.rs"));
    }

    pub mod packet {
        include!(concat!(env!("OUT_DIR"), "/jito/packet.rs"));
    }

    pub mod shredstream {
        include!(concat!(env!("OUT_DIR"), "/jito/shredstream.rs"));
    }
}

// Solana-related types used independently — not nested under `generated`
pub mod solana_storage {
    include!(concat!(
        env!("OUT_DIR"),
        "/jito/solana.storage.confirmed_block.rs"
    ));
}

pub mod geyser {
    include!(concat!(env!("OUT_DIR"), "/jito/geyser.rs"));
}
