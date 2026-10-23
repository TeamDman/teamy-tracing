# Observable JSON contract

Default event keys are `timestamp`, uppercase `level`, nested `fields`, `target`,
and (when applicable) current `span` and root-to-leaf `spans`. Source and thread
keys are opt-in: `filename`, numeric `line_number`, `threadName`, `threadId`.
An unnamed thread falls back to the formatted ID under `threadName` only when
thread names are enabled and a separate ID was not requested.

The schema is upstream-compatible for ordinary contextual events, not guaranteed
byte-identical. Structural object ordering is not a consumer contract. Default
event fields retain recording order; span fields use sorted keys and last update
wins. Empty/unrecorded fields are absent until recorded; span metadata `name` is
written after recorded fields. A producer field named `name` can therefore create
duplicate span keys, matching upstream. Updating an undeclared span field remains
tracing's responsibility and does not introduce a new field declaration.

| Field input | Encoding |
| --- | --- |
| i64/u64, bool | JSON integer/boolean, no conversion through dynamic Value |
| f64 | JSON number; integral floats retain `.0`, including `-0.0`; NaN and infinities become null |
| i128/u128 | Decimal string, matching upstream 0.3.23's visitor fallback |
| string | Escaped JSON string, no interpretation as raw JSON |
| `%Display`, `?Debug` | Escaped rendered string; formatting errors are counted |
| `dyn Error` | Display string, without invented source-chain arrays |
| bytes in event | Lowercase hex debug string, e.g. `"[00 ff]"` |
| bytes in span | JSON byte array, e.g. `[0,255]`, matching upstream's different visitor |

Facet standard mode supplies scalar encoding. A small normalization retains `.0`
for integral floats, because default Facet otherwise emits integer-looking tokens.
Finite float spelling, especially exponents, may differ from upstream's zmij output;
boundary tests check exact f64 round trips. Span caches preserve validated JSON
lexemes rather than round-tripping through Facet Value's coercive dynamic numbers.

Flattening moves event fields into the top-level object and preserves duplicates
and stream order: built-in timestamp/level, event fields, then target/source/context.
Consumers differ in duplicate-key handling. Keep nested mode when producers can
use metadata names; no silent collision resolution is performed.

Two intentional differences from upstream 0.3.23:

1. Explicit event parent selection governs both `span` and `spans`, and explicit
   root events inherit neither. Upstream's span-list path uses the current scope
   in some explicit-parent cases. This library uses `FmtContext::event_scope()`.
2. Malformed/multiline span caches fail the whole event with a counter rather than
   panicking in debug or adding upstream's schema-dependent `field_error` fallback.

`log` facade normalization and unstable `valuable` support are not implemented.
This matches the inspected consumers' disabled tracing-log/valuable feature paths,
but is not full upstream feature parity. Typed application payloads should use a
typed custom event wrapper, as in the startup example, rather than unchecked JSON.

Span close events remain upstream generated, including string `time.busy` and
`time.idle`. Filters must admit the span level. This library does not invent new
timing units, profiler schemas, progress counters, or lifecycle events.
