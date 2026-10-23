use std::io;
use std::io::Write;
#[derive(Debug, Default)]
pub struct PartialWriter {
    pub bytes: Vec<u8>,
    pub fail_after: Option<usize>,
    pub fail_flush: bool,
}
impl Write for PartialWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self
            .fail_after
            .is_some_and(|limit| self.bytes.len() >= limit)
        {
            return Err(io::Error::other("injected failure"));
        }
        let count = bytes
            .len()
            .min(3)
            .min(self.fail_after.unwrap_or(usize::MAX) - self.bytes.len());
        self.bytes.extend_from_slice(&bytes[..count]);
        std::thread::yield_now();
        Ok(count)
    }
    fn flush(&mut self) -> io::Result<()> {
        if self.fail_flush {
            Err(io::Error::other("injected flush failure"))
        } else {
            Ok(())
        }
    }
}
