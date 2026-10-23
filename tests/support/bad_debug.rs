use std::fmt;
pub struct BadDebug;
impl fmt::Debug for BadDebug {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        Err(fmt::Error)
    }
}
