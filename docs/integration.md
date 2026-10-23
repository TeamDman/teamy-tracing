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

`examples/typed_startup` demonstrates retaining typed `argv` as an array through a
custom `FormatEvent<S, FacetJsonFields>`, delegating ordinary events to `FacetJson`.
It decodes the reserved field through a known Facet type and reserializes it; it
does not splice producer JSON. A real template adapter must also preserve its
timestamp/source fields and account for wrapper failures in its diagnostic policy.
The example is intentionally a small schema example, not a drop-in template patch.

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
