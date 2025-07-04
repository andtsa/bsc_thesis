//! definition of rankings and ties

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fmt::Display;

use anyhow::Result;
use anyhow::anyhow;
use anyhow::bail;
use anyhow::ensure;
use itertools::Itertools;
use regex::Regex;

pub type Element = char;

pub type TieGroup = Vec<Element>;

pub type PartialOrder = Vec<TieGroup>;

pub type StrictOrder = Vec<Option<Element>>;

impl Ranking for StrictOrder {
    fn new_empty(size: usize) -> StrictOrder {
        let mut v = Vec::with_capacity(size);
        for _ in 0..size {
            v.push(None);
        }
        v
    }

    fn rank_eq(&self, other: &Self) -> bool {
        self.eq(other)
    }

    fn is_defined(&self) -> bool {
        self.iter().all(|e| e.is_some())
    }

    fn ensure_defined(&self) -> Result<Vec<Vec<Element>>> {
        self.iter()
            .map(|x| {
                x.map(|xx| vec![xx]).ok_or(anyhow!(
                    "ranking is not fully defined: {}",
                    total_to_string(self)
                ))
            })
            .collect::<Result<Vec<Vec<Element>>>>()
    }

    fn ensure_conjoint(&self, other: &Self) -> Result<(usize, BTreeSet<Element>)> {
        let mut item_set = BTreeSet::new();
        for a in self.iter().flatten() {
            item_set.insert(*a);
        }
        for b in other {
            ensure!(
                b.is_some_and(|bb| item_set.contains(&bb)),
                "conjointness failed!\na=[{}]\nb=[{}]",
                total_to_string(self),
                total_to_string(other),
            );
        }
        Ok((item_set.len(), item_set))
    }

    fn set_size(&self) -> usize {
        self.len()
    }

    fn set_eq(&self, other: &StrictOrder) -> bool {
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
}

fn sort_eq(a: &[Element], b: &[Element]) -> bool {
    a.iter()
        .sorted()
        .zip(b.iter().sorted())
        .all(|(x, y)| x.eq(y))
}

impl Ranking for PartialOrder {
    fn new_empty(size: usize) -> PartialOrder {
        let mut v = Vec::with_capacity(size);
        for _ in 0..size {
            v.push(Vec::new());
        }
        v
    }

    fn rank_eq(&self, other: &Self) -> bool {
        self.iter().zip(other).all(|(tga, tgb)| sort_eq(tga, tgb))
    }

    fn is_defined(&self) -> bool {
        self.iter().all(|e| !e.is_empty())
    }

    fn ensure_defined(&self) -> Result<Vec<Vec<Element>>> {
        self.iter()
            .map(|x| {
                if x.is_empty() {
                    Err(anyhow!(
                        "ranking is not fully defined: {}",
                        partial_to_string(self)
                    ))
                } else {
                    Ok(x.clone())
                }
            })
            .collect::<Result<Vec<Vec<Element>>>>()
    }

    fn set_size(&self) -> usize {
        self.iter().map(|x| x.len()).sum()
    }

    fn ensure_conjoint(&self, other: &Self) -> Result<(usize, BTreeSet<Element>)> {
        let mut item_set = BTreeSet::new();
        for tg in self {
            for a in tg {
                item_set.insert(*a);
            }
        }
        for tg in other {
            for b in tg {
                ensure!(
                    item_set.contains(b),
                    "conjointness failed!\na=[{}]\nb=[{}]",
                    partial_to_string(self),
                    partial_to_string(other)
                );
            }
        }
        Ok((item_set.len(), item_set))
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
        self.iter()
            .map(|x| (1..=(x.len() as u128)).fold(1, mul))
            .fold(1, mul)
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

pub fn partial_to_repl_string(
    r: &PartialOrder,
    rmap: &BTreeMap<Element, String>,
) -> String {
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

pub fn strict_from_partial(p: &PartialOrder) -> Result<StrictOrder> {
    if !p.iter().all(|x| x.len() == 1) {
        bail!("{} is not a strict order", partial_to_string(p))
    }
    Ok(p.iter().map(|x| Some(x[0])).collect_vec())
}

pub fn total_to_string(o: &StrictOrder) -> String {
    o.iter()
        .map(|x| x.map_or("<empty>".to_string(), |e| e.to_string()))
        .collect::<Vec<String>>()
        .join(" ")
}

pub fn total_to_repl_string(o: &StrictOrder, rmap: &BTreeMap<Element, String>) -> String {
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
    fn rank_eq(&self, other: &Self) -> bool;
    fn is_defined(&self) -> bool;
    fn ensure_defined(&self) -> Result<Vec<Vec<Element>>>;
    fn ensure_conjoint(&self, other: &Self) -> Result<(usize, BTreeSet<Element>)>;
    fn set_size(&self) -> usize;
    fn set_eq(&self, other: &Self) -> bool;
    fn item_set(&self) -> BTreeSet<Element>;
    fn get_at(&self, idx: usize) -> Option<Element>;
    fn all_possible_at(&self, idx: usize) -> Vec<Element>;
    fn insert_at(&mut self, e: Element, p: usize) -> Result<()>;
    fn fixed_indices(&self) -> Vec<usize>;
    fn linear_ext_count(&self) -> u128;
}

pub struct Bound {
    pub t: f64,
    pub a: Vec<StrictOrder>,
    pub b: Vec<StrictOrder>,
}

pub struct TauBounds {
    pub lb: Option<Bound>,
    pub ub: Option<Bound>,
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
