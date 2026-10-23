use teamy_tracing::FacetJsonFields;
use tracing_subscriber::Layer;
use tracing_subscriber::fmt::FormattedFields;
use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;

/// Mutate public upstream extensions after the formatter has warmed its cache.
#[derive(Debug)]
pub struct CacheMutationLayer(pub &'static str);
impl<S> Layer<S> for CacheMutationLayer
where
    S: tracing::Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_record(
        &self,
        id: &tracing::span::Id,
        _: &tracing::span::Record<'_>,
        ctx: Context<'_, S>,
    ) {
        let span = ctx.span(id).unwrap();
        span.extensions_mut()
            .get_mut::<FormattedFields<FacetJsonFields>>()
            .unwrap()
            .fields = self.0.into();
    }
}
