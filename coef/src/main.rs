use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Instant;

use anyhow::Result;
use anyhow::anyhow;
use clap::Parser;
use clap_derive::Parser;
use csv::StringRecord;
use csv::Writer;
use indicatif::ParallelProgressIterator;
use rayon::iter::IntoParallelIterator;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use solver::def::Element;
use solver::def::Ranking;
use solver::def::partial_from_string;
use verifier::parsing::progress_bar;
use verifier::parsing::read_glob_csv;
use verifier::AlgoOut;
use verifier::runner::Case;
use verifier::runner::run_solver_on;
use verifier::verify::parse_algo_sol;

#[derive(Parser, Debug)]
#[command(version)]
pub struct Cli {
    pub solver: PathBuf,
    pub data: String,
    pub output: PathBuf,
}

#[derive(Debug, Clone, serde_derive::Deserialize, PartialEq, Eq)]
pub struct InCsvRow {
    pub a: String,
    pub b: String,
}

#[derive(Debug, Clone, serde_derive::Deserialize, serde_derive::Serialize, PartialEq)]
pub struct OutCsvRow {
    // pub ta: f64, // todo: convert tb to ta by multiplication.
    pub t_b: f64,
    pub t_max: f64,
    pub t_min: f64,
    pub sum_of_tie_lengths: usize,
    pub tie_count: usize,
    pub longest_tie: usize,
    pub permutation_count: u128,
}

fn main() -> Result<()> {
    let args = Cli::parse();

    let start = Instant::now();

    let rows = read_glob_csv(&args.data, vec!["a", "b"])?;

    let mut writer = Writer::from_path(&args.output)?;

    let mut cases = rows
        .par_iter()
        .map(parse_row)
        .collect::<Result<Vec<InCsvRow>>>()?;
    cases.dedup();
    cases.sort_by_key(|row| row.a.len());

    let num_tests = cases.len();

    let pb = progress_bar(num_tests as u64)?;

    cases.chunks(1000).try_for_each(|group| {
        let outputs = group
            .into_par_iter()
            .map(|c| run_solver_on(&args.solver, &c).map(|x| (x, c)))
            .progress_with(pb.clone())
            .collect::<Result<Vec<_>>>()
            .map_err(|e| anyhow!("runner err: {e:?}"))?
            .into_par_iter()
            .map(|(x, c)| parse_algo_sol(x).map(|x| x.map(|xx| (xx, c))))
            .collect::<Result<Vec<_>>>()
            .map_err(|e| anyhow!("parser err: {e:?}"))?
            .into_iter()
            .flatten()
            .map(map_to_out)
            .collect::<Result<Vec<_>>>()?;

        outputs
            .into_iter()
            .try_for_each(|o| writer.serialize(o))
            .map_err(|e| anyhow!("writer err: {e:?}"))
    })?;

    println!("{num_tests} done in {}s", start.elapsed().as_secs_f32());
    Ok(())
}

fn map_to_out(xc: (AlgoOut, &InCsvRow)) -> Result<OutCsvRow> {
    let mut inp_map: BTreeMap<String, Element> = BTreeMap::new();
    let rank_a = partial_from_string(&xc.1.a, &mut inp_map)?;
    let rank_b = partial_from_string(&xc.1.b, &mut inp_map)?;

    let t = rank_a.tau(&rank_b)?;

    Ok(OutCsvRow {
        t_b: t,
        t_max: xc.0.tmax.unwrap(),
        t_min: xc.0.tmin.unwrap(),
        tie_count: rank_a
            .iter()
            .chain(rank_b.iter())
            .map(|x| x.len())
            .filter(|x| x > &1)
            .count(),
        longest_tie: rank_a
            .iter()
            .chain(rank_b.iter())
            .map(|x| x.len())
            .max()
            .unwrap_or_default(),
        sum_of_tie_lengths: rank_a
            .iter()
            .chain(rank_b.iter())
            .map(|x| x.len())
            .filter(|x| x > &1)
            .sum(),
        permutation_count: rank_a.linear_ext_count().saturating_mul(rank_b.linear_ext_count()),
    })
}

impl Case for &&InCsvRow {
    fn algo_args(&self) -> Vec<String> {
        vec![self.a.to_string(), self.b.to_string()]
    }
}

pub fn parse_row(row: &StringRecord) -> Result<InCsvRow> {
    let p: InCsvRow = row.deserialize(None)?;
    Ok(p)
}

