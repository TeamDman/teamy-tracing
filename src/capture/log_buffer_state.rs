// SPDX-License-Identifier: LiLiQ-R-1.1
// Adapted from Cloud Terrastodon; modified for reusable application-owned capture.
// See THIRD_PARTY_NOTICES.md and LICENSES/Cloud-Terrastodon-LiLiQ-R-1.1*.txt.
use super::LogRecord;
use std::collections::VecDeque;

/// Retained log records and their absolute sequence position.
#[derive(Debug)]
pub(super) struct LogBufferState {
    pub records: VecDeque<LogRecord>,
    pub first_index: usize,
    pub capacity: Option<usize>,
}
