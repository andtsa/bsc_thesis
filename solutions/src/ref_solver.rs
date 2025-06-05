use std::collections::BTreeMap;

use anyhow::Result;
use anyhow::bail;
use lib::AlgoOutputRow;
use lib::RankingsCsvRow;
use solver::bounds::bf::tau_bounds_bf;
use solver::def::TotalOrder;
use solver::def::partial_from_string;
use solver::def::total_to_repl_string;

pub fn run_solver(inp: &RankingsCsvRow) -> Result<Option<AlgoOutputRow>> {
    let mut inp_map: BTreeMap<String, char> = BTreeMap::new();
    let rank_a = partial_from_string(&inp.a, &mut inp_map)?;
    let rank_b = partial_from_string(&inp.b, &mut inp_map)?;

    match tau_bounds_bf(&rank_a, &rank_b) {
        Err(_) => Ok(None),
        Ok(bounds) => {
            if let (Some(lb), Some(ub)) = (bounds.lb, bounds.ub) {
                let rmap = inp_map
                    .iter()
                    .map(|(x, y)| (*y, x.clone()))
                    .collect::<BTreeMap<char, String>>();
                let join_sols = |aa: &Vec<TotalOrder>, bb: &Vec<TotalOrder>| {
                    aa.iter()
                        .zip(bb.iter())
                        .map(|(a, b)| {
                            format!(
                                "{}/{}",
                                total_to_repl_string(a, &rmap),
                                total_to_repl_string(b, &rmap)
                            )
                        })
                        .collect::<Vec<String>>()
                        .join("|")
                };
                let min_sols = join_sols(&lb.a, &lb.b);
                let max_sols = join_sols(&ub.a, &ub.b);
                Ok(Some(AlgoOutputRow {
                    a: inp.a.clone(),
                    b: inp.b.clone(),
                    tmin: lb.t,
                    tmax: ub.t,
                    pmin: min_sols,
                    pmax: max_sols,
                }))
            } else {
                bail!("reference solution did not return full solution (???)")
            }
        }
    }
}
