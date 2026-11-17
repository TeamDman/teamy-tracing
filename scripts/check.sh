#!/bin/sh
# Checkout-only: omitted from the crate archive with the comparison workspace.
set -eu
cd "$(dirname "$0")/.."
rustup run nightly-2026-05-24 cargo fmt --all -- --check
rustup run nightly-2026-05-24 cargo fmt --manifest-path validation/upstream-comparison/Cargo.toml --all -- --check
cargo clippy --offline --locked --all-targets --no-deps -- -D warnings
cargo test --offline --locked
for features in capture eyre egui; do
    cargo check --offline --locked --no-default-features --features "$features"
done
cargo clippy --offline --locked --all-targets --all-features --no-deps -- -D warnings
cargo test --offline --locked --all-features
RUSTDOCFLAGS='-D warnings' cargo doc --offline --locked --no-deps
RUSTDOCFLAGS='-D warnings' cargo doc --offline --locked --all-features --no-deps
cargo run --offline --locked --example independent_sinks
cargo run --offline --locked --example typed_startup
cargo test --offline --locked --manifest-path validation/upstream-comparison/Cargo.toml
cargo clippy --offline --locked --manifest-path validation/upstream-comparison/Cargo.toml --all-targets --no-deps -- -D warnings
viewer_manifest=examples/synthetic-telemetry-viewer/Cargo.toml
rustup run nightly-2026-05-24 cargo fmt --manifest-path "$viewer_manifest" --all -- --check
# Compile the standalone viewer without opening a window.
cargo clippy --offline --locked --manifest-path "$viewer_manifest" --all-targets --no-deps -- -D warnings
# Optional GUI dependencies may use Serde internally; capture remains independent.
for features in default capture,eyre; do
    if [ "$features" = default ]; then
        graph=$(cargo tree --offline --locked -e normal,build --prefix none --format '{p}')
    else
        graph=$(cargo tree --offline --locked --no-default-features --features "$features" -e normal,build --prefix none --format '{p}')
    fi
    printf '%s\n' "$graph"
    if printf '%s\n' "$graph" | grep -Eq '^(serde|serde_core|serde_json|tracing-serde) v'; then
        printf '%s\n' "Unexpected Serde dependency in normal/build graph (features: $features)" >&2
        exit 1
    fi
done
