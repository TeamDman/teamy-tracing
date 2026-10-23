#[derive(Debug, facet::Facet)]
pub struct BenchmarkRecord<'a> {
    pub formatter: &'a str,
    pub scenario: &'a str,
    pub iterations: u64,
    pub samples: usize,
    pub median_ns_per_iteration: f64,
    pub minimum_ns_per_iteration: f64,
    pub maximum_ns_per_iteration: f64,
}
