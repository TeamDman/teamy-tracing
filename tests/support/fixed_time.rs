use std::fmt;
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::time::FormatTime;

#[derive(Debug, Clone, Copy)]
pub struct FixedTime;
impl FormatTime for FixedTime {
    fn format_time(&self, writer: &mut Writer<'_>) -> fmt::Result {
        writer.write_str("2026-10-23T00:00:00Z")
    }
}
