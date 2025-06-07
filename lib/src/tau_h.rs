use std::collections::BTreeMap;

use anyhow::Result;
use itertools::Itertools;

use crate::def::Element;
use crate::def::PartialOrder;
use crate::def::Ranking;
use crate::def::StrictOrder;

pub enum TauVariants {
    A,
    B,
    W,
}

pub fn unweighted(_: (usize, usize), _: (usize, usize)) -> f64 {
    1.0
}

pub fn tau_unweighted(a: &StrictOrder, b: &StrictOrder) -> Result<f64> {
    tau_h(a, b, unweighted, TauVariants::A)
}

/// compute kendall's tau under weight function w
///
/// runs in O(n^2)
///
/// example:
/// ```
/// # use lib::tau_h::tau_h;
/// let A = vec![Some('a'), Some('b'), Some('c'), Some('d')];
/// let B = vec![Some('a'), Some('b'), Some('c'), Some('d')];
/// let w = |i,j| 1.0 / ((i + j + 1) as f64);
/// assert_eq!(tau_h(&A, &B, w, lib::tau_h::TauVariants::A).unwrap(), 1.0);
/// ```
pub fn tau_h<F: Fn((usize, usize), (usize, usize)) -> f64>(
    a: &StrictOrder,
    b: &StrictOrder,
    w: F,
    variant: TauVariants,
) -> Result<f64> {
    let va = a.ensure_defined()?;
    let vb = b.ensure_defined()?;
    let (_l, item_set) = a.ensure_conjoint(b)?;

    // build map element->index for O(1) comparisons
    let mut map: BTreeMap<char, (usize, usize)> = BTreeMap::new();
    for (i, (a, b)) in va.iter().flatten().zip(vb.iter().flatten()).enumerate() {
        map.entry(*a)
            .and_modify(|e| (e).0 = i)
            .or_insert((i, usize::MAX));
        map.entry(*b)
            .and_modify(|e| (e).1 = i)
            .or_insert((usize::MAX, i));
    }
    let mut num = 0.0;
    let mut total_weight = 0.0;

    let items = item_set.iter().cloned().sorted().collect_vec();
    for (i, x) in items.iter().enumerate() {
        for (_j, y) in items.iter().enumerate().skip(i + 1) {
            let (xa, xb) = *map.get(x).unwrap();
            let (ya, yb) = *map.get(y).unwrap();
            let weight = w((xa, xb), (ya, yb));
            num += weight * sign(xa, ya) * sign(xb, yb);
            total_weight += weight;
        }
    }

    let denom = match variant {
        TauVariants::A => total_weight,
        TauVariants::B => total_weight, // on strict orders A==B
        TauVariants::W => unimplemented!(),
    };

    Ok(num / denom)
}

pub fn sign(a: usize, b: usize) -> f64 {
    if a > b {
        1.0
    } else if a < b {
        -1.0
    } else {
        0.0
    }
}

pub type RankIndexMap = BTreeMap<Element, (usize, usize)>;

/// create a map Element->usize index
///
/// e.g.:
/// ```
/// # use lib::tau_h::index_map;
/// let A = vec![vec!['a','b'], vec!['c']];
/// let B = vec![vec!['b'], vec!['c', 'a']];
/// let map = index_map(&A,&B);
/// assert_eq!(map.get(&'a'), Some((0,1)).as_ref());
/// assert_eq!(map.get(&'b'), Some((0,0)).as_ref());
/// assert_eq!(map.get(&'c'), Some((2,1)).as_ref());
/// ```
pub fn index_map(rank_a: &PartialOrder, rank_b: &PartialOrder) -> RankIndexMap {
    let mut map: BTreeMap<Element, (usize, usize)> = BTreeMap::new();
    let mut idxa = 0;
    for tga in rank_a {
        for a in tga {
            map.entry(*a)
                .and_modify(|e| (e).0 = idxa)
                .or_insert((idxa, usize::MAX));
        }
        idxa += tga.len()
    }
    let mut idxb = 0;
    for tgb in rank_b {
        for b in tgb {
            map.entry(*b)
                .and_modify(|e| (e).1 = idxb)
                .or_insert((usize::MAX, idxb));
        }
        idxb += tgb.len()
    }

    map
}
