use crate::Diagnostics;
use crate::writer_state::WriterState;
use std::io;
use std::io::Write;
use std::sync::MutexGuard;
use std::sync::atomic::Ordering;

/// Writer guard obtained from SharedWriter; holds the mutex through write_all.
#[derive(Debug)]
pub struct RecordGuard<'a, W> {
    state: Option<MutexGuard<'a, WriterState<W>>>,
    diagnostics: Diagnostics,
    counted: bool,
}

impl<'a, W> RecordGuard<'a, W> {
    pub(crate) fn new(
        state: Option<MutexGuard<'a, WriterState<W>>>,
        diagnostics: Diagnostics,
    ) -> Self {
        Self {
            state,
            diagnostics,
            counted: false,
        }
    }

    fn ready(&mut self) -> io::Result<&mut WriterState<W>> {
        if self.state.as_ref().is_none_or(|state| state.failed) {
            if !self.counted {
                self.diagnostics
                    .0
                    .suppressed_records
                    .fetch_add(1, Ordering::Relaxed);
                self.counted = true;
            }
            return Err(io::Error::other(
                "log writer unavailable after I/O error or mutex poisoning",
            ));
        }
        Ok(self.state.as_deref_mut().expect("checked above"))
    }

    fn failed(&mut self) {
        if let Some(state) = self.state.as_mut() {
            state.failed = true;
        }
        self.diagnostics
            .0
            .write_errors
            .fetch_add(1, Ordering::Relaxed);
        self.counted = true;
    }
}

impl<W: Write> Write for RecordGuard<'_, W> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let result = self.ready()?.writer.write(bytes).and_then(|count| {
            if count == 0 && !bytes.is_empty() {
                Err(io::Error::new(
                    io::ErrorKind::WriteZero,
                    "log writer made no progress",
                ))
            } else {
                Ok(count)
            }
        });
        if result.is_err() {
            self.failed();
        }
        result
    }
    fn write_all(&mut self, bytes: &[u8]) -> io::Result<()> {
        let result = self.ready()?.writer.write_all(bytes);
        if result.is_err() {
            self.failed();
        }
        result
    }
    fn flush(&mut self) -> io::Result<()> {
        let result = self.ready()?.writer.flush();
        if result.is_err() {
            if let Some(state) = self.state.as_mut() {
                state.failed = true;
            }
            self.counted = true;
            self.diagnostics
                .0
                .flush_errors
                .fetch_add(1, Ordering::Relaxed);
        }
        result
    }
}
