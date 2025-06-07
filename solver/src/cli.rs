//! calculation of $\tau_{min}, \tau_{max}$
use std::collections::BTreeMap;

use anyhow::Result;
use clap::Parser;
use clap_derive::Parser;
use lib::def::Element;
use lib::def::PartialOrder;
use lib::def::TauBounds;
use lib::def::partial_from_string;

#[derive(Parser, Debug)]
#[command(version)]
pub struct Cli {
    pub a: String,
    pub b: String,
}

pub fn compute<Algo>(algo: Algo) -> Result<()>
where
    Algo: Fn(&PartialOrder, &PartialOrder) -> Result<TauBounds>,
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

    let bounds = algo(&rank_a, &rank_b)?;

    println!("{}", bounds.print_with_repl(&inp_map)?);

    Ok(())
}
