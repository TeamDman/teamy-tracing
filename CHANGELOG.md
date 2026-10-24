# Changelog

## 0.1.0 — 2026-10-24

- Facet JSON event/span formatting using upstream tracing-subscriber interfaces.
- Typed scalar fidelity, span updates, explicit parent/root handling and independent
  per-layer filtering, with validated span rendering reuse.
- Supplied writers, whole-record locking, fail-stop delivery and diagnostic counters.
- Exact owned registry dependency family: teamy-facet rc.7 and JSON/format/value rc.9.
- Strict validation of externally changed span caches before raw JSON reuse, plus
  documented shared-span diagnostic attribution and observational inspect semantics.
- Linux source/package qualification, executable API/startup examples and documented
  performance limits. The Windows x86_64 MSVC release gate passed with 36 behavioral
  tests, one API doctest, two upstream comparisons, strict Clippy, formatting,
  warning-free rustdoc, both runnable examples and a no-Serde normal/build graph.

Native Tracy/cancellation and actual consumer startup integrations are outside the
qualified release scope.
