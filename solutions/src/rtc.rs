pub mod ref_solver;

use std::path::PathBuf;
use std::time::Instant;

use anyhow::Result;
use anyhow::anyhow;
use anyhow::bail;
use clap::Parser;
use clap_derive::Parser;
use csv::Reader;
use csv::StringRecord;
use csv::Writer;
use glob::glob;
use indicatif::ParallelProgressIterator;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use ref_solver::run_solver;
#[derive(Parser, Debug)]
#[command(version)]
pub struct Cli {
    pub input: String,
    pub output: PathBuf,
}

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

fn main() -> Result<()> {
    let args = Cli::parse();

    let start = Instant::now();

    let mut rows: Vec<StringRecord> = vec![];

    for srcfile in glob(&args.input)? {
        let mut csv_read = Reader::from_path(&srcfile?)?;

        if csv_read.headers()? != vec!["a", "b"] {
            bail!("Incompatible CSV data: expected header of `a,b`");
        }

        rows.append(&mut csv_read.records().collect::<Result<Vec<_>, csv::Error>>()?);
    }

    let num_cases = rows.len();

    let mut cases = rows
        .par_iter()
        .map(parse_row)
        .collect::<Result<Vec<InCsvRow>>>()?;
    cases.dedup();
    cases.sort_by_key(|row| row.a.len());

    let pb = ProgressBar::new(num_cases as u64).with_style(ProgressStyle::default_bar().template(
        "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
    )?);

    let mut count = 0;

    let mut writer = Writer::from_path(&args.output)?;

    cases.chunks(1000).try_for_each(|group| {
        let outputs = group
            .par_iter()
            .map(run_solver)
            .progress_with(pb.clone())
            .collect::<Result<Vec<_>>>()
            .map_err(|e| anyhow!("runner err: {e:?}"))?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        outputs
            .into_iter()
            .try_for_each(|o| {
                count += 1;
                writer.serialize(o)
            })
            .map_err(|e| anyhow!("writer err: {e:?}"))
    })?;

    println!(
        "evaluated {count} test cases in {}s",
        start.elapsed().as_secs_f32()
    );

    Ok(())
}

pub fn parse_row(row: &StringRecord) -> Result<InCsvRow> {
    let p: InCsvRow = row.deserialize(None)?;
    Ok(p)
}
