# Changelog

## Unreleased — initial release candidate

- Facet JSON event/span formatting using upstream tracing-subscriber interfaces.
- Typed scalar fidelity, span updates, explicit parent/root handling and independent
  per-layer filtering, with validated span rendering reuse.
- Supplied writers, whole-record locking, fail-stop delivery and diagnostic counters.
- Exact owned registry dependency family: teamy-facet rc.7 and JSON/format/value rc.9.
- Strict validation of externally changed span caches before raw JSON reuse, plus
  documented shared-span diagnostic attribution and observational inspect semantics.
- Linux source/package qualification, executable API/startup examples and documented
  performance limits. Windows release qualification is pending.

The manifest version is 0.1.0; publication remains disabled. Native Tracy/cancellation
and actual consumer startup integrations are outside the qualified release scope.
