# teamy-tracing

Facet JSON event and span formatting for upstream `tracing-subscriber`. This is a
library: callers own subscriber installation, filters, terminal ownership and sinks.
No consumer has been changed. Public source is MPL-2.0 with the applicable upstream
MIT notice retained. This is an initial implementation checkpoint;
`publish = false` keeps crates.io publication disabled.

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

## Dependencies and validation

The normal library graph keeps upstream subscriber `json` disabled. All Facet
crates use the same source reference `rev = "23ad4dcd0"`, matching existing Teamy
consumers; the lockfile records full commit
`23ad4dcd034d123f0c38943305ffb077519040ce`. The Facet `fast` feature is disabled.
`facet` supplies reflection for diagnostic snapshots, `facet-json` supplies JSON
encoding/validated cache parsing, and `facet-format` supplies dynamic scalar APIs.
No first-party source calls Serde. Another dependency can still unify subscriber
`json` back in; inspect each consumer's resolved graph.

`validation/upstream-comparison` is a separate workspace with its own lockfile.
Only it enables upstream JSON for comparison. Its external Serde dependencies are
intentional and cannot be selected through a feature of this library. Do not use
that graph as the library runtime dependency report. `facet-value`, `tracing-error`
and EnvFilter are library dev dependencies for behavioral validation.

Run `scripts/check.ps1` on Windows or `sh scripts/check.sh` on Linux, with the
documented toolchains and cached dependencies. The scripts install nothing and use
offline, locked resolution. [Library acceptance](docs/library-acceptance.md) records 28 behavioral tests, the
proposed typed startup adapter and current measurement/integration limits.
[Linux qualification](docs/linux-qualification.md) retains the earlier optimization
and repeatable before/after measurements. [Validation](docs/validation.md) records coverage and
measurement limits; [cloud proposal](docs/cloud-setup.md) retains the original
Linux environment proposal with `teamy-docs` beside the repository.

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
