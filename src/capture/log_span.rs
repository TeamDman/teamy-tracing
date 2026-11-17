// SPDX-License-Identifier: LiLiQ-R-1.1
// Adapted from Cloud Terrastodon; modified for reusable application-owned capture.
// See THIRD_PARTY_NOTICES.md and LICENSES/Cloud-Terrastodon-LiLiQ-R-1.1*.txt.
use super::LogValue;
use std::sync::Arc;

/// Captured span context; report fields are flattened to avoid ownership cycles.
#[derive(Clone, Debug)]
pub struct LogSpan {
    pub name: Arc<str>,
    pub fields: Vec<(String, LogValue)>,
}
