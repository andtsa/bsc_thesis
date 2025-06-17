use anyhow::Result;
use anyhow::bail;
use csv::ReaderBuilder;
use csv::StringRecord;
use glob::glob;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use itertools::Itertools;

pub mod def;
pub mod tau_w;
pub mod weights;

pub const PRECISION: f64 = 1e-6f64;
pub const CHUNK_SIZE: usize = 256;

#[derive(Debug, Clone, serde_derive::Deserialize, PartialEq, Eq)]
pub struct RankingsCsvRow {
    pub a: String,
    pub b: String,
}

#[derive(Debug, Clone, serde_derive::Serialize)]
pub struct AlgoOutputRow {
    pub a: String,
    pub b: String,
    pub tmin: f64,
    pub tmax: f64,
    pub pmin: String,
    pub pmax: String,
}

#[derive(Debug, Default)]
pub struct AlgoOut {
    pub tmin: Option<f64>,
    pub tmax: Option<f64>,
    pub minp: Vec<(String, String)>,
    pub maxp: Vec<(String, String)>,
}

pub fn parse_row(row: &StringRecord) -> Result<RankingsCsvRow> {
    let p: RankingsCsvRow = row.deserialize(None)?;
    Ok(p)
}

pub fn read_glob_csv(g: &str, header: Vec<&str>) -> Result<Vec<StringRecord>> {
    let mut rows = vec![];

    for src in glob(g)? {
        let path = src?;
        let mut csv_read = ReaderBuilder::new().has_headers(true).from_path(&path)?;

        // check header
        if !header.is_empty() && csv_read.headers()? != header {
            bail!(
                "Incompatible CSV data ({}): expected header {header:?}, got {:?}",
                path.display(),
                csv_read.headers()?.iter().collect_vec()
            );
        }

        // iterate over data
        rows.append(&mut csv_read.records().collect::<Result<Vec<_>, csv::Error>>()?);
    }

    rows.dedup();

    Ok(rows)
}

pub fn progress_bar(n: u64) -> Result<ProgressBar> {
    Ok(
        ProgressBar::new(n).with_style(ProgressStyle::default_bar().template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
        )?),
    )
}

use std::path::PathBuf;
use std::process::Command;

pub trait Case {
    fn algo_args(&self) -> Vec<String>;
}

pub fn run_solver_on<C: Case>(algo: &PathBuf, inp: C) -> Result<String> {
    let mut cmd = Command::new(algo);

    for arg in inp.algo_args() {
        cmd.arg(arg);
    }

    let out = cmd.output()?;

    if !out.status.success() {
        bail!(
            "process failed: {}. stderr: {}",
            out.status,
            String::from_utf8_lossy(&out.stderr)
        )
    }

    Ok(String::from_utf8(out.stdout)?)
}

impl Case for &&RankingsCsvRow {
    fn algo_args(&self) -> Vec<String> {
        vec![self.a.to_string(), self.b.to_string()]
    }
}

impl PartialEq for AlgoOut {
    fn eq(&self, other: &Self) -> bool {
        if self
            .tmin
            .is_some_and(|vl| other.tmin.is_some_and(|vr| (vl - vr).abs() > PRECISION))
        {
            return false;
        }
        if self
            .tmax
            .is_some_and(|vl| other.tmax.is_some_and(|vr| (vl - vr).abs() > PRECISION))
        {
            return false;
        }

        // we can't really compare the permutations returned by each algorithm:
        // there are possibly many many optimal solutions,
        // and the algorithms don't need to return the same ones.
        // we trust that the tau values weren't pulled out of thin air
        true
    }
}

pub fn display_cases(cases: &[(String, String)]) -> String {
    cases
        .iter()
        .map(|(l, r)| format!("{l}/{r}"))
        .collect::<Vec<String>>()
        .join("|")
}
