use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Duration;
use std::time::Instant;

use anyhow::Result;
use anyhow::anyhow;
use clap::Parser;
use clap_derive::Parser;
use csv::StringRecord;
use csv::Writer;
use eval::OutCsvRow;
use indicatif::ParallelProgressIterator;
use lib::AlgoOut;
use lib::CHUNK_SIZE;
use lib::PRECISION;
use lib::RankingsCsvRow;
use lib::def::Element;
use lib::def::Ranking;
use lib::def::partial_from_string;
use lib::def::strict_from_partial;
use lib::progress_bar;
use lib::read_glob_csv;
use lib::run_solver_on;
use lib::tau_w::TauVariants;
use lib::tau_w::tau_partial;
use lib::tau_w::tau_w;
use lib::weights::ap_weight;
use rayon::iter::IntoParallelIterator;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use verifier::verify::parse_algo_sol;

#[derive(Parser, Debug)]
#[command(version)]
pub struct Cli {
    pub solver: PathBuf,
    pub output: PathBuf,
    pub data: String,
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

    println!(
        "loading {} test cases ({}s)",
        cases.len(),
        start.elapsed().as_secs_f32()
    );
    cases.dedup_by(|r1, r2| {
        let mut map = BTreeMap::new();
        let p1 = partial_from_string(&r1.a, &mut map).unwrap();
        let p2 = partial_from_string(&r1.b, &mut map).unwrap();
        let mut map = BTreeMap::new();
        let p3 = partial_from_string(&r2.a, &mut map).unwrap();
        let p4 = partial_from_string(&r2.b, &mut map).unwrap();
        p1.rank_eq(&p3) && p2.rank_eq(&p4)
    });
    cases.sort_unstable_by_key(|row| row.a.len());

    let num_tests = cases.len();
    println!(
        "parsed {} into {num_tests} lines of input in {}s",
        rows.len(),
        start.elapsed().as_secs_f32()
    );

    let pb = progress_bar(num_tests as u64)?;

    cases.chunks(CHUNK_SIZE * 2).try_for_each(|group| {
        let outputs = group
            .into_par_iter()
            .map(|c| run_solver_on(&args.solver, &c).map(|x| (x, c)))
            .progress_with(pb.clone())
            .collect::<Result<Vec<_>>>()
            .map_err(|e| anyhow!("runner err: {e:?}"))?
            .into_par_iter()
            .map(|((x, dt), c)| parse_algo_sol(x).map(|x| x.map(|xx| (xx, c, dt))))
            .collect::<Result<Vec<_>>>()
            .map_err(|e| anyhow!("parser err: {e:?}"))?
            .into_iter()
            .flatten()
            .map(map_to_out)
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        outputs
            .into_iter()
            .try_for_each(|o| writer.serialize(o))
            .map_err(|e| anyhow!("writer err: {e:?}"))
    })?;

    println!("{num_tests} done in {}s", start.elapsed().as_secs_f32());
    Ok(())
}

fn map_to_out(xc: (AlgoOut, &RankingsCsvRow, Duration)) -> Result<Option<OutCsvRow>> {
    let mut inp_map: BTreeMap<String, Element> = BTreeMap::new();
    let rank_a = partial_from_string(&xc.1.a, &mut inp_map)?;
    let rank_b = partial_from_string(&xc.1.b, &mut inp_map)?;

    let mut sol_inp_map: BTreeMap<String, Element> = BTreeMap::new();
    let sol_str_a = partial_from_string(&xc.0.maxp[0].0, &mut sol_inp_map)?;
    let sol_str_b = partial_from_string(&xc.0.maxp[0].1, &mut sol_inp_map)?;
    let p_max_a = strict_from_partial(&sol_str_a)?;
    let p_max_b = strict_from_partial(&sol_str_b)?;

    let mut sol_inp_map: BTreeMap<String, Element> = BTreeMap::new();
    let sol_str_a = partial_from_string(&xc.0.minp[0].0, &mut sol_inp_map)?;
    let sol_str_b = partial_from_string(&xc.0.minp[0].1, &mut sol_inp_map)?;
    let p_min_a = strict_from_partial(&sol_str_a)?;
    let p_min_b = strict_from_partial(&sol_str_b)?;

    let t_a = tau_partial(&rank_a, &rank_b, ap_weight, TauVariants::A)?;
    let t_b = tau_partial(&rank_a, &rank_b, ap_weight, TauVariants::B)?;

    if t_b.is_nan() {
        return Ok(None);
    }

    let t_max = tau_w(&p_max_a, &p_max_b, ap_weight)?;
    let t_min = tau_w(&p_min_a, &p_min_b, ap_weight)?;

    assert!(
        t_min - PRECISION < t_b,
        "error in: \"{}\" \"{}\"\ntb:{t_b}\ntmin:{t_min}\npmin:{}/{}",
        &xc.1.a,
        &xc.1.b,
        &xc.0.minp[0].0,
        &xc.0.minp[0].1,
    );
    assert!(
        t_max + PRECISION > t_b,
        "error in: \"{}\" \"{}\"\ntb:{t_b}\ntmax:{t_max}\npmax:{}/{}",
        &xc.1.a,
        &xc.1.b,
        &xc.0.minp[0].0,
        &xc.0.minp[0].1,
    );

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

    Ok(Some(OutCsvRow {
        t_a,
        t_b,
        t_max,
        t_min,
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
        compute_time: xc.2.as_secs_f32(),
    }))
}

pub fn parse_row(row: &StringRecord) -> Result<RankingsCsvRow> {
    let p = row.deserialize(None)?;
    Ok(p)
}
