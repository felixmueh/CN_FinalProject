use rand::{seq::SliceRandom, Rng};

use crate::{Graph, GraphEdgeList};

pub trait Adaptor
where
    Self: Sync + Send,
{
    fn adapt(&self, graph: &GraphEdgeList) -> GraphEdgeList;
}

pub struct NoOpAdaptor;

impl Adaptor for NoOpAdaptor {
    fn adapt(&self, graph: &GraphEdgeList) -> GraphEdgeList {
        GraphEdgeList {
            edges: graph.edges.clone(),
        }
    }
}

pub struct MaxDegreeAdaptor {
    pub max_degree: usize,
}

impl Adaptor for MaxDegreeAdaptor {
    fn adapt(&self, graph: &GraphEdgeList) -> GraphEdgeList {
        let mut n_neighbors = vec![0; graph.n_nodes()];
        let mut edges = graph.edges.clone();
        let edges_len = edges.len();
        edges.shuffle(&mut rand::thread_rng());
        let mut new_edges = Vec::new();
        for edge in edges {
            if n_neighbors[edge.0] < self.max_degree && n_neighbors[edge.1] < self.max_degree {
                n_neighbors[edge.0] += 1;
                n_neighbors[edge.1] += 1;
                new_edges.push(edge);
            }
        }
        println!(
            "GreedyMaxConnetionsAdaptor: removed {} edges",
            edges_len - new_edges.len()
        );
        GraphEdgeList { edges: new_edges }
    }
}

pub struct MaxDegreeCheatAdaptor {
    pub max_degree: usize,
    pub cheat_prop: f64,
}

impl Adaptor for MaxDegreeCheatAdaptor {
    fn adapt(&self, graph: &GraphEdgeList) -> GraphEdgeList {
        let mut ran = rand::thread_rng();
        let mut n_neighbors = vec![0; graph.n_nodes()];
        let mut allow_cheat = vec![false; graph.n_nodes()];
        for i in 0..graph.n_nodes() {
            if ran.gen::<f64>() < self.cheat_prop {
                allow_cheat[i] = true;
            }
        }
        let mut edges = graph.edges.clone();
        let edges_len = edges.len();
        edges.shuffle(&mut rand::thread_rng());
        let mut new_edges = Vec::new();
        for edge in edges {
            if (allow_cheat[edge.0] || n_neighbors[edge.0] < self.max_degree)
                && (allow_cheat[edge.1] || n_neighbors[edge.1] < self.max_degree)
            {
                n_neighbors[edge.0] += 1;
                n_neighbors[edge.1] += 1;
                new_edges.push(edge);
            }
        }
        println!(
            "GreedyMaxCheatConnetionsAdaptor: removed {} edges",
            edges_len - new_edges.len()
        );
        GraphEdgeList { edges: new_edges }
    }
}
