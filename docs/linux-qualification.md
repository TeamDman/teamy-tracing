# Linux qualification and performance — 23 October 2026

This continuation starts at `1da6902fb5d7874a0f36e88ed65ede665b2341ff` on
`implementation`. Guidance was read from the provisioned teamy-docs revision
`7fe98c909c7f61b50264930a0d7e0358edf950bb` (the handoff's older reference remains
recorded). The requested `instructions/.agents/skills/teamy-docs` path was absent;
the adjacent AGENTS.md, TeamyStyle, code standards and library handoff supplied the
applicable instructions. No consumer was edited. Neither dependency lockfile nor
Facet pin changed. First-party MPL-2.0 and adapted-source MIT attribution remain.

## Implementation

`ValidatedSpan` stores the exact source cache and a validated serialized span object
in upstream span extensions. A read lock and byte equality permit reuse; a miss
acquires the write lock and rechecks current source before replacement. External
changes to public `FormattedFields<FacetJsonFields>` cannot reuse stale validation.
The full scalar lexemes survive; no conversion through dynamic Value occurs.
Only parsed JSON and Facet-generated scalars reach raw serialization. Span updates
retain the previous cache on failure, and event failures still discard the whole
record and count once. A new test found Facet accepts `[]` as an empty map: both
cache paths now explicitly require an object before decoding.

Span-field serialization emits the validated map through Facet's serializer API,
avoiding a second reflection traversal. No dependency or companion Layer is needed;
public layer and formatter types remain unchanged. Upstream subscriber identities,
explicit parents/roots and per-layer filtering remain intact.

The cache adds two owned strings per rendered live span (source plus full output),
released with that span; it has no process-global or thread-local growth. Equality
is linear in source length. Misses still allocate and parse, and serialize under
the span extension write lock. Concurrent updates can yield different versions in
`span` and `spans` within an event, as the prior separately read implementation could;
there is no whole-event atomic snapshot promise. Each rendering is validated and
corresponds to a source observed under its lock.

Writer fixes make a nonempty direct zero-byte write return `WriteZero` and stop
delivery, and prevent a guard flush failure from counting its own subsequent write
as a separately suppressed record. Empty direct writes remain successful on a
healthy sink. The existing complete-record lock and fail-stop policy are retained.

## Qualification

Host: x86_64 Linux; Rust/Cargo 1.96.0 (`ac68faa20`, LLVM 22.1.2), release defaults.
Pinned nightly-2026-05-24 rustfmt was installed and used for both workspaces.
`sh scripts/check.sh` passed end to end with locked offline Cargo operations:

- 24 behavioral tests (14 failure/composition, 10 formatting).
- Two isolated upstream nested/flattened comparisons and both runnable examples.
- Formatting and strict all-target Clippy for both workspaces.
- Library normal/build graph contains none of serde, serde_core, serde_json,
  tracing-serde; the gate now fails if any appears. Separate normal, build and dev
  graphs were also inspected and contain none. Upstream `json` and Facet `fast`
  remain disabled in the library graph. The isolated comparison intentionally uses
  upstream JSON/Serde. Weavy remains in the existing pinned build graph; no network
  or security settings were changed.

New acceptance includes warm-cache corruption and valid external replacement,
float bit preservation through updates/cache hits, duplicate/escaped span names,
concurrent updates and final visibility, two enabled independently filtered sinks,
write-zero, mutex poisoning, direct guard failures and actual file append/flush/
reopen. Existing parent/root, bad Debug/timer, partial writes, concurrent short writes,
tracing-error and span-close tests remain green.

The Windows script received equivalent gates but was not rerun on Windows. The
historical Windows receipt is preserved; this does not claim Windows qualification
of these changes. No dependency additions were needed or proposed.

## Measured outcomes

The original commit was exported to a temporary source directory and built with
this checkpoint's expanded `formatting` benchmark, otherwise unchanged. Both builds
used the same lockfiles, toolchain, release defaults, host, `io::sink`, disabled
timestamps/target, per-layer EnvFilter, events and spans. Every invocation uses
1,000 warm-up iterations, 20,000 measured iterations, seven samples and alternating
formatter order. Three baseline and three final invocations are retained under
[measurements](measurements/linux-2026-10-23/). The second and third baseline/final
runs alternated executables sequentially, without concurrent test/build processes.
Numbers below are medians of the three per-invocation medians, in ns/iteration.

| Workload | Original Facet | Final Facet | Final upstream | Final / upstream |
| --- | ---: | ---: | ---: | ---: |
| Events | 482.86 | 482.19 | 575.10 | 0.84 |
| Three reused spans | 4906.22 | 726.74 | 1928.17 | 0.38 |
| Update plus event | 6667.52 | 4690.84 | 2120.26 | 2.21 |
| Updates only | 2314.64 | 1977.66 | 481.13 | 4.11 |
| Create span, emit once, close | 3631.02 | 2459.03 | 1662.36 | 1.48 |

Reused spans improved about 6.75x against the original on this host; update-plus-
event improved about 1.42x. Events were essentially unchanged. Update-heavy and
short-lived spans are still slower than upstream. Upstream medians also shifted
between binaries/runs, so small differences must not be attributed solely to source.
The final second run has substantial update/churn noise; it is retained, not removed.
Disabled-loop results are retained but not interpreted as serializer performance.
Do not compare these Linux timings against the Windows receipt as a speedup.

The `cache_costs` operation profile measured a representative two-field cache:
parsing into `BTreeMap<String, RawJson>` took 1473.89 ns median; reflection encoding
of that map took 403.34 ns. This isolates operations, not call-stack or allocation
sampling. Together with update-only timings it identifies repeated parsing as a
remaining concrete cost. Native sampling tools were unavailable; no profiler or
new dependency was installed. Replacing parsing with a typed-cache Layer would
require a separate design review of multiple sinks, cache identity and updates.

## Reproduction and limits

Run the check script, then:

```sh
cargo bench --offline --locked --manifest-path validation/upstream-comparison/Cargo.toml --bench formatting
cargo bench --offline --locked --manifest-path validation/upstream-comparison/Cargo.toml --bench cache_costs
```

For the baseline, export commit `1da6902fb5d7874a0f36e88ed65ede665b2341ff`
with `git archive` into a separate temporary directory; copy only the current
`validation/upstream-comparison/benches/formatting.rs` into the matching path.
Run its formatting benchmark using the same toolchain. This adds update-only and
churn workloads without changing baseline library source. The raw files retain
sample counts and min/median/max summaries; individual sample values were not
captured. Avoid concurrent workloads when repeating measurements.

These are microbenchmarks, not allocation, RSS, filesystem throughput, tail latency
or representative application measurements. Added per-span memory has not been
quantified. File tests qualify append and explicit flush/reopen, not fsync, power-loss
recovery, rotation or safe recovery of a partially written existing file.

Native Tracy/cancellation composition and actual consumer startup/cancellation
payloads remain consumer acceptance work. Existing dependencies do not include
those integrations, and adding dependencies requires parent review. Log-facade
normalization and unstable valuable remain unsupported. Reentrant writer callbacks
(including calling inspect/flush while holding its guard) remain forbidden because
they can deadlock; producer Debug panics still propagate. No catch-all recovery,
package publication, default-branch merge or consumer migration is implied.
