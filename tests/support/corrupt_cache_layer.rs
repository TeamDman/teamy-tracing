use teamy_tracing::FacetJsonFields;
use tracing_subscriber::Layer;
use tracing_subscriber::fmt::FormattedFields;
use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;
#[derive(Debug)]
pub struct CorruptCacheLayer;
impl<S> Layer<S> for CorruptCacheLayer
where
    S: tracing::Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(
        &self,
        _: &tracing::span::Attributes<'_>,
        id: &tracing::span::Id,
        ctx: Context<'_, S>,
    ) {
        let span = ctx.span(id).unwrap();
        let mut ext = span.extensions_mut();
        ext.get_mut::<FormattedFields<FacetJsonFields>>()
            .unwrap()
            .fields = "{\"injected\": [\n1]}".into();
    }
}
