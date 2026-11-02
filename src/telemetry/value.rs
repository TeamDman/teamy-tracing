use super::Attribute;
use facet::Facet;

/// Values supported by OTLP AnyValue; empty differs from a missing attribute.
#[derive(Clone, Debug, Facet, PartialEq)]
#[repr(u8)]
pub enum TelemetryValue {
    Empty,
    String(String),
    Bool(bool),
    Integer(i64),
    Float(f64),
    Bytes(Vec<u8>),
    Array(Vec<TelemetryValue>),
    Map(Vec<Attribute>),
}

impl TelemetryValue {
    pub(crate) fn logical_size(&self) -> usize {
        let payload = match self {
            Self::Empty | Self::Bool(_) | Self::Integer(_) | Self::Float(_) => 8,
            Self::String(value) => value.len(),
            Self::Bytes(value) => value.len(),
            Self::Array(values) => values
                .iter()
                .fold(0usize, |n, value| n.saturating_add(value.logical_size())),
            Self::Map(values) => values.iter().fold(0usize, |n, value| {
                n.saturating_add(value.key.len())
                    .saturating_add(value.value.logical_size())
            }),
        };
        payload.saturating_add(64)
    }
}
