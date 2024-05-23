use std::fs::File;
use std::io::Write;

use sir_sim::GraphAdjacencyList;
use sir_sim::GraphEdgeList;
use sir_sim::SIRGraph;

const VERBOSE: bool = false;

fn conv_plot(in_file: &str, out_file: &str, mu: f64, beta: f64) {
    let mut file = File::open(in_file).expect(format!("Failed to open file {}", in_file).as_str());
    let g = GraphEdgeList::read_pajek(&mut file).unwrap().into();
    let mut g = SIRGraph::from_graph(&g, mu, beta, 0.2);

    let mut results = Vec::new();
    for _ in 0..100 {
        g.step();
        results.push(g.prop_infected());
    }
    let mut file = File::create(out_file).expect("Failed to create file");
    for prop in results.iter() {
        writeln!(file, "{}", prop).unwrap();
    }
}

fn beta_sweep(in_file: &str, out_file: &str, n_beta: usize, beta_min: f64, beta_max: f64, mu: f64) {
    let mut file = File::open(in_file).expect(format!("Failed to open file {}", in_file).as_str());
    let g: GraphAdjacencyList = GraphEdgeList::read_pajek(&mut file).unwrap().into();
    let mut g = SIRGraph::from_graph(&g, 0.0, 0.0, 0.2);

    let max_sim = 5000;
    let window = 10;
    let eps = 0.001; // average required value of the derivative to stop simulation

    let mut results = Vec::new();
    let mut values: Vec<f64> = vec![0.0; window];
    let mut window_pnt = 0;

    for i in 0..n_beta {
        let beta = beta_min + (beta_max - beta_min) * (i as f64) / (n_beta as f64);
        g.reset(beta, mu);

        for i in 0.. {
            g.step();

            values[window_pnt] = g.prop_infected();
            window_pnt = (window_pnt + 1) % window;
            if i >= window {
                let current_value = values[window_pnt];
                let most_dist_value = values[(window_pnt - 1 + window) % window];
                let diff = current_value - most_dist_value;
                if diff.abs() < eps {
                    if VERBOSE {
                        println!("Converged after {} runs to {}", i, values[window_pnt]);
                    }
                    break;
                }
            }
            if i >= max_sim {
                if VERBOSE {
                    println!(
                        "Max number of simulations reached, last change: {}, converged to {}",
                        (values[window_pnt] - values[(window_pnt - 1 + window) % window]).abs(),
                        values[window_pnt]
                    );
                }
                break;
            }
        }
        results.push((beta, g.prop_infected()));
    }
    let mut file = File::create(out_file).expect("Failed to create file");
    for (beta, prop) in results {
        writeln!(file, "{},{}", beta, prop).unwrap();
    }
}

fn main() {
    let n_beta = 51;
    let beta_min = 0.0;
    let beta_max = 1.0;

    let args: Vec<String> = std::env::args().collect();
    let program = args[1].as_str();
    let in_file = &args[2];
    let out_file = &args[3];
    let mu = args[4].parse::<f64>().unwrap();

    if program == "beta" {
        beta_sweep(in_file, out_file, n_beta, beta_min, beta_max, mu);
    } else if program == "conv_plot" {
        let beta = args[5].parse::<f64>().unwrap();
        conv_plot(in_file, out_file, mu, beta);
    } else if program == "full" {
        let n = 1000;
        let mut full_graph_nodes = Vec::new();
        for i in 0..n {
            for j in 0..n {
                if i != j {
                    full_graph_nodes.push((i, j));
                }
            }
        }
        let graph = GraphEdgeList {
            edges: full_graph_nodes,
        };
        let graph_adj_list: GraphAdjacencyList = graph.into();
        let mut g = SIRGraph::from_graph(&graph_adj_list, 0.1, 0.9, 0.2);
        let mut results = Vec::new();
        for _ in 0..5000 {
            g.step();
            results.push(g.prop_infected());
        }
        let mut file = File::create("out.csv").expect("Failed to create file");
        for prop in results.iter() {
            writeln!(file, "{}", prop).unwrap();
        }
    } else {
        panic!("Unknown program {}", program);
    }
}
