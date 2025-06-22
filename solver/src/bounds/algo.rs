//! compute tau
use std::cmp::Ordering;
use std::collections::BTreeMap;

use anyhow::Result;
use anyhow::ensure;
use itertools::Itertools;
use lib::def::*;
use lib::tau_w::RankIndexMap;
use lib::tau_w::index_map;
use lib::tau_w::tau_w;
use petgraph::acyclic::Acyclic;
use petgraph::data::DataMap;
use petgraph::graph::DiGraph;
use petgraph::graph::Edge;
use petgraph::graph::NodeIndex;

use crate::bounds::trivial_alloc;

pub type PartialRankGraph = DiGraph<Element, (Element, Element)>;
type NX = NodeIndex;

pub fn tau_bound<F: Fn((usize, usize), (usize, usize)) -> f64>(
    rank_a: &PartialOrder,
    rank_b: &PartialOrder,
    is_minimising: bool,
    w: F,
) -> Result<Bound> {
    #[cfg(debug_assertions)]
    println!(
        "{} {}/{}",
        if is_minimising {
            "minimising"
        } else {
            "maximising"
        },
        partial_to_string(rank_a),
        partial_to_string(rank_b)
    );
    // checks
    let (length, _item_set) = rank_a.ensure_conjoint(rank_b)?;

    ensure!(
        length >= 2,
        "ranks are too short ({}): {rank_a:?}/{rank_b:?}",
        length
    );

    let mut final_a = StrictOrder::new_empty(length);
    let mut final_b = StrictOrder::new_empty(length);

    // check if ties exist to exit early
    if rank_a.len() == length && rank_b.len() == length {
        trivial_alloc(&mut final_a, &mut final_b, rank_a, rank_b);
        let (t, _sig) = kendalls::tau_b(&final_a, &final_b)?;
        return Ok(Bound {
            a: vec![final_a],
            b: vec![final_b],
            t,
        });
    }

    // initialise graphs
    let item_set = rank_a.item_set();
    debug_assert_eq!(item_set, rank_b.item_set());
    // construct the graphs here in order also create node-lists for petgraph's
    // internal indexing.
    let mut ga = PartialRankGraph::new();
    let nla = item_set
        .iter()
        .map(|e| (*e, ga.add_node(*e)))
        .collect::<BTreeMap<Element, _>>();
    let mut gb = PartialRankGraph::new();
    let nlb = item_set
        .iter()
        .map(|e| (*e, gb.add_node(*e)))
        .collect::<BTreeMap<Element, _>>();

    // find all edges
    // in n^2
    partial_edges(rank_a, &mut ga, &nla);
    partial_edges(rank_b, &mut gb, &nlb);

    // result must be a strict total order,
    // which is equivalent to acyclic tournament.
    // + petgraph enforces acyclic,
    // + we promise it's a tournament.
    // it's much much easier to preserve acyclic property when adding edges
    // than to find if removing an edge will eventually make the graph acyclic,
    // hence we create new graphs under petgraph's acyclic invariant.

    // fill target graphs with nodes, keep track of their indices (as chars)
    let mut gfa = PartialRankGraph::new();
    let nlfa = item_set
        .iter()
        .map(|e| (*e, gfa.add_node(*e)))
        .collect::<BTreeMap<Element, _>>();
    let mut gfb = PartialRankGraph::new();
    let nlfb = item_set
        .iter()
        .map(|e| (*e, gfb.add_node(*e)))
        .collect::<BTreeMap<Element, _>>();

    // "creating" an acyclic graph is a fallible operation,
    // hence can't be done implicitly.
    let mut gfa =
        Acyclic::try_from_graph(gfa).expect("can't convert vertex graph to acyclic (??)");
    let mut gfb =
        Acyclic::try_from_graph(gfb).expect("can't convert vertex graph to acyclic (??)");

    // sanity check: assert that the node indices still correspond to the node-lists
    verify_internal_node_indices(&gfa, &nla)?;
    verify_internal_node_indices(&gfb, &nlb)?;

    let rank_index_map = index_map(rank_a, rank_b);
    let sort_cmp = |a, b| edge_cmp(&a, &b, &w, &rank_index_map, false);
    // fill in gfa based on the edges in gb
    let other_edges = gb
        .raw_edges()
        .iter()
        .sorted_by(|a, b| sort_cmp(*a, *b))
        .collect_vec();
    for edge in other_edges {
        // to _minimise_ concordance, we want to try to add the inverse yx of every
        // edge xy that's in the other graph (since only one of xy,yx will be added,
        // managing to add yx results in +1 discordant pair)
        let (scid, dsid) = if is_minimising {
            (edge.weight.1, edge.weight.0)
        } else {
            edge.weight
        };
        // is adding this edge even an option in the first place?
        // the strict order we end up with _must_ be a linear extension of the partial
        // order we started with, so we can't add any edges that weren't already
        // there
        if ga.contains_edge(nla[&scid], nla[&dsid]) {
            let other_scidx = nlfa.get(&scid).expect("indices changed unexpectedly"); // the indices ought to stay the same between the two graphs
            let other_dsidx = nlfa.get(&dsid).expect("indices changed unexpectedly");
            match gfa.try_add_edge(*other_scidx, *other_dsidx, (scid, dsid)) {
                Ok(_a) => {
                    #[cfg(debug_assertions)]
                    println!("inserted (concordant) in gfa ({scid},{dsid})[{_a:?}]");
                }
                Err(_e) => {
                    #[cfg(debug_assertions)]
                    println!("failed (concordant) ({scid},{dsid})->gfa: {_e:?}");
                }
            }
        } else if ga.contains_edge(nla[&dsid], nla[&scid]) {
            // we couldn't add the edge x->y above, but we still need an edge connecting x
            // and y. this means we must have y->x, which is discordant.
            let other_scidx = nlfa.get(&dsid).expect("indices changed unexpectedly"); // the indices ought to stay the same between the two graphs
            let other_dsidx = nlfa.get(&scid).expect("indices changed unexpectedly");
            match gfa.try_add_edge(*other_scidx, *other_dsidx, (scid, dsid)) {
                Ok(_a) => {
                    #[cfg(debug_assertions)]
                    println!("inserted (discordant) in gfa ({scid},{dsid})[{_a:?}]");
                }
                Err(_e) => {
                    #[cfg(debug_assertions)]
                    println!("failed (discordant) ({scid},{dsid})->gfa: {_e:?}");
                }
            }
        } else {
            unreachable!("ga contains neither {scid}->{dsid} nor {dsid}->{scid}");
        }
    }

    // if minimising, then we want to always pick the opposite edge of a 2-cycle
    // (opposite of the previous graph creation)
    // For example, if we have 2 elements `x,y` which are tied in both rankings,
    // then xy ∈ ga and yx ∈ ga and xy ∈ gb and yx ∈ gb, so
    // - it doesn't matter if a<b or b<a in the resulting orders,
    // - in the maximisation case we want to pick the same order,
    // - in the minimisation case we want to pick the reverse order,
    // - which of the two xy or yx we pick depends only on which one we encounter
    //   first.
    let sort_cmp = |a, b| edge_cmp(&a, &b, &w, &rank_index_map, is_minimising);
    let other_edges = ga
        .raw_edges()
        .iter()
        .sorted_by(|a, b| sort_cmp(*a, *b))
        .collect_vec();
    // depending on the underlying implementation of the graph (specifically
    // `.raw_edges()` edge iteratior) sorting may not be necessary–and for
    // petgraph this is the case. the only reason I include sorting here is to
    // illustrate that this algorithm's optimality depends on the order in which
    // we see edges when adding them to gfa,gfb.
    for edge in other_edges {
        let (scid, dsid) = if is_minimising {
            (edge.weight.1, edge.weight.0)
        } else {
            edge.weight
        };
        // is adding this edge even an option in the first place?
        if gb.contains_edge(nlb[&scid], nlb[&dsid]) {
            let other_scidx = nlfb.get(&scid).expect("indices changed unexpectedly");
            let other_dsidx = nlfb.get(&dsid).expect("indices changed unexpectedly");
            match gfb.try_add_edge(*other_scidx, *other_dsidx, (scid, dsid)) {
                Ok(_a) => {
                    #[cfg(debug_assertions)]
                    println!("inserted (concordant) in gfb ({scid},{dsid})[{_a:?}]");
                }
                Err(_e) => {
                    #[cfg(debug_assertions)]
                    println!("failed (concordant) ({scid},{dsid})->gfb: {_e:?}");
                }
            }
        } else if gb.contains_edge(nlb[&dsid], nlb[&scid]) {
            // we couldn't add the edge x->y above, but we still need an edge connecting x
            // and y. this means we must have y->x, which is discordant.
            let other_scidx = nlfb.get(&dsid).expect("indices changed unexpectedly"); // the indices ought to stay the same between the two graphs
            let other_dsidx = nlfb.get(&scid).expect("indices changed unexpectedly");
            match gfb.try_add_edge(*other_scidx, *other_dsidx, (scid, dsid)) {
                Ok(_a) => {
                    #[cfg(debug_assertions)]
                    println!("inserted (discordant) in gfb ({scid},{dsid})[{_a:?}]");
                }
                Err(_e) => {
                    #[cfg(debug_assertions)]
                    println!("failed (discordant) ({scid},{dsid})->gfb: {_e:?}");
                }
            }
        } else {
            unreachable!("gb contains neither {scid}->{dsid} nor {dsid}->{scid}");
        }
    }

    // construct the list-represented [`TotalOrder`]s from their graph
    // representations
    let final_a = thm_acy_tnm_sto(rank_a, &gfa, &nlfa, length);
    let final_b = thm_acy_tnm_sto(rank_b, &gfb, &nlfb, length);

    // let t = final_a.tau(&final_b)?;
    let t = tau_w(&final_a, &final_b, w)?;
    Ok(Bound {
        a: vec![final_a], // we only construct 1 solution!
        b: vec![final_b], /* there are often multiple optimal solutions, but we can't
                           * find them all */
        t, /* in polynomial time since there can be exponentially many optimal
            * solutions. */
    })
}

