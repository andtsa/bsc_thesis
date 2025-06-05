use std::path::PathBuf;
use std::time::Instant;

use anyhow::Result;
use anyhow::anyhow;
use clap::Parser;
use clap_derive::Parser;
use csv::Writer;
use indicatif::ParallelProgressIterator;
use lib::RankingsCsvRow;
use lib::parse_row;
use lib::progress_bar;
use lib::read_glob_csv;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use solutions::ref_solver::run_solver;

#[derive(Parser, Debug)]
#[command(version)]
pub struct Cli {
    pub input: String,
    pub output: PathBuf,
}

fn main() -> Result<()> {
    let args = Cli::parse();

    let start = Instant::now();

    let rows = read_glob_csv(&args.input, vec!["a", "b"])?;

    let num_cases = rows.len();

    let mut cases = rows
        .par_iter()
        .map(parse_row)
        .collect::<Result<Vec<RankingsCsvRow>>>()?;
    cases.dedup();
    cases.sort_by_key(|row| row.a.len());

    let pb = progress_bar(num_cases as u64)?;
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
