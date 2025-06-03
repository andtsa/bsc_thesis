//! definition of rankings and ties

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use anyhow::Result;
use anyhow::bail;
use regex::Regex;

pub type Element = char;

pub type TieGroup = Vec<Element>;

pub type PartialOrder = Vec<TieGroup>;

pub type TotalOrder = Vec<Option<Element>>;

impl Ranking for TotalOrder {
    fn new_empty(size: usize) -> TotalOrder {
        let mut v = Vec::with_capacity(size);
        for _ in 0..size {
            v.push(None);
        }
        v
    }
    fn is_defined(&self) -> bool {
        self.iter().all(|e| e.is_some())
    }
    fn set_size(&self) -> usize {
        self.len()
    }
    fn set_eq(&self, other: &TotalOrder) -> bool {
        let set = self.iter().flatten().collect::<BTreeSet<&Element>>();
        other.iter().flatten().all(|e| set.contains(e))
    }
    fn item_set(&self) -> BTreeSet<Element> {
        self.iter().flatten().cloned().collect()
    }
    fn get_at(&self, idx: usize) -> Option<Element> {
        self[idx]
    }
    fn all_possible_at(&self, _idx: usize) -> Vec<Element> {
        unimplemented!()
    }

    fn insert_at(&mut self, e: Element, p: usize) -> Result<()> {
        if self[p].is_none() {
            self[p] = Some(e);
            Ok(())
        } else {
            bail!(
                "spot taken! tried to insert {e} in [{p}] of {}",
                total_to_string(self)
            )
        }
    }
    fn fixed_indices(&self) -> Vec<usize> {
        self.iter()
            .enumerate()
            .flat_map(|(i, e)| e.as_ref().map(|_| i))
            .collect()
    }
    fn linear_ext_count(&self) -> u128 {
        1
    }
    fn tau(&self, other: &Self) -> Result<f64> {
        assert!(self.is_defined());
        assert!(other.is_defined());
        // collect items in the order they appear in `self`
        let items: Vec<Element> = self.iter().map(|opt| opt.unwrap()).collect();
        let n = items.len();
        let mut rank_in_self: BTreeMap<Element, usize> = BTreeMap::new();

        // 1. map element -> position in `self`
        for (pos, &opt_e) in self.iter().enumerate() {
            let e = opt_e.unwrap(); // asserted `is_defined()`
            rank_in_self.insert(e, pos);
        }

        // 2. map element -> position in `other`
        let mut rank_in_other: BTreeMap<Element, usize> = BTreeMap::new();
        for (pos, &opt_e) in other.iter().enumerate() {
            let e = opt_e.unwrap();
            rank_in_other.insert(e, pos);
        }

        let mut vec_x = Vec::with_capacity(n);
        let mut vec_y = Vec::with_capacity(n);
        for &e in &items {
            // `items` enumerates each element exactly once, in the order of `self`.
            vec_x.push(*rank_in_self.get(&e).unwrap());
            vec_y.push(*rank_in_other.get(&e).unwrap());
        }

        let (t, _sig) = kendalls::tau_b(&vec_x, &vec_y)?;
        Ok(t)
    }
}

impl Ranking for PartialOrder {
    fn new_empty(size: usize) -> PartialOrder {
        let mut v = Vec::with_capacity(size);
        for _ in 0..size {
            v.push(Vec::new());
        }
        v
    }
    fn is_defined(&self) -> bool {
        self.iter().all(|e| !e.is_empty())
    }
    fn set_size(&self) -> usize {
        self.iter().map(|x| x.len()).sum()
    }
    fn set_eq(&self, other: &PartialOrder) -> bool {
        let set = self.iter().flatten().collect::<BTreeSet<&Element>>();
        other.iter().flatten().all(|e| set.contains(e))
    }
    fn item_set(&self) -> BTreeSet<Element> {
        self.iter().flatten().cloned().collect()
    }
    fn get_at(&self, idx: usize) -> Option<Element> {
        let mut i = 0;
        for tg in self {
            if tg.len() == 1 && i == idx {
                return Some(tg[0]);
            } else {
                i += tg.len()
            }
            if i > idx {
                return None;
            }
        }
        debug_assert!(
            false,
            "called get_at with index {idx} for PO {}",
            partial_to_string(self)
        );
        None
    }

    fn all_possible_at(&self, idx: usize) -> Vec<Element> {
        let mut i = 0;
        for tg in self {
            if i >= idx {
                return tg.clone();
            } else {
                i += tg.len();
            }
        }
        Vec::new()
    }
    fn insert_at(&mut self, _e: Element, _p: usize) -> Result<()> {
        unimplemented!()
    }
    fn fixed_indices(&self) -> Vec<usize> {
        let mut idx = 0;
        let mut out = vec![];
        for tg in self {
            if tg.len() == 1 {
                out.push(idx);
            }
            idx += tg.len();
        }
        out
    }
    fn linear_ext_count(&self) -> u128 {
        let mul = |a: u128, b: u128| a.saturating_mul(b);
        self.iter().map(|x| (1..=(x.len() as u128)).fold(1, mul)).fold(1, mul)
    }
    fn tau(&self, other: &Self) -> Result<f64> {
        assert!(self.is_defined());
        let m = self
            .iter()
            .flatten()
            .enumerate()
            .map(|(a, b)| (*b, a))
            .collect::<BTreeMap<Element, usize>>();
        let flatten_ranking = |r: &Self| {
            r.iter()
                .flatten()
                .map(|e| *m.get(e).expect("???"))
                .collect::<Vec<_>>()
        };

        let (t, _sig) = kendalls::tau_b(&flatten_ranking(self), &flatten_ranking(other))?;
        Ok(t)
    }
}

