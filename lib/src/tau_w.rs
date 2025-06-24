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

/// compute kendall's tau under weight function w
///
/// runs in O(n^2)
///
/// example:
/// ```
/// # use lib::tau_w::tau_w;
/// let A = vec![Some('a'), Some('b'), Some('c'), Some('d')];
/// let B = vec![Some('a'), Some('b'), Some('c'), Some('d')];
/// let w = |i,j| 1.0 / ((i + j + 1) as f64);
/// assert_eq!(tau_w(&A, &B, w, lib::tau_w::TauVariants::A).unwrap(), 1.0);
/// ```
pub fn tau_w<F: Fn((usize, usize), (usize, usize)) -> f64>(
    a: &StrictOrder,
    b: &StrictOrder,
    w: F,
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

    Ok(num / total_weight)
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
        let average_position = ((idxa + idxa + tga.len() - 1) / 2) + 1;
        for a in tga {
            map.entry(*a)
                .and_modify(|e| (e).0 = average_position)
                .or_insert((average_position, usize::MAX));
        }
        idxa += tga.len()
    }
    let mut idxb = 0;
    for tgb in rank_b {
        let average_position = ((idxb + idxb + tgb.len() - 1) / 2) + 1;
        for b in tgb {
            map.entry(*b)
                .and_modify(|e| (e).1 = average_position)
                .or_insert((usize::MAX, average_position));
        }
        idxb += tgb.len()
    }

    map
}

pub fn tau_partial<F: Fn((usize, usize), (usize, usize)) -> f64>(
    a: &PartialOrder,
    b: &PartialOrder,
    w: F,
    variant: TauVariants,
) -> Result<f64> {
    let map = index_map(a, b);
    let items = map.keys().sorted().cloned().collect_vec();

    let mut concordance = 0.0;
    let mut total_weight = 0.0;
    let mut ties_a = 0.0;
    let mut ties_b = 0.0;
    let mut ties_both = 0.0;

    for (i, x) in items.iter().enumerate() {
        for y in items.iter().skip(i + 1) {
            let (xa, xb) = *map.get(x).unwrap();
            let (ya, yb) = *map.get(y).unwrap();
            let weight = w((xa, xb), (ya, yb));

            let sa = sign(xa, ya);
            let sb = sign(xb, yb);

            match (sa, sb) {
                (0.0, 0.0) => ties_both += weight,
                (0.0, _) => ties_a += weight,
                (_, 0.0) => ties_b += weight,
                _ => concordance += weight * sa * sb,
            }
            total_weight += weight;
        }
    }

    let denom = match variant {
        TauVariants::A => total_weight,
        TauVariants::B => {
            let sum_cd = total_weight - (ties_a + ties_b + ties_both); // = C+D
            let denom_a = sum_cd + ties_a;
            let denom_b = sum_cd + ties_b;
            (denom_a * denom_b).sqrt()
        }
        TauVariants::W => unimplemented!(), // waiting on Gazeel (2025)
    };

    #[cfg(debug_assertions)]
    {
        println!("{concordance} / {denom}");
    }
    Ok(concordance / denom)
}
