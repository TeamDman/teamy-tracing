mod benchmark_record;
use benchmark_record::BenchmarkRecord;
use std::hint::black_box;
use std::io::Write;
use std::time::Instant;

fn workload(scenario: &str, iterations: u64) {
    match scenario {
        "events" | "disabled" => {
            for n in 0..iterations {
                tracing::info!(
                    n = black_box(n),
                    rate = 1.25,
                    success = true,
                    "work finished"
                );
            }
        }
        "deep_spans" => {
            let a = tracing::info_span!("one", job = 42).entered();
            let b = tracing::info_span!("two", path = "synthetic").entered();
            let c = tracing::info_span!("three", attempt = 2).entered();
            for n in 0..iterations {
                tracing::info!(n = black_box(n), "progress");
            }
            drop((c, b, a));
        }
        "span_churn" => {
            for n in 0..iterations {
                let _entered = tracing::info_span!("cold", count = black_box(n)).entered();
                tracing::info!("once");
            }
        }
        "updates_only" => {
            let span = tracing::info_span!("updated", count = 0, immutable = "context");
            for n in 0..iterations {
                span.record("count", black_box(n));
            }
        }
        "span_updates" => {
            let span = tracing::info_span!("updated", count = 0, immutable = "context");
            let _entered = span.enter();
            for n in 0..iterations {
                span.record("count", black_box(n));
                tracing::info!("updated");
            }
        }
        _ => unreachable!(),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    const ITERATIONS: u64 = 20_000;
    let mut output = std::io::stdout().lock();
    for scenario in [
        "disabled",
        "events",
        "deep_spans",
        "span_updates",
        "updates_only",
        "span_churn",
    ] {
        let dispatches = [
            teamy_tracing_upstream_comparison::subscriber(false, scenario != "disabled"),
            teamy_tracing_upstream_comparison::subscriber(true, scenario != "disabled"),
        ];
        let mut timings = [Vec::new(), Vec::new()];
        for dispatch in &dispatches {
            tracing::dispatcher::with_default(dispatch, || workload(scenario, 1000));
        }
        for sample in 0..7 {
            // Alternate order to reduce systematic warm-up/thermal bias.
            for index in [sample % 2, 1 - sample % 2] {
                tracing::dispatcher::with_default(&dispatches[index], || {
                    let start = Instant::now();
                    workload(scenario, ITERATIONS);
                    timings[index].push(start.elapsed().as_secs_f64() * 1e9 / ITERATIONS as f64);
                });
            }
        }
        for (index, values) in timings.iter_mut().enumerate() {
            values.sort_by(f64::total_cmp);
            let record = BenchmarkRecord {
                formatter: ["facet_standard", "upstream_serde"][index],
                scenario,
                iterations: ITERATIONS,
                samples: values.len(),
                median_ns_per_iteration: values[3],
                minimum_ns_per_iteration: values[0],
                maximum_ns_per_iteration: values[6],
            };
            writeln!(output, "{}", facet_json::to_string(&record)?)?;
        }
    }
    output.flush()?;
    Ok(())
}
