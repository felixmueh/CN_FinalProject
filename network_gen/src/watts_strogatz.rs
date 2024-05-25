use rand::Rng;

use crate::{
    graphs::*,
    utils::{generate_ring_graph, order_pair},
};
use std::collections::HashSet;

pub fn generate_watts_strogatz(n: usize, k: usize, p: f64) -> Option<GraphEdgeList> {
    if k % 2 != 0 {
        eprintln!("k should be even");
        return None;
    }
    if !(0.0..=1.0).contains(&p) {
        eprintln!("p should be in the range [0, 1]");
        return None;
    }
    if n <= k {
        eprintln!("n should be greater than k");
        return None;
    }
    if (k as f64) <= (n as f64).ln() {
        eprintln!(
            "k should be greater than ln(N)=ln({})={}",
            n,
            (n as f64).ln()
        );
        return None;
    }
    let mut ring_graph = generate_ring_graph(n, k / 2)?;
    let mut edges_set: HashSet<(usize, usize)> =
        ring_graph.edges.iter().map(|p| order_pair(*p)).collect();
    for edge in ring_graph.edges.iter_mut() {
        if rand::random::<f64>() > p {
            continue;
        }
        // This essentially allows the same edge to be picked again.
        edges_set.remove(&order_pair(*edge));
        let new_idx = loop {
            let new_idx = rand::thread_rng().gen_range(0..n);
            if new_idx == edge.0 || edges_set.contains(&order_pair((edge.0, new_idx))) {
                continue;
            }
            break new_idx;
        };
        edges_set.insert(order_pair((edge.0, new_idx)));
        edge.1 = new_idx;
    }

    Some(ring_graph)
}
