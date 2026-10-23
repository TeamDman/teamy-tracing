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

Owned Facet sources are pinned to
<https://github.com/TeamDman/facet/tree/23ad4dcd034d123f0c38943305ffb077519040ce>.
Their manifests declare MIT OR Apache-2.0. They remain dependencies; their source
has not been copied into this library. Review the final distribution and resolved
dependencies before publication. No package publication is configured.
