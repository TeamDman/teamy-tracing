// Envelope behavior follows tracing-subscriber 0.3.23. See THIRD_PARTY_NOTICES.md.
use crate::Diagnostics;
use crate::FacetJsonFields;
use crate::json_options::JsonOptions;
use crate::json_visitor::JsonVisitor;
use crate::serialize_scalar::serialize_scalar;
use crate::validated_span::ValidatedSpan;
use facet_format::FormatSerializer;
use facet_format::ScalarValue;
use facet_json::JsonSerializer;
use std::borrow::Cow;
use std::fmt;
use tracing::Event;
use tracing::Subscriber;
use tracing_subscriber::fmt::FmtContext;
use tracing_subscriber::fmt::FormatEvent;
use tracing_subscriber::fmt::FormattedFields;
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::time::FormatTime;
use tracing_subscriber::fmt::time::SystemTime;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::registry::SpanRef;

/// JSON envelope formatter. Configuration belongs here, not on upstream Format.
#[derive(Debug, Clone)]
pub struct FacetJson<T = SystemTime> {
    timer: T,
    options: JsonOptions,
    diagnostics: Diagnostics,
}

impl Default for FacetJson {
    fn default() -> Self {
        Self {
            timer: SystemTime,
            options: JsonOptions::default(),
            diagnostics: Diagnostics::default(),
        }
    }
}

impl<T> FacetJson<T> {
    pub fn with_timer<U>(self, timer: U) -> FacetJson<U> {
        FacetJson {
            timer,
            options: self.options,
            diagnostics: self.diagnostics,
        }
    }
    pub fn with_diagnostics(mut self, diagnostics: Diagnostics) -> Self {
        self.diagnostics = diagnostics;
        self
    }
    pub fn without_time(mut self) -> Self {
        self.options.timestamp = false;
        self
    }
    pub fn with_level(mut self, enabled: bool) -> Self {
        self.options.level = enabled;
        self
    }
    pub fn with_target(mut self, enabled: bool) -> Self {
        self.options.target = enabled;
        self
    }
    pub fn with_file(mut self, enabled: bool) -> Self {
        self.options.filename = enabled;
        self
    }
    pub fn with_line_number(mut self, enabled: bool) -> Self {
        self.options.line_number = enabled;
        self
    }
    pub fn with_current_span(mut self, enabled: bool) -> Self {
        self.options.current_span = enabled;
        self
    }
    pub fn with_span_list(mut self, enabled: bool) -> Self {
        self.options.span_list = enabled;
        self
    }
    pub fn with_thread_names(mut self, enabled: bool) -> Self {
        self.options.thread_name = enabled;
        self
    }
    pub fn with_thread_ids(mut self, enabled: bool) -> Self {
        self.options.thread_id = enabled;
        self
    }
    pub fn flatten_event(mut self, enabled: bool) -> Self {
        self.options.flatten = enabled;
        self
    }
}

fn string_field(serializer: &mut JsonSerializer, name: &str, value: &str) -> fmt::Result {
    serializer.field_key(name).map_err(|_| fmt::Error)?;
    serializer
        .scalar(ScalarValue::Str(Cow::Borrowed(value)))
        .map_err(|_| fmt::Error)
}

