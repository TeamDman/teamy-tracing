use super::CorrelationKey;
use super::StoreLimits;
use super::StoreStats;
use super::TelemetryRecord;
use super::TelemetryValue;
use std::collections::VecDeque;

/// An application-owned FIFO. Synchronization and repaint signaling belong to the caller.
#[derive(Debug)]
pub struct RecordStore {
    limits: StoreLimits,
    records: VecDeque<(usize, TelemetryRecord)>,
    stats: StoreStats,
}

impl RecordStore {
    pub fn new(limits: StoreLimits) -> Self {
        Self {
            limits,
            records: VecDeque::new(),
            stats: StoreStats::default(),
        }
    }

    /// Reject an individually oversized record; evict oldest records to make room.
    pub fn push(&mut self, record: TelemetryRecord) -> bool {
        let size = record.logical_size();
        if self.limits.max_records == 0 || size > self.limits.max_logical_bytes {
            self.stats.rejected_oversize = self.stats.rejected_oversize.saturating_add(1);
            return false;
        }
        while self.records.len() >= self.limits.max_records
            || self.stats.logical_bytes.saturating_add(size) > self.limits.max_logical_bytes
        {
            if let Some((old_size, _)) = self.records.pop_front() {
                self.stats.logical_bytes -= old_size;
                self.stats.evicted = self.stats.evicted.saturating_add(1);
            } else {
                break;
            }
        }
        if matches!(
            record.attribute("output_truncated"),
            Some(TelemetryValue::Bool(true))
        ) {
            self.stats.upstream_truncated = self.stats.upstream_truncated.saturating_add(1);
        }
        self.stats.accepted = self.stats.accepted.saturating_add(1);
        self.stats.logical_bytes += size;
        self.records.push_back((size, record));
        self.stats.records = self.records.len();
        true
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &TelemetryRecord> {
        self.records.iter().map(|(_, record)| record)
    }

    /// Return only records with all three matching identity fields.
    pub fn matching_call<'a>(
        &'a self,
        key: &'a CorrelationKey,
    ) -> impl Iterator<Item = &'a TelemetryRecord> + 'a {
        self.iter()
            .filter(move |record| CorrelationKey::from_record(record).as_ref() == Some(key))
    }

    pub fn stats(&self) -> StoreStats {
        self.stats
    }

    pub fn clear(&mut self) {
        self.records.clear();
        self.stats.records = 0;
        self.stats.logical_bytes = 0;
    }
}

impl Default for RecordStore {
    fn default() -> Self {
        Self::new(StoreLimits::default())
    }
}
