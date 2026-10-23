/// Proposed adapter payload; the provisioned template has no typed startup schema.
#[derive(Debug, Clone, PartialEq, Eq, facet::Facet)]
pub struct StartupFields {
    pub message: String,
    pub argv: Vec<String>,
    /// The caller supplies the template's rich version/build identity string.
    pub version: String,
}
