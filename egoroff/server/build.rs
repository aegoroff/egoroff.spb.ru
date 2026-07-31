use std::env;
use std::path::PathBuf;

fn main() {
    // rust-embed and Askama bake ../../static (and friends) into the binary at
    // compile time. Without these directives Cargo may keep a Fresh `server`
    // artifact after only UI assets change (new Vite hashes), so Docker/local
    // builds can ship a stale frontend. Mirror `just local`'s `cargo clean -p server`.
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    for rel in ["../../static", "../../apache", "../../templates/apache"] {
        println!("cargo:rerun-if-changed={}", manifest_dir.join(rel).display());
    }
}
