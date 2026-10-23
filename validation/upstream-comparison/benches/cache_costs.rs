//! Operation timings for the representative two-field span cache. This is a
//! focused cost profile, not stack sampling or allocation attribution.
mod benchmark_record;
use benchmark_record::BenchmarkRecord;
use facet_json::RawJson;
use std::collections::BTreeMap;
use std::hint::black_box;
use std::io::Write;
use std::time::Instant;

fn measure(mut operation: impl FnMut(), scenario: &str) -> BenchmarkRecord<'_> {
    const ITERATIONS: u64 = 20_000;
    for _ in 0..1000 {
        operation();
    }
    let mut samples = [0.0; 7];
    for sample in &mut samples {
        let start = Instant::now();
        for _ in 0..ITERATIONS {
            operation();
        }
        *sample = start.elapsed().as_secs_f64() * 1e9 / ITERATIONS as f64;
    }
    samples.sort_by(f64::total_cmp);
    BenchmarkRecord {
        formatter: "facet_standard",
        scenario,
        iterations: ITERATIONS,
        samples: samples.len(),
        median_ns_per_iteration: samples[3],
        minimum_ns_per_iteration: samples[0],
        maximum_ns_per_iteration: samples[6],
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = r#"{"count":42,"immutable":"context"}"#;
    let values: BTreeMap<String, RawJson<'static>> = facet_json::from_str(source)?;
    let mut output = std::io::stdout().lock();
    let parse = measure(
        || {
            let decoded: BTreeMap<String, RawJson<'static>> =
                facet_json::from_str(black_box(source)).unwrap();
            black_box(decoded);
        },
        "parse_span_cache",
    );
    let encode = measure(
        || {
            black_box(facet_json::to_string(black_box(&values)).unwrap());
        },
        "reflect_encode_span_cache",
    );
    for record in [parse, encode] {
        writeln!(output, "{}", facet_json::to_string(&record)?)?;
    }
    output.flush()?;
    Ok(())
}