/// an acyclic tournament is equivalent to a strict total order
pub fn thm_acy_tnm_sto(
    rank: &PartialOrder,
    tnm: &Acyclic<PartialRankGraph>,
    nl: &BTreeMap<Element, NX>,
    must_size: usize,
) -> StrictOrder {
    let mut o = StrictOrder::new_empty(must_size);
    let mut idx = 0;
    for tg in rank {
        if tg.len() == 1 {
            o[idx] = Some(tg[0]);
            idx += 1;
        } else {
            // need to sort the tie group according to the tournament
            let mut tgc = tg.clone();
            tgc.sort_by(|a, b| {
                let idxa = nl[a];
                let idxb = nl[b];
                match (tnm.contains_edge(idxa, idxb), tnm.contains_edge(idxb, idxa)) {
                    (false, false) => unreachable!("not a tournament!"),
                    (true, false) => Ordering::Less,
                    (false, true) => Ordering::Greater,
                    (true, true) => unreachable!("graph has cycles!"),
                }
            });

            for elem in tgc {
                o[idx] = Some(elem);
                idx += 1;
            }
        }
    }

    assert!(o.is_defined());
    o
}

/// construct the graph representation of a partial total order.
///
/// takes:
/// - a partial total order
/// - a graph
/// - a node-list, mapping [`Element`]s (ascii characters) to the internal
///   indexing of petgraph vertices
#[allow(unused_labels)]
pub fn partial_edges(
    rank: &PartialOrder,
    g: &mut PartialRankGraph,
    nl: &BTreeMap<Element, NX>,
) {
    // all edges start from one tie-group,
    'fst_group: for (i, tg) in rank.iter().enumerate() {
        // and go to a second (later or equal) tie-group
        'snd_group: for stg in rank.iter().skip(i) {
            'fst_elem: for x in tg {
                'snd_elem: for y in stg {
                    // in the first iteration of 'snd_group, tg==stg, and we want all the
                    // edges between tied elements _except_ for
                    // self-loops.
                    if x != y {
                        g.add_edge(nl[x], nl[y], (*x, *y));
                    }
                }
            }
        }
    }
}

