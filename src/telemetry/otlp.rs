//! Decode OTLP/HTTP protobuf log requests into an application-owned store.
//!
//! This is a wire adapter, not an HTTP server. The caller owns transport,
//! admission, authentication, request limits, response and lifecycle.
use super::Attribute;
use super::RecordStore;
use super::TelemetryRecord;
use super::TelemetryValue;
use opentelemetry_proto::tonic::collector::logs::v1::ExportLogsServiceRequest;
use opentelemetry_proto::tonic::common::v1::AnyValue;
use opentelemetry_proto::tonic::common::v1::KeyValue;
use opentelemetry_proto::tonic::common::v1::any_value;
use prost::Message;

/// Result of decoding and admitting one request.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct IngestResult {
    pub accepted: usize,
    pub rejected: usize,
}

/// Decode a bounded binary request. No record is inserted if protobuf decoding fails.
/// The returned rejection count must be reflected in the transport's response.
pub fn ingest_protobuf(
    bytes: &[u8],
    max_request_bytes: usize,
    store: &mut RecordStore,
) -> Result<IngestResult, IngestError> {
    if bytes.len() > max_request_bytes {
        return Err(IngestError::RequestTooLarge);
    }
    let request =
        ExportLogsServiceRequest::decode(bytes).map_err(|_| IngestError::InvalidProtobuf)?;
    let mut result = IngestResult::default();
    for resource_logs in request.resource_logs {
        let resource = resource_logs.resource.unwrap_or_default();
        let resource_attributes = attributes(resource.attributes);
        for scope_logs in resource_logs.scope_logs {
            let scope = scope_logs.scope.unwrap_or_default();
            let scope_attributes = attributes(scope.attributes);
            for log in scope_logs.log_records {
                let record = TelemetryRecord {
                    resource_attributes: resource_attributes.clone(),
                    scope_attributes: scope_attributes.clone(),
                    attributes: attributes(log.attributes),
                    resource_schema_url: resource_logs.schema_url.clone(),
                    scope_schema_url: scope_logs.schema_url.clone(),
                    scope_name: scope.name.clone(),
                    scope_version: scope.version.clone(),
                    time_unix_nano: log.time_unix_nano,
                    observed_time_unix_nano: log.observed_time_unix_nano,
                    severity_number: log.severity_number,
                    severity_text: log.severity_text,
                    event_name: log.event_name,
                    body: log.body.map(value),
                    trace_id: log.trace_id,
                    span_id: log.span_id,
                    flags: log.flags,
                    dropped_attributes_count: log.dropped_attributes_count,
                    resource_dropped_attributes_count: resource.dropped_attributes_count,
                    scope_dropped_attributes_count: scope.dropped_attributes_count,
                };
                if store.push(record) {
                    result.accepted += 1;
                } else {
                    result.rejected += 1;
                }
            }
        }
    }
    Ok(result)
}

/// Errors intentionally contain no request data.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IngestError {
    RequestTooLarge,
    InvalidProtobuf,
}

fn attributes(values: Vec<KeyValue>) -> Vec<Attribute> {
    values
        .into_iter()
        .map(|entry| Attribute {
            key: entry.key,
            value: entry.value.map_or(TelemetryValue::Empty, value),
        })
        .collect()
}

fn value(value: AnyValue) -> TelemetryValue {
    match value.value {
        None => TelemetryValue::Empty,
        Some(any_value::Value::StringValue(value)) => TelemetryValue::String(value),
        Some(any_value::Value::BoolValue(value)) => TelemetryValue::Bool(value),
        Some(any_value::Value::IntValue(value)) => TelemetryValue::Integer(value),
        Some(any_value::Value::DoubleValue(value)) => TelemetryValue::Float(value),
        Some(any_value::Value::BytesValue(value)) => TelemetryValue::Bytes(value),
        Some(any_value::Value::ArrayValue(values)) => {
            TelemetryValue::Array(values.values.into_iter().map(self::value).collect())
        }
        Some(any_value::Value::KvlistValue(values)) => {
            TelemetryValue::Map(attributes(values.values))
        }
    }
}
