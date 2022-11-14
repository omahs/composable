set -o errexit
cargo +nightly test
RUST_BACKTRACE=full cosmwasm-check ./target/wasm32-unknown-unknown/release/xcm_executor.wasm