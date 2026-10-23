use super::startup_fields::StartupFields;
#[derive(Debug, facet::Facet)]
pub struct StartupRecord {
    pub level: String,
    pub fields: StartupFields,
    pub target: String,
}
