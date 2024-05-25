use std::collections::HashSet;

use rand::Rng;
use shuffle::{fy::FisherYates, shuffler::Shuffler};

const N_RETRIES_FIND_EDGE: usize = 200;
const N_COMPLETE_RETRIES: usize = 20;

use crate::{graphs::*, utils::order_pair};

pub trait DegreeDistribution {
    fn sample(&self, rng: &mut rand::rngs::ThreadRng) -> usize;
    fn prob(&self, degree: usize) -> f64;
}

#[derive(Debug)]
pub struct DiscreteDegreeDistriubtion {
    // index is degree, value is probability
    distrib: Vec<f64>,
}

impl DiscreteDegreeDistriubtion {
    pub fn from_unnormalized(distrib: Vec<f64>) -> Self {
        assert!(!distrib.is_empty(), "degree distribution must be non-empty");
        assert!(
            distrib.iter().all(|x| *x >= 0.0),
            "probabilities must be non-negative"
        );
        let sum: f64 = distrib.iter().sum();
        assert!(sum > 0.0, "sum of probabilities must be positive");
        let distrib = distrib.iter().map(|x| x / sum).collect();
        Self { distrib }
    }
}

impl DegreeDistribution for DiscreteDegreeDistriubtion {
    fn sample(&self, rng: &mut rand::rngs::ThreadRng) -> usize {
        let r = rng.gen_range(0.0..1.0);
        let mut sum = 0.0;
        for (i, p) in self.distrib.iter().enumerate() {
            sum += p;
            if r < sum {
                return i;
            }
        }
        self.distrib.len() - 1
    }

    fn prob(&self, degree: usize) -> f64 {
        self.distrib[degree]
    }
}

fn validate_edge(edges: &HashSet<(usize, usize)>, u: usize, v: usize) -> bool {
    u != v && !edges.contains(&order_pair((u, v)))
}

pub fn generate_configuration_model<T>(n: usize, degree_distribution: &T) -> Option<GraphEdgeList>
where
    T: DegreeDistribution,
{
    let mut rng = rand::thread_rng();
    for _ in 0..N_COMPLETE_RETRIES {
        if let Some(edges) = generate_configuration_model_inner(n, degree_distribution, &mut rng) {
            return Some(edges);
        }
    }
    eprintln!(
        "coult not generate configuration model after {} retries",
        N_COMPLETE_RETRIES
    );
    None
}

fn generate_configuration_model_inner<T>(
    n: usize,
    degree_distribution: &T,
    rng: &mut rand::rngs::ThreadRng,
) -> Option<GraphEdgeList>
where
    T: DegreeDistribution,
{
    let mut degrees = (0..n)
        .map(|_| degree_distribution.sample(rng))
        .collect::<Vec<_>>();

    // regenerate any degree such that the sum is even
    let mut n_stubs = degrees.iter().sum::<usize>();
    if n_stubs % 2 != 0 {
        let degree_0 = degrees[0];
        let mut gen_degree = degree_distribution.sample(rng);
        while (degree_0 + gen_degree) % 2 == 0 {
            gen_degree = degree_distribution.sample(rng);
        }
        n_stubs = n_stubs - degree_0 + gen_degree;
        degrees[0] = gen_degree;
    }
    debug_assert!(degrees.iter().sum::<usize>() % 2 == 0);
    let n_edges = n_stubs / 2;
    let mut stubs = Vec::with_capacity(n_stubs);
    for (i, &d) in degrees.iter().enumerate() {
        for _ in 0..d {
            stubs.push(i);
        }
    }
    debug_assert!(stubs.len() == n_stubs);
    let mut fy = FisherYates::default();
    fy.shuffle(&mut stubs, rng).expect("shuffle failed");

    // let mut edges = Vec::with_capacity(n_edges);
    let mut edges = HashSet::with_capacity(n_edges);
    for i in 0..n_edges {
        let idx_stub = i * 2;
        let u = stubs[idx_stub];
        // try a simple "forward" correction (by choosing another stub)
        if !validate_edge(&edges, u, stubs[idx_stub + 1]) {
            let mut fixed = false;
            // try forward fix
            if i != n_edges - 1 {
                for _ in 0..N_RETRIES_FIND_EDGE {
                    let new_stub = rng.gen_range((idx_stub + 2)..n_stubs);
                    if validate_edge(&edges, u, stubs[new_stub]) {
                        stubs.swap(idx_stub + 1, new_stub);
                        debug_assert!(stubs[idx_stub] != stubs[idx_stub + 1]);
                        fixed = true;
                        break;
                    }
                }
            }
            if !fixed {
                // try to rewire existing edge
                for _ in 0..N_RETRIES_FIND_EDGE {
                    let backwards_stub = rng.gen_range(0..idx_stub);
                    let backwards_edge_other_stub = if backwards_stub % 2 == 0 {
                        backwards_stub + 1
                    } else {
                        backwards_stub - 1
                    };
                    // both the new edge and the old edge that has been changed
                    // must be valid
                    if validate_edge(&edges, u, stubs[backwards_stub])
                        && validate_edge(
                            &edges,
                            stubs[backwards_edge_other_stub],
                            stubs[idx_stub + 1],
                        )
                    {
                        edges.remove(&order_pair((
                            stubs[backwards_stub],
                            stubs[backwards_edge_other_stub],
                        )));
                        stubs.swap(backwards_stub, idx_stub + 1);
                        edges.insert(order_pair((u, stubs[idx_stub + 1])));
                        edges.insert(order_pair((
                            stubs[backwards_stub],
                            stubs[backwards_edge_other_stub],
                        )));
                        fixed = true;
                        break;
                    }
                }
            }
            if !fixed {
                eprintln!("could not fix edge");
                return None;
            }
        }
        edges.insert(order_pair((u, stubs[idx_stub + 1])));
    }
    let edges: Vec<_> = edges.into_iter().collect();
    debug_assert!(edges.len() == n_edges);
    Some(GraphEdgeList { edges })
}
