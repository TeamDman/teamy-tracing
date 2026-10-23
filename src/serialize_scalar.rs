use facet_format::FormatSerializer;
use facet_format::ScalarValue;
use facet_json::JsonSerializer;
use std::fmt;

/// Retain float spelling (especially -0.0) with Facet's standard serializer.
pub(crate) fn serialize_scalar(
    serializer: &mut JsonSerializer,
    value: ScalarValue<'_>,
) -> fmt::Result {
    if let ScalarValue::F64(number) = value
        && number.is_finite()
        && number.fract() == 0.0
    {
        let mut scalar = JsonSerializer::new();
        scalar
            .scalar(ScalarValue::F64(number))
            .map_err(|_| fmt::Error)?;
        let mut encoded = String::from_utf8(scalar.finish()).map_err(|_| fmt::Error)?;
        if !encoded.contains(['.', 'e', 'E']) {
            encoded.push_str(".0");
        }
        // Only a validated, serializer-generated number enters raw_scalar.
        return serializer.raw_scalar(&encoded).map_err(|_| fmt::Error);
    }
    serializer.scalar(value).map_err(|_| fmt::Error)
}
