use std::io::Write;

pub trait Graph {
    fn n_nodes(&self) -> usize;
    fn write_to_file(&self, file: &mut std::fs::File) -> std::io::Result<()>;
}

pub struct GraphAdjacencyList {
    pub nodes: Vec<Vec<usize>>,
}

pub struct GraphEdgeList {
    pub edges: Vec<(usize, usize)>,
}

impl Graph for GraphEdgeList {
    fn n_nodes(&self) -> usize {
        self.edges.iter().map(|(a, b)| *a.max(b)).max().unwrap() + 1
    }

    fn write_to_file(&self, file: &mut std::fs::File) -> std::io::Result<()> {
        writeln!(file, "*Vertices {}", self.n_nodes())?;
        for i in 0..self.n_nodes() {
            writeln!(file, "{} \"{}\"", i, i)?;
        }
        writeln!(file, "*Edges")?;
        for (a, b) in &self.edges {
            writeln!(file, "{} {}", a, b)?;
        }
        Ok(())
    }
}

impl From<GraphAdjacencyList> for GraphEdgeList {
    fn from(val: GraphAdjacencyList) -> Self {
        let mut edges = Vec::new();
        for (i, node) in val.nodes.iter().enumerate() {
            for &j in node {
                edges.push((i, j));
            }
        }
        GraphEdgeList { edges }
    }
}

impl From<GraphEdgeList> for GraphAdjacencyList {
    fn from(val: GraphEdgeList) -> Self {
        let max_node = val.edges.iter().map(|(a, b)| *a.max(b)).max().unwrap();
        let mut nodes = vec![Vec::new(); max_node + 1];
        for (a, b) in val.edges {
            nodes[a].push(b);
        }
        GraphAdjacencyList { nodes }
    }
}
