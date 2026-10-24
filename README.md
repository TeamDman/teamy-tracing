# teamy-tracing

Facet JSON event and span formatting for upstream `tracing-subscriber`. This is a
library: callers own subscriber installation, filters, terminal ownership and sinks.
No consumer has been changed. Public source is MPL-2.0 with the applicable upstream
MIT notice retained. This is a release-preparation checkout, not a crates.io release.
`publish = false` remains in effect pending release approval and Windows qualification.

## Installation

Use the reviewed local checkout while publication remains disabled:

```toml
[dependencies]
teamy-tracing = { path = "../teamy-tracing" }
tracing = { version = "0.1.44", default-features = false, features = ["std"] }
tracing-subscriber = { version = "0.3.23", default-features = false, features = ["fmt", "registry"] }
```

After an approved release, replace the path dependency with the published version.
`0.1.0` is the current candidate, not an announcement that it is available on
crates.io. The same public API applies to the packaged crate. Applications own
subscriber installation and may opt into upstream `env-filter` or text features.
Adding upstream `json` elsewhere can reintroduce Serde through feature unification.

## Use

```rust
use teamy_tracing::{Diagnostics, SharedWriter};
use tracing_subscriber::prelude::*;

let diagnostics = Diagnostics::default();
let writer = SharedWriter::new(Vec::<u8>::new(), diagnostics.clone());
let subscriber = tracing_subscriber::registry()
    .with(teamy_tracing::layer(writer.clone(), diagnostics.clone()));
tracing::subscriber::with_default(subscriber, || {
    let span = tracing::info_span!("import", completed = 0);
    let _entered = span.enter();
    span.record("completed", 3);
    tracing::info!(bytes_written = 1200u64, "import complete");
});
writer.flush().unwrap();
assert_eq!(diagnostics.snapshot(), Default::default());
```

For custom settings, replace the event formatter with
`FacetJson::default().with_diagnostics(diagnostics.clone()).with_target(false)`.
Timestamp implementations use upstream `FormatTime`; `without_time()` skips the
timer entirely. Upstream `Format` builder settings do not configure a replacement
`FacetJson` instance. Use `layer.set_span_events(FmtSpan::CLOSE)` on the returned
layer when close receipts are wanted.

`FacetJsonFields` implements upstream `FormatFields`, including updates.
`FacetJson` implements `FormatEvent<S, FacetJsonFields>`. These are composable with
upstream registry, filters, error layers and other upstream layers. The helper
disables ANSI and upstream plain-text internal-error fallback. Keep that fallback
disabled for NDJSON. Nothing installs a global subscriber or emits recursive logs.

See [consumer integration](docs/integration.md), the compiled examples, and the
[observable schema](docs/schema.md). `SharedWriter` is optional; an existing
whole-record writer can be passed directly if its concurrency/error policy is suitable.

## Delivery and failures

The event is fully serialized before fmt submits it. Successful records are compact
JSON objects followed by exactly one LF; string CR/LF/control characters are escaped.
Serialization errors increment `Diagnostics::formatting_errors` and discard the
event. A failed initial span serialization stores `{}` and counts the failure,
avoiding upstream's raw diagnostic fallback; failed span updates retain the prior
cache. Arbitrary panics in a producer's `Debug` implementation are not caught.

`SharedWriter` holds a mutex for the lifetime of each upstream writer request,
including all short writes performed by `write_all`. After an I/O failure it stops
later records, so they cannot be appended to an incomplete JSON tail. The first
failure and suppressed records are counted. Mutex poisoning also suppresses delivery.
Open a fresh sink explicitly after inspection; there is no implicit retry or truncation.
`flush()` is an explicit, fallible application shutdown boundary. It is not fsync,
rotation, crash durability, or interception of writes performed outside this writer.
An I/O failure can still leave a truncated final line. Tracing macros cannot return
delivery errors; inspect the typed, Facet-enabled diagnostic snapshot outside logging.
Do not call `inspect()` recursively from a writer callback: it takes the same mutex.
Its callback receives `&W`, which does not enforce read-only access for every writer
(e.g. File). Keep it observational; writes through that reference or a cloned
handle bypass the record lock/counter contract.

