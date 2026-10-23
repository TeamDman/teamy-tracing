Latest continuation: [Linux qualification and measurements](linux-qualification.md).
The receipt below is historical Windows evidence, retained unchanged.

# Local validation receipt — 23 October 2026

Platform: Windows x86_64 MSVC, Rust/Cargo 1.96.0. All commands used cached dependencies
and offline resolution. No Linux, native Tracy capture, or real-consumer test was run.

Completed before the handoff checkpoint:

- 15 library integration tests passed: scalar/escape/schema behavior, exact float
  boundary round trips, span updates and parents, flattened collisions, source/thread
  options, concurrent short writes, partial/flush failures, malformed cache rejection,
  timer/Debug failure handling, per-sink filters, tracing-error and close receipts.
- Two isolated upstream nested/flattened schema comparison tests passed. Their
  expected values are parsed with Facet, with no first-party Serde API usage.
- Strict Clippy passed for the library, tests and examples: `--all-targets --no-deps
  -- -D warnings` using the copied Teamy method/type/macro restrictions.
- Nightly-2026-05-24 formatting checks passed for both workspaces. Both lockfiles are retained.
- Both consumer examples were run successfully (independent sinks and typed startup).
- The actual repository's default normal/build graph contained no serde,
  serde_json, tracing-serde or serde_core path. The comparison workspace is separate.

The 15 behavioral tests and two comparisons were rerun successfully after the final
code changes. Documentation/license and line-ending normalization followed. Check
commands were executed individually with an external target directory; the scripts
themselves have not been run end to end. They install nothing and use offline Cargo.

## Release microbenchmark

`cargo bench --offline --bench formatting --manifest-path
validation/upstream-comparison/Cargo.toml` measured 20,000 iterations per sample,
seven samples, alternating implementation order, after 1,000 warm-up iterations.
Both formatters used the same process, `io::sink`, no timestamps, no target metadata,
the same events/spans, and a per-layer EnvFilter. Facet's fast feature was disabled.
The comparison resolved subscriber 0.3.23 and external serde_json 1.0.151.

| Scenario | Facet median ns/iteration | Upstream median ns/iteration | Facet/upstream |
| --- | ---: | ---: | ---: |
| Event fields | 784.395 | 477.135 | 1.64 |
| Three entered spans | 6977.645 | 1983.285 | 3.52 |
| Span update plus event | 8327.600 | 2293.395 | 3.63 |

Disabled-event measurements were approximately 0.43 ns/iteration for both; the tiny
optimized filtering loop is not a useful serialization comparison. Raw samples'
min/median/max summaries are in `benchmark-windows.ndjson`. Allocation counts,
filesystem latency, tail latency, process memory, binary size and cloud performance
were not measured. No performance improvement is claimed. The current span cache
is parsed on every event and update, sometimes twice for span plus spans; this is
the first optimization candidate, not proof of its isolated share of runtime.

The recorded benchmark was run before formatting and equivalent if-let Clippy
rewrites; it includes the numeric preservation and multiline-cache fixes. A future
performance change needs a fresh benchmark on the same host/workload.
