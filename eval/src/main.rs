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
use lib::AlgoOut;
use lib::CHUNK_SIZE;
use lib::RankingsCsvRow;
use lib::def::Element;
use lib::def::Ranking;
use lib::def::partial_from_string;
use lib::progress_bar;
use lib::read_glob_csv;
use lib::run_solver_on;
use rayon::iter::IntoParallelIterator;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use verifier::verify::parse_algo_sol;

#[derive(Parser, Debug)]
#[command(version)]
pub struct Cli {
    pub solver: PathBuf,
    pub data: String,
    pub output: PathBuf,
}

#[derive(Debug, Clone, serde_derive::Deserialize, serde_derive::Serialize, PartialEq)]
pub struct OutCsvRow {
    // pub ta: f64, // todo: convert tb to ta by multiplication. // <- stupid, ta=tb
    pub t_b: f64,
    pub t_max: f64,
    pub t_min: f64,
    pub length: usize,
    // pub frac_ties_x: f64,
    // pub frac_ties_y: f64,
    pub frac_ties: f64,
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
        .collect::<Result<Vec<RankingsCsvRow>>>()?;
    cases.dedup_by(|r1, r2| {
        let mut map = BTreeMap::new();
        let p1 = partial_from_string(&r1.a, &mut map).unwrap();
        let p2 = partial_from_string(&r1.b, &mut map).unwrap();
        let mut map = BTreeMap::new();
        let p3 = partial_from_string(&r2.a, &mut map).unwrap();
        let p4 = partial_from_string(&r2.b, &mut map).unwrap();
        p1.rank_eq(&p3) && p2.rank_eq(&p4)
    });
    cases.sort_by_key(|row| row.a.len());

    let num_tests = cases.len();

    let pb = progress_bar(num_tests as u64)?;

    cases.chunks(CHUNK_SIZE).try_for_each(|group| {
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

fn map_to_out(xc: (AlgoOut, &RankingsCsvRow)) -> Result<OutCsvRow> {
    let mut inp_map: BTreeMap<String, Element> = BTreeMap::new();
    let rank_a = partial_from_string(&xc.1.a, &mut inp_map)?;
    let rank_b = partial_from_string(&xc.1.b, &mut inp_map)?;

    let t = rank_a.tau(&rank_b)?;
    let tie_count = rank_a
        .iter()
        .chain(rank_b.iter())
        .map(|x| x.len())
        .filter(|x| x > &1)
        .count();
    let items_in_ties: usize = rank_a
        .iter()
        .chain(rank_b.iter())
        .map(|x| x.len())
        .filter(|x| x > &1)
        .sum();
    
    let length = rank_a.set_size();

    Ok(OutCsvRow {
        t_b: t,
        t_max: xc.0.tmax.unwrap(),
        t_min: xc.0.tmin.unwrap(),
        length,
        frac_ties: items_in_ties as f64 / (2.0 * length as f64),
        tie_count,
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
        permutation_count: rank_a
            .linear_ext_count()
            .saturating_mul(rank_b.linear_ext_count()),
    })
}

pub fn parse_row(row: &StringRecord) -> Result<RankingsCsvRow> {
    let p = row.deserialize(None)?;
    Ok(p)
}
