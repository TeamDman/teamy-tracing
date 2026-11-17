# Consumer integration

Keep `tracing-subscriber` from crates.io, with
`default-features = false` and the consumer's `fmt`, `registry`, `env-filter`,
`ansi` requirements; remove `json` only when no remaining code calls `.json()`,
`Json`, `JsonFields`, or a wrapper that still delegates to them.

Use `teamy_tracing::json_layer(writer, diagnostics)` for NDJSON. Reuse upstream text
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

Use the exact owned registry family: `facet` aliases `teamy-facet =0.50.0-rc.10`;
JSON uses `teamy-facet-json =0.50.0-rc.12`; Format and Value use
their `teamy-` packages at `=0.50.0-rc.11`. Both workspaces select
Core/Reflect 0.50.0-rc.8 and unchanged derive macros at rc.9.
`DiagnosticSnapshot` implements that Facet identity; old Core, Git, and upstream
identities must not remain in the same interacting consumer graph.

The new path proxy borrows UTF-8 text rather than cloning it; its holder and
generic map-key output can still allocate. Low-level `OwnedPeek::as_peek()`
callers must move their inspection into `with_peek` callbacks and return owned
results. Invariant and contravariant proxy shapes are rejected. These are
reflection-family changes; the tracing layer API is unchanged. Consumer
adoption still requires its own checks.

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

For an unpublished checkout, pin an audited full commit after review. For the
registry release, use the exact published `teamy-tracing` version. Retain the
existing upstream subscriber package identity and approved exact owned registry
dependency family. The formatter/layer API is unchanged by the span cache
optimization. Never manipulate its private validated cache; if an adapter edits
upstream FormattedFields, the next event revalidates the changed bytes.

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

## Cooperative shutdown boundary

Cancellation belongs to the application. Check it at safe worker boundaries;
teamy-tracing does not accept a token or interrupt an underlying synchronous write.
Stop and join producers, then drop entered guards and remaining span handles so any
requested close receipts are emitted, and only then flush each sink explicitly.
A successful flush does not stop other owners from emitting later events. Do not
flush from a writer callback or while holding that writer's guard.

Treat a failed shutdown flush as incomplete delivery and inspect that sink's
Diagnostics outside tracing. Separate diagnostic handles let one failed sink be
distinguished from another healthy one; include any custom startup-adapter counter
as well. The [Linux lifecycle fixture](linux-lifecycle-qualification.md) exercises
caller-owned cooperative cancellation during an in-flight short write, close
receipts and shutdown failures. It does not qualify a native cancellation library,
signals, async task cancellation or the real consumer's shutdown path.


## Counter attribution across layers

Upstream stores one `FormattedFields<FacetJsonFields>` extension per span. Multiple
fmt layers using this same field-formatter type reuse its initial cache; an initial
span-field failure is counted only by the layer that initializes it. The remaining
layers can emit the valid empty-context fallback without another error increment.
Do not assume one span-format error per sink. Record updates can invoke each layer's
field formatter; counters reflect formatter invocations, not unique spans/events.

Writer failures and suppressed writer requests are attributed to the Diagnostics
handle supplied to that SharedWriter. Event formatter failures use the formatter's
handle; custom adapters may have their own counters. Inspect all handles or use an
intentionally shared aggregate handle. Separate writer counters still identify a
failed sink, as tested, but a zero formatting counter on one layer does not prove
that shared span context was originally captured without error. Snapshot fields
are sampled independently and are not durable-delivery receipts.

## Live report-aware capture

Enable `capture` for live events independently of JSON formatting. The
`eyre` feature adds retained reports and enables capture. The `egui` feature adds
reusable views and the existing telemetry model.

Choose separate stores for distinct delivery responsibilities. An unbounded
`LogBuffer::new()` can retain human output owed while a terminal is owned; a
bounded `LogBuffer::bounded(NonZeroUsize)` suits an independent debug viewer.
Clones of either buffer share its store. This is application composition, not two
collectors hidden inside a widget. Apply a separate upstream per-layer filter
for each capture layer; a viewer filter never changes capture or terminal output.

`CaptureLayer::new(&buffer, active_probe, inactive_sink)` routes each event exactly
once for that layer: capture while the probe returns true, otherwise invoke the
provided sink. An always-true probe is suitable for an independently retained
GUI history. A terminal layer can probe the application's terminal coordinator
and send inactive records through a supplied terminal writer. Neither probe nor
sink runs under a store or span-extension lock. Do not emit recursively into the
same layer from those callbacks, a formatter or a writer.

