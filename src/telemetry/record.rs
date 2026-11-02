use super::Attribute;
use super::TelemetryValue;
use facet::Facet;

/// A source record, preserving OTLP attributes and attribution without re-emitting it.
#[derive(Clone, Debug, Default, Facet, PartialEq)]
pub struct TelemetryRecord {
    pub resource_attributes: Vec<Attribute>,
    pub scope_attributes: Vec<Attribute>,
    pub attributes: Vec<Attribute>,
    pub resource_schema_url: String,
    pub scope_schema_url: String,
    pub scope_name: String,
    pub scope_version: String,
    pub time_unix_nano: u64,
    pub observed_time_unix_nano: u64,
    pub severity_number: i32,
    pub severity_text: String,
    pub event_name: String,
    pub body: Option<TelemetryValue>,
    pub trace_id: Vec<u8>,
    pub span_id: Vec<u8>,
    pub flags: u32,
    pub dropped_attributes_count: u32,
    pub resource_dropped_attributes_count: u32,
    pub scope_dropped_attributes_count: u32,
}

impl TelemetryRecord {
    /// Return the first matching event attribute without changing its type.
    pub fn attribute(&self, key: &str) -> Option<&TelemetryValue> {
        self.attributes
            .iter()
            .find(|attribute| attribute.key == key)
            .map(|attribute| &attribute.value)
    }

    /// The conservative logical budget is not a measurement of allocator overhead.
    pub fn logical_size(&self) -> usize {
        let mut size = 512usize;
        for text in [
            &self.resource_schema_url,
            &self.scope_schema_url,
            &self.scope_name,
            &self.scope_version,
            &self.severity_text,
            &self.event_name,
        ] {
            size = size.saturating_add(text.len());
        }
        size = size
            .saturating_add(self.trace_id.len())
            .saturating_add(self.span_id.len());
        if let Some(body) = &self.body {
            size = size.saturating_add(body.logical_size());
        }
        for attr in self
            .resource_attributes
            .iter()
            .chain(&self.scope_attributes)
            .chain(&self.attributes)
        {
            size = size
                .saturating_add(attr.key.len())
                .saturating_add(attr.value.logical_size());
        }
        size
    }
}
