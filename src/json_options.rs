// These independent flags model the upstream JSON envelope options.
#[derive(Debug, Clone)]
pub(crate) struct JsonOptions {
    pub timestamp: bool,
    pub level: bool,
    pub target: bool,
    pub filename: bool,
    pub line_number: bool,
    pub current_span: bool,
    pub span_list: bool,
    pub thread_name: bool,
    pub thread_id: bool,
    pub flatten: bool,
}

impl Default for JsonOptions {
    fn default() -> Self {
        Self {
            timestamp: true,
            level: true,
            target: true,
            filename: false,
            line_number: false,
            current_span: true,
            span_list: true,
            thread_name: false,
            thread_id: false,
            flatten: false,
        }
    }
}
