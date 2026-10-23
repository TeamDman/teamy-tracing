# Third-party notices

`src/hex_bytes.rs` adapts the formatting algorithm of `HexBytes` from
`tracing-core` 0.1.36, `src/field.rs`. The event envelope, span update behavior and
compatibility tests were developed against `tracing-subscriber` 0.3.23 and
`tracing-serde` 0.2.0. The former's source provenance is commit
`54ede4d5d85a536aed5485c5213011d9ec961935` in
<https://github.com/tokio-rs/tracing>. These components are MIT licensed.
The full notice is retained in `LICENSES/tracing-MIT.txt`; preserve it in copies
and distributions containing the adapted material. First-party source is MPL-2.0,
as recorded in LICENSE. The MIT notice remains applicable to adapted material.

Owned Facet dependencies use registry packages `teamy-facet` 0.50.0-rc.7 and
`teamy-facet-json`, `teamy-facet-format`, `teamy-facet-value` 0.50.0-rc.9, with
checksums in the lockfiles. Their manifests declare MIT OR Apache-2.0. Cached
registry source provenance identifies `teamy-facet` at
`74b1070dc1c889f429ba79ff458a38dae46209a5` (TeamDman/facet), JSON/format at
`c77cfc601397e3b1fdf3b63c59eb624c3825960c`, and value at
`642d268fc01eced0a861bded044342ff88733eec` (TeamDman/facet-format).
These are dependencies, not source copied into this library. The old Git pin
`23ad4dcd034d123f0c38943305ffb077519040ce` applies only to historical receipts.
Review the final distribution and resolved dependencies before publication.
Package publication remains disabled.
