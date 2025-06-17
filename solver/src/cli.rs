//! calculation of $\tau_{min}, \tau_{max}$
#![allow(unused_variables)]
use std::collections::BTreeMap;

use anyhow::Result;
use clap::Parser;
use clap_derive::Parser;
use lib::def::Element;
use lib::def::PartialOrder;
use lib::def::TauBounds;
use lib::def::partial_from_string;
use lib::weights::ap_high_weight;
use lib::weights::ap_weight;
use lib::weights::const_weight_42;
use lib::weights::expo_thresh_weight;
use lib::weights::hyper_left_weight;
use lib::weights::hyperbolic_addtv_weight;
use lib::weights::hyperbolic_mult_weight;
use lib::weights::hyperbolic_sym_mult_weight;
use lib::weights::rbo_other_weight;
use lib::weights::rbo_weight;
use lib::weights::threshold_weight;
use lib::weights::unweighted;
use lib::weights::weight_inv_left;
use lib::weights::weight_inv_log;
use lib::weights::weight_inv_right;
use lib::weights::weight_left;
use lib::weights::weight_right;
use lib::weights::weight_sum;
use lib::weights::weight_zero;

#[derive(Parser, Debug)]
#[command(version)]
pub struct Cli {
    pub a: String,
    pub b: String,
}

pub fn compute<Algo>(algo: Algo) -> Result<()>
where
    Algo: Fn(&PartialOrder, &PartialOrder, fn((usize, usize), (usize, usize)) -> f64) -> Result<TauBounds>,
{
    let args = Cli::parse();

    // we can accept elements being strings (such as "i1")
    // but we want to work with characters, so we just create
    // a bijection str-char
    let mut inp_map: BTreeMap<String, Element> = BTreeMap::new();

    // move out of args
    let rank_a = partial_from_string(&args.a, &mut inp_map)?;
    let rank_b = partial_from_string(&args.b, &mut inp_map)?;

    #[cfg(debug_assertions)]
    {
        use lib::def::partial_to_repl_string;
        let rmap = inp_map
            .iter()
            .map(|(x, y)| (*y, x.clone()))
            .collect::<BTreeMap<char, String>>();
        println!("{rank_a:?}");
        println!("{}", partial_to_repl_string(&rank_a, &rmap));
        println!("{}", partial_to_repl_string(&rank_b, &rmap));
        println!("{rank_b:?}");
    }

    // passes tests
    let w = ap_weight;
    let w = unweighted;
    let w = const_weight_42;
    let w = ap_high_weight;
    let w = weight_zero;
    let w = weight_inv_log;

    // this seems interesting
    let w = rbo_weight;
    // passes, but i dont know why??
    let w = weight_inv_left;

    // fails tests
    let w = hyperbolic_addtv_weight;
    let w = hyperbolic_mult_weight;
    let w = weight_inv_right;
    let w = weight_left;
    let w = weight_right;
    let w = weight_sum;
    let w = threshold_weight;
    let w = expo_thresh_weight;
    let w = rbo_other_weight;
    let w = hyperbolic_sym_mult_weight;
    let w = hyper_left_weight;

    // current test
    let w = unweighted;

    let bounds = match algo(&rank_a, &rank_b, w) {
        Ok(sol) => sol,
        Err(e) => {
            if format!("{e}").contains("skipped") {
                println!("skipped: {e}");
                return Ok(());
            } else {
                return Err(e);
            }
        }
    };

    println!("{}", bounds.print_with_repl(&inp_map)?);

    Ok(())
}
