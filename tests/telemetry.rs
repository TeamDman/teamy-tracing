#![cfg(feature = "telemetry")]

use teamy_tracing::telemetry::Attribute;
use teamy_tracing::telemetry::RecordStore;
use teamy_tracing::telemetry::StoreLimits;
use teamy_tracing::telemetry::TelemetryRecord;
use teamy_tracing::telemetry::TelemetryValue;

fn record(call: &str, truncated: bool) -> TelemetryRecord {
    TelemetryRecord {
        attributes: vec![
            Attribute {
                key: "call_id".into(),
                value: TelemetryValue::String(call.into()),
            },
            Attribute {
                key: "output_truncated".into(),
                value: TelemetryValue::Bool(truncated),
            },
        ],
        ..TelemetryRecord::default()
    }
}

#[test]
fn bounded_store_counts_eviction_rejection_and_upstream_truncation() {
    let first = record("first", false);
    let limit = first.logical_size() + 100;
    let mut store = RecordStore::new(StoreLimits {
        max_records: 1,
        max_logical_bytes: limit,
    });
    assert!(store.push(first));
    assert!(store.push(record("second", true)));
    let mut big = record("big", false);
    big.body = Some(TelemetryValue::String("x".repeat(limit)));
    assert!(!store.push(big));
    let stats = store.stats();
    assert_eq!(
        (
            stats.accepted,
            stats.evicted,
            stats.rejected_oversize,
            stats.upstream_truncated,
            stats.records
        ),
        (2, 1, 1, 1, 1)
    );
    assert_eq!(
        store.iter().next().unwrap().attribute("call_id"),
        Some(&TelemetryValue::String("second".into()))
    );
    store.clear();
    assert_eq!(store.stats().records, 0);
    assert_eq!(store.stats().evicted, 1);
}

#[test]
fn facet_round_trip_preserves_nested_values_and_unknown_fields() {
    let record = TelemetryRecord {
        attributes: vec![Attribute {
            key: "unrecognized".into(),
            value: TelemetryValue::Array(vec![
                TelemetryValue::Integer(42),
                TelemetryValue::Bool(true),
                TelemetryValue::String("42".into()),
            ]),
        }],
        ..TelemetryRecord::default()
    };
    let json = facet_json::to_string(&record).unwrap();
    let restored: TelemetryRecord = facet_json::from_str(&json).unwrap();
    assert_eq!(restored, record);
}

