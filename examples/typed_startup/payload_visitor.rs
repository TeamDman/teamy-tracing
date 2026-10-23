use tracing::field::Field;
use tracing::field::Visit;
#[derive(Debug, Default)]
pub struct PayloadVisitor {
    pub encoded: Option<String>,
}
impl Visit for PayloadVisitor {
    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "encoded" {
            self.encoded = Some(value.into());
        }
    }
    fn record_debug(&mut self, _: &Field, _: &dyn std::fmt::Debug) {}
}
