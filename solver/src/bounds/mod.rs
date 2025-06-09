//! code for either bound calculation
pub mod algo;
pub mod bf;

use algo::tau_bound;
use anyhow::Result;
use lib::def::PartialOrder;
use lib::def::Ranking;
use lib::def::StrictOrder;
use lib::def::TauBounds;
use lib::tau_h::hyperbolic_addtv_weight;

pub fn find_tau_bounds(rank_a: &PartialOrder, rank_b: &PartialOrder) -> Result<TauBounds> {
    let lb = tau_bound(rank_a, rank_b, true, hyperbolic_addtv_weight)?;
    let ub = tau_bound(rank_a, rank_b, false, hyperbolic_addtv_weight)?;
    Ok(TauBounds {
        lb: Some(lb),
        ub: Some(ub),
    })
}

pub fn alloc_fixed(
    final_a: &mut StrictOrder,
    final_b: &mut StrictOrder,
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
    final_a: &mut StrictOrder,
    final_b: &mut StrictOrder,
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
