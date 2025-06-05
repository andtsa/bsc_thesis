use lib::AlgoOut;
use lib::Case;

pub mod parsing;
pub mod verify;

#[derive(Debug, Clone, serde_derive::Deserialize)]
pub struct CsvRow {
    pub a: String,
    pub b: String,
    pub tmin: f64,
    pub tmax: f64,
    pub min_sols: String,
    pub max_sols: String,
}

#[derive(Debug, Clone)]
pub struct TestCase {
    pub a: String,
    pub b: String,
    pub tmin: f64,
    pub tmax: f64,
    pub min_sol_pairs: Vec<(String, String)>,
    pub max_sol_pairs: Vec<(String, String)>,
}

/// failure types. value is (actual, expected)
#[derive(Debug)]
pub enum FailType {
    Tmin(f64, f64),
    Tmax(f64, f64),
    MinP,
    MaxP,
}

type FailInfo = (TestCase, AlgoOut);

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum TestResult {
    Pass,
    Skipped,
    Empty(TestCase),
    Fail(FailInfo, FailType),
}

impl Case for TestCase {
    fn algo_args(&self) -> Vec<String> {
        vec![self.a.to_string(), self.b.to_string()]
    }
}
