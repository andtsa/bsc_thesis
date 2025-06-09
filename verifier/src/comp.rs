//! compare the outputs of two algorithms

use std::collections::BTreeMap;
use std::ops::Sub;
use std::path::PathBuf;
use std::time::Instant;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use anyhow::Result;
use anyhow::anyhow;
use anyhow::bail;
use clap::Parser;
use clap_derive::Parser;
use csv::Writer;
use indicatif::ParallelProgressIterator;
use lib::def::partial_from_string;
use lib::def::Ranking;
use lib::AlgoOut;
use lib::CHUNK_SIZE;
use lib::RankingsCsvRow;
use lib::display_cases;
use lib::parse_row;
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
    pub algo_one: PathBuf,
    pub algo_two: PathBuf,
    pub output: PathBuf,
    pub data: String,
}

enum CompResult {
    LeftMissing(AlgoOut, RankingsCsvRow),
    RightMissing(AlgoOut, RankingsCsvRow),
    BothMissing(RankingsCsvRow),
    TauNotEqual(AlgoOut, AlgoOut, RankingsCsvRow),
    SolNotEqual(AlgoOut, AlgoOut, RankingsCsvRow),
    Equal,
}

#[derive(Debug, Clone, serde_derive::Serialize)]
struct CompOutRow {
    pub err_type: String,
    pub a: String,
    pub b: String,
    pub dtmin: f64,
    pub ltmin: f64,
    pub rtmin: f64,
    pub dtmax: f64,
    pub ltmax: f64,
    pub rtmax: f64,
    pub lpmin: String,
    pub rpmin: String,
    pub lpmax: String,
    pub rpmax: String,
}