fn span_fields<S>(serializer: &mut JsonSerializer, span: &SpanRef<'_, S>) -> fmt::Result
where
    S: for<'a> LookupSpan<'a>,
{
    {
        let extensions = span.extensions();
        let source = &extensions
            .get::<FormattedFields<FacetJsonFields>>()
            .ok_or(fmt::Error)?
            .fields;
        if let Some(cached) = extensions.get::<ValidatedSpan>()
            && cached.source == *source
        {
            return serializer
                .raw_scalar(&cached.encoded)
                .map_err(|_| fmt::Error);
        }
    }
    // Recheck under the write lock: another thread may have recorded or rendered
    // this span after the read lock was released. Never emit stale validation.
    let mut extensions = span.extensions_mut();
    let source = extensions
        .get_mut::<FormattedFields<FacetJsonFields>>()
        .ok_or(fmt::Error)?
        .fields
        .clone();
    if extensions
        .get_mut::<ValidatedSpan>()
        .is_none_or(|cached| cached.source != source)
    {
        let validated = ValidatedSpan::new(source, span.metadata().name())?;
        extensions.replace(validated);
    }
    serializer
        .raw_scalar(
            &extensions
                .get_mut::<ValidatedSpan>()
                .ok_or(fmt::Error)?
                .encoded,
        )
        .map_err(|_| fmt::Error)
}

impl<T: FormatTime> FacetJson<T> {
    fn encode<S>(
        &self,
        ctx: &FmtContext<'_, S, FacetJsonFields>,
        event: &Event<'_>,
    ) -> Result<Vec<u8>, fmt::Error>
    where
        S: Subscriber + for<'a> LookupSpan<'a>,
    {
        let mut serializer = JsonSerializer::new();
        let metadata = event.metadata();
        serializer.begin_struct().map_err(|_| fmt::Error)?;
        if self.options.timestamp {
            let mut timestamp = String::new();
            self.timer.format_time(&mut Writer::new(&mut timestamp))?;
            string_field(&mut serializer, "timestamp", &timestamp)?;
        }
        if self.options.level {
            string_field(&mut serializer, "level", metadata.level().as_str())?;
        }
        if !self.options.flatten {
            serializer.field_key("fields").map_err(|_| fmt::Error)?;
            serializer.begin_struct().map_err(|_| fmt::Error)?;
        }
        let mut visitor = JsonVisitor::new(
            |name: &str, value| {
                serializer.field_key(name).map_err(|_| fmt::Error)?;
                serialize_scalar(&mut serializer, value)
            },
            false,
        );
        event.record(&mut visitor);
        visitor.finish()?;
        if !self.options.flatten {
            serializer.end_struct().map_err(|_| fmt::Error)?;
        }
        if self.options.target {
            string_field(&mut serializer, "target", metadata.target())?;
        }
        if self.options.filename
            && let Some(file) = metadata.file()
        {
            string_field(&mut serializer, "filename", file)?;
        }
        if self.options.line_number
            && let Some(line) = metadata.line()
        {
            serializer
                .field_key("line_number")
                .map_err(|_| fmt::Error)?;
            serializer
                .scalar(ScalarValue::U64(u64::from(line)))
                .map_err(|_| fmt::Error)?;
        }
        // Deliberate correction: explicit parent/root governs both span and spans.
        if (self.options.current_span || self.options.span_list)
            && let Some(scope) = ctx.event_scope()
        {
            let spans: Vec<_> = scope.from_root().collect();
            if self.options.current_span
                && let Some(span) = spans.last()
            {
                serializer.field_key("span").map_err(|_| fmt::Error)?;
                span_fields(&mut serializer, span)?;
            }
            if self.options.span_list {
                serializer.field_key("spans").map_err(|_| fmt::Error)?;
                serializer.begin_seq().map_err(|_| fmt::Error)?;
                for span in &spans {
                    span_fields(&mut serializer, span)?;
                }
                serializer.end_seq().map_err(|_| fmt::Error)?;
            }
        }
        if self.options.thread_name || self.options.thread_id {
            let thread = std::thread::current();
            if self.options.thread_name {
                if let Some(name) = thread.name() {
                    string_field(&mut serializer, "threadName", name)?;
                } else if !self.options.thread_id {
                    string_field(&mut serializer, "threadName", &format!("{:?}", thread.id()))?;
                }
            }
            if self.options.thread_id {
                string_field(&mut serializer, "threadId", &format!("{:?}", thread.id()))?;
            }
        }
        serializer.end_struct().map_err(|_| fmt::Error)?;
        Ok(serializer.finish())
    }
}

impl<S, T> FormatEvent<S, FacetJsonFields> for FacetJson<T>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    T: FormatTime,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, FacetJsonFields>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        let result = self.encode(ctx, event).and_then(|bytes| {
            let text = std::str::from_utf8(&bytes).map_err(|_| fmt::Error)?;
            writer.write_str(text)?;
            writer.write_char('\n')
        });
        if result.is_err() {
            self.diagnostics.format_error();
        }
        result
    }
}