#[cfg(feature = "otlp")]
#[test]
fn protobuf_ingestion_keeps_source_call_types_and_truncation() {
    use opentelemetry_proto::tonic::collector::logs::v1::ExportLogsServiceRequest;
    use opentelemetry_proto::tonic::common::v1::AnyValue;
    use opentelemetry_proto::tonic::common::v1::KeyValue;
    use opentelemetry_proto::tonic::common::v1::any_value::Value;
    use opentelemetry_proto::tonic::logs::v1::LogRecord;
    use opentelemetry_proto::tonic::logs::v1::ResourceLogs;
    use opentelemetry_proto::tonic::logs::v1::ScopeLogs;
    use opentelemetry_proto::tonic::resource::v1::Resource;
    use prost::Message;
    use teamy_tracing::telemetry::otlp::IngestError;
    use teamy_tracing::telemetry::otlp::ingest_protobuf;

    fn kv(key: &str, value: Value) -> KeyValue {
        KeyValue {
            key: key.into(),
            value: Some(AnyValue { value: Some(value) }),
        }
    }
    let request = ExportLogsServiceRequest {
        resource_logs: vec![ResourceLogs {
            resource: Some(Resource {
                attributes: vec![kv("service.name", Value::StringValue("synthetic".into()))],
                dropped_attributes_count: 2,
                ..Default::default()
            }),
            scope_logs: vec![ScopeLogs {
                log_records: vec![LogRecord {
                    attributes: vec![
                        kv("event.name", Value::StringValue("codex.tool_result".into())),
                        kv("conversation.id", Value::StringValue("session-1".into())),
                        kv("call_id", Value::StringValue("call-1".into())),
                        kv(
                            "arguments",
                            Value::StringValue(r#"{"cmd":"Get-Date","shell":"powershell"}"#.into()),
                        ),
                        kv("output_truncated", Value::BoolValue(true)),
                        kv("unknown_number", Value::IntValue(17)),
                    ],
                    dropped_attributes_count: 3,
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        }],
    };
    let bytes = request.encode_to_vec();
    let mut store = RecordStore::default();
    let result = ingest_protobuf(&bytes, bytes.len(), &mut store).unwrap();
    assert_eq!((result.accepted, result.rejected), (1, 0));
    let record = store.iter().next().unwrap();
    assert_eq!(
        record.attribute("unknown_number"),
        Some(&TelemetryValue::Integer(17))
    );
    assert_eq!(
        record.attribute("call_id"),
        Some(&TelemetryValue::String("call-1".into()))
    );
    assert_eq!(
        record.resource_attributes[0].value,
        TelemetryValue::String("synthetic".into())
    );
    assert_eq!(
        (
            record.resource_dropped_attributes_count,
            record.dropped_attributes_count,
            store.stats().upstream_truncated
        ),
        (2, 3, 1)
    );
    assert_eq!(
        ingest_protobuf(&bytes, 0, &mut store),
        Err(IngestError::RequestTooLarge)
    );
    assert_eq!(
        ingest_protobuf(&[0xff], 1, &mut store),
        Err(IngestError::InvalidProtobuf)
    );
    assert_eq!(store.stats().accepted, 1);
}

#[test]
fn correlation_requires_source_session_and_call() {
    use teamy_tracing::telemetry::CorrelationKey;
    let mut decision = record("call-1", false);
    decision.resource_attributes.push(Attribute {
        key: "service.instance.id".into(),
        value: TelemetryValue::String("producer-1".into()),
    });
    decision.attributes.push(Attribute {
        key: "conversation.id".into(),
        value: TelemetryValue::String("session-1".into()),
    });
    decision.event_name = "codex.tool_decision".into();
    let key = CorrelationKey::from_record(&decision).unwrap();
    let mut result = decision.clone();
    result.event_name = "codex.tool_result".into();
    let mut other = result.clone();
    other.resource_attributes.clear();
    assert!(CorrelationKey::from_record(&other).is_none());
    let mut store = RecordStore::default();
    assert!(store.push(decision));
    assert!(store.push(result));
    assert!(store.push(other));
    assert_eq!(store.matching_call(&key).count(), 2);
}

#[cfg(feature = "egui")]
#[test]
fn view_filter_and_display_escape_controls() {
    use teamy_tracing::telemetry::view::escape_display;
    use teamy_tracing::telemetry::view::matches_filter;
    let record = record("synthetic-call", false);
    assert!(matches_filter(&record, "SYNTHETIC"));
    assert!(!matches_filter(&record, "missing"));
    assert_eq!(
        escape_display("a\u{1b}b\u{202e}\nc"),
        "a\\u{1b}b\\u{202e}\nc"
    );
    let mut store = RecordStore::default();
    assert!(store.push(record));
    let mut view = teamy_tracing::telemetry::view::RecordView::default();
    let context = egui::Context::default();
    let output = context.run(egui::RawInput::default(), |context| {
        egui::CentralPanel::default().show(context, |ui| view.show(ui, &store));
    });
    assert!(!output.shapes.is_empty());
}

#[cfg(feature = "otlp-http")]
#[test]
fn loopback_receiver_accepts_protobuf_and_rejects_bad_requests() {
    use opentelemetry_proto::tonic::collector::logs::v1::ExportLogsServiceRequest;
    use opentelemetry_proto::tonic::collector::logs::v1::ExportLogsServiceResponse;
    use opentelemetry_proto::tonic::logs::v1::LogRecord;
    use opentelemetry_proto::tonic::logs::v1::ResourceLogs;
    use opentelemetry_proto::tonic::logs::v1::ScopeLogs;
    use prost::Message;
    use std::sync::Arc;
    use std::sync::Mutex;
    use teamy_tracing::telemetry::otlp_http::Receiver;
    use teamy_tracing::telemetry::otlp_http::ReceiverLimits;
    use tokio::io::AsyncReadExt;
    use tokio::io::AsyncWriteExt;

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async {
        let store = Arc::new(Mutex::new(RecordStore::default()));
        let receiver = Receiver::bind_loopback(0, ReceiverLimits { max_request_bytes: 256, body_timeout: std::time::Duration::from_millis(100), ..ReceiverLimits::default() }, Arc::clone(&store)).await.unwrap();
        assert!(receiver.address().ip().is_loopback());
        assert!(Receiver::bind_loopback(receiver.address().port(), ReceiverLimits::default(), Arc::clone(&store)).await.is_err());
        async fn send(address: std::net::SocketAddr, content_type: &str, payload: &[u8]) -> Vec<u8> {
            let mut stream = tokio::net::TcpStream::connect(address).await.unwrap();
            let header = format!("POST /v1/logs HTTP/1.1\r\nHost: localhost\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", payload.len());
            stream.write_all(header.as_bytes()).await.unwrap();
            stream.write_all(payload).await.unwrap();
            let mut response = Vec::new();
            stream.read_to_end(&mut response).await.unwrap();
            response
        }
        let request = ExportLogsServiceRequest { resource_logs: vec![ResourceLogs { scope_logs: vec![ScopeLogs { log_records: vec![LogRecord { event_name: "codex.tool_result".into(), ..Default::default() }], ..Default::default() }], ..Default::default() }] }.encode_to_vec();
        let valid = send(receiver.address(), "application/x-protobuf", &request).await;
        assert!(valid.starts_with(b"HTTP/1.1 200"));
        assert_eq!(store.lock().unwrap().stats().accepted, 1);
        assert!(send(receiver.address(), "application/json", b"{}").await.starts_with(b"HTTP/1.1 415"));
        assert!(send(receiver.address(), "application/x-protobuf", &[0xff]).await.starts_with(b"HTTP/1.1 400"));
        assert!(send(receiver.address(), "application/x-protobuf", &[0; 257]).await.starts_with(b"HTTP/1.1 413"));
        let mut slow = tokio::net::TcpStream::connect(receiver.address()).await.unwrap();
        slow.write_all(b"POST /v1/logs HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/x-protobuf\r\nContent-Length: 2\r\nConnection: close\r\n\r\n").await.unwrap();
        let mut timed_out = Vec::new();
        slow.read_to_end(&mut timed_out).await.unwrap();
        assert!(timed_out.starts_with(b"HTTP/1.1 408"));
        assert_eq!(store.lock().unwrap().stats().accepted, 1);
        receiver.shutdown().await.unwrap();
        let rejecting = Arc::new(Mutex::new(RecordStore::new(StoreLimits { max_records: 0, max_logical_bytes: 0 })));
        let receiver = Receiver::bind_loopback(0, ReceiverLimits::default(), Arc::clone(&rejecting)).await.unwrap();
        let response = send(receiver.address(), "application/x-protobuf", &request).await;
        let body_offset = response.windows(4).position(|window| window == b"\r\n\r\n").unwrap() + 4;
        let decoded = ExportLogsServiceResponse::decode(&response[body_offset..]).unwrap();
        assert_eq!(decoded.partial_success.unwrap().rejected_log_records, 1);
        assert_eq!(rejecting.lock().unwrap().stats().rejected_oversize, 1);
        receiver.shutdown().await.unwrap();
    });
}
