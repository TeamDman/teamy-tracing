# Linux library acceptance — 23 October 2026

This bounded pass starts at `e539d15e4f9fa1fe6bfe761095fbda47440d918e`.
It completes acceptance of the library and its proposed startup adapter against the
available source and documented sink contract. Consumer integration and Windows
qualification remain separate. No production library code, dependency manifest,
lockfile, consumer repository, or package-publication setting changed in this pass.

## Reference boundary

The provisioned teamy-rust-cli checkout and verified public `main` both resolve to
`0dfae8080f780d93bb5b5606050237b6a7b7d478`. Its `src/lib.rs` defines the rich
version/build string; `src/logging_init.rs` configures file/line metadata, upstream
formatting layers, cancellation and optional Tracy. This revision has no typed
startup formatter, argv payload or independent file filter. It still uses shared
filters and truncating file creation. Those historical behaviors are not copied.

The independent-sink contract comes from teamy-docs
`7fe98c909c7f61b50264930a0d7e0358edf950bb`, `src/logging.md`, which explicitly labels
the newer template behavior as local, unpublished work: per-sink filters, file
append, file-only close receipts, no global filter and regex-disabled EnvFilter.
A specific newer startup formatter revision was requested but was not available
for this pass. Exact compatibility with that unseen formatter is **not qualified**.
The remaining prerequisite is its reviewed source/schema; do not infer it from the
proposed fixture's field names. No consumer source was modified or fetched into its
checkout, and no replacement consumer dependency graph was resolved.

## Accepted fixtures

The runnable `typed_startup` example now preserves typed `argv`, a startup message,
and the caller's rich version/build string alongside timestamp, level, target,
filename and line number. Values are synthetic, including empty argv entries,
quotes, backslashes, Unicode and embedded CR/LF. The adapter decodes a known Facet
type and serializes it before handing any output to upstream fmt. It never splices
producer JSON. Its reserved target is `example::startup`; this is an example
protocol, not a claim about a consumer's actual target or schema.

`tests/startup_contract.rs` adds four acceptance tests:

- Two NDJSON sinks with terminal/file filter pairs off/debug, warn/debug, info/off
  and info/debug. Typed startup fields survive; debug progress remains file-only;
  close receipts are enabled only on the file layer; updated u64::MAX span fields
  retain their exact lexemes. A tracing-error layer composes with both sinks.
- A plain-text terminal alongside typed NDJSON, with an independently supplied
  result writer. Diagnostics do not contaminate that result buffer. This is a
  library fixture, not subprocess proof of a consumer's stdout/stderr routing.
- Malformed, incomplete, wrongly typed and missing reserved payloads drop the
  entire event. An ordinary event containing a non-JSON string still uses fallback.
- Startup and fallback timer failures each drop the record and are counted once
  by the owning formatter.

The example's startup error counter is separate from library Diagnostics because
that API intentionally exposes no counter mutation. Applications must inspect both.
Ordinary events use FacetJson with the same timer and source options and preserve
its span handling and diagnostics. The example emits startup before application
spans; it does not implement a general custom envelope preserving context on the
reserved target. Missing source metadata is represented as null. It remains a
reviewable adapter, not a drop-in consumer migration.

## Linux checks

`sh scripts/check.sh` passed end to end under Rust 1.96.0 and pinned
nightly-2026-05-24 rustfmt, using locked offline Cargo operations:

- 28 behavioral tests: 14 failure/composition, 10 formatting, 4 startup acceptance.
- Two isolated upstream comparisons and both executable examples.
- Formatting and strict all-target Clippy in both workspaces.
- No Serde in the library normal/build graph; separate normal/build/dev graph
  inspections also passed. Upstream JSON remains confined to the comparison
  workspace. All existing scalar, collision, cache-update and writer tests pass.

No Windows test was run. MPL-2.0, the upstream MIT notice and `publish = false`
remain intact. Only the approved implementation branch is a publication target.

## Update cost and rejected optimization

The focused release profile now parses runtime-owned input and adds a sixteen-field
case. Two-field parsing measured 1444.31 ns median; sixteen-field parsing measured
9230.14 ns. Reflection encoding of the two-field map measured 404.94 ns. These are
operation timings, not CPU stack sampling or allocation counts. The reflective
encoding case is a historical comparison operation; production already directly
emits the map through Facet's serializer API.

Borrowing raw values and keys appeared faster on static input (about 13% for the
two-field parse), but it cannot safely apply to live caches at the retained pin:
Facet's `facet-core/src/impls/alloc/btreemap.rs` implementation requires `K` and `V`
to be `'static`. Switching production to `from_str_borrowed` correctly failed to
compile for runtime-owned strings. The experiment was discarded, with no unsafe
lifetime extension, custom JSON parser, dependency upgrade or cache redesign.
No new production speedup is claimed. A further update optimization needs a
separate reviewed design or dependency capability, not a static-input benchmark.

## Reproducible comparison

The unchanged formatting benchmark was run for three sequential baseline/final
pairs on this same Linux host. The baseline executable was retained from the
starting checkpoint, and the final executable was rebuilt with locked offline
Cargo. They have the **identical SHA-256**:
`67f67978cd4649abe006aa3264af1425bf1611e369ebd07809ad82dc01271a8a`.
Differences below are run variation, not an implementation improvement.

Both formatters use io::sink, no timestamp/target, the same workloads and per-layer
EnvFilter, 1,000 warm-up iterations, 20,000 measured iterations, seven samples and
alternating formatter order. Values are medians of three per-run medians in ns.

| Workload | Starting checkpoint | Final | Final upstream | Final / upstream |
| --- | ---: | ---: | ---: | ---: |
| Events | 480.67 | 482.26 | 577.07 | 0.84 |
| Three reused spans | 722.77 | 726.81 | 1887.74 | 0.39 |
| Update plus event | 4534.27 | 4475.73 | 2038.97 | 2.20 |
| Updates only | 1866.67 | 1855.35 | 466.57 | 3.98 |
| Create span, emit once, close | 2425.92 | 2389.70 | 1589.18 | 1.50 |

All summaries are retained in [measurements](measurements/linux-acceptance-2026-10-23/).
Run `cargo bench --offline --locked --manifest-path
validation/upstream-comparison/Cargo.toml --bench formatting` and the corresponding
`--bench cache_costs` command. The earlier [optimization receipt](linux-qualification.md)
compares against the original `1da6902` source; its improvements remain historical
and are not counted again as gains from this acceptance pass. Update-heavy workloads
still trail upstream. These are microbenchmarks, not real file/application timings.

## Remaining integration prerequisites

Cancellation/Tracy composition was **not attempted** in the tracing workspaces:
neither manifest or lock contains teamy-cancellation or tracing-tracy. The reference
consumer lock selects teamy-cancellation 0.6.0, tracing-tracy 0.11.4 and tracy-client
0.18.4; their sources are cached, but cache presence does not add them to this
library's declared graph. A compile/runtime composition fixture therefore requires
reviewed dependency declarations, feature choices and a corresponding isolated
lockfile change, which this pass prohibits. No packages were fetched or installed.
Native runtime/capture behavior would then need its own test; compilation alone
would not establish it.

For consumer acceptance, supply the exact newer startup formatter, check actual
CLI startup/result/log-file/cancellation behavior and rerun relevant Windows tests.
The bounded Linux library milestone is complete; it does not authorize or claim
consumer cutover, native profiler validation, a default-branch merge or release.
