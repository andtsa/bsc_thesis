#![allow(unused_variables)]
use std::collections::BTreeMap;

use anyhow::Result;
use clap::Parser;
use clap_derive::Parser;
use lib::def::Element;
use lib::def::partial_from_string;
use lib::def::strict_from_partial;
use lib::tau_w::TauVariants;
use lib::tau_w::tau_partial;
use lib::tau_w::tau_w;
use lib::weights::ap_weight;
use lib::weights::unweighted;

#[derive(Parser, Debug)]
pub struct Cli {
    pub a: String,
    pub b: String,
}

fn main() -> Result<()> {
    let args = Cli::parse();

    // we can accept elements being strings (such as "i1")
    // but we want to work with characters, so we just create
    // a bijection str-char
    let mut inp_map: BTreeMap<String, Element> = BTreeMap::new();

    // move out of args
    let rank_a = partial_from_string(&args.a, &mut inp_map)?;
    let rank_b = partial_from_string(&args.b, &mut inp_map)?;

    let weight = unweighted;
    let weight = ap_weight;
    let variant = TauVariants::B;

    let tau = if let (Ok(strict_a), Ok(strict_b)) =
        (strict_from_partial(&rank_a), strict_from_partial(&rank_b))
    {
        tau_w(&strict_a, &strict_b, weight)?
    } else {
        tau_partial(&rank_a, &rank_b, weight, variant)?
    };

    println!("{tau}");

    Ok(())
}
