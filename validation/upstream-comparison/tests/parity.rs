use facet_value::Value;
use teamy_tracing::Diagnostics;
use teamy_tracing::FacetJson;
use teamy_tracing::SharedWriter;
use tracing_subscriber::prelude::*;

fn records(upstream: bool, flatten: bool, emit: fn()) -> Vec<Value> {
    let diagnostics = Diagnostics::default();
    let writer = SharedWriter::new(Vec::<u8>::new(), diagnostics.clone());
    if upstream {
        let subscriber = tracing_subscriber::registry().with(
            tracing_subscriber::fmt::layer()
                .json()
                .without_time()
                .with_target(false)
                .flatten_event(flatten)
                .with_writer(writer.clone()),
        );
        tracing::subscriber::with_default(subscriber, emit);
    } else {
        let subscriber = tracing_subscriber::registry().with(
            teamy_tracing::layer(writer.clone(), diagnostics.clone()).event_format(
                FacetJson::default()
                    .without_time()
                    .with_target(false)
                    .flatten_event(flatten),
            ),
        );
        tracing::subscriber::with_default(subscriber, emit);
    }
    assert_eq!(diagnostics.snapshot(), Default::default());
    let bytes = writer.inspect(Clone::clone).unwrap();
    std::str::from_utf8(&bytes)
        .unwrap()
        .lines()
        .map(|line| facet_json::from_str(line).unwrap())
        .collect()
}

fn fixture() {
    let parent = tracing::info_span!(
        "parent",
        integer = i64::MIN,
        counter = tracing::field::Empty,
        bytes = &[3u8, 255][..],
        wide = u128::MAX
    );
    let _entered = parent.enter();
    let leaf = tracing::debug_span!("leaf", float = 1.25);
    let _leaf = leaf.enter();
    let error = std::io::Error::other("source\nerror");
    tracing::info!(integer = u64::MAX, flag = true, float = 1.25, nan = f64::NAN,
        wide = u128::MAX, bytes = &[3u8, 255][..], error = &error as &dyn std::error::Error,
        debug = ?vec![1, 2], text = "\"\\\r\n\t\u{1b}日本語", "message");
    parent.record("counter", 1u64);
    parent.record("counter", 2u64);
    tracing::warn!("updated");
}

#[test]
fn nested_schema_and_values_match_upstream() {
    assert_eq!(
        records(false, false, fixture),
        records(true, false, fixture)
    );
}
#[test]
fn flattened_schema_and_values_match_upstream() {
    assert_eq!(records(false, true, fixture), records(true, true, fixture));
}
