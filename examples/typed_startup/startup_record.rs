use super::startup_fields::StartupFields;
#[derive(Debug, facet::Facet)]
pub struct StartupRecord {
    pub timestamp: String,
    pub level: String,
    pub fields: StartupFields,
    pub target: String,
    pub filename: Option<String>,
    pub line_number: Option<u32>,
}
