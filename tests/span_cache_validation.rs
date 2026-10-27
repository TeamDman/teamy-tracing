#[path = "support/cache_mutation_layer.rs"]
mod cache_mutation_layer;
use teamy_tracing::Diagnostics;
use teamy_tracing::SharedWriter;
use tracing_subscriber::prelude::*;

#[test]
fn malformed_nested_cache_and_trailing_input_fail_events_and_updates() {
    for source in [
        r#"{"injected":[1 2]}"#,
        r#"{"injected":{"a" 1}}"#,
        r#"{"injected":{"a":1 "b":2}}"#,
        r#"{"injected":[1,]}"#,
        r#"{"injected":{"a":1,}}"#,
        r#"{"injected":1,}"#,
        r#"{"injected":1} {"other":2}"#,
        r#"{"injected":1} trailing"#,
        r#"{"injected":[,1]}"#,
        r#"{"injected":[1,,2]}"#,
        r#"{"injected":[01]}"#,
        r#"{"injected":[/*comment*/1]}"#,
        r#"{"injected":1.}"#,
        r#"{"injected":1e}"#,
        r#"{"injected":-01}"#,
        r#"{"injected":1.e2}"#,
        r#"{"injected":1e+}"#,
        r#"{"injected":"\q"}"#,
        r#"{"injected":"\u00"}"#,
        r#"{"injected":"\u€€"}"#,
        r#"{"\u€€":1}"#,
        "{\"injected\":\"raw\tcontrol\"}",
        "{\"injected\":\"raw\u{0001}control\"}",
    ] {
        let diagnostics = Diagnostics::default();
        let writer = SharedWriter::new(Vec::<u8>::new(), diagnostics.clone());
        let subscriber = tracing_subscriber::registry()
            .with(teamy_tracing::json_layer(
                writer.clone(),
                diagnostics.clone(),
            ))
            .with(cache_mutation_layer::CacheMutationLayer(source));
        tracing::subscriber::with_default(subscriber, || {
            let span = tracing::info_span!("cache", value = 0u64);
            let _entered = span.enter();
            tracing::info!("warm");
            span.record("value", 1u64); // External layer installs malformed source.
            tracing::info!("must drop");
            span.record("value", 2u64); // Merge must also reject that source.
            tracing::info!("must still drop");
        });
        let text = String::from_utf8(writer.inspect(Clone::clone).unwrap()).unwrap();
        assert_eq!(text.lines().count(), 1, "cache accepted: {source}");
        assert_eq!(diagnostics.snapshot().formatting_errors, 3, "{source}");
        assert_eq!(diagnostics.snapshot().write_errors, 0);
    }
}

#[test]
fn valid_nested_cache_preserves_lexemes_and_string_delimiters() {
    let source = r#" {"injected":[18446744073709551615,-0.0,1e+2,-1.25E-3,{"text":"comma,} bracket] quote\""}],"flags":[true,false,null],"value":0} "#;
    let diagnostics = Diagnostics::default();
    let writer = SharedWriter::new(Vec::<u8>::new(), diagnostics.clone());
    let subscriber = tracing_subscriber::registry()
        .with(teamy_tracing::json_layer(
            writer.clone(),
            diagnostics.clone(),
        ))
        .with(cache_mutation_layer::CacheMutationLayer(source));
    tracing::subscriber::with_default(subscriber, || {
        let span = tracing::info_span!("cache", value = 0u64);
        let _entered = span.enter();
        span.record("value", 1u64);
        tracing::info!("first validation");
        tracing::info!("cached rendering");
        span.record("value", 2u64);
        tracing::info!("valid merge");
    });
    let text = String::from_utf8(writer.inspect(Clone::clone).unwrap()).unwrap();
    assert_eq!(text.lines().count(), 3);
    for line in text.lines() {
        assert!(line.contains(
            r#"[18446744073709551615,-0.0,1e+2,-1.25E-3,{"text":"comma,} bracket] quote\""}]"#
        ));
        let _: facet_value::Value = facet_json::from_str(line).unwrap();
    }
    assert_eq!(diagnostics.snapshot(), Default::default());
}
