use crate::Diagnostics;
use crate::RecordGuard;
use crate::writer_state::WriterState;
use std::io;
use std::io::Write;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::Ordering;
use tracing_subscriber::fmt::MakeWriter;

/// Locks for the complete formatted record and stops delivery after any I/O error.
/// Reopen a fresh sink explicitly after inspecting diagnostics; no silent recovery.
#[derive(Debug)]
pub struct SharedWriter<W> {
    state: Arc<Mutex<WriterState<W>>>,
    diagnostics: Diagnostics,
}

impl<W> Clone for SharedWriter<W> {
    fn clone(&self) -> Self {
        Self {
            state: Arc::clone(&self.state),
            diagnostics: self.diagnostics.clone(),
        }
    }
}

impl<W> SharedWriter<W> {
    pub fn new(writer: W, diagnostics: Diagnostics) -> Self {
        Self {
            state: Arc::new(Mutex::new(WriterState {
                writer,
                failed: false,
            })),
            diagnostics,
        }
    }
    /// Run an observational callback with `&W` while holding the sink mutex.
    ///
    /// This does not enforce read-only access: types such as `File` permit writes
    /// or cloned writing handles through `&W`. Such writes bypass this writer's
    /// record/counter contract. Do not use those routes or re-enter this writer
    /// from the callback; re-entry can deadlock.
    pub fn inspect<R>(&self, inspect: impl FnOnce(&W) -> R) -> io::Result<R> {
        let guard = self
            .state
            .lock()
            .map_err(|_| io::Error::other("log writer mutex poisoned"))?;
        Ok(inspect(&guard.writer))
    }
}

impl<W: Write> SharedWriter<W> {
    /// Explicit shutdown boundary; flushing is not fsync or crash durability.
    pub fn flush(&self) -> io::Result<()> {
        let mut guard = self
            .state
            .lock()
            .map_err(|_| io::Error::other("log writer mutex poisoned"))?;
        if guard.failed {
            return Err(io::Error::other(
                "log writer stopped after an earlier error",
            ));
        }
        let result = guard.writer.flush();
        if result.is_err() {
            guard.failed = true;
            self.diagnostics
                .0
                .flush_errors
                .fetch_add(1, Ordering::Relaxed);
        }
        result
    }
}

impl<'a, W: Write + 'a> MakeWriter<'a> for SharedWriter<W> {
    type Writer = RecordGuard<'a, W>;
    fn make_writer(&'a self) -> Self::Writer {
        RecordGuard::new(self.state.lock().ok(), self.diagnostics.clone())
    }
}
