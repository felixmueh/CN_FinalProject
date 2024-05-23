use std::{fs, path::Path, sync::Arc};

use graph_sd_adaptor::{Adaptor, Graph, GraphEdgeList, NoOpAdaptor};
use threadpool::ThreadPool;

const N_THREADS: usize = 10;

fn get_adaptor(adaptor: &str, _args: &[String]) -> Box<dyn Adaptor> {
    return match adaptor {
        "noop" => Box::new(NoOpAdaptor {}),
        _ => panic!("Unknown adaptor {}", adaptor),
    };
}

fn main() {
    // adapts all graphs in a folder using a specified adaptor

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 4 {
        println!("Usage: graph-sd-adaptor <input-folder> <output-folder> <adaptor> [adaptor-args]");
        return;
    }

    let input_folder = Path::new(&args[1]);
    let output_folder = Path::new(&args[2]);
    let adaptor: &str = &args[3];
    let adaptor_args = &args[4..];
    let adator = Arc::new(get_adaptor(adaptor, adaptor_args));

    let input_files = fs::read_dir(input_folder).expect("Unable to read input folder");
    let pool = ThreadPool::new(N_THREADS);

    for entry in input_files {
        let entry = entry.expect("Unable to read entry");
        let path = entry.path();
        if path.is_file() {
            let output_path = output_folder.join(path.file_name().unwrap());
            fs::create_dir_all(output_path.parent().unwrap()).expect("Unable to create directory");
            let adator = adator.clone();
            pool.execute(move || {
                let mut file = std::fs::File::open(&path).expect("Unable to open file");
                let graph = GraphEdgeList::read_pajek(&mut file).expect("Unable to read graph");
                let adapted_graph = adator.adapt(graph);
                let mut output_file =
                    std::fs::File::create(&output_path).expect("Unable to create file");
                adapted_graph
                    .write_to_file(&mut output_file)
                    .expect("Unable to write graph");
            });
        }
    }
    pool.join();
}
