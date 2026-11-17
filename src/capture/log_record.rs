// SPDX-License-Identifier: LiLiQ-R-1.1
// Adapted from Cloud Terrastodon; modified for reusable application-owned capture.
// See THIRD_PARTY_NOTICES.md and LICENSES/Cloud-Terrastodon-LiLiQ-R-1.1*.txt.
use super::LogLevel;
use super::LogSpan;
use super::LogValue;
use std::sync::Arc;
use std::time::SystemTime;

/// An owned event snapshot with reports retained until presentation.
#[derive(Clone, Debug)]
pub struct LogRecord {
    pub timestamp: SystemTime,
    pub level: LogLevel,
    pub target: Arc<str>,
    pub message: Arc<str>,
    pub fields: Vec<(String, LogValue)>,
    pub spans: Vec<LogSpan>,
    pub file: Option<Arc<str>>,
    pub line: Option<u32>,
}
