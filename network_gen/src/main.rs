use network_gen::{
    generate_configuration_model, generate_watts_strogatz, DegreeDistribution,
    DiscreteDegreeDistriubtion, Graph,
};
use rand::Rng;
use std::fs::File;

struct PoissionDegreeDistrib {
    lambda: f64,
    l: f64,
}

impl PoissionDegreeDistrib {
    fn new(lambda: f64) -> Self {
        Self {
            lambda,
            l: (-lambda).exp(),
        }
    }
}

fn generate_power_law_degree_distrib(
    alpha: f64,
    k_min: usize,
    size: usize,
) -> DiscreteDegreeDistriubtion {
    // It's important to not make the tail too long, otherwise a lot of the probability mass will be there.
    // Cutoff prop controls the cutoff point of the power law distribution.
    let mut p = vec![0.0; size];
    for k in k_min..size {
        p[k] = (k as f64).powf(-alpha);
    }
    DiscreteDegreeDistriubtion::from_unnormalized(p)
}

impl DegreeDistribution for PoissionDegreeDistrib {
    fn sample(&self, rng: &mut rand::rngs::ThreadRng) -> usize {
        assert!(self.lambda <= 30.0, "lambda is too large");
        let mut k = 0;
        let mut p = 1.0;
        loop {
            k += 1;
            let u: f64 = rng.gen();
            p *= u;
            if p < self.l {
                break k - 1;
            }
        }
    }
    fn prob(&self, k: usize) -> f64 {
        assert!(self.lambda <= 30.0, "lambda is too large");
        self.l * self.lambda.powi(k as i32) / (1..=k).product::<usize>() as f64
    }
}

#[allow(dead_code)]
fn write_distrib_to_file<T>(file: &mut File, distrib: &T) -> std::io::Result<()>
where
    T: DegreeDistribution,
{
    let from_writer = csv::Writer::from_writer(file);
    let mut writer = from_writer;
    for k in 0.. {
        let prob = distrib.prob(k);
        if prob <= 0.001 {
            break;
        }
        writer.write_record(&[k.to_string(), prob.to_string()])?;
    }
    writer.flush()?;
    Ok(())
}

fn main() -> std::io::Result<()> {
    let args_cl: Vec<String> = std::env::args().collect();
    let mut args = args_cl.iter().skip(1);

    let graph = args.next().expect("No graph type specified");

    if graph == "WS" {
        // expected parameters: -s <size> -k <k> -p <p>

        let mut size = None;
        let mut k = None;
        let mut p = None;
        let mut output = None;

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-n" => {
                    size = Some(args.next().expect("No size specified").parse().unwrap());
                }
                "-k" => {
                    k = Some(args.next().expect("No k specified").parse().unwrap());
                }
                "-p" => {
                    p = Some(args.next().expect("No p specified").parse().unwrap());
                }
                "-o" => {
                    output = Some(args.next().expect("No output file specified"));
                }
                _ => {
                    eprintln!("Unknown argument: {}", arg);
                    return Ok(());
                }
            }
        }
        if size.is_none() || k.is_none() || p.is_none() || output.is_none() {
            panic!("Missing arguments");
        }
        let graph = generate_watts_strogatz(size.unwrap(), k.unwrap(), p.unwrap());
        if let Some(graph) = graph {
            let mut file = File::create(output.unwrap())?;
            graph.write_to_file(&mut file)?;
        } else {
            panic!("Failed to generate graph");
        }
    } else if graph == "CM-Poisson" {
        // expected parameters: -lambda <lambda> -n <size> -o <filepath>

        let mut lambda = None;
        let mut size = None;
        let mut output = None;

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-lambda" => {
                    lambda = Some(args.next().expect("No lambda specified").parse().unwrap());
                }
                "-n" => {
                    size = Some(args.next().expect("No size specified").parse().unwrap());
                }
                "-o" => {
                    output = Some(args.next().expect("No output file specified"));
                }
                _ => {
                    eprintln!("Unknown argument: {}", arg);
                    return Ok(());
                }
            }
        }
        if lambda.is_none() || size.is_none() || output.is_none() {
            panic!("Missing arguments");
        }
        let graph = generate_configuration_model(
            size.unwrap(),
            &PoissionDegreeDistrib::new(lambda.unwrap()),
        );
        if let Some(graph) = graph {
            let mut file = File::create(output.unwrap())?;
            graph.write_to_file(&mut file)?;
        } else {
            panic!("Failed to generate graph");
        }
    } else if graph == "CM-Powerlaw" {
        // expected parameters: -gamma <alpha> -k_min <k_min> -n <size> -o <filepath>

        let mut gamma = None;
        let mut k_min = None;
        let mut size = None;
        let mut output = None;

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-gamma" => {
                    gamma = Some(args.next().expect("No gamma specified").parse().unwrap());
                }
                "-k-min" => {
                    k_min = Some(args.next().expect("No k_min specified").parse().unwrap());
                }
                "-n" => {
                    size = Some(args.next().expect("No size specified").parse().unwrap());
                }
                "-o" => {
                    output = Some(args.next().expect("No output file specified"));
                }
                _ => {
                    eprintln!("Unknown argument: {}", arg);
                    return Ok(());
                }
            }
        }
        if gamma.is_none() || k_min.is_none() || size.is_none() || output.is_none() {
            panic!("Missing arguments");
        }
        let graph = generate_configuration_model(
            size.unwrap(),
            &generate_power_law_degree_distrib(gamma.unwrap(), k_min.unwrap(), size.unwrap()),
        );
        if let Some(graph) = graph {
            let mut file = File::create(output.unwrap())?;
            graph.write_to_file(&mut file)?;
        } else {
            panic!("Failed to generate graph");
        }
    } else {
        panic!("Unknown graph type: {}", graph);
    }

    Ok(())
}
