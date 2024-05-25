use std::fs::File;
use std::io::Write;
use std::path::Path;

use sir_sim::GraphAdjacencyList;
use sir_sim::GraphEdgeList;
use sir_sim::SIRGraph;

use threadpool::ThreadPool;
use threadpool_scope::scope_with;

const N_THREADS: usize = 10;

const VERBOSE: bool = false;

fn conv_plot(in_file: &str, out_file: &str, mu: f64, beta: f64, init_prop: f64) {
    let mut file =
        File::open(in_file).unwrap_or_else(|_| panic!("Failed to open file {}", in_file));
    let g = GraphEdgeList::read_pajek(&mut file)
        .unwrap_or_else(|_| panic!("Failed to read pajek file {}", in_file))
        .into();
    let mut g = SIRGraph::from_graph(&g, mu, beta, init_prop);

    let mut file = File::create(out_file).expect("Failed to create file");
    writeln!(file, "susceptible, infected, recovered").unwrap();
    writeln!(
        file,
        "{},{},{}",
        g.n_susceptible(),
        g.n_infected(),
        g.n_recovered()
    )
    .unwrap();
    for _ in 0..100 {
        g.step();
        writeln!(
            file,
            "{},{},{}",
            g.n_susceptible(),
            g.n_infected(),
            g.n_recovered()
        )
        .unwrap();
    }
}

fn beta_sweep(in_file: &str, out_file: &str, n_beta: usize, beta_min: f64, beta_max: f64, mu: f64) {
    let mut file =
        File::open(in_file).unwrap_or_else(|_| panic!("Failed to open file {}", in_file));
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

struct SIRSimProps<'a> {
    beta: f64,
    mu: f64,
    init_prop: f64,
    n_sim: usize,
    in_dir: &'a Path,
    out_dir: &'a Path,
    do_sd: bool,
    max_contacts: usize,
}

fn sir_expr(sir_sim: SIRSimProps) {
    // Runs a sir simulation for all graphs in the data folder (in parallel)
    // For every graph, the simulation is run n_sim times and maximum number of infected, the time after the peak is reached and the final number of susceptible and the total time till eradication is recoreded

    let pool = ThreadPool::new(N_THREADS);
    std::fs::create_dir_all(sir_sim.out_dir).unwrap();

    scope_with(&pool, |scope| {
        for entry in sir_sim.in_dir.read_dir().unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            scope.execute(move || {
                let mut file = File::open(&path).unwrap();
                let g = GraphEdgeList::read_pajek(&mut file).unwrap().into();
                let mut g = if sir_sim.do_sd {
                    SIRGraph::from_graph_with_sd(
                        &g,
                        sir_sim.mu,
                        sir_sim.beta,
                        sir_sim.init_prop,
                        sir_sim.max_contacts,
                    )
                } else {
                    SIRGraph::from_graph(&g, sir_sim.mu, sir_sim.beta, sir_sim.init_prop)
                };

                let mut results = Vec::new();

                for _ in 0..sir_sim.n_sim {
                    g.reset(sir_sim.beta, sir_sim.mu);
                    let mut max_infected = 0;
                    let mut time_peak = 0;
                    let mut time_eradication = 0;
                    let mut n_susceptible_eradication = 0;
                    for i in 0.. {
                        g.step();
                        let n_infected = g.n_infected();
                        if n_infected > max_infected {
                            max_infected = n_infected;
                            time_peak = i;
                        }
                        if n_infected == 0 {
                            time_eradication = i;
                            n_susceptible_eradication = g.n_susceptible();
                            break;
                        }
                    }
                    results.push((
                        max_infected,
                        time_peak,
                        time_eradication,
                        n_susceptible_eradication,
                    ));
                }

                let out_file = sir_sim.out_dir.join(format!(
                    "{}.csv",
                    path.file_stem().unwrap().to_str().unwrap()
                ));
                let mut file = File::create(out_file).expect("Failed to create file");
                writeln!(
                    file,
                    "max_infected,peak_time,eradication_time,n_susceptible_eradication"
                )
                .unwrap();
                for (peak, peak_time, eradication_time, n_susceptible_eradication) in results {
                    writeln!(
                        file,
                        "{},{},{},{}",
                        peak, peak_time, eradication_time, n_susceptible_eradication
                    )
                    .unwrap();
                }
            });
        }
    });
}

fn main() {
    let n_beta = 51;
    let beta_min = 0.0;
    let beta_max = 1.0;

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        panic!("Usage: sir_sim <program> [args]");
    }
    let program = args[1].as_str();

    if program == "beta" {
        if args.len() < 5 {
            panic!("Usage: sir_sim beta <in_file> <out_file> <n_beta> <mu>");
        }
        let in_file = &args[2];
        let out_file = &args[3];
        let mu = args[4].parse::<f64>().unwrap();
        beta_sweep(in_file, out_file, n_beta, beta_min, beta_max, mu);
    } else if program == "conv_plot" {
        if args.len() < 7 {
            panic!("Usage: sir_sim conv_plot <in_file> <out_file> <beta> <mu> <init_prop>");
        }
        let in_file = &args[2];
        let out_file = &args[3];
        let beta = args[4].parse::<f64>().unwrap();
        let mu = args[5].parse::<f64>().unwrap();
        let init_prop = args[6].parse::<f64>().unwrap();
        conv_plot(in_file, out_file, mu, beta, init_prop);
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
    } else if program == "sir_expr" {
        if args.len() < 8 {
            panic!("Usage: sir_sim sir_expr <in_dir> <out_dir> <beta> <mu> <init_prop> <n_sim>");
        }
        let in_dir = Path::new(&args[2]);
        let out_dir = Path::new(&args[3]);
        let beta = args[4].parse::<f64>().unwrap();
        let mu = args[5].parse::<f64>().unwrap();
        let init_prop = args[6].parse::<f64>().unwrap();
        let n_sim = args[7].parse::<usize>().unwrap();
        let sir_sim = SIRSimProps {
            beta,
            mu,
            init_prop,
            n_sim,
            in_dir,
            out_dir,
            do_sd: false,
            max_contacts: 0,
        };
        sir_expr(sir_sim);
    } else if program == "sir_expr_sd" {
        if args.len() < 9 {
            panic!("Usage: sir_sim sir_expr <in_dir> <out_dir> <beta> <mu> <init_prop> <n_sim>");
        }
        let in_dir = Path::new(&args[2]);
        let out_dir = Path::new(&args[3]);
        let beta = args[4].parse::<f64>().unwrap();
        let mu = args[5].parse::<f64>().unwrap();
        let init_prop = args[6].parse::<f64>().unwrap();
        let n_sim = args[7].parse::<usize>().unwrap();
        let max_contacts = args[8].parse::<usize>().unwrap();
        let sir_sim = SIRSimProps {
            beta,
            mu,
            init_prop,
            n_sim,
            in_dir,
            out_dir,
            do_sd: true,
            max_contacts,
        };
        sir_expr(sir_sim);
    } else {
        panic!("Unknown program {}", program);
    }
}
