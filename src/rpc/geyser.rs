use crate::rpc::solana_storage::solana::storage::confirmed_block;

pub mod geyser {
    include!(concat!(env!("OUT_DIR"), "/jito/geyser.rs"));
}
