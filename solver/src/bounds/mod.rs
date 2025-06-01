//! code for either bound calculation
pub mod algo;
pub mod bf;

use std::collections::BTreeMap;
use std::fmt::Display;

use algo::tau_bound;
use anyhow::Result;

use crate::def::PartialOrder;
use crate::def::Ranking;
use crate::def::TauBounds;
use crate::def::TotalOrder;
use crate::def::total_to_repl_string;
use crate::def::total_to_string;

pub fn find_tau_bounds(rank_a: &PartialOrder, rank_b: &PartialOrder) -> Result<TauBounds> {
    let lb = tau_bound(rank_a, rank_b, true)?;
    let ub = tau_bound(rank_a, rank_b, false)?;
    Ok(TauBounds {
        lb: Some(lb),
        ub: Some(ub),
    })
}

pub fn alloc_fixed(
    final_a: &mut TotalOrder,
    final_b: &mut TotalOrder,
    rank_a: &PartialOrder,
    rank_b: &PartialOrder,
) -> Result<()> {
    for i in 0..final_a.len() {
        if let Some(e) = rank_a.get_at(i) {
            final_a.insert_at(e, i)?;
        }
        if let Some(e) = rank_b.get_at(i) {
            final_b.insert_at(e, i)?;
        }
    }
    Ok(())
}

pub fn trivial_alloc(
    final_a: &mut TotalOrder,
    final_b: &mut TotalOrder,
    rank_a: &PartialOrder,
    rank_b: &PartialOrder,
) {
    let mut idxa = 0;
    for tg in rank_a {
        if tg.len() == 1 {
            // single (fixed) element, insert it into its current position
            final_a[idxa] = Some(tg[0]);
            idxa += 1;
        } else {
            // tie-group! need to somehow sort it.

            // for now, nothing
            for elem in tg {
                final_a[idxa] = Some(*elem);
                idxa += 1;
            }
        }
    }

    let mut idxb = 0;
    for tg in rank_b {
        if tg.len() == 1 {
            // single (fixed) element, insert it into its current position
            final_b[idxb] = Some(tg[0]);
            idxb += 1;
        } else {
            // tie-group! need to somehow sort it.

            // for now, nothing
            for elem in tg {
                final_b[idxb] = Some(*elem);
                idxb += 1;
            }
        }
    }
}

impl Display for TauBounds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f)?;
        if let Some(lb) = &self.lb {
            writeln!(f, "tmin:{:?}", lb.t)?;
            // just print the first solution
            writeln!(f, "mina:{}", total_to_string(&lb.a[0]))?;
            writeln!(f, "minb:{}", total_to_string(&lb.b[0]))?;
        }
        if let Some(ub) = &self.ub {
            writeln!(f, "tmax:{:?}", ub.t)?;
            writeln!(f, "maxa:{}", total_to_string(&ub.a[0]))?;
            writeln!(f, "maxb:{}", total_to_string(&ub.b[0]))?;
        }
        std::fmt::Result::Ok(())
    }
}

impl TauBounds {
    /// separate from display because we need the input map argument in order to
    /// show the same elements we were given.
    pub fn print_with_repl(&self, inp_map: &BTreeMap<String, char>) -> Result<String> {
        let rmap = inp_map
            .iter()
            .map(|(x, y)| (*y, x.clone()))
            .collect::<BTreeMap<char, String>>();

        let mut out = String::from("\n");
        if let Some(lb) = &self.lb {
            out.push_str(&format!("tmin:{:?}\n", lb.t));
            for (mina, minb) in lb.a.iter().zip(lb.b.iter()) {
                out.push_str(&format!(
                    "minp:{}/{}\n",
                    total_to_repl_string(mina, &rmap),
                    total_to_repl_string(minb, &rmap)
                ));
            }
        }
        if let Some(ub) = &self.ub {
            out.push_str(&format!("tmax:{:?}\n", ub.t));
            for (maxa, maxb) in ub.a.iter().zip(ub.b.iter()) {
                out.push_str(&format!(
                    "maxp:{}/{}\n",
                    total_to_repl_string(maxa, &rmap),
                    total_to_repl_string(maxb, &rmap)
                ));
            }
        }
        Ok(out)
    }
}
