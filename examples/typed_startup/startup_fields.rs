#[derive(Debug, facet::Facet)]
pub struct StartupFields {
    pub argv: Vec<String>,
}
