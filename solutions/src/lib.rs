use csv::StringRecord;
use anyhow::Result;

pub mod ref_solver;

#[derive(Debug, Clone, serde_derive::Deserialize, PartialEq, Eq)]
pub struct InCsvRow {
    pub a: String,
    pub b: String,
}

#[derive(Debug, Clone, serde_derive::Serialize)]
pub struct OutputRow {
    pub a: String,
    pub b: String,
    pub tmin: f64,
    pub tmax: f64,
    pub pmin: String,
    pub pmax: String,
}

pub fn parse_row(row: &StringRecord) -> Result<InCsvRow> {
    let p: InCsvRow = row.deserialize(None)?;
    Ok(p)
}
