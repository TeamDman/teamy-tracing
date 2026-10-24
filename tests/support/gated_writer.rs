use std::io;
use std::io::Write;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;
use std::time::Duration;

/// Lets a test request cancellation while the first record is in flight.
#[derive(Debug)]
pub struct GatedWriter {
    pub bytes: Vec<u8>,
    pub started: Option<SyncSender<()>>,
    pub release: Receiver<()>,
    pub fail_flush: bool,
}
impl Write for GatedWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if let Some(started) = self.started.take() {
            started.send(()).map_err(io::Error::other)?;
            self.release
                .recv_timeout(Duration::from_secs(5))
                .map_err(io::Error::other)?;
        }
        let count = bytes.len().min(7);
        self.bytes.extend_from_slice(&bytes[..count]);
        Ok(count)
    }
    fn flush(&mut self) -> io::Result<()> {
        if self.fail_flush {
            Err(io::Error::other("synthetic shutdown flush failure"))
        } else {
            Ok(())
        }
    }
}
