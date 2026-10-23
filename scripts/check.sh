#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
rustup run nightly-2026-05-24 cargo fmt --all -- --check
cargo clippy --offline --locked --all-targets --no-deps -- -D warnings
cargo test --offline --locked
cargo run --offline --locked --example independent_sinks
cargo run --offline --locked --example typed_startup
cargo test --offline --locked --manifest-path validation/upstream-comparison/Cargo.toml
cargo tree --offline --locked -e normal,build --prefix none --format '{p}'
