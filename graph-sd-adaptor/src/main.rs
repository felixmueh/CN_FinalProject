use std::{fs, path::Path, sync::Arc};

use graph_sd_adaptor::{Adaptor, Graph, GraphEdgeList, NoOpAdaptor};
use threadpool::ThreadPool;

const N_THREADS: usize = 10;

fn get_adaptor(adaptor: &str, args: &[String]) -> Box<dyn Adaptor> {
    return match adaptor {
        "noop" => Box::new(NoOpAdaptor {}),
        "max_degree" => {
            if args.len() < 1 {
                panic!("Usage: graph-sd-adaptor <input-folder> <output-folder> max_degree <max-degree>");
            }
            let max_degree = args[0].parse::<usize>().expect("Invalid max degree");
            Box::new(graph_sd_adaptor::MaxDegreeAdaptor { max_degree })
        }
        _ => panic!("Unknown adaptor {}", adaptor),
    };
}

fn main() {
    // adapts all graphs in a folder using a specified adaptor

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 5 {
        println!("Usage: graph-sd-adaptor <folder/file> <input-folder> <output-folder> <adaptor> [adaptor-args]");
        return;
    }

    let command = &args[1];
    if command == "folder" {
        let input_folder = Path::new(&args[2]);
        let output_folder = Path::new(&args[3]);
        let adaptor: &str = &args[4];
        let adaptor_args = &args[5..];
        let adator = Arc::new(get_adaptor(adaptor, adaptor_args));

        let input_files = fs::read_dir(input_folder).expect("Unable to read input folder");
        let pool = ThreadPool::new(N_THREADS);

        for entry in input_files {
            let entry = entry.expect("Unable to read entry");
            let path = entry.path();
            if path.is_file() {
                let output_path = output_folder.join(path.file_name().unwrap());
                fs::create_dir_all(output_path.parent().unwrap())
                    .expect("Unable to create directory");
                let adator = adator.clone();
                pool.execute(move || {
                    let mut file = std::fs::File::open(&path).expect("Unable to open file");
                    let graph = GraphEdgeList::read_pajek(&mut file).expect("Unable to read graph");
                    let adapted_graph = adator.adapt(&graph);
                    let mut output_file =
                        std::fs::File::create(&output_path).expect("Unable to create file");
                    adapted_graph
                        .write_to_file(&mut output_file)
                        .expect("Unable to write graph");
                });
            }
        }
        pool.join();
    } else if command == "file" {
        let n_adaptions: i64 = args[2].parse().expect("Invalid number of adaptions");
        let input_file = Path::new(&args[3]);
        let output_folder = Path::new(&args[4]);
        let adaptor: &str = &args[5];
        let adaptor_args = &args[6..];
        let adator = get_adaptor(adaptor, adaptor_args);

        fs::create_dir_all(output_folder).expect("Unable to create directory");

        let mut file = std::fs::File::open(input_file).expect("Unable to open file");
        let graph = GraphEdgeList::read_pajek(&mut file).expect("Unable to read graph");
        for i in 0..n_adaptions {
            let new_file_name = format!(
                "{}-{}.net",
                input_file.file_stem().unwrap().to_str().unwrap(),
                i
            );
            let output_file = output_folder.join(new_file_name);
            let adapted_graph = adator.adapt(&graph);
            let mut output_file =
                std::fs::File::create(output_file).expect("Unable to create file");
            adapted_graph
                .write_to_file(&mut output_file)
                .expect("Unable to write graph");
        }
    } else {
        panic!("Unknown command {}", command);
    }
}
