#!/usr/bin/env python3

from pathlib import Path
import os
import sys

from gen_graphs import gen_graphs
from analyze_sir_results import analyze_sir_results

this_dir = Path(__file__).resolve().parent
exec_path_adapt = Path(__file__).resolve().parent / "graph-sd-adaptor/target/release/graph-sd-adaptor"
exec_path_sim = Path(__file__).resolve().parent / "network-sir/target/release/sir-sim"

# main directory that contains graphs
# for every type of graph there is a subdirectory
GRAPHS_DIR = Path("graphs")
RESULTS_DIR = Path("results")


def build_path_adaptor_lib():
    print("Runner: building graph-sd-adaptor...")
    current_dir = os.getcwd()
    os.chdir(this_dir / "graph-sd-adaptor")
    os.system("cargo build --release")
    os.chdir(current_dir)


def build_sir_sim_lib():
    print("Runner: building sir-sim...")
    current_dir = os.getcwd()
    os.chdir(this_dir / "network-sir")
    os.system("cargo build --release")
    os.chdir(current_dir)


def call_graph_sd_adaptor(in_file, out_file, *args):
    print(f"Runner: calling graph-sd-adaptor with args: {args}")
    os.system(f"{exec_path_adapt} {in_file} {out_file} {' '.join(map(str, args))}")


def call_sir_sim(*args):
    print(f"Runner: calling sir-sim with args: {args}")
    os.system(f"{exec_path_sim} {' '.join(map(str, args))}")


adaptations = [
    # postfix, args
    ("_max_degree_3", ["max_degree", 3]),
    ("_max_degree_5", ["max_degree", 5]),
    ("_max_degree_10", ["max_degree", 10]),
]

sd_sims = [
    3, 5, 10
]

def get_sd_sim_results_suffix(sd):
    return f"_sd_max_contacts_{sd}"


def make_entry(result, column_size=35):
    # name:  value +- std (std of mean)
    mean, std, std_of_mean = result
    basic_str = f"{mean:.2f} +- {std:.2f} ({std_of_mean:.2f})"
    return basic_str + " " * (column_size - len(basic_str))


def make_row(analysis_result, name):
    return f"{name:<35}{make_entry(analysis_result['peak'])} {make_entry(analysis_result['peak_time'])} {make_entry(analysis_result['t_end'])} {make_entry(analysis_result['n_sus_end'])}"


# TODO: clean existing directories?
if __name__ == "__main__":
    if len(sys.argv) < 2:
        print(f"Usage: {sys.argv[0]} [commands]")
        sys.exit(1)
    for command in sys.argv[1:]:
        if command == "gen":
            gen_graphs(GRAPHS_DIR)
        if command == "adapt":
            build_path_adaptor_lib()
            for graph_dir in os.listdir(GRAPHS_DIR):
                if not os.path.isdir(GRAPHS_DIR / graph_dir):
                    continue
                for suffix, args in adaptations:
                    new_dir = Path(str(GRAPHS_DIR) + suffix) / (graph_dir + suffix)
                    if os.path.exists(new_dir):
                        print(f"Runner: {new_dir} already exists, skipping...")
                        continue
                    call_graph_sd_adaptor(GRAPHS_DIR / graph_dir, new_dir, *args)
        if command == "sim_all":
            build_sir_sim_lib()
            mu = 0.1
            beta = 0.1
            prop_init = 0.05
            for graph_dir in os.listdir(GRAPHS_DIR):
                if not os.path.isdir(GRAPHS_DIR / graph_dir):
                    continue
                results_dir_graph = f"{RESULTS_DIR}/basic/{graph_dir}_mu_{mu}_beta_{beta}_prop_init_{prop_init}"
                if os.path.exists(results_dir_graph):
                    print(f"Runner: {results_dir_graph} already exists, skipping...")
                else:
                    call_sir_sim("sir_expr", GRAPHS_DIR / graph_dir, results_dir_graph, beta, mu, prop_init, 20)

                for suffix, _ in adaptations:
                    # don't add suffix to graph to allow matching of graph names
                    new_dir = Path(str(GRAPHS_DIR) + suffix) / (graph_dir + suffix)
                    results_dir_graph = f"{RESULTS_DIR}/{suffix}/{graph_dir}_mu_{mu}_beta_{beta}_prop_init_{prop_init}"
                    if os.path.exists(results_dir_graph):
                        print(f"Runner: {results_dir_graph} already exists, skipping...")
                        continue
                    call_sir_sim("sir_expr", new_dir, results_dir_graph, beta, mu, prop_init, 20)
                for sd in sd_sims:
                    suffix = get_sd_sim_results_suffix(sd)
                    results_dir_graph = f"{RESULTS_DIR}/{suffix}/{graph_dir}_mu_{mu}_beta_{beta}_prop_init_{prop_init}"
                    if os.path.exists(results_dir_graph):
                        print(f"Runner: {results_dir_graph} already exists, skipping...")
                        continue
                    call_sir_sim("sir_expr_sd", new_dir, results_dir_graph, beta, mu, prop_init, 20, sd)

        if command == "analyze":
            # for every graph in basic result get metrics and report these together with the adapted graphs
            with open("results_summary.txt", "w") as f:
                for graph_dir in os.listdir(RESULTS_DIR / "basic"):
                    if not os.path.isdir(RESULTS_DIR / "basic" / graph_dir):
                        continue
                    print(f"Runner: analyzing {graph_dir}")
                    results_basic = analyze_sir_results(RESULTS_DIR / "basic" / graph_dir, verbose=False)
                    results_adapted = []
                    for suffix, _ in adaptations:
                        new_dir = Path(suffix) / graph_dir
                        print(f"Runner: analyzing {new_dir}")
                        results = analyze_sir_results(RESULTS_DIR / new_dir, verbose=False)
                        results_adapted.append((suffix, results))
                    results_sd_sims = []
                    for sd in sd_sims:
                        suffix = get_sd_sim_results_suffix(sd)
                        new_dir = Path(suffix) / graph_dir
                        print(f"Runner: analyzing {new_dir}")
                        results = analyze_sir_results(RESULTS_DIR / new_dir, verbose=False)
                        results_sd_sims.append((suffix, results))
                    f.write(f"Results for {graph_dir}:\n")
                    # write in columns for basic and then every adaptation:
                    # adaptor:    peak += std (std of mean)    peak_time += std (std of mean)    t_end += std (std of mean)    n_sus_end += std (std of mean)
                    column_size = 35
                    f.write(f"{'':<35}{'Peak':<35}{'Peak Time':<35}{'T End':<35}{'N Sus End':<35}\n")
                    f.write(make_row(results_basic, "basic") + "\n")
                    for suffix, results in results_adapted + results_sd_sims:
                        f.write(make_row(results, suffix) + "\n")
                    f.write("\n")
