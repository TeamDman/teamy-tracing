#[derive(Debug)]
pub(crate) struct WriterState<W> {
    pub writer: W,
    pub failed: bool,
}
