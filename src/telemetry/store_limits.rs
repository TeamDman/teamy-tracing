/// Retention bounds. A zero bound rejects every record.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StoreLimits {
    pub max_records: usize,
    /// Logical record size, not total process memory or a network request limit.
    pub max_logical_bytes: usize,
}

impl Default for StoreLimits {
    fn default() -> Self {
        Self {
            max_records: 1_000,
            max_logical_bytes: 8 * 1024 * 1024,
        }
    }
}
