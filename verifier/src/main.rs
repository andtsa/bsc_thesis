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
use clap::Parser;
use clap_derive::Parser;
use indicatif::ParallelProgressIterator;
use lib::progress_bar;
use lib::read_glob_csv;
use lib::run_solver_on;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use verifier::TestCase;
use verifier::TestResult;
use verifier::parsing::parse_entry;
use verifier::parsing::pretty_print;
use verifier::verify::verify_result;

#[derive(Parser, Debug)]
#[command(version)]
pub struct Cli {
    pub exec: PathBuf,
    pub data: String,
}

fn main() -> Result<()> {
    let args = Cli::parse();

    let start = Instant::now();

    let rows = read_glob_csv(&args.data, vec!["a", "b", "tmin", "tmax", "pmin", "pmax"])?;

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

    let pb = progress_bar(num_tests as u64)?;

    let results = inputs
        .par_iter()
        .map(|(e, i)| run_solver_on(e, i.clone()).map(|out| (out, i.clone())))
        .progress_with(pb.clone())
        .map(verify_result)
        .collect::<Result<Vec<_>>>()?;

    let successes = results
        .iter()
        .filter(|x| matches!(x, TestResult::Pass | TestResult::Complete))
        .count();

    let completes = results
        .iter()
        .filter(|x| matches!(x, TestResult::Complete))
        .count();

    let skips = results
        .iter()
        .filter(|x| matches!(x, TestResult::Skipped))
        .count();

    let failures = results
        .iter()
        .filter_map(|x| match x {
            TestResult::Pass | TestResult::Skipped | TestResult::Complete => None,
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

    println!("{successes}/{num_tests} test cases passed,");
    println!("{completes}/{successes} solutions were complete.");
    println!("total time: {}s", start.elapsed().as_secs_f32());

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
