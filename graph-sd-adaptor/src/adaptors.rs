use rand::seq::SliceRandom;

use crate::{Graph, GraphAdjacencyList, GraphEdgeList};

pub trait Adaptor
where
    Self: Sync + Send,
{
    fn adapt(&self, graph: GraphEdgeList) -> GraphEdgeList;
}

pub struct NoOpAdaptor;

impl Adaptor for NoOpAdaptor {
    fn adapt(&self, graph: GraphEdgeList) -> GraphEdgeList {
        graph
    }
}

pub struct MaxDegreeAdaptor {
    pub max_degree: usize,
}

impl Adaptor for MaxDegreeAdaptor {
    fn adapt(&self, graph: GraphEdgeList) -> GraphEdgeList {
        let mut n_neighbors = vec![0; graph.n_nodes()];
        let mut edges = graph.edges;
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
