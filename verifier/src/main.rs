//! # Solution Verifier
//! this program should
//! - be run through the command line
//! - take in a path to an executable algorithm
//! - take in a path to a file with test data
//!
//! and then:
//! - open a reader for the data
//! - for each input:
//!     - start a child process for the algorithm execution
//!     - pass the input
//!     - assert the output is correct
//!
//! exit:
//! - code 0: all tests passed
//! - anything else: something failed

use std::path::PathBuf;
use std::time::Instant;

use anyhow::Result;
use anyhow::bail;
use clap::Parser;
use clap_derive::Parser;
use csv::Reader;
use glob::glob;
use indicatif::ParallelProgressIterator;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use verifier::parsing::parse_entry;
use verifier::parsing::pretty_print;
use verifier::runner::run_solver_on;
use verifier::verify::verify_result;
use verifier::TestCase;
use verifier::TestResult;

#[derive(Parser, Debug)]
#[command(version)]
pub struct Cli {
    pub exec: PathBuf,
    pub data: String,
}

fn main() -> Result<()> {
    let args = Cli::parse();

    let start = Instant::now();

    let mut rows = vec![];

    for src in glob(&args.data)? {
        let mut csv_read = Reader::from_path(&src?)?;

        // check header
        if csv_read.headers()? != vec!["a", "b", "tmin", "tmax", "pmin", "pmax"] {
            bail!("Incompatible CSV data: expected header of `a,b,tmin,tmax,pmin,pmax`");
        }

        // iterate over data
        rows.append(&mut csv_read.records().collect::<Result<Vec<_>, csv::Error>>()?);
    }

    let num_tests = rows.len();

    println!("running {num_tests} tests...");

    let inputs = rows
        .par_iter()
        .map(parse_entry)
        .collect::<Result<Vec<TestCase>>>()?
        .into_iter()
        .map(|i| (args.exec.clone(), i))
        .collect::<Vec<_>>();

    println!("parsed input in {}s", start.elapsed().as_secs_f32());

    let pb = ProgressBar::new(num_tests as u64).with_style(ProgressStyle::default_bar().template(
        "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
    )?);

    let results = inputs
        .par_iter()
        .map(|(e, i)| run_solver_on(e, i.clone()).map(|out| (out, i.clone())))
        .progress_with(pb.clone())
        .map(verify_result)
        .collect::<Result<Vec<_>>>()?;

    let successes = results
        .iter()
        .filter(|x| matches!(x, TestResult::Pass))
        .count();

    let skips = results
        .iter()
        .filter(|x| matches!(x, TestResult::Skipped))
        .count();

    let failures = results
        .iter()
        .filter_map(|x| match x {
            TestResult::Pass | TestResult::Skipped => None,
            TestResult::Fail(i, t) => Some((i, t)),
            TestResult::Empty(f) => {
                println!(
                    "empty algo out on test case {}",
                    pretty_print(f, &Default::default())
                );
                None
            }
        })
        .collect::<Vec<_>>();

    println!(
        "{successes}/{num_tests} test cases passed. total time: {}s",
        start.elapsed().as_secs_f32()
    );

    if skips > 0 {
        println!("{skips} run(s) skipped");
    }

    match failures.len() {
        0 => {}
        1..=5 => {
            println!("{}/{num_tests} cases failed.", failures.len());
            for (f, t) in failures {
                println!("reason: {t:?},\n{}\n", pretty_print(&f.0, &f.1));
            }
        }
        _ => {
            println!(
                "{}/{num_tests} cases failed, showing first 5",
                failures.len()
            );
            for (f, t) in failures.iter().take(5) {
                println!("reason: {t:?},\n{}\n", pretty_print(&f.0, &f.1));
            }
        }
    }

    Ok(())
}
