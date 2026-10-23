use std::fmt;
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::time::FormatTime;
#[derive(Debug, Clone)]
pub struct FailingTime;
impl FormatTime for FailingTime {
    fn format_time(&self, _: &mut Writer<'_>) -> fmt::Result {
        Err(fmt::Error)
    }
}
