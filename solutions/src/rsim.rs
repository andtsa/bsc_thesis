//! Data generation (ranking simulation)
#![allow(unused)]
use std::path::PathBuf;
use std::time::Instant;

use anyhow::Result;
use anyhow::bail;
use clap::Parser;
use clap_derive::Parser;
use csv::Reader;
use csv::StringRecord;
use csv::Writer;
use indicatif::ParallelProgressIterator;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use solver::def::PartialOrder;

const ABC: &str = "abcdefghijklmnopqrstuvwxyz";

#[derive(Parser, Debug)]
#[command(version)]
pub struct Cli {
    pub lens_min: usize,
    pub lens_max: usize,
    pub ties_min: usize,
    pub ties_max: usize,
    pub taus_min: f32,
    pub taus_max: f32,
}

fn main() -> Result<()> {
    Ok(())
}

/// # integer representation of a ranking with ties
/// - one u64 has 16 * 4 bits with indices 0-15
/// - the index of the 4 bit number within the u64 corresponds to an index in
///   &ABC, which gives us the name (char) of an element
/// - the value of the 4 bit number is the rank of the element from the previous
///   step
pub fn int_to_rwt(i: u64) -> PartialOrder {
    todo!()
}
