use std::fmt;

pub struct PrefixBadDebug;
impl fmt::Debug for PrefixBadDebug {
    fn fmt(&self, writer: &mut fmt::Formatter<'_>) -> fmt::Result {
        writer.write_str("must not leak this prefix")?;
        Err(fmt::Error)
    }
}
