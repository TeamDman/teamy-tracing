// Adapted from tracing-core 0.1.36 field::HexBytes; see THIRD_PARTY_NOTICES.md.
use std::fmt;
use std::fmt::Write;

pub(crate) struct HexBytes<'a>(pub &'a [u8]);

impl fmt::Debug for HexBytes<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_char('[')?;
        for (index, byte) in self.0.iter().enumerate() {
            if index != 0 {
                f.write_char(' ')?;
            }
            write!(f, "{byte:02x}")?;
        }
        f.write_char(']')
    }
}
