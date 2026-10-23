use crate::hex_bytes::HexBytes;
use facet_format::ScalarValue;
use std::borrow::Cow;
use std::fmt;
use std::fmt::Write;
use tracing::field::Field;
use tracing::field::Visit;

/// A borrowing visitor; scalar types go directly to Facet, never through Value.
pub(crate) struct JsonVisitor<F> {
    emit: F,
    result: fmt::Result,
    span: bool,
}

impl<F> JsonVisitor<F>
where
    F: FnMut(&str, ScalarValue<'_>) -> fmt::Result,
{
    pub fn new(emit: F, span: bool) -> Self {
        Self {
            emit,
            result: Ok(()),
            span,
        }
    }
    pub fn finish(self) -> fmt::Result {
        self.result
    }
    fn emit(&mut self, name: &str, value: ScalarValue<'_>) {
        if self.result.is_ok() {
            self.result = (self.emit)(name, value);
        }
    }
}

impl<F> Visit for JsonVisitor<F>
where
    F: FnMut(&str, ScalarValue<'_>) -> fmt::Result,
{
    fn record_i64(&mut self, field: &Field, value: i64) {
        self.emit(field.name(), ScalarValue::I64(value));
    }
    fn record_u64(&mut self, field: &Field, value: u64) {
        self.emit(field.name(), ScalarValue::U64(value));
    }
    fn record_f64(&mut self, field: &Field, value: f64) {
        self.emit(field.name(), ScalarValue::F64(value));
    }
    fn record_bool(&mut self, field: &Field, value: bool) {
        self.emit(field.name(), ScalarValue::Bool(value));
    }
    fn record_str(&mut self, field: &Field, value: &str) {
        self.emit(field.name(), ScalarValue::Str(Cow::Borrowed(value)));
    }
    // Like upstream 0.3.23, i128/u128 and Error use Visit's debug/display fallbacks.
    fn record_bytes(&mut self, field: &Field, value: &[u8]) {
        if self.span {
            self.emit(field.name(), ScalarValue::Bytes(Cow::Borrowed(value)));
        } else {
            self.record_debug(field, &HexBytes(value));
        }
    }
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        if self.result.is_err() {
            return;
        }
        let mut text = String::new();
        if write!(&mut text, "{value:?}").is_err() {
            self.result = Err(fmt::Error);
            return;
        }
        let name = if self.span {
            field.name().strip_prefix("r#").unwrap_or(field.name())
        } else {
            field.name()
        };
        self.emit(name, ScalarValue::Str(Cow::Owned(text)));
    }
}
