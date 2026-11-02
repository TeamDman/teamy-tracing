use super::Attribute;
use super::TelemetryRecord;
use super::TelemetryValue;

/// A conservative source/session/call key. Missing attribution is not guessed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CorrelationKey {
    pub source: String,
    pub session: String,
    pub call: String,
}

impl CorrelationKey {
    /// Uses resource `service.instance.id`, event `conversation.id` and `call_id`.
    /// Other producers may need their own key adapter; `None` means uncorrelated.
    pub fn from_record(record: &TelemetryRecord) -> Option<Self> {
        fn string<'a>(attributes: &'a [Attribute], key: &str) -> Option<&'a str> {
            attributes
                .iter()
                .find(|attribute| attribute.key == key)
                .and_then(|attribute| match &attribute.value {
                    TelemetryValue::String(value) if !value.is_empty() => Some(value.as_str()),
                    _ => None,
                })
        }
        Some(Self {
            source: string(&record.resource_attributes, "service.instance.id")?.into(),
            session: string(&record.attributes, "conversation.id")?.into(),
            call: string(&record.attributes, "call_id")?.into(),
        })
    }
}
