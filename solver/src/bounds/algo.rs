//! compute tau
use std::cmp::Ordering;
use std::collections::BTreeMap;

use anyhow::Result;
use anyhow::ensure;
use itertools::Itertools;
use petgraph::acyclic::Acyclic;
use petgraph::data::DataMap;
use petgraph::graph::DiGraph;
use petgraph::graph::NodeIndex;

use crate::bounds::trivial_alloc;
use crate::def::*;

pub type PartialRankGraph = DiGraph<Element, (Element, Element)>;
type NX = NodeIndex;

pub fn tau_bound(
    rank_a: &PartialOrder,
    rank_b: &PartialOrder,
    is_minimising: bool,
) -> Result<Bound> {
    // checks
    let rank_a_size = rank_a.set_size();
    let rank_b_size = rank_b.set_size();
    ensure!(
        rank_a_size == rank_b_size,
        "non-conjoint rankings: diff length"
    );
    ensure!(rank_a.set_eq(rank_b), "non-conjoint rankings: set neq");

    // unambiguously equal length
    let length = rank_a_size;

    ensure!(
        length >= 2,
        "ranks are too short ({}): {rank_a:?}/{rank_b:?}",
        length
    );

    let mut final_a = TotalOrder::new_empty(length);
    let mut final_b = TotalOrder::new_empty(length);
    ensure!(final_a.set_size() == rank_b.set_size());

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
    let mut gfa = Acyclic::try_from_graph(gfa).expect("can't convert vertex graph to acyclic (??)");
    let mut gfb = Acyclic::try_from_graph(gfb).expect("can't convert vertex graph to acyclic (??)");

    // sanity check: assert that the node indices still correspond to the node-lists
    verify_internal_node_indices(&gfa, &nla)?;
    verify_internal_node_indices(&gfb, &nlb)?;

    // fill in gfa based on the edges in gb
    let other_edges = gb.raw_edges().iter().sorted_by(edge_cmp).collect_vec();
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
                    println!("inserted in gfa ({scid},{dsid})[{_a:?}]");
                }
                Err(_e) => {
                    #[cfg(debug_assertions)]
                    println!("failed ({scid},{dsid})->gfa: {_e:?}");
                }
            }
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
    let mut other_edges = ga.raw_edges().iter().sorted_by(edge_cmp).collect_vec();
    // depending on the underlying implementation of the graph (specifically
    // `.raw_edges()` edge iteratior) sorting may not be necessary–and for
    // petgraph this is the case. the only reason I include sorting here is to
    // illustrate that this algorithm's optimality depends on the order in which
    // we see edges when adding them to gfa,gfb.
    if is_minimising {
        other_edges.reverse();
    }
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
                    println!("inserted in gfb ({scid},{dsid})[{_a:?}]");
                }
                Err(_e) => {
                    #[cfg(debug_assertions)]
                    println!("failed ({scid},{dsid})->gfb: {_e:?}");
                }
            }
        }
    }

    // construct the list-represented [`TotalOrder`]s from their graph
    // representations
    let final_a = thm_acy_tnm_sto(rank_a, &gfa, &nlfa, length);
    let final_b = thm_acy_tnm_sto(rank_b, &gfb, &nlfb, length);

    let t = final_a.tau(&final_b)?;
    Ok(Bound {
        a: vec![final_a], // we only construct 1 solution!
        b: vec![final_b], // there are often multiple optimal solutions, but we can't find them all
        t,                /* in polynomial time since there can be exponentially many optimal
                           * solutions. */
    })
}

/// an acyclic tournament is equivalent to a strict total order
pub fn thm_acy_tnm_sto(
    rank: &PartialOrder,
    tnm: &Acyclic<PartialRankGraph>,
    nl: &BTreeMap<Element, NX>,
    must_size: usize,
) -> TotalOrder {
    let mut o = TotalOrder::new_empty(must_size);
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
pub fn partial_edges(rank: &PartialOrder, g: &mut PartialRankGraph, nl: &BTreeMap<Element, NX>) {
    // all edges start from one tie-group,
    'fst_group: for (i, tg) in rank.iter().enumerate() {
        // and go to a second (later or equal) tie-group
        'snd_group: for stg in rank.iter().skip(i) {
            'fst_elem: for x in tg {
                'snd_elem: for y in stg {
                    // in the first iteration of 'snd_group, tg==stg, and we want all the edges
                    // between tied elements _except_ for self-loops.
                    if x == y {
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
pub fn edge_cmp(
    a: &&petgraph::graph::Edge<(Element, Element)>,
    b: &&petgraph::graph::Edge<(Element, Element)>,
) -> Ordering {
    match a.weight.0.cmp(&b.weight.0) {
        Ordering::Equal => a.weight.1.cmp(&b.weight.1),
        x => x,
    }
}
