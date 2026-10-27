//! Executable proposed adapter and documented independent-sink contract.
//! The provisioned template revision has no typed startup formatter; see receipt.
#[path = "support/failing_time.rs"]
mod failing_time;
#[path = "support/fixed_time.rs"]
mod fixed_time;
#[path = "../examples/typed_startup/payload_visitor.rs"]
mod payload_visitor;
#[path = "../examples/typed_startup/startup_fields.rs"]
mod startup_fields;
#[path = "../examples/typed_startup/startup_formatter.rs"]
mod startup_formatter;
#[path = "../examples/typed_startup/startup_record.rs"]
mod startup_record;
use fixed_time::FixedTime;
use startup_fields::StartupFields;
use startup_formatter::StartupFormatter;
use startup_record::StartupRecord;
use teamy_tracing::Diagnostics;
use teamy_tracing::SharedWriter;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::prelude::*;

fn payload() -> StartupFields {
    StartupFields {
        message: "Application started".into(),
        argv: ["synthetic-tool", "inspect", "--source", "quoted\" path\\日本語\r\n.json", "", "--log-file", "fixture.ndjson"]
            .into_iter().map(String::from).collect(),
        version: "0.1.0 (repo https://example.invalid/tool, branch fixture, rev synthetic, worktree clean, built 2026-10-23T00:00:00Z)".into(),
    }
}
fn filter(value: &str) -> EnvFilter {
    EnvFilter::builder().with_regex(false).parse(value).unwrap()
}
fn emit(encoded: &str) {
    tracing::info!(target: "example::startup", encoded);
    let span = tracing::info_span!("operation", completed = 0u64);
    let _entered = span.enter();
    span.record("completed", u64::MAX);
    tracing::debug!(phase = "complete", "progress");
    tracing::warn!("fixture warning");
}
fn check_startup(text: &str) {
    let startup: StartupRecord = facet_json::from_str(text.lines().next().unwrap()).unwrap();
    assert_eq!(startup.fields, payload());
    assert_eq!(startup.timestamp, "2026-10-23T00:00:00Z");
    assert_eq!(startup.level, "INFO");
    assert_eq!(startup.target, "example::startup");
    assert!(startup.filename.unwrap().ends_with("startup_contract.rs"));
    assert!(startup.line_number.unwrap() > 0);
    assert!(text.lines().next().unwrap().contains("\"argv\":["));
}
fn check_ndjson(text: &str, count: usize) {
    assert_eq!(text.lines().count(), count);
    assert_eq!(text.bytes().filter(|b| *b == b'\n').count(), count);
    assert!(!text.contains(['\r', '\u{1b}']));
    for line in text.lines() {
        let _: facet_value::Value = facet_json::from_str(line).unwrap();
    }
}

#[test]
fn typed_startup_two_ndjson_sinks_preserve_independent_filters_and_close_policy() {
    for (terminal_filter, file_filter, terminal_count, file_count) in [
        ("off", "debug", 0, 4),
        ("warn", "debug", 1, 4),
        ("info", "off", 2, 0),
        ("info", "debug", 2, 4),
    ] {
        let diagnostics = Diagnostics::default();
        let terminal = SharedWriter::new(Vec::<u8>::new(), diagnostics.clone());
        let file = SharedWriter::new(Vec::<u8>::new(), diagnostics.clone());
        let formatter = StartupFormatter::new(diagnostics.clone()).with_timer(FixedTime);
        let mut file_layer = teamy_tracing::json_layer(file.clone(), diagnostics.clone())
            .event_format(formatter.clone());
        file_layer.set_span_events(FmtSpan::CLOSE);
        let subscriber = tracing_subscriber::registry()
            .with(tracing_error::ErrorLayer::default())
            .with(
                teamy_tracing::json_layer(terminal.clone(), diagnostics.clone())
                    .event_format(formatter.clone())
                    .with_filter(filter(terminal_filter)),
            )
            .with(file_layer.with_filter(filter(file_filter)));
        let encoded = facet_json::to_string(&payload()).unwrap();
        tracing::subscriber::with_default(subscriber, || emit(&encoded));
        terminal.flush().unwrap();
        file.flush().unwrap();
        let terminal = String::from_utf8(terminal.inspect(Clone::clone).unwrap()).unwrap();
        let file = String::from_utf8(file.inspect(Clone::clone).unwrap()).unwrap();
        check_ndjson(&terminal, terminal_count);
        check_ndjson(&file, file_count);
        assert!(!terminal.contains("\"message\":\"close\""));
        assert!(!terminal.contains("\"message\":\"progress\""));
        if terminal_filter == "info" {
            check_startup(&terminal);
        }
        if file_count > 0 {
            check_startup(&file);
            assert!(file.contains("\"message\":\"close\""));
            assert!(file.contains("\"time.busy\":"));
            assert!(file.contains("\"completed\":18446744073709551615"));
        }
        assert_eq!(formatter.formatting_errors(), 0);
        assert_eq!(diagnostics.snapshot(), Default::default());
    }
}

