// SPDX-License-Identifier: LiLiQ-R-1.1
// Adapted from Cloud Terrastodon; modified for reusable application-owned capture.
// See THIRD_PARTY_NOTICES.md and LICENSES/Cloud-Terrastodon-LiLiQ-R-1.1*.txt.
use crate::diagnostic_report::DiagnosticReport;
use std::fmt;

/// A rendering view that retains only the report's constrained cosmetic styling.
#[derive(Debug)]
pub(crate) struct DiagnosticReportDisplay<'a> {
    pub report: &'a DiagnosticReport,
}

impl fmt::Display for DiagnosticReportDisplay<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.report.format_report(formatter, true)
    }
}
