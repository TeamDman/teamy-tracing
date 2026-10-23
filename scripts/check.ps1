# Checkout-only: omitted from the crate archive with the comparison workspace.
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
Push-Location $root
try {
    & rustup run nightly-2026-05-24 cargo fmt --all -- --check
    if ($LASTEXITCODE -ne 0) { throw 'Formatting failed' }
    & rustup run nightly-2026-05-24 cargo fmt --manifest-path validation/upstream-comparison/Cargo.toml --all -- --check
    if ($LASTEXITCODE -ne 0) { throw 'Comparison formatting failed' }
    & cargo clippy --offline --locked --all-targets --no-deps -- -D warnings
    if ($LASTEXITCODE -ne 0) { throw 'Clippy failed' }
    & cargo test --offline --locked
    if ($LASTEXITCODE -ne 0) { throw 'Tests failed' }
    $previousRustdocFlags = $env:RUSTDOCFLAGS
    try {
        $env:RUSTDOCFLAGS = '-D warnings'
        & cargo doc --offline --locked --no-deps
        if ($LASTEXITCODE -ne 0) { throw 'Documentation failed' }
    } finally { $env:RUSTDOCFLAGS = $previousRustdocFlags }
    & cargo run --offline --locked --example independent_sinks
    if ($LASTEXITCODE -ne 0) { throw 'Sink example failed' }
    & cargo run --offline --locked --example typed_startup
    if ($LASTEXITCODE -ne 0) { throw 'Typed example failed' }
    & cargo test --offline --locked --manifest-path validation/upstream-comparison/Cargo.toml
    if ($LASTEXITCODE -ne 0) { throw 'Comparison failed' }
    & cargo clippy --offline --locked --manifest-path validation/upstream-comparison/Cargo.toml --all-targets --no-deps -- -D warnings
    if ($LASTEXITCODE -ne 0) { throw 'Comparison Clippy failed' }
    $graph = & cargo tree --offline --locked -e normal,build --prefix none --format '{p}'
    if ($LASTEXITCODE -ne 0) { throw 'Graph inspection failed' }
    $graph
    if ($graph -match '^(serde|serde_core|serde_json|tracing-serde) v') {
        throw 'Unexpected Serde dependency in normal/build graph'
    }
} finally { Pop-Location }
