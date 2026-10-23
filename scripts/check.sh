#!/bin/sh
# Checkout-only: omitted from the crate archive with the comparison workspace.
set -eu
cd "$(dirname "$0")/.."
rustup run nightly-2026-05-24 cargo fmt --all -- --check
rustup run nightly-2026-05-24 cargo fmt --manifest-path validation/upstream-comparison/Cargo.toml --all -- --check
cargo clippy --offline --locked --all-targets --no-deps -- -D warnings
cargo test --offline --locked
RUSTDOCFLAGS='-D warnings' cargo doc --offline --locked --no-deps
cargo run --offline --locked --example independent_sinks
cargo run --offline --locked --example typed_startup
cargo test --offline --locked --manifest-path validation/upstream-comparison/Cargo.toml
cargo clippy --offline --locked --manifest-path validation/upstream-comparison/Cargo.toml --all-targets --no-deps -- -D warnings
graph=$(cargo tree --offline --locked -e normal,build --prefix none --format '{p}')
printf '%s\n' "$graph"
if printf '%s\n' "$graph" | grep -Eq '^(serde|serde_core|serde_json|tracing-serde) v'; then
    printf '%s\n' 'Unexpected Serde dependency in normal/build graph' >&2
    exit 1
fi
