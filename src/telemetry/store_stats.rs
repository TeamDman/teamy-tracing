/// Counts are cumulative for the lifetime of a store, including after clear.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StoreStats {
    pub accepted: u64,
    pub evicted: u64,
    pub rejected_oversize: u64,
    pub upstream_truncated: u64,
    pub records: usize,
    pub logical_bytes: usize,
}
