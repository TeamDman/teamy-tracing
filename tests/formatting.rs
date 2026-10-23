mod support;
use facet_value::Value;
use support::capture;
use support::format;
use support::raw;
use tracing::field;

#[test]
fn typed_values_escaping_and_one_line() {
    let (text, status) = capture(format().with_target(false), || {
        let error = std::io::Error::other("oops\nsecond line");
        tracing::info!(signed = i64::MIN, unsigned = u64::MAX, float = 1.25, integral_float = 1.0,
            negative_zero = -0.0, not_number = f64::NAN, infinite = f64::INFINITY,
            wide = i128::MIN, widest = u128::MAX, boolean = true, error = &error as &dyn std::error::Error,
            bytes = &[0u8, 15, 255][..], debug = ?vec![1, 2], display = %"multi\nline",
            escaped = "quote\" slash\\\r\n\t\u{1b} 日本語", "hello\nworld");
    });
    assert_eq!(text.lines().count(), 1);
    assert!(text.ends_with('\n'));
    assert!(!text.contains('\r'));
    assert!(!text.contains('\u{1b}'));
    let _: Value = facet_json::from_str(text.trim_end()).unwrap();
    let outer = raw(&text);
    let fields = raw(outer["fields"].as_str());
    assert_eq!(fields["signed"].as_str(), i64::MIN.to_string());
    assert_eq!(fields["unsigned"].as_str(), u64::MAX.to_string());
    assert_eq!(fields["float"].as_str(), "1.25");
    assert_eq!(fields["negative_zero"].as_str(), "-0.0");
    assert_eq!(fields["integral_float"].as_str(), "1.0");
    assert_eq!(fields["not_number"].as_str(), "null");
    assert_eq!(fields["infinite"].as_str(), "null");
    assert_eq!(fields["wide"].as_str(), format!("\"{}\"", i128::MIN));
    assert_eq!(fields["widest"].as_str(), format!("\"{}\"", u128::MAX));
    assert_eq!(fields["boolean"].as_str(), "true");
    assert_eq!(fields["bytes"].as_str(), "\"[00 0f ff]\"");
    assert_eq!(fields["error"].as_str(), "\"oops\\nsecond line\"");
    assert_eq!(fields["display"].as_str(), "\"multi\\nline\"");
    assert_eq!(status.formatting_errors, 0);
}

#[test]
fn span_updates_preserve_numbers_and_replace_fields() {
    let (text, status) = capture(format(), || {
        let span = tracing::info_span!(
            "operation",
            answer = field::Empty,
            counter = 1u64,
            exact = u64::MAX,
            fraction = -0.0,
            bytes = &[0u8, 255][..]
        );
        let _entered = span.enter();
        tracing::info!("before");
        span.record("answer", "arrived\nlate");
        span.record("counter", 2u64);
        span.record("counter", 3u64);
        tracing::info!("after");
    });
    let lines: Vec<_> = text.lines().map(raw).collect();
    let before = raw(lines[0]["span"].as_str());
    let after = raw(lines[1]["span"].as_str());
    assert!(!before.contains_key("answer"));
    assert_eq!(after["counter"].as_str(), "3");
    assert_eq!(after["exact"].as_str(), u64::MAX.to_string());
    assert_eq!(after["fraction"].as_str(), "-0.0");
    assert_eq!(after["bytes"].as_str(), "[0,255]");
    assert_eq!(after["answer"].as_str(), "\"arrived\\nlate\"");
    assert_eq!(after["name"].as_str(), "\"operation\"");
    assert_eq!(status.formatting_errors, 0);
}

#[test]
fn explicit_parent_and_explicit_root_control_both_span_fields() {
    let (text, _) = capture(format(), || {
        let parent = tracing::info_span!("parent");
        let leaf = tracing::info_span!(parent: &parent, "leaf");
        let unrelated = tracing::info_span!("unrelated");
        let _entered = unrelated.enter();
        tracing::info!(parent: &leaf, "explicit");
        tracing::info!(parent: None, "root");
    });
    let lines: Vec<_> = text.lines().map(raw).collect();
    assert_eq!(raw(lines[0]["span"].as_str())["name"].as_str(), "\"leaf\"");
    assert_eq!(
        lines[0]["spans"].as_str(),
        "[{\"name\":\"parent\"},{\"name\":\"leaf\"}]"
    );
    assert!(!lines[1].contains_key("span"));
    assert!(!lines[1].contains_key("spans"));
}

#[test]
fn flattened_collisions_remain_in_stream_order() {
    let (text, _) = capture(format().flatten_event(true), || {
        tracing::info!(
            timestamp = "producer timestamp",
            target = "producer target",
            "flattened"
        );
    });
    assert_eq!(text.matches("\"timestamp\":").count(), 2);
    assert_eq!(text.matches("\"target\":").count(), 2);
    assert!(!text.contains("\"fields\":"));
    assert!(text.find("producer target").unwrap() < text.rfind("\"target\":").unwrap());
    let _: Value = facet_json::from_str(&text).unwrap();
}

