// SPDX-License-Identifier: LiLiQ-R-1.1
// Adapted from Cloud Terrastodon; modified for reusable application-owned capture.
// See THIRD_PARTY_NOTICES.md and LICENSES/Cloud-Terrastodon-LiLiQ-R-1.1*.txt.
use super::LogBuffer;
use super::LogRecord;
use super::log_buffer_state::LogBufferState;
use std::sync::Mutex;
use std::sync::Weak;

/// A nonowning capture writer that cannot keep its subscriber alive through a
/// retained report's span trace. The application owns the retained state.
#[derive(Clone, Debug)]
pub(super) struct WeakLogBuffer {
    inner: Weak<Mutex<LogBufferState>>,
}

impl WeakLogBuffer {
    pub(super) fn new(inner: Weak<Mutex<LogBufferState>>) -> Self {
        Self { inner }
    }

    pub(super) fn push(&self, record: LogRecord) {
        if let Some(inner) = self.inner.upgrade() {
            LogBuffer::from_state(inner).push(record);
        }
    }
}
