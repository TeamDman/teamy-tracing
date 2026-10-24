# Candidate/package review — 24 October 2026

Cloud review began at exact local candidate
`76c72efa4d9bc36f8d4e14c5b6f12e8599f515cd`, version 0.1.0, on
`release-preparation/owned-registry`. An independent read-only reviewer examined
API behavior, compatibility claims, attribution and the fixes below. The primary
agent executed the tests and package checks. No consumer repositories were edited.
The final reviewed commit and archive checksum are recorded in the local completion
receipt; the code changes from the starting candidate are confined to cache
validation. Dependencies, lockfiles, toolchains and `publish = false` remain frozen.

## Findings and disposition

1. **Correctness blocker — fixed.** The original cache path treated successful
   deserialization into RawJson as strict JSON validation. A regression against the
   starting implementation reproduced emission of an externally installed nested
   array with a trailing comma, contradicting malformed-cache rejection. Further
   testing of the initial repair reproduced acceptance of an unknown string escape.
   Missing-comma cases were investigated too, but already rejected in the actual
   original reproduction; they are not claimed as observed bypasses.

   Both span rendering and field merging now validate the complete single-line
   object before raw lexemes are reused. The existing Facet parser is traversed
   without raw skipping, using its checked decoding path. Additional token checks
   enforce JSON number and string-escape grammar, disallow raw controls/trailing
   commas, and require only permitted whitespace after the root object. The checked
   path is important before inspecting malformed non-ASCII Unicode escapes.
   Accepted values are still emitted from the original RawJson lexemes, not a
   lossy numeric round trip. Failed updates retain their previous source and events
   with invalid context are discarded/counted. Validation is skipped on an exact
   validated-cache hit, but runs before merging changed fields.

   Regressions cover malformed nested containers, multiple root values/trailing
   junk, invalid numbers/escapes/controls, both event and update paths, and valid
   nested values retaining u64::MAX, negative zero, exponent spelling, escaped
   quotes, booleans and null. Existing typed scalar/float boundary tests pass.
   The reviewer reread the final fix and found no remaining concrete correctness
   blocker in it. This is source review plus listed tests, not proof for all inputs.

2. **inspect documentation overclaim — fixed.** The method passes `&W` under its
   mutex; it cannot enforce read-only access for types such as File. Rustdoc and
   README now warn that writes through that reference or cloned handles bypass the
   record/counter contract, and re-entry may deadlock. The signature is unchanged.

3. **Shared span diagnostic attribution — clarified and tested.** Upstream layers
   using the same FacetJsonFields type share initial span-cache formatting. A bad
   initial field increments only the initializing layer's counter while both sinks
   may emit empty-context fallback. A regression verifies exactly one initial error
   across two handles. Docs distinguish writer errors, event formatter errors and
   shared span-field initialization; counters are not per-sink loss receipts.

External extension caches also remain subject to the pinned parser's scalar
limits, including integers beyond i128/u128. They are not an arbitrary-precision
JSON interface. This does not narrow the documented ordinary i64/u64/f64 tracing
inputs or i128/u128 string fallback. Formatting source shapes and public signatures
otherwise remain unchanged; log normalization and valuable remain unsupported.

## Completed review evidence

- **Archive:** the starting candidate packaged twice with locked offline Cargo and
  produced the identical SHA-256
  `b7273b2554a2cc2b5af5785d17408e39c2e3e2b655157fac789e491cdd840b39`.
  Its VCS metadata matched the clean starting commit. This establishes repeatability
  for those runs, not cross-platform reproducible binaries. The corrected candidate
  is repackaged and checked again after the review commit.
- **Inventory and attribution:** audited regular text entries, normalized dependency
  aliases, source/tests/examples/configuration and documentation. Maintainer scripts
  and the isolated comparison workspace remain excluded. LICENSE, retained upstream
  MIT notice and THIRD_PARTY_NOTICES.md are included. No unexpected path types,
  credentials or selected secret-pattern matches were found. This is an inventory/
  provenance review, not a claim of independent security certification.
- **Dependency provenance:** all 54 cached registry crate archives matched the
  checksums in Cargo.lock. The owned rc.7 core/reflect/macro family and rc.9 JSON/
  format/value identities remain coherent; published-source VCS identities are
  retained in the notices. No package has an unspecified license in resolved root
  metadata. No source was fetched, installed or upgraded for this review.
- **API and compatibility:** registry Facet identity, upstream subscriber identity,
  scalar lexemes, explicit parent behavior, duplicate keys, span updates, writer
  failure policy, caller-owned shutdown and unsupported integrations were reviewed
  against source and tests. The illustrative startup schema remains clearly scoped.
- **Validation after fixes:** 36 behavioral tests, one API doctest, two isolated
  upstream comparisons, both examples, strict all-target Clippy/formatting,
  warning-free rustdoc and no-Serde normal/build graph gate pass. Corrected archive
  verification and its extracted behavioral/doctest suite pass as well.

## Performance cost

Strict cache validation adds scanning on cache misses and span-field updates.
The same release benchmark was rerun three times with io::sink, timestamps/target
disabled, equivalent filters/workloads, 1,000 warm-up iterations, 20,000 measured
iterations, seven samples and alternating formatter order. Medians of the three
per-run medians (ns/iteration) are below; all summaries are retained in
[measurements](measurements/review-2026-10-24/).

| Workload | Corrected candidate | Upstream | Candidate / upstream |
| --- | ---: | ---: | ---: |
| Events | 614.84 | 742.23 | 0.83 |
| Three reused spans | 753.29 | 2158.38 | 0.35 |
| Update plus event | 6643.33 | 2610.73 | 2.54 |
| Updates only | 2441.78 | 517.99 | 4.71 |
| Create span, emit once, close | 2794.99 | 1967.33 | 1.42 |

Host/run variation is visible, including in unchanged upstream timings. These are
not a controlled before/after estimate of the validation overhead or application
throughput. No speedup is claimed. Update-heavy workloads remain slower, and JSON
integrity takes precedence over retaining the earlier update timings.

## Release checklist

| Gate | Status and next action |
| --- | --- |
| Cloud candidate/API/package review | Complete, with the three findings above addressed. Do not treat this as an unspecified pending review. |
| Frozen-graph Linux qualification | Passed, including corrected archive tests and documented performance limits. |
| Windows qualification | **Required and unrun.** Run the existing Windows gate on this corrected registry candidate before claiming Windows qualification. Desktop remains off. |
| Release decision | **Pending authorization.** Choose/approve the initial version and intended publication. Version 0.1.0 is a candidate, not an existing release. |
| Publication-enabling change | **Not performed.** After authorization, a narrowly reviewed manifest change must enable the intended registry; version/manifest edits change the artifact, so repackage, verify/audit and retain its exact digest before upload. |
| Credentials/upload/push | Not needed for completed cloud checks; none requested or used. No push, package publication, tag or merge occurred. |

Native cancellation/Tracy, signal/async-runtime behavior, real-consumer startup and
Vox/Terminal cutovers are **optional separate integrations**, not unresolved blockers
for this deliberately narrower formatter/writer release. They require separate
acceptance if later advertised. The cooperative AtomicBool fixture does not qualify
those capabilities. No new general cloud qualification task is identified by this
review; the remaining required gate is Windows plus the explicit release decisions.
