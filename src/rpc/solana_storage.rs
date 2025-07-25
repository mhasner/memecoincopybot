pub mod solana {
    pub mod storage {
        pub mod confirmed_block {
            include!(concat!(
                env!("OUT_DIR"),
                "/jito/solana.storage.confirmed_block.rs"
            ));
        }
    }
}
