use crate::validate_span_cache::validate_span_cache;
use facet_format::FormatSerializer;
use facet_format::ScalarValue;
use facet_json::JsonSerializer;
use facet_json::RawJson;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fmt;

/// One validated rendering per live span. Source equality detects external edits
/// to upstream's public FormattedFields as well as ordinary span.record updates.
#[derive(Debug)]
pub(crate) struct ValidatedSpan {
    pub source: String,
    pub encoded: String,
}

impl ValidatedSpan {
    pub fn new(source: String, name: &str) -> Result<Self, fmt::Error> {
        validate_span_cache(&source)?;
        let fields: BTreeMap<String, RawJson<'static>> =
            facet_json::from_str(&source).map_err(|_| fmt::Error)?;
        let mut serializer = JsonSerializer::new();
        serializer.begin_struct().map_err(|_| fmt::Error)?;
        for (key, value) in fields {
            serializer.field_key(&key).map_err(|_| fmt::Error)?;
            serializer
                .raw_scalar(value.as_str())
                .map_err(|_| fmt::Error)?;
        }
        serializer.field_key("name").map_err(|_| fmt::Error)?;
        serializer
            .scalar(ScalarValue::Str(Cow::Borrowed(name)))
            .map_err(|_| fmt::Error)?;
        serializer.end_struct().map_err(|_| fmt::Error)?;
        Ok(Self {
            source,
            encoded: String::from_utf8(serializer.finish()).map_err(|_| fmt::Error)?,
        })
    }
}