Multiple fmt layers using `FacetJsonFields` share upstream's initial span-field
cache. An initial span-field formatting failure is counted by the layer that
initializes that cache, not necessarily once per sink. Writer failures belong to
the supplied writer's diagnostics; event-format failures belong to the formatter's
diagnostics. Inspect all configured handles, or deliberately share one handle for
aggregate accounting. These counters are not a count of lost events or durable
records, and their independently sampled fields are not an atomic snapshot.

## Dependencies and validation

The default normal/build graph keeps upstream subscriber `json` and Facet `fast`
disabled and contains no Serde. The approved owned registry pins are:

| Import name | Registry package | Exact version |
| --- | --- | --- |
| `facet` | `teamy-facet` | `0.50.0-rc.7` |
| `facet-json` | `teamy-facet-json` | `0.50.0-rc.9` |
| `facet-format` | `teamy-facet-format` | `0.50.0-rc.9` |
| `facet-value` (tests) | `teamy-facet-value` | `0.50.0-rc.9` |

These packages share a single owned rc.7 core/reflect/macro family; matching crate
names or version numbers from upstream or Git do not imply the same Rust type
identity. `DiagnosticSnapshot` implements the registry `teamy-facet` trait. A
consumer serializing it can use:

```toml
facet-json = { package = "teamy-facet-json", version = "=0.50.0-rc.9", default-features = false }
```

Then call `facet_json::to_string(&diagnostics.snapshot())`. Consumers deriving their
own compatible types also need `facet = { package = "teamy-facet", version =
"=0.50.0-rc.7" }`. The earlier Git identity is not interchangeable. Both lockfiles
record registry checksums; release preparation requalifies this dependency change
rather than assuming the old Git measurements still apply.

`validation/upstream-comparison` is a separate workspace with its own lockfile.
Only it enables upstream JSON for comparison. Its external Serde dependencies are
intentional and cannot be selected through a feature of this library. Do not use
that graph as the library runtime dependency report. `facet-value`, `tracing-error`
and EnvFilter are library dev dependencies for behavioral validation.

Maintainer scripts are **checkout-only**: run `scripts/check.ps1` on Windows or
`sh scripts/check.sh` on Linux with the documented toolchains and cached dependencies.
They install nothing and use offline, locked Cargo. The scripts and isolated
comparison workspace are excluded from the crate archive. From an extracted crate,
run `cargo test --offline --locked`, both examples, `cargo clippy --offline --locked
--all-targets --no-deps -- -D warnings`, and `cargo doc --offline --locked --no-deps`.
Populate dependency caches before requesting offline validation.

[Candidate/package review](docs/candidate-review.md) records the completed independent
review, cache-validation correction, 36 behavioral tests and exact remaining gates.
[Linux lifecycle qualification](docs/linux-lifecycle-qualification.md) adds cancellation
boundary and failure-isolation coverage (33 behavioral tests).
[Registry release preparation](docs/release-preparation.md) records the graph checks,
package verification and remaining release gates. [Library acceptance](docs/library-acceptance.md)
and [Linux qualification](docs/linux-qualification.md) retain historical Git-baseline
coverage and measurements. The startup adapter is illustrative, not qualified
against the unavailable newer template formatter. Native Tracy/cancellation
integration, real-consumer acceptance and a Windows rerun remain separate.

## Provenance and licensing

TeamyStyle source: adjacent `teamy-docs` over revision
`50e89d6897ce2a65167ef65b17c5fa83fe849158`, including `teamystyle.md`,
`code-standards.md`, `logging.md` and `teamystyle-template-audit.md`.
One file per type, supplied writers, scoped configuration, typed diagnostics,
coherent Facet sources and meaningful acceptance fixtures apply here. A CLI,
Figue parser, Windows resources and CLI build metadata do not apply to this library.

Upstream behavior and the adapted byte formatter are attributed in
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md). The upstream MIT notice must remain
with adapted code. Facet declares MIT OR Apache-2.0. First-party source is MPL-2.0;
public repository publication is approved. Package publication and consumer adoption
remain separate decisions. See [handoff](docs/handoff.md) for unfinished work.
