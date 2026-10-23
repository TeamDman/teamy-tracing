use crate::Diagnostics;
use crate::json_visitor::JsonVisitor;
use crate::serialize_scalar::serialize_scalar;
use facet_json::JsonSerializer;
use facet_json::RawJson;
use std::collections::BTreeMap;
use std::fmt;
use tracing::span::Record;
use tracing_subscriber::field::RecordFields;
use tracing_subscriber::fmt::FormatFields;
use tracing_subscriber::fmt::FormattedFields;
use tracing_subscriber::fmt::format::Writer;

/// Compact span cache with last-recorded-value wins, preserving JSON lexemes.
#[derive(Debug, Default, Clone)]
pub struct FacetJsonFields {
    diagnostics: Diagnostics,
}

impl FacetJsonFields {
    pub fn new(diagnostics: Diagnostics) -> Self {
        Self { diagnostics }
    }

    fn merge<R: RecordFields>(&self, previous: &str, fields: R) -> Result<String, fmt::Error> {
        if previous.contains(['\n', '\r']) {
            return Err(fmt::Error);
        }
        let mut values: BTreeMap<String, RawJson<'static>> = if previous.is_empty() {
            BTreeMap::new()
        } else {
            facet_json::from_str(previous).map_err(|_| fmt::Error)?
        };
        let mut visitor = JsonVisitor::new(
            |name: &str, value| {
                let mut serializer = JsonSerializer::new();
                serialize_scalar(&mut serializer, value)?;
                let encoded = String::from_utf8(serializer.finish()).map_err(|_| fmt::Error)?;
                values.insert(name.into(), RawJson::from_owned(encoded));
                Ok(())
            },
            true,
        );
        fields.record(&mut visitor);
        visitor.finish()?;
        facet_json::to_string(&values).map_err(|_| fmt::Error)
    }
}

impl<'a> FormatFields<'a> for FacetJsonFields {
    fn format_fields<R: RecordFields>(&self, mut writer: Writer<'_>, fields: R) -> fmt::Result {
        match self.merge("", fields) {
            Ok(encoded) => writer.write_str(&encoded),
            Err(_) => {
                // Avoid upstream on_new_span's raw stderr fallback. Keep valid empty context.
                self.diagnostics.format_error();
                writer.write_str("{}")
            }
        }
    }

    fn add_fields(
        &self,
        current: &'a mut FormattedFields<Self>,
        fields: &Record<'_>,
    ) -> fmt::Result {
        match self.merge(&current.fields, fields) {
            Ok(encoded) => {
                current.fields = encoded;
                Ok(())
            }
            Err(error) => {
                self.diagnostics.format_error();
                Err(error)
            }
        }
    }
}
