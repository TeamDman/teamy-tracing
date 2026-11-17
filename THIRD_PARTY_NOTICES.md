# Third-party notices

`src/hex_bytes.rs` adapts the formatting algorithm of `HexBytes` from
`tracing-core` 0.1.36, `src/field.rs`. The event envelope, span update behavior and
compatibility tests were developed against `tracing-subscriber` 0.3.23 and
`tracing-serde` 0.2.0. The former's source provenance is commit
`54ede4d5d85a536aed5485c5213011d9ec961935` in
<https://github.com/tokio-rs/tracing>. These components are MIT licensed.
The full notice is retained in `LICENSES/tracing-MIT.txt`; preserve it in copies
and distributions containing the adapted material. Other first-party source is MPL-2.0,
as recorded in LICENSE; the Cloud Terrastodon extraction described below retains
LiLiQ-R-1.1. The MIT notice remains applicable to adapted material.

Historical qualification used registry packages `teamy-facet` 0.50.0-rc.7 and
`teamy-facet-json`, `teamy-facet-format`, `teamy-facet-value` 0.50.0-rc.9, with
checksums in that release's lockfiles. Their manifests declare MIT OR Apache-2.0. Cached
registry source provenance identifies `teamy-facet` at
`74b1070dc1c889f429ba79ff458a38dae46209a5` (TeamDman/facet), JSON/format at
`c77cfc601397e3b1fdf3b63c59eb624c3825960c`, and value at
`642d268fc01eced0a861bded044342ff88733eec` (TeamDman/facet-format).
These are dependencies, not source copied into this library. The old Git pin
`23ad4dcd034d123f0c38943305ffb077519040ce` applies only to historical receipts.

Current dependencies are Facet rc.10, JSON rc.12, and Format/Value rc.11, with
registry checksums in the current lockfiles. Their MIT OR Apache-2.0 licensing
is unchanged; the historical source identities above remain provenance records.

## Cloud Terrastodon live diagnostic extraction

The following source is adapted from
[Cloud Terrastodon](https://github.com/AAFC-Cloud/Cloud-Terrastodon), whose workspace
declares **LiLiQ-R-1.1**:

- All files under [src/capture](src/capture), extracted and split from the original
  structured-log visitor/record/formatter and terminal-buffer implementation.
- [src/diagnostic_report.rs](src/diagnostic_report.rs) and
  [src/diagnostic_report_display.rs](src/diagnostic_report_display.rs), retaining
  original eyre handlers, typed causes, captured spans and constrained styling.
- [src/gui/diagnostic_report_view.rs](src/gui/diagnostic_report_view.rs) and
  [src/gui/diagnostic_text_style.rs](src/gui/diagnostic_text_style.rs), adapted from
  the original CT egui diagnostic viewer and ANSI text-style conversion.

The source checkout's base revision was
eb3490c768f06c024c6119d089198f59bb1fb34c. The extraction also includes
uncommitted diagnostic-view enhancements from that checkout; this base revision
alone does not claim to reproduce those unpublished changes.

The extracted files are modified for reusable capture, optional eyre/egui
features, caller-owned sinks and retention, and the egui 0.36.2 API. Per-file
notices identify this modified origin. They remain subject to LiLiQ-R-1.1; the
existing MPL-2.0 license does not relicense them. Exact original English and
French license texts accompany the code in
[LICENSES/Cloud-Terrastodon-LiLiQ-R-1.1.txt](LICENSES/Cloud-Terrastodon-LiLiQ-R-1.1.txt)
and
[LICENSES/Cloud-Terrastodon-LiLiQ-R-1.1.fr-CA.txt](LICENSES/Cloud-Terrastodon-LiLiQ-R-1.1.fr-CA.txt).
Preserve those license texts and source attributions in distributions.

The package's **MPL-2.0 AND MIT AND LiLiQ-R-1.1** expression identifies cumulative
licenses for its included source components. It does not offer an OR choice,
assert a new permission to relicense, or change the original components' licenses.
