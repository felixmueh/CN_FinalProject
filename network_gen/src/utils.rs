use crate::GraphEdgeList;

pub fn order_pair(p: (usize, usize)) -> (usize, usize) {
    if p.0 < p.1 {
        p
    } else {
        (p.1, p.0)
    }
}

pub fn generate_ring_graph(n: usize, r: usize) -> Option<GraphEdgeList> {
    if r > n / 2 {
        eprintln!("r must be less than or equal to n / 2 to generate ring graph");
        return None;
    }

    let mut edges: Vec<(usize, usize)> = Vec::with_capacity(n * r);
    for i in 0..n {
        for offset in 1..(r + 1) {
            edges.push((i, (i + offset) % n));
        }
    }

    Some(GraphEdgeList { edges })
}