#[test]
fn optional_metadata_and_named_thread() {
    let text = std::thread::Builder::new()
        .name("worker\nname".into())
        .spawn(|| {
            capture(
                format()
                    .without_time()
                    .with_level(false)
                    .with_target(false)
                    .with_file(true)
                    .with_line_number(true)
                    .with_thread_names(true)
                    .with_thread_ids(true),
                || tracing::warn!("location"),
            )
            .0
        })
        .unwrap()
        .join()
        .unwrap();
    let outer = raw(&text);
    assert!(!outer.contains_key("timestamp"));
    assert!(!outer.contains_key("level"));
    assert!(!outer.contains_key("target"));
    assert_eq!(outer["threadName"].as_str(), "\"worker\\nname\"");
    assert!(outer["threadId"].as_str().contains("ThreadId"));
    assert!(outer["filename"].as_str().contains("formatting.rs"));
    assert!(outer.contains_key("line_number"));
    assert_eq!(text.lines().count(), 1);
}

#[test]
fn empty_span_and_disabled_context_are_valid() {
    let (text, _) = capture(
        format().with_current_span(false).with_span_list(false),
        || {
            let _entered = tracing::info_span!("empty").entered();
            tracing::info!("event");
        },
    );
    let outer = raw(&text);
    assert!(!outer.contains_key("span"));
    assert!(!outer.contains_key("spans"));
}

#[test]
fn finite_float_boundaries_round_trip_without_dynamic_value_coercion() {
    let values = [
        0.0f64,
        -0.0,
        1.0,
        -1.0,
        f64::MAX,
        f64::MIN,
        f64::MIN_POSITIVE,
        f64::from_bits(1),
        9007199254740992.0,
        1e-100,
        1e100,
        0.1,
    ];
    let (text, status) = capture(format(), || {
        for value in values {
            tracing::info!(value);
        }
    });
    for (line, expected) in text.lines().zip(values) {
        let event = raw(line);
        let fields = raw(event["fields"].as_str());
        let actual: f64 = facet_json::from_str(fields["value"].as_str()).unwrap();
        assert_eq!(
            actual.to_bits(),
            expected.to_bits(),
            "{}",
            fields["value"].as_str()
        );
    }
    assert_eq!(text.lines().count(), values.len());
    assert_eq!(status.formatting_errors, 0);
}

#[test]
fn control_characters_and_unicode_round_trip() {
    let value: String = (0..=127)
        .filter_map(char::from_u32)
        .chain("日本語😀\u{2028}\u{2029}".chars())
        .collect();
    let (text, _) = capture(format(), || tracing::info!(value = value.as_str()));
    assert_eq!(text.bytes().filter(|byte| *byte == b'\n').count(), 1);
    let event = raw(&text);
    let fields = raw(event["fields"].as_str());
    let decoded: String = facet_json::from_str(fields["value"].as_str()).unwrap();
    assert_eq!(decoded, value);
}

#[test]
fn cached_spans_preserve_all_float_bits_across_updates() {
    let values = [
        -0.0,
        0.0,
        f64::MAX,
        f64::MIN_POSITIVE,
        f64::from_bits(1),
        1e100,
    ];
    let (text, status) = capture(format(), || {
        let span = tracing::info_span!("floats", value = 0.0, exact = u64::MAX);
        let _entered = span.enter();
        for value in values {
            span.record("value", value);
            tracing::info!("first");
            tracing::info!("cached");
        }
    });
    for (index, line) in text.lines().enumerate() {
        let event = raw(line);
        let span = raw(event["span"].as_str());
        let value: f64 = facet_json::from_str(span["value"].as_str()).unwrap();
        assert_eq!(value.to_bits(), values[index / 2].to_bits());
        assert_eq!(span["exact"].as_str(), u64::MAX.to_string());
        assert_eq!(
            event["spans"].as_str(),
            format!("[{}]", event["span"].as_str())
        );
    }
    assert_eq!(text.lines().count(), values.len() * 2);
    assert_eq!(status, Default::default());
}

#[test]
fn span_name_collision_and_escaped_names_survive_cache_hits() {
    let (text, status) = capture(format(), || {
        let _entered = tracing::info_span!("meta\"\nname", name = "producer").entered();
        tracing::info!("first");
        tracing::info!("cached");
    });
    for line in text.lines() {
        assert_eq!(line.matches("\"name\":").count(), 4);
        assert!(line.contains("meta\\\"\\nname"));
        let _: Value = facet_json::from_str(line).unwrap();
    }
    assert_eq!(status, Default::default());
}
