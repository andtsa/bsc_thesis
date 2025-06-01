//! brute force as reference solution

use anyhow::Result;
use anyhow::bail;
use itertools::Itertools;

use crate::def::*;

impl BruteForce for PartialOrder {
    fn completions(&self) -> Vec<TotalOrder> {
        // 1. for each tie-group, collect all intra-group permutations
        let group_perms: Vec<Vec<Vec<Element>>> = self
            .iter()
            .map(|group| group.iter().cloned().permutations(group.len()).collect())
            .collect();

        // 2. cartesian product across groups, flatten each into a TotalOrder
        group_perms
            .into_iter()
            .multi_cartesian_product() // one Vec<Vec<Element>> per group
            .map(|combo: Vec<Vec<Element>>| {
                // `combo` e.g. [ vec!['B','C'], vec!['A'], vec!['D','E'] ]
                let flat: Vec<Element> = combo.into_iter().flatten().collect();
                let mut to = TotalOrder::new_empty(flat.len());
                for (i, e) in flat.into_iter().enumerate() {
                    to.insert_at(e, i).unwrap();
                }
                to
            })
            .collect()
    }
}

pub fn tau_bounds_bf(a: &PartialOrder, b: &PartialOrder) -> Result<TauBounds> {
    let na = a.linear_ext_count();
    let nb = b.linear_ext_count();
    let le_count = na.saturating_mul(nb);
    if le_count > 25_000_000 {
        bail!("too many linear extensions ({le_count})");
    }

    let le_a = a.completions();
    let le_b = b.completions();

    let mut lb = f64::INFINITY;
    let mut ub = f64::NEG_INFINITY;
    let mut min_pairs = Vec::new();
    let mut max_pairs = Vec::new();

    for x in &le_a {
        for y in &le_b {
            if min_pairs.len() + max_pairs.len() >= 8000 {
                bail!("too many solutions")
            }
            let t = x.tau(y)?;
            if t < lb {
                lb = t;
                min_pairs.clear();
                min_pairs.push((x.clone(), y.clone()));
            } else if t == lb {
                min_pairs.push((x.clone(), y.clone()));
            }
            if t > ub {
                ub = t;
                max_pairs.clear();
                max_pairs.push((x.clone(), y.clone()));
            } else if t == ub {
                max_pairs.push((x.clone(), y.clone()));
            }
        }
    }

    Ok(TauBounds {
        lb: Some(Bound {
            t: lb,
            a: min_pairs.iter().map(|(a, _b)| a.clone()).collect(),
            b: min_pairs.into_iter().map(|(_a, b)| b).collect(),
        }),
        ub: Some(Bound {
            t: ub,
            a: max_pairs.iter().map(|(a, _b)| a.clone()).collect(),
            b: max_pairs.into_iter().map(|(_a, b)| b).collect(),
        }),
    })
}

pub trait BruteForce {
    fn completions(&self) -> Vec<TotalOrder>;
}
