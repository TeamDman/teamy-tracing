# Consumer integration proposal

No consumer has been changed. First qualify a reviewed consumer checkout separately
from active Bend/Vox work. Keep `tracing-subscriber` from crates.io, with
`default-features = false` and the consumer's `fmt`, `registry`, `env-filter`,
`ansi` requirements; remove `json` only when no remaining code calls `.json()`,
`Json`, `JsonFields`, or a wrapper that still delegates to them.

Use `teamy_tracing::layer(writer, diagnostics)` for NDJSON. Reuse upstream text
layers and per-layer filters. Put formatter options on `FacetJson`, because replacing
upstream `Format` does not transfer its target/time/source configuration. After
construction, `set_span_events(FmtSpan::CLOSE)` enables the file's timing receipts.
The caller still owns filter validation, opening append-mode files, file paths,
terminal detection, startup and shutdown. `SharedWriter<File>` locks actual writes
rather than just cloning a file handle.

The compiled `examples/independent_sinks.rs` demonstrates an independently filtered
file with close receipts alongside ordinary stderr. Its in-memory sink keeps the
example inert; supply the already-opened file in a real application.

`examples/typed_startup` demonstrates retaining typed `argv`, a startup message and
caller-supplied rich version/build identity through a custom formatter. Timestamp,
level, target and source metadata are retained; ordinary events delegate to
FacetJson. It decodes a known Facet schema and reserializes it instead of splicing
producer JSON. Inspect its separate startup-error counter alongside Diagnostics.
The fixture emits startup before spans and is not a general context-preserving
custom envelope. Its schema/target are proposed: the available template revision
has no typed startup formatter. Obtain the exact newer source before claiming
consumer compatibility. See [library acceptance](library-acceptance.md) for the
four executable adapter/sink tests and remaining prerequisites.

Keep all directly referenced Facet crates on the identical Git source reference,
currently `rev = "23ad4dcd0"`, resolved to full commit
`23ad4dcd034d123f0c38943305ffb077519040ce`. Different `rev` URL spellings or registry
releases can create distinct Facet identities even when names/versions look alike.
`DiagnosticSnapshot` derives the pinned Facet, so downstream serialization must use
that compatible source. This is not permission to upgrade profiler's older Facet.

The existing cancellation, tracing-error and Tracy layers retain upstream Layer
identity. The local behavior tests exercise tracing-error and independent filters;
native Tracy runtime/capture and cancellation integration remain consumer acceptance
work. A compile-only external composition fixture is suitable before launching any
native capture. Never equate a type check with runtime profiler validation.

Before adoption check default/Tracy runtime graphs separately from build/dev graphs,
run the consumer's actual CLI fixtures (typed startup payload, stdout, stderr,
file append, filtering, close receipts, failure and cancellation), and compare a
representative release workload. `teamy-profiler` can retain NDJSON beside the Tracy
capture, but its CPU summaries come from `.tracy`; JSON formatting is a separate cost.


## Checkpoint adoption gate

The public implementation branch is source-only (`publish = false`). Pin an audited
full commit after review; retain the existing upstream subscriber package identity
and identical owned Facet source URL/revision. The formatter/layer API is unchanged
by the span cache optimization. Never manipulate its private validated cache; if an
adapter edits upstream FormattedFields, the next event revalidates the changed bytes.

Before rollout, inspect the sink's DiagnosticSnapshot outside tracing, flush on both
normal and cancellation shutdown, and treat flush/write failures as incomplete
logging. Do not call writer inspect/flush or emit tracing from inside its callbacks.
Reopening an existing file after a partial write requires caller reconciliation of
the truncated tail; opening a fresh SharedWriter alone does not repair that file.

Review the [Linux measurements](linux-qualification.md): reused spans improved,
but update-heavy and short-lived spans still trail upstream. Benchmark the consumer's
actual schema and sinks, account for two extra cached strings per rendered live span,
and rerun Windows checks where applicable. Native Tracy/cancellation and the real
startup payload remain unqualified; obtain parent review before adding integration
dependencies. The existing examples are guidance, not a completed consumer cutover.
