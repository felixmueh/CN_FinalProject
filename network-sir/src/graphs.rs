use std::io::{BufRead, Write};

pub trait Graph {
    fn n_nodes(&self) -> usize;
    fn write_to_file(&self, file: &mut std::fs::File) -> std::io::Result<()>;
}

#[derive(Debug)]
pub struct GraphAdjacencyList {
    pub nodes: Vec<Vec<usize>>,
}

#[derive(Debug)]
pub struct GraphEdgeList {
    // undirected graph
    pub edges: Vec<(usize, usize)>,
}

impl GraphEdgeList {
    pub fn read_pajek(file: &mut std::fs::File) -> std::io::Result<Self> {
        let mut lines = std::io::BufReader::new(file).lines();
        let mut edges = Vec::new();
        while let Some(Ok(line)) = lines.next() {
            if line.starts_with("*Vertices") {
            } else if line.starts_with("*Edges") {
                break;
            }
        }
        for line in lines {
            let line = line?;
            let mut parts = line.split_whitespace();
            let a = parts.next().unwrap().parse().unwrap();
            let b = parts.next().unwrap().parse().unwrap();
            edges.push((a, b));
        }
        Ok(GraphEdgeList { edges })
    }
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

impl Graph for GraphAdjacencyList {
    fn n_nodes(&self) -> usize {
        self.nodes.len()
    }

    fn write_to_file(&self, file: &mut std::fs::File) -> std::io::Result<()> {
        // NOTE: Not tested
        writeln!(file, "*Vertices {}", self.n_nodes())?;
        for i in 0..self.n_nodes() {
            writeln!(file, "{} \"{}\"", i, i)?;
        }
        writeln!(file, "*Edges")?;
        for (i, node) in self.nodes.iter().enumerate() {
            for &j in node {
                writeln!(file, "{} {}", i, j)?;
            }
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
        if val.edges.is_empty() {
            return GraphAdjacencyList { nodes: Vec::new() };
        }
        let max_node = val.edges.iter().map(|(a, b)| *a.max(b)).max().unwrap();
        let mut nodes = vec![Vec::new(); max_node + 1];
        for (a, b) in val.edges {
            nodes[a].push(b);
            nodes[b].push(a);
        }
        GraphAdjacencyList { nodes }
    }
}
