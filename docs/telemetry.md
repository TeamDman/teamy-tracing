# Optional telemetry preview

The `telemetry` feature exposes `TelemetryRecord`, `TelemetryValue`, `Attribute`,
`RecordStore`, `StoreLimits` and `StoreStats`. Values preserve OTLP primitive,
array, map, byte and empty values, including unrecognized attribute keys. The
record carries resource/scope attributes, timestamps, trace and span ID bytes,
severity, flags and upstream dropped-attribute counts. Facet serializes the
model; the caller chooses and owns any file or console sink. No received record
is re-emitted through `tracing` or an exporter. The existing JSON formatter and
`SharedWriter` do not automatically accept these records.

The default store keeps at most 1,000 records and 8 MiB of *logical record size*.
This accounting is not a process-memory or allocation bound. Old records are
evicted first; a record too large for the configured budget or a zero-record
store rejects it. Accepted, evicted, rejected and upstream `output_truncated`
counts are cumulative, including after `clear`. The store is not internally
synchronized; applications choose synchronization, delivery and repaint policy.
`clear` removes records from the store but is not secure memory erasure. No
persistence, rotation, redaction or replay is implemented. Treat all fields as
potentially sensitive, including tool arguments and output.

`CorrelationKey::from_record` requires `service.instance.id` on the resource
and `conversation.id` plus `call_id` on the event. `matching_call` returns only
exact matches. Many real events may lack an instance ID; they remain
uncorrelated rather than acquiring a fabricated producer. Producer-specific
correlation and presentation remain application work.

`otlp::ingest_protobuf` decodes a bounded `ExportLogsServiceRequest` with the
OpenTelemetry-generated protobuf types and returns accepted/rejected counts.
Malformed or oversized wire requests insert no records. Unknown *attribute
keys* survive; unknown protobuf fields are not retained by the generated
decoder. Only OTLP logs over protobuf are supported, not OTLP JSON, gRPC, traces
or metrics. The caller must select a request limit and implement transport and
acknowledgement if it uses this adapter directly.

`otlp-http::Receiver::bind_loopback` binds **only** IPv4 `127.0.0.1` and exposes
`POST /v1/logs` for `application/x-protobuf`. The caller supplies its Tokio
runtime and `Arc<Mutex<RecordStore>>`, chooses a port and explicitly awaits
shutdown. Defaults cap each body at 1 MiB, allow eight in-flight handler bodies,
and time out body reads after ten seconds. The server reports invalid data,
unsupported content, oversized requests and admission saturation via HTTP
status; rejected individual records produce an OTLP partial-success response.
The store reports evictions separately, and a successful acknowledgement means
memory admission, not durable storage. Some senders do not inspect partial
success. Limits do not bound open TCP connections or all decoder/allocator
overhead; the listener is not authenticated against other local processes and
is not intended as an exposed network collector. Receiver diagnostics do not
include incoming content. No listener is started automatically.

The `egui` feature supplies `RecordView`, a filterable, escaped, expandable
record view. It has no eframe dependency or window lifecycle. It currently scans
the retained records and formats matching details rather than virtualizing a
large history; it is not yet a correlated Codex tool-call or PowerShell viewer.
The caller owns the UI state and repaint. Captured commands are displayed as
data, never executed.

The synthetic tests cover Facet round trips, protobuf decoding, limits, loss,
correlation, filtering, a headless egui frame and local HTTP. They do not test
real Codex telemetry, offline sender behavior, capture fidelity or a live user
configuration. The local socket test requires an environment that permits
loopback binds.
