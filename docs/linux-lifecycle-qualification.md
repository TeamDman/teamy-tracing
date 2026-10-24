# Linux lifecycle qualification — 24 October 2026

This bounded continuation starts from local owned-registry checkpoint
`b4be8e454558a98187056faa040dbfe06c41ac84`. The dependency manifests, both
lockfiles, toolchain inputs and production implementation remain unchanged.
Publication remains disabled. No consumer repository was edited; consumer
integration proceeds separately.

Repository AGENTS.md and adjacent TeamyStyle, code standards and cooperative
cancellation guidance were consulted. No applicable checkout SKILL.md was present.
All checks use the existing standard/non-fast dependency graph and installed tools;
no installs, credentials or network/security changes were needed.

## Gaps closed

Five new tests exercise observable public behavior:

- **Caller-owned cooperative cancellation during a write.** A gated short writer
  confirms that cancellation is requested while the first record is in flight.
  After the writer is released, the worker completes that record, observes the
  request at its next loop boundary and closes the span. Exactly two valid NDJSON
  records remain: the event and a close receipt with the final span update. The
  caller joins the worker before flushing. Both successful and failing shutdown
  flushes are exercised; a failed flush suppresses later delivery and is counted.
- **Sink failure isolation in both layer orders.** One independently filtered sink
  fails after seven bytes. The healthy sink still receives all three admitted
  events, including its debug-only event. Per-sink counters distinguish the initial
  failure, suppressed later record and healthy delivery.
- **Interrupted short writes.** A writer alternates Interrupted errors with short
  progress. The complete-record write_all path retries, emits two whole records
  and leaves diagnostics clear. This tests the actual upstream delivery path; it
  does not change the direct RecordGuard::write fail-stop policy.
- **Transactional multi-field span updates.** A Debug implementation writes a prefix
  and then fails after an earlier field in the same update was visited. No prefix
  leaks, the entire previous cache remains intact, and a later valid update works.
  An event with the same failing Debug value is discarded and counted separately.
- **Unused formatter components are skipped.** Disabling timestamps and both span
  context outputs permits an event despite a failing timer and a deliberately
  corrupt unused span cache; no formatting failure is counted.

The cancellation fixture uses a standard-library AtomicBool and deterministic
channels. It is not a teamy-cancellation, signal handler, async-runtime, Tracy or
real-application integration test. In particular, cancellation does not interrupt
an underlying blocked synchronous writer; the fixture explicitly releases it.
Its five-second channel timeout bounds a broken test and is not a cancellation
latency or user-interaction requirement.

## Verification

`sh scripts/check.sh` passed under the frozen Rust 1.96.0 toolchain and
nightly-2026-05-24 formatter:

- 33 behavioral tests (16 failure/composition, 10 formatting, 3 lifecycle,
  4 startup), one API doctest and two isolated upstream comparisons.
- Both examples, strict all-target Clippy and formatting for both workspaces,
  warning-free rustdoc, and the no-Serde normal/build graph gate.
- Cargo package verification with publication disabled, plus all 33 behavioral
  tests and the API doctest from the extracted crate. The new support fixtures
  are included in the archive and checkout-only scripts remain excluded.

The archive was audited for attribution and unintended files; MPL-2.0 and the
upstream MIT notice remain included. Final clean-commit/package identity and logs
are retained in the local completion receipt. No production code or dependency
change warrants another performance claim; the registry-graph measurements in
[release preparation](release-preparation.md) remain the applicable prior evidence.

## Release boundary

The useful public-API Linux gaps identified in this pass are closed without new
capabilities or dependencies. Remaining release gates are Windows qualification
on the owned registry graph, review of the exact candidate/version/package, and
explicit publication approval. Desktop being unavailable does not make the Windows
gate pass. No push, package upload, tag or merge was performed.

Shared prerequisites for the separate consumer integration are the approved owned
registry Facet identities, independent sink/adapter diagnostic ownership and a
shutdown sequence that stops producers, drops spans, then checks flush. Consumer
startup schemas, native cancellation/Tracy behavior and actual process stream/file
routing require their own acceptance evidence. They are not prerequisites for
claiming the narrower formatter/writer contract, and are not promised by this crate.
