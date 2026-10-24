use std::io;
use std::io::Write;

#[derive(Debug, Default)]
pub struct InterruptedWriter {
    pub bytes: Vec<u8>,
    pub calls: usize,
}
impl Write for InterruptedWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.calls += 1;
        if self.calls % 2 == 1 {
            return Err(io::ErrorKind::Interrupted.into());
        }
        let count = bytes.len().min(3);
        self.bytes.extend_from_slice(&bytes[..count]);
        Ok(count)
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
