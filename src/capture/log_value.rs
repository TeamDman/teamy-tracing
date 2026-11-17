// SPDX-License-Identifier: LiLiQ-R-1.1
// Adapted from Cloud Terrastodon; modified for reusable application-owned capture.
// See THIRD_PARTY_NOTICES.md and LICENSES/Cloud-Terrastodon-LiLiQ-R-1.1*.txt.
#[cfg(feature = "eyre")]
use crate::DiagnosticReport;
use std::fmt;

/// A captured log field. Reports retain their original sources and handler until
/// a presentation surface renders them; ordinary tracing values remain text.
#[derive(Clone, Debug)]
pub enum LogValue {
    Text(String),
    #[cfg(feature = "eyre")]
    Report(DiagnosticReport),
}

impl fmt::Display for LogValue {
    /// Plain presentation for file logs and GUI text surfaces.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Text(text) => formatter.write_str(text),
            #[cfg(feature = "eyre")]
            Self::Report(report) => fmt::Display::fmt(report, formatter),
        }
    }
}

impl From<String> for LogValue {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<&str> for LogValue {
    fn from(value: &str) -> Self {
        Self::Text(value.to_owned())
    }
}
