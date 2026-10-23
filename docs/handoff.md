# Bounded Linux library acceptance

The next checkpoint adds realistic proposed typed-startup and independent-sink
fixtures: 28 behavioral tests, two upstream comparisons, both examples, formatting,
strict Clippy and graph checks pass. See [acceptance](library-acceptance.md).
Production library source and dependency manifests/locks are unchanged from e539d15;
the borrowed-cache optimization was rejected because the pinned map reflection
requires static types. Repeat measurements claim no new speedup.

The newer consumer startup formatter is not in the provisioned/public template
revision. Supply its exact source before claiming schema compatibility. Cancellation
and Tracy sources are cached but absent from the tracing manifests/locks; an isolated
composition fixture needs reviewed dependency/lock changes. These, actual consumer
integration and a Windows rerun remain separate. No consumer was edited.

# Linux continuation

The implementation branch now includes validated per-span rendering reuse, direct
Facet map emission, writer edge-case fixes and 24 behavioral tests. See
[Linux qualification](linux-qualification.md) for commands, retained before/after
measurements, memory/concurrency contracts and remaining adoption limits.
No dependencies or consumer pins changed. Parent coordinates consumer acceptance;
update-heavy performance, native Tracy/cancellation, actual startup payloads and
Windows reruns remain explicit follow-up work. Publication is a source checkpoint,
not package release or approval to change consumers.

The original handoff follows as historical provenance.

# Cloud continuation checkpoint

This initial local implementation is intended to be published as a public MPL-2.0
source checkpoint on branch `implementation`. Preserve the upstream MIT notice.
Package publication remains disabled. No consumer was edited, and no environment
was created. Use the remote commit SHA reported with the handoff; do not assume a
moving branch still names the same source.

The repository contains all first-party source, examples, tests, policy config,
both dependency lockfiles, check scripts, benchmark source/results and Linux setup
proposal. It does not require a desktop-only path dependency. Supply the public
TeamDman/teamy-docs repository beside it; the inspected guidance revision was
`50e89d6897ce2a65167ef65b17c5fa83fe849158`. Cargo's owned Facet Git dependencies
resolve to `23ad4dcd034d123f0c38943305ffb077519040ce`.

## Continue in order

1. Check out the verified immutable library checkpoint and teamy-docs revision.
   Read AGENTS.md, README.md, schema.md, validation.md and cloud-setup.md.
2. Provision the proposed Linux toolchains/cache only through the approved
   environment setup. Run `sh scripts/check.sh`; record that Linux was actually
   exercised, rather than relying on the Windows receipt.
3. Inspect normal/build/dev Cargo graphs separately. The separate comparison
   workspace intentionally enables Serde; the library must not.
4. Profile the span-heavy cases. Consider a typed-cache companion Layer or a
   validated cache optimization while preserving updates, per-layer filtering,
   explicit parent/root semantics, error counters and writer failure policy.
5. Extend qualification where needed: native cancellation/Tracy composition,
   poison/write-zero and reentrancy boundaries, optional log facade normalization,
   actual consumer startup payload and durable file lifecycle. The existing
   typed-startup example illustrates an adapter; it is not a consumer migration.
6. Propose consumer adoption after compatibility/performance review. Do not infer
   crates.io publication or unrelated application changes from this checkpoint.

Existing behavior intentionally differs from upstream for explicit event parents
and malformed span-cache fallback; see schema.md. Current microbenchmarks are slower
than upstream, especially with spans. Preserve that evidence until new measurements
justify a different conclusion.

## Publication audit

The source set was created locally for this library. It contains synthetic fixture
values, public source coordinates and toolchain/version data. Raw conversation,
credentials, private review material, machine-specific paths, build caches and
desktop environment exports are not required inputs and must not be committed.
The pre-push audit inspected the complete 53-file staged inventory, found no matches
for credential/private-path/conversation patterns, retained both licenses/notices,
checked staged whitespace and confirmed `publish = false`. Pattern scanning is
not a guarantee against every possible secret. No build caches or untracked extras
were selected for publication.
Verification of the remote SHA is required before calling the handoff cloud-ready.
