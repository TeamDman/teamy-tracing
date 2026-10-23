Read the adjacent teamy-docs/src/teamystyle.md and code-standards.md before changes.
This is a library, not a CLI template. Keep one discoverable file per type.
Use owned Facet crates from the same immutable revision; no first-party Serde APIs.
Never emit tracing events from formatters or writers: that can recurse.
Run the documented offline formatting, Clippy, behavioral and graph checks.
Preserve the upstream MIT notice alongside the first-party MPL-2.0 license.
Public source publication is approved; crates.io publication remains disabled.
Do not edit or upgrade consumers as part of library changes without authorization.