pub fn partial_from_string(
    s: &str,
    inp_map: &mut BTreeMap<String, Element>,
) -> Result<PartialOrder> {
    let re = Regex::new(
        r"^(?:[A-Za-z0-9]+|\([A-Za-z0-9]+(?: [A-Za-z0-9]+)+\))(?: (?:[A-Za-z0-9]+|\([A-Za-z0-9]+(?: [A-Za-z0-9]+)+\)))*$",
    )?;
    if !re.is_match(s) {
        bail!("{s} must be a string representation of a ranking using alphanumeric tokens");
    }

    let mut out: PartialOrder = Vec::new();
    let mut in_group = false;

    for token in s.split_whitespace() {
        let start = token.starts_with('(');
        let end = token.ends_with(')');

        // strip any leading "(" and trailing ")"
        let core = token.trim_start_matches('(').trim_end_matches(')');

        // core is guaranteed alphanumeric by the regex
        let elem = if let Some(e) = inp_map.get(&core.to_string()) {
            *e
        } else {
            let c = char::from_u32(97 + (inp_map.len() as u32))
                .expect("invalid conversion from string index to char");
            inp_map.insert(core.to_string(), c);
            c
        };

        match (start, end, in_group) {
            // “(x” → begin new group containing x
            (true, false, _) => {
                out.push(vec![elem]);
                in_group = true;
            }
            // “y)” → end group: append y, then close
            (false, true, true) => {
                out.last_mut().unwrap().push(elem);
                in_group = false;
            }
            // “(z)” → single‐element group
            (true, true, _) => {
                out.push(vec![elem]);
                // in_group remains false
            }
            // inside a group: just append
            (false, false, true) => {
                out.last_mut().unwrap().push(elem);
            }
            // standalone element: new singleton group
            (false, false, false) => {
                out.push(vec![elem]);
            }
            // any other combination should never happen
            _ => unreachable!("L"),
        }
    }

    Ok(out)
}

pub fn partial_to_string(r: &PartialOrder) -> String {
    r.iter()
        .map(|tg| {
            debug_assert!(!tg.is_empty());
            if tg.len() == 1 {
                tg[0].to_string()
            } else {
                format!("({})", join_chars(tg, " "))
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}

pub fn partial_to_repl_string(r: &PartialOrder, rmap: &BTreeMap<Element, String>) -> String {
    r.iter()
        .map(|tg| {
            debug_assert!(!tg.is_empty());
            if tg.len() == 1 {
                rmap.get(&tg[0])
                    .map_or("<nf>".to_string(), |c| c.to_string())
            } else {
                format!(
                    "({})",
                    tg.iter()
                        .map(|c| rmap.get(c).map_or("<nf>".to_string(), |s| s.to_string()))
                        .collect::<Vec<String>>()
                        .join(" ")
                )
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}

pub fn total_to_string(o: &TotalOrder) -> String {
    o.iter()
        .map(|x| x.map_or("<empty>".to_string(), |e| e.to_string()))
        .collect::<Vec<String>>()
        .join(" ")
}

pub fn total_to_repl_string(o: &TotalOrder, rmap: &BTreeMap<Element, String>) -> String {
    o.iter()
        .map(|x| {
            x.map_or("<empty>".to_string(), |e| {
                rmap.get(&e).map_or("<nf>".to_string(), |s| s.to_string())
            })
        })
        .collect::<Vec<String>>()
        .join(" ")
}

fn join_chars(c: &[char], sep: &str) -> String {
    c.iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
        .join(sep)
}

pub trait Ranking {
    fn new_empty(size: usize) -> Self;
    fn is_defined(&self) -> bool;
    fn set_size(&self) -> usize;
    fn set_eq(&self, other: &Self) -> bool;
    fn item_set(&self) -> BTreeSet<Element>;
    fn get_at(&self, idx: usize) -> Option<Element>;
    fn all_possible_at(&self, idx: usize) -> Vec<Element>;
    fn insert_at(&mut self, e: Element, p: usize) -> Result<()>;
    fn fixed_indices(&self) -> Vec<usize>;
    fn linear_ext_count(&self) -> u128;
    fn tau(&self, other: &Self) -> Result<f64>;
}

pub struct Bound {
    pub t: f64,
    pub a: Vec<TotalOrder>,
    pub b: Vec<TotalOrder>,
}

pub struct TauBounds {
    pub lb: Option<Bound>,
    pub ub: Option<Bound>,
}