Keep the `LogBuffer` owner alive independently of its subscriber. Capture layers
only hold weak buffer references. Supplied callbacks must not strongly capture
that buffer or an owner containing it, which would reintroduce the same cycle.
A captured report may retain a SpanTrace, which
retains the dispatch and layer: a strong layer-to-buffer reference would close a
cycle through that report. Report-valued span fields are rendered to text during
recording instead of retained in span extensions, preventing a report from
owning the span that owns it. Event fields other than `message` may retain typed
reports. The stores release acknowledged or evicted reports outside locks, so
error destructors that emit new events cannot deadlock the store.

With `eyre`, construct `DiagnosticReport::new(report)` and record an event field
as `error = diagnostic.as_error()`. Its `Arc<eyre::Report>` retains typed causes,
the report handler and any sections or captured traces. `%diagnostic`,
`?diagnostic`, a `message` field or a span field is text and cannot retain that
contract. Handler installation remains the application boundary. The library
does not install color-eyre, mutate global color settings or manufacture a new
report from an error string.

Read `records_since(&mut cursor)` to snapshot and advance an absolute position.
An old cursor starts at the first surviving record after eviction or delivery.
The absolute position is a `usize` even for a bounded buffer. On 32-bit targets,
replace buffers before they reach 2^32 cumulative records.
Only call `acknowledge_through(cursor)` after successful consumption; records
arriving after the read remain available. Repeated acknowledgements are safe.
`replay_to(writer)` snapshots before formatting, propagates writer failures and
does not acknowledge anything. `formatted_records_since` uses the same terminal
formatter as immediate output. A replay receipt is neither fsync nor durability.
For shutdown, stop and join producers before the final read/write/flush boundary.

A bounded buffer limits retained **record count**, not bytes. Arbitrary report
handlers and source errors have unknown sizes. Even a small count may retain
substantial memory, and snapshots can temporarily keep evicted reports alive.
`new()` has no retention bound until acknowledged. Neither store is a durable
journal, crash-recovery mechanism, secret scrubber or secure-memory erasure API.

`DiagnosticReport` plain Display/Debug retain the complete diagnostic while
removing supported styling and escaping unsupported control sequences.
`terminal()` honors stderr color policy; `styled()` keeps permitted ANSI colors
and text emphasis independently of a terminal. The allowed styling excludes
terminal commands and concealment. Report-provided content remains potentially
sensitive: presentation control escaping is not field redaction.

### Reusable egui views

With `egui`, own `gui::LogView` next to application window state and call
`view.show(ui, &buffer)`. It has text/severity filters, context expandos and a
Follow latest control. Every frame reads the buffer's currently retained history;
it does not cache a second collector, acknowledge terminal output or flatten
reports to `TelemetryRecord`. Absolute record cursors give stable expando IDs
when older records are evicted. Use distinct egui UI scopes when displaying more
than one buffer. Filtering scans retained record metadata, field names, and
plain text including span fields. Report details render when opened. The caller
schedules repaints.

With both `egui` and `eyre`, `gui::DiagnosticReportView::new(&diagnostic).show(ui)`
displays supported report colors as selectable egui text. Copy error puts the
full plain diagnostic on the application clipboard, without terminal escapes.
`layout_job()` exposes the same styled diagnostic for another application-owned
layout. The dark inset background matches the usual color-eyre palette. The view
does not own the report or change its handler. CT keeps portal layout, repaint,
toast and account/resource policy outside these generic widgets.

The widgets use egui **0.36.2**. Match that version in applications passing
`egui::Ui` to the widgets. The standalone example owns the native window and
chooses eframe **0.36.2** with glow. Its manifest enables the library's `egui` and
`eyre` features through a local path dependency.

From the repository root, check the example or launch the preview of synthetic data.

```sh
cargo check --offline --locked --manifest-path examples/synthetic-telemetry-viewer/Cargo.toml
cargo run --locked --manifest-path examples/synthetic-telemetry-viewer/Cargo.toml
```

Maintainer scripts retain default checks and separately compile `capture`,
`eyre` and `egui`, then run all-feature Clippy, headless behavior tests and docs.
They also check the standalone example's formatting and compile it with Clippy.
The no-Serde graph rule applies to default and capture/eyre normal/build graphs.
Optional GUI dependencies may use Serde internally; that is not first-party
Serde API adoption or a claim that the all-feature graph is Serde-free. Loopback
OTLP tests remain explicitly synthetic local networking, not service validation.
