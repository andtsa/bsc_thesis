use anyhow::Result;
use anyhow::bail;
use csv::StringRecord;

use crate::AlgoOut;
use crate::CsvRow;
use crate::TestCase;

/// get the input & solution from csv row
pub fn parse_entry(row: &StringRecord) -> Result<TestCase> {
    if row.len() != 6 {
        bail!("improper row length ({}): {row:?}", row.len());
    }

    let parsed_row: CsvRow = row.deserialize(None)?;

    // parse the min/max permutations
    let parse_permutations = |sols: String| {
        sols.split('|')
            .map(|p| {
                let mut rs = p.split('/');
                if let (Some(fst), Some(snd)) = (rs.next(), rs.next()) {
                    Ok((fst.to_string(), snd.to_string()))
                } else {
                    bail!("malformed permutation data: {rs:?}")
                }
            })
            .collect::<Result<Vec<_>>>()
    };

    let min_sol_pairs = parse_permutations(parsed_row.min_sols)?;
    let max_sol_pairs = parse_permutations(parsed_row.max_sols)?;

    Ok(TestCase {
        a: parsed_row.a,
        b: parsed_row.b,
        tmin: parsed_row.tmin,
        tmax: parsed_row.tmax,
        min_sol_pairs,
        max_sol_pairs,
    })
}

pub fn pretty_print(tc: &TestCase, ao: &AlgoOut) -> String {
    let show = |o: &Option<f64>| {
        o.as_ref()
            .map_or_else(|| "none".to_string(), |v| v.to_string())
    };
    format!(
        "test case:\n\
         > a: {}\n\
         > b: {}\n\
         tmin\n \
         | sol: {}\n \
         | alg: {}\n\
         tmax\n \
         | sol: {}\n \
         | alg: {}\n\
         pmin\n \
         | sol: {:?}\n \
         | alg minp: {:?}\n\
         pmax\n \
         | sol: {:?}\n \
         | alg maxp: {:?}\n \
        ",
        tc.a,
        tc.b,
        tc.tmin,
        show(&ao.tmin),
        tc.tmax,
        show(&ao.tmax),
        tc.min_sol_pairs,
        ao.minp,
        tc.max_sol_pairs,
        ao.maxp,
    )
}
