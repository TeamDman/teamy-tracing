//! Isolated comparison oracle. Its Serde dependencies never enter the library graph.
use teamy_tracing::Diagnostics;
use teamy_tracing::FacetJson;
use tracing_subscriber::prelude::*;

pub fn subscriber(upstream: bool, enabled: bool) -> tracing::Dispatch {
    let filter = if enabled { "trace" } else { "off" };
    if upstream {
        tracing::Dispatch::new(
            tracing_subscriber::registry().with(
                tracing_subscriber::fmt::layer()
                    .json()
                    .without_time()
                    .with_target(false)
                    .with_writer(std::io::sink)
                    .with_filter(tracing_subscriber::EnvFilter::new(filter)),
            ),
        )
    } else {
        tracing::Dispatch::new(
            tracing_subscriber::registry().with(
                teamy_tracing::layer(std::io::sink, Diagnostics::default())
                    .event_format(FacetJson::default().without_time().with_target(false))
                    .with_filter(tracing_subscriber::EnvFilter::new(filter)),
            ),
        )
    }
}
