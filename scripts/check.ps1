$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
Push-Location $root
try {
    & rustup run nightly-2026-05-24 cargo fmt --all -- --check
    if ($LASTEXITCODE -ne 0) { throw 'Formatting failed' }
    & cargo clippy --offline --locked --all-targets --no-deps -- -D warnings
    if ($LASTEXITCODE -ne 0) { throw 'Clippy failed' }
    & cargo test --offline --locked
    if ($LASTEXITCODE -ne 0) { throw 'Tests failed' }
    & cargo run --offline --locked --example independent_sinks
    if ($LASTEXITCODE -ne 0) { throw 'Sink example failed' }
    & cargo run --offline --locked --example typed_startup
    if ($LASTEXITCODE -ne 0) { throw 'Typed example failed' }
    & cargo test --offline --locked --manifest-path validation/upstream-comparison/Cargo.toml
    if ($LASTEXITCODE -ne 0) { throw 'Comparison failed' }
    & cargo tree --offline --locked -e normal,build --prefix none --format '{p}'
    if ($LASTEXITCODE -ne 0) { throw 'Graph inspection failed' }
} finally { Pop-Location }
