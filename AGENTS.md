Read the adjacent teamy-docs/src/teamystyle.md and code-standards.md before changes.
This is a library, not a CLI template. Keep one discoverable file per type.
Use the approved exact owned registry Facet package pins as one coherent graph;
no first-party Serde APIs. Registry migration supersedes the historical Git pin.
Never emit tracing events from formatters or writers: that can recurse.
Run the documented offline formatting, Clippy, behavioral and graph checks.
Preserve the upstream MIT notice alongside the first-party MPL-2.0 license.
On release-preparation branches, commit locally only unless separately authorized
to push. Crates.io publication remains disabled.
Do not edit or upgrade consumers as part of library changes without authorization.