/// sanity check
pub fn verify_internal_node_indices(
    g: &Acyclic<PartialRankGraph>,
    nl: &BTreeMap<Element, NX>,
) -> Result<()> {
    for (nn, ni) in nl.iter() {
        ensure!(g.node_weight(*ni) == Some(nn));
    }
    Ok(())
}

/// the order to use for picking edges from `other` graph.
/// it doesn't matter as long as it's transitive & total.
pub fn edge_cmp<W: Fn((usize, usize), (usize, usize)) -> f64>(
    e1: &&Edge<(Element, Element)>,
    e2: &&Edge<(Element, Element)>,
    w: W,
    m: &RankIndexMap,
    is_minimising: bool,
) -> Ordering {
    let (a1, b1) = e1.weight;
    let (a2, b2) = e2.weight;
    let cmp = w(m[&a1], m[&b1]).partial_cmp(&w(m[&a2], m[&b2]));
    match cmp {
        Some(Ordering::Equal) | None => {
            if is_minimising {
                match a2.cmp(&a1) {
                    Ordering::Equal => b2.cmp(&b1),
                    x => x,
                }
            } else {
                match a1.cmp(&a2) {
                    Ordering::Equal => b1.cmp(&b2),
                    x => x,
                }
            }
        }
        Some(x) => x,
    }
}
