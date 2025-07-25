use std::path::Path;
use std::{env, fs};

fn main() {
    let out_dir = Path::new(&env::var("OUT_DIR").unwrap()).join("jito");
    fs::create_dir_all(&out_dir).unwrap();

    tonic_build::configure()
        .build_server(true)
        .out_dir(&out_dir)
        .compile(
            &[
                "proto/geyser.proto",
                "proto/solana-storage.proto",
                "protos/jito-protos/searcher.proto",
                "protos/jito-protos/shared.proto",
                "protos/jito-protos/bundle.proto",
                "protos/jito-protos/relayer.proto",
                "protos/jito-protos/packet.proto",
                "protos/jito-protos/shredstream.proto",
            ],
            &["proto", "protos/jito-protos"],
        )
        .unwrap();

    println!("cargo:rerun-if-changed=proto/");
    println!("cargo:rerun-if-changed=protos/jito-protos/");

    let patch_paths = [
        (
            "relayer.rs",
            vec![
                (
                    "super::crate::generated::shared",
                    "crate::generated::shared",
                ),
                (
                    "super::crate::generated::packet",
                    "crate::generated::packet",
                ),
            ],
        ),
        (
            "bundle.rs",
            vec![("super::packet", "crate::generated::packet")],
        ),
        (
            "geyser.rs",
            vec![(
                "super::solana::storage::confirmed_block",
                "crate::rpc::solana_storage::solana::storage::confirmed_block",
            )],
        ),
    ];

    for (file, replacements) in patch_paths {
        let path = out_dir.join(file);
        if path.exists() {
            let mut content = fs::read_to_string(&path).unwrap();
            for (from, to) in replacements {
                content = content.replace(from, to);
            }
            fs::write(&path, content).unwrap();
        }
    }
}
