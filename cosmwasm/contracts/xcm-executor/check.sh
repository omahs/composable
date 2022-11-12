set -o errexit
cargo +nightly test
cosmwasm-check ./target/wasm32-unknown-unknown/release/xcm_executor.wasm