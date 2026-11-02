use super::TelemetryValue;
use facet::Facet;

/// An attribute retains its original key and typed value, including unknown keys.
#[derive(Clone, Debug, Facet, PartialEq)]
pub struct Attribute {
    pub key: String,
    pub value: TelemetryValue,
}