fn main() -> Result<()> {
    let args = Cli::parse();

    let start = Instant::now();

    if !args.algo_one.is_file() {
        bail!("algo one not found: {}", args.algo_one.display());
    }
    if !args.algo_two.is_file() {
        bail!("algo two not found: {}", args.algo_two.display());
    }

    let rows = read_glob_csv(&args.data, vec!["a", "b"])?;


    let mut inputs = rows
        .par_iter()
        .map(parse_row)
        .collect::<Result<Vec<RankingsCsvRow>>>()?;
    inputs.dedup_by(|r1, r2| {
        let mut map = BTreeMap::new();
        let p1 = partial_from_string(&r1.a, &mut map).unwrap();
        let p2 = partial_from_string(&r1.b, &mut map).unwrap();
        let mut map = BTreeMap::new();
        let p3 = partial_from_string(&r2.a, &mut map).unwrap();
        let p4 = partial_from_string(&r2.b, &mut map).unwrap();
        p1.rank_eq(&p3) && p2.rank_eq(&p4)
    });

    let num_tests = inputs.len();
    println!("parsed {} into {num_tests} lines of input in {}s", rows.len(), start.elapsed().as_secs_f32());

    let pb = progress_bar(num_tests as u64)?;

    let log_file = if args.output.is_dir() {
        // create a new logfile with the current timestamp as name
        let new_file_name = format!(
            "{}_comp_log.csv",
            (SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() / 30) & 0xfff
        );
        let p = args.output.join(new_file_name);
        std::fs::File::create_new(&p)
            .map_err(|e| anyhow!("error creating file {}: {e}", p.display()))?;
        println!("creating {}", p.display());
        Ok::<PathBuf, anyhow::Error>(p)
    } else {
        // whatever the user pointed to, i dont care
        Ok(args.output)
    }?;
    let mut writer = Writer::from_path(&log_file)
        .map_err(|e| anyhow!("couldn't open output file ({}): {e}", log_file.display()))?;

    let mut equals = 0;
    let mut l_missing = 0;
    let mut r_missing = 0;
    let mut both_missing = 0;
    let mut tau_not_eq = 0;
    let mut sol_not_eq = 0;

    println!("running {num_tests} tests...");

    inputs.chunks(CHUNK_SIZE).try_for_each(|group| {
        let comp_results = group
            .par_iter()
            .map(|i| {
                run_solver_on(&args.algo_one, &i)
                    .map(|a| run_solver_on(&args.algo_two, &i).map(|b| (a, b, i)))
            })
            .progress_with(pb.clone())
            .collect::<Result<Result<Vec<_>>>>()??
            .par_iter()
            .map(|(a, b, i)| {
                parse_algo_sol(a.to_string()).map(|aa| {
                    parse_algo_sol(b.to_string()).map(|bb| (aa, bb, (*i).clone()))
                })
            })
            .collect::<Result<Result<Vec<_>>>>()??
            .into_par_iter()
            .map(compare_results)
            .collect::<Result<Vec<_>>>()?;

        comp_results
            .into_iter()
            .flat_map(|cr| match cr {
                CompResult::Equal => {
                    equals += 1;
                    None
                }
                CompResult::LeftMissing(r, c) => {
                    l_missing += 1;
                    Some(CompOutRow {
                        err_type: "left missing".into(),
                        a: c.a,
                        b: c.b,
                        dtmin: 0.0,
                        ltmin: 0.0,
                        rtmin: r.tmin.unwrap_or(f64::NAN),
                        dtmax: 0.0,
                        ltmax: 0.0,
                        rtmax: r.tmax.unwrap_or(f64::NAN),
                        lpmin: "none".into(),
                        rpmin: display_cases(&r.minp),
                        lpmax: "none".into(),
                        rpmax: display_cases(&r.maxp),
                    })
                }
                CompResult::RightMissing(l, c) => {
                    r_missing += 1;
                    Some(CompOutRow {
                        err_type: "right missing".into(),
                        a: c.a,
                        b: c.b,
                        dtmin: 0.0,
                        rtmin: 0.0,
                        ltmin: l.tmin.unwrap_or(f64::NAN),
                        dtmax: 0.0,
                        rtmax: 0.0,
                        ltmax: l.tmax.unwrap_or(f64::NAN),
                        rpmin: "none".into(),
                        lpmin: display_cases(&l.minp),
                        rpmax: "none".into(),
                        lpmax: display_cases(&l.maxp),
                    })
                }
                CompResult::TauNotEqual(l, r, c) => {
                    tau_not_eq += 1;
                    Some(CompOutRow {
                        err_type: "bounds not equal".into(),
                        a: c.a,
                        b: c.b,
                        dtmin: l.tmin.unwrap_or(f64::NAN).sub(r.tmin.unwrap_or(f64::NAN)).abs(),
                        ltmin: l.tmin.unwrap_or(f64::NAN),
                        rtmin: r.tmin.unwrap_or(f64::NAN),
                        dtmax: l.tmax.unwrap_or(f64::NAN).sub(r.tmax.unwrap_or(f64::NAN)).abs(),
                        ltmax: l.tmax.unwrap_or(f64::NAN),
                        rtmax: r.tmax.unwrap_or(f64::NAN),
                        lpmin: display_cases(&l.minp),
                        rpmin: display_cases(&r.minp),
                        lpmax: display_cases(&l.maxp),
                        rpmax: display_cases(&r.maxp),
                    })
                }
                CompResult::SolNotEqual(l, r, c) => {
                    sol_not_eq += 1;
                    Some(CompOutRow {
                        err_type: "rankings not equal".into(),
                        a: c.a,
                        b: c.b,
                        dtmin: l.tmin.unwrap_or(f64::NAN).sub(r.tmin.unwrap_or(f64::NAN)).abs(),
                        ltmin: l.tmin.unwrap_or(f64::NAN),
                        rtmin: r.tmin.unwrap_or(f64::NAN),
                        dtmax: l.tmax.unwrap_or(f64::NAN).sub(r.tmax.unwrap_or(f64::NAN)).abs(),
                        ltmax: l.tmax.unwrap_or(f64::NAN),
                        rtmax: r.tmax.unwrap_or(f64::NAN),
                        lpmin: display_cases(&l.minp),
                        rpmin: display_cases(&r.minp),
                        lpmax: display_cases(&l.maxp),
                        rpmax: display_cases(&r.maxp),
                    })
                }
                CompResult::BothMissing(c) => {
                    both_missing += 1;
                    Some(CompOutRow {
                        err_type: "missing both".into(),
                        a: c.a,
                        b: c.b,
                        dtmax: 0.0,
                        dtmin: 0.0, 
                        ltmin: 0.0,
                        rtmin: 0.0,
                        ltmax: 0.0,
                        rtmax: 0.0,
                        lpmin: "none".into(),
                        rpmin: "none".into(),
                        lpmax: "none".into(),
                        rpmax: "none".into(),
                    })
                }
            })
            .try_for_each(|o| writer.serialize(o))
            .map_err(|e| anyhow!("writer err: {e:?}"))
    }).unwrap();

    let not_eqs = tau_not_eq + sol_not_eq;

    println!("{num_tests} done in {}s", start.elapsed().as_secs_f32());
    if equals > 0 {
        println!("success: {equals}");
    }
    if not_eqs > 0 {
        println!("failures: {not_eqs}");
        println!("| sol ≠: {sol_not_eq}");
        println!("| tau ≠: {tau_not_eq}");
    }
    if l_missing > 0 {
        println!("left empty: {l_missing}");
    }
    if r_missing > 0 {
        println!("right empty: {r_missing}");
    }
    if both_missing > 0 {
        println!("both empty: {both_missing}");
    }

    Ok(())
}

fn compare_results(
    out: (Option<AlgoOut>, Option<AlgoOut>, RankingsCsvRow),
) -> Result<CompResult> {
    let case = out.2;
    let (left, right) = match (out.0, out.1) {
        (None, None) => return Ok(CompResult::BothMissing(case)),
        (Some(l), None) => return Ok(CompResult::RightMissing(l, case)),
        (None, Some(r)) => return Ok(CompResult::LeftMissing(r, case)),
        (Some(l), Some(r)) => (l, r),
    };

    if left.eq(&right) {
        Ok(CompResult::Equal)
    } else {
        // check that neither solution is a superset of the other
        if either_set_rel(&left.minp, &right.minp)
            && either_set_rel(&left.maxp, &right.maxp)
        {
            // the algorithms agree on the solutions,
            // so it's just the tau calculation that's wrong
            Ok(CompResult::TauNotEqual(left, right, case))
        } else {
            Ok(CompResult::SolNotEqual(left, right, case))
        }
    }
}

fn either_set_rel(a: &[(String, String)], b: &[(String, String)]) -> bool {
    superset_rel(a, b) || superset_rel(b, a)
}

/// is the left slice a superset of the right?
fn superset_rel(left: &[(String, String)], right: &[(String, String)]) -> bool {
    for el in right {
        if !left.contains(el) {
            return false;
        }
    }
    true
}