#[test]
fn text_terminal_and_typed_file_keep_result_stream_and_filters_independent() {
    use std::io::Write;
    let diagnostics = Diagnostics::default();
    let terminal = SharedWriter::new(Vec::<u8>::new(), diagnostics.clone());
    let file = SharedWriter::new(Vec::<u8>::new(), diagnostics.clone());
    let formatter = StartupFormatter::new(diagnostics.clone()).with_timer(FixedTime);
    let mut file_layer = teamy_tracing::json_layer(file.clone(), diagnostics.clone())
        .event_format(formatter.clone());
    file_layer.set_span_events(FmtSpan::CLOSE);
    let subscriber = tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_ansi(false)
                .without_time()
                .with_writer(terminal.clone())
                .with_filter(filter("warn")),
        )
        .with(file_layer.with_filter(filter("debug")));
    let mut result = Vec::new();
    tracing::subscriber::with_default(subscriber, || {
        emit(&facet_json::to_string(&payload()).unwrap());
        writeln!(result, "{{\"completed\":3}}").unwrap();
    });
    terminal.flush().unwrap();
    file.flush().unwrap();
    assert_eq!(result, b"{\"completed\":3}\n");
    let terminal = String::from_utf8(terminal.inspect(Clone::clone).unwrap()).unwrap();
    assert!(terminal.contains("fixture warning"));
    assert!(!terminal.contains("Application started"));
    assert!(!terminal.contains("progress"));
    assert!(!terminal.contains('\u{1b}'));
    let file = String::from_utf8(file.inspect(Clone::clone).unwrap()).unwrap();
    check_startup(&file);
    check_ndjson(&file, 4);
    assert_eq!(formatter.formatting_errors(), 0);
    assert_eq!(diagnostics.snapshot(), Default::default());
}

#[test]
fn malformed_reserved_payloads_drop_whole_records_and_count_adapter_failures() {
    let diagnostics = Diagnostics::default();
    let writer = SharedWriter::new(Vec::<u8>::new(), diagnostics.clone());
    let formatter = StartupFormatter::new(diagnostics.clone()).with_timer(FixedTime);
    let subscriber = tracing_subscriber::registry().with(
        teamy_tracing::json_layer(writer.clone(), diagnostics.clone())
            .event_format(formatter.clone()),
    );
    tracing::subscriber::with_default(subscriber, || {
        for encoded in [
            "{",
            "{}",
            r#"{"message":"start","argv":"not an array","version":"fixture"}"#,
        ] {
            tracing::info!(target: "example::startup", encoded);
        }
        tracing::info!(target: "example::startup", "missing encoded");
        tracing::info!(target: "example::startup", encoded = ?payload());
        tracing::info!(target: "ordinary", encoded = "{not raw JSON}", "fallback");
    });
    let text = String::from_utf8(writer.inspect(Clone::clone).unwrap()).unwrap();
    check_ndjson(&text, 1);
    assert!(text.contains("{not raw JSON}"));
    assert_eq!(formatter.formatting_errors(), 5);
    assert_eq!(diagnostics.snapshot(), Default::default());
}

#[test]
fn startup_and_fallback_timer_failures_are_accounted_separately() {
    let diagnostics = Diagnostics::default();
    let writer = SharedWriter::new(Vec::<u8>::new(), diagnostics.clone());
    let formatter =
        StartupFormatter::new(diagnostics.clone()).with_timer(failing_time::FailingTime);
    let subscriber = tracing_subscriber::registry().with(
        teamy_tracing::json_layer(writer.clone(), diagnostics.clone())
            .event_format(formatter.clone()),
    );
    tracing::subscriber::with_default(subscriber, || {
        let encoded = facet_json::to_string(&payload()).unwrap();
        tracing::info!(target: "example::startup", encoded = encoded.as_str());
        tracing::info!("fallback timer fails");
    });
    assert!(writer.inspect(Vec::is_empty).unwrap());
    assert_eq!(formatter.formatting_errors(), 1);
    assert_eq!(diagnostics.snapshot().formatting_errors, 1);
}
