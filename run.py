#!/usr/bin/env python3

from pathlib import Path
import os
import sys

from collections import defaultdict

from gen_graphs import gen_graphs
from analyze_sir_results import analyze_sir_results

import matplotlib.pyplot as plt

import numpy as np

this_dir = Path(__file__).resolve().parent
exec_path_adapt = Path(__file__).resolve().parent / "graph-sd-adaptor/target/release/graph-sd-adaptor"
exec_path_sim = Path(__file__).resolve().parent / "network-sir/target/release/sir-sim"

# main directory that contains graphs
# for every type of graph there is a subdirectory
GRAPHS_DIR = Path("graphs")
BASIC_DIR_NAME = "basic"
RESULTS_DIR = Path("results")
PLOT_DIR = Path("plots")

CMAXs = np.arange(1, 20, 1)

ADAPTORS = [
    # (name, args)
    (lambda n: f"simple_max_degree_{n}", lambda n: ["max_degree", n])
]

mu = 0.1
beta = 0.1
prop_init = 0.05

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


def get_result_dir_name(mu, beta, prop_init):
    return f"mu-{mu}_beta-{beta}_prop_init-{prop_init}"


def get_sd_sim_results_name(sd):
    return f"sd_max_contacts_{sd}"


def make_entry(result, column_size=35):
    # name:  value +- std (std of mean)
    mean, std, std_of_mean = result
    basic_str = f"{mean:.2f} +- {std:.2f} ({std_of_mean:.2f})"
    return basic_str + " " * (column_size - len(basic_str))


def make_row(analysis_result, name):
    return f"{name:<35}{make_entry(analysis_result['peak'])} {make_entry(analysis_result['peak_time'])} {make_entry(analysis_result['t_end'])} {make_entry(analysis_result['n_sus_end'])}"


def cmd_gen():
    gen_graphs(GRAPHS_DIR / BASIC_DIR_NAME)


def cmd_adapt():
    build_path_adaptor_lib()
    for graph_dir in os.listdir(GRAPHS_DIR / BASIC_DIR_NAME):
        if not os.path.isdir(GRAPHS_DIR / BASIC_DIR_NAME / graph_dir):
            continue
        for f_name, f_args in ADAPTORS:
            for cmax in CMAXs:
                name, args = f_name(cmax), f_args(cmax)
                new_dir = GRAPHS_DIR / name / graph_dir
                if os.path.exists(new_dir):
                    print(f"Runner: {new_dir} already exists, skipping...")
                    continue
                call_graph_sd_adaptor(GRAPHS_DIR / BASIC_DIR_NAME / graph_dir, new_dir, *args)

def cmd_sim_all():
    build_sir_sim_lib()
    r_folder_name = get_result_dir_name(mu, beta, prop_init)
    # normal sim for basic graphs and all adapted graphs
    for graph_type_dir in os.listdir(GRAPHS_DIR):
        if not os.path.isdir(GRAPHS_DIR / graph_type_dir):
            continue
        for graph_dir in os.listdir(GRAPHS_DIR / graph_type_dir):
            total_graph_dir = GRAPHS_DIR / graph_type_dir / graph_dir
            if not os.path.isdir(total_graph_dir):
                continue
            results_dir_graph = RESULTS_DIR / r_folder_name / graph_type_dir / graph_dir
            if os.path.exists(results_dir_graph):
                print(f"Runner: {results_dir_graph} already exists, skipping...")
            else:
                call_sir_sim("sir_expr", total_graph_dir, results_dir_graph, beta, mu, prop_init, 20)

    # implicit sd sim for all basic graphs
    for graph_dir in os.listdir(GRAPHS_DIR / BASIC_DIR_NAME):
        if not os.path.isdir(GRAPHS_DIR / BASIC_DIR_NAME / graph_dir):
            continue
        for cmax in CMAXs:
            results_dir_graph = RESULTS_DIR / r_folder_name / get_sd_sim_results_name(cmax) / graph_dir
            total_graph_dir = GRAPHS_DIR / BASIC_DIR_NAME / graph_dir
            if os.path.exists(results_dir_graph):
                print(f"Runner: {results_dir_graph} already exists, skipping...")
                continue
            call_sir_sim("sir_expr_sd", total_graph_dir, results_dir_graph, beta, mu, prop_init, 20, cmax)


def overview_results(results):
    # with open("results_summary.txt", "w") as f:
    # for graph_dir in os.listdir(RESULTS_DIR / "basic"):
    #     if not os.path.isdir(RESULTS_DIR / "basic" / graph_dir):
    #         continue
    #     sd_sim_results = {}
    #     adapted_results = {}
    #     print(f"Runner: analyzing {graph_dir}")
    #     results_basic = analyze_sir_results(RESULTS_DIR / "basic" / graph_dir, verbose=False)
    #     results_adapted = []
    #     for name, [_, n] in adaptations:
    #         new_dir = Path(name) / graph_dir
    #         print(f"Runner: analyzing {new_dir}")
    #         results = analyze_sir_results(RESULTS_DIR / new_dir, verbose=False)
    #         results_adapted.append((name, results))
    #         adapted_results[n] = results
    #     results_sd_sims = []
    #     for cmax in sd_sims:
    #         name = get_sd_sim_results_suffix(cmax)
    #         new_dir = Path(name) / graph_dir
    #         print(f"Runner: analyzing {new_dir}")
    #         results = analyze_sir_results(RESULTS_DIR / new_dir, verbose=False)
    #         results_sd_sims.append((name, results))
    #         sd_sim_results[cmax] = results
    #     f.write(f"Results for {graph_dir}:\n")
    #     column_size = 35
    #     f.write(f"{'':<35}{'Peak':<35}{'Peak Time':<35}{'T End':<35}{'N Sus End':<35}\n")
    #     f.write(make_row(results_basic, "basic") + "\n")
    #     for name, results in results_adapted + results_sd_sims:
    #         f.write(make_row(results, name) + "\n")
    #     f.write("\n")
    pass


def make_sd_efficacy_graphs(results, plot_dir):
    # Results is mapping from (sd_type, cmax) to a dictionary of results for different graphs
    # transpose results to map form graph to (sd_type, cmax)
    os.makedirs(plot_dir, exist_ok=True)
    graph_results = defaultdict(lambda: defaultdict(dict))
    for (sd_type, cmax), graph_results_type in results.items():
        for graph, graph_result in graph_results_type.items():
            graph_results[graph][sd_type][cmax] = graph_result

    # assume we have values for all CMAXs for each graph

    def plot_metric(metric_name, graph_name, graph_results):
        fig, ax = plt.subplots()
        fig.set_size_inches(10, 6)
        ax.set_xlim(1, max(CMAXs))
        max_std_mean = max(sd_result[cmax][metric_name][2] for cmax in CMAXs for (name, sd_result) in graph_results.items() if name != "basic")
        ax.set_xlabel("max contacts")
        ax.set_ylabel(metric_name)

        for sd_type, sd_results in graph_results.items():
            if sd_type == "basic":
                continue
            ax.plot(CMAXs, [sd_results[cmax][metric_name][0] for cmax in CMAXs], label=sd_type)
            ax.fill_between(CMAXs, [sd_results[cmax][metric_name][0] - sd_results[cmax][metric_name][1] for cmax in CMAXs],
                            [sd_results[cmax][metric_name][0] + sd_results[cmax][metric_name][1] for cmax in CMAXs], alpha=0.5)
        results_basic = graph_results["basic"]
        ax.axhline(results_basic[np.inf][metric_name][0], color="black", linestyle="--", label="no_sd")
        ax.legend()
        ax.set_title(f"{graph_name} {metric_name}\n Shaded area is standard deviation\n Max std of mean: {max_std_mean:.2f}")
        fig.savefig(plot_dir / f"{graph_name}_{metric_name}.png")
        plt.close(fig)

    for graph, graph_results in graph_results.items():
        plot_metric("peak", graph, graph_results)
        plot_metric("peak_time", graph, graph_results)
        plot_metric("t_end", graph, graph_results)
        plot_metric("n_sus_end", graph, graph_results)


def cmd_analyze():
    print("Loading results ...")
    os.makedirs(PLOT_DIR, exist_ok=True)
    results_dir = RESULTS_DIR / get_result_dir_name(mu, beta, prop_init)
    results = defaultdict(dict)
    for folder in os.listdir(results_dir):
        if not os.path.isdir(results_dir / folder):
            continue
        if folder == BASIC_DIR_NAME:
            f_type = ("basic", np.inf)
        elif folder.startswith("sd_max_contacts"):
            f_type = ("sd_sim", int(folder.split("_")[-1]))
        elif folder.startswith("simple_max_degree"):
            f_type = ("simple_max_degree", int(folder.split("_")[-1]))
        else:
            f_type = (str(folder), np.inf)
        for graph in os.listdir(results_dir / folder):
            if not os.path.isdir(results_dir / folder / graph):
                continue
            results[f_type][graph] = analyze_sir_results(results_dir / folder / graph, verbose=False)
    print("Results loaded, making plots ...")
    make_sd_efficacy_graphs(results, PLOT_DIR / get_result_dir_name(mu, beta, prop_init))


# TODO: clean existing directories?
if __name__ == "__main__":
    if len(sys.argv) < 2:
        print(f"Usage: {sys.argv[0]} [commands]")
        sys.exit(1)
    for command in sys.argv[1:]:
        if command == "gen":
            cmd_gen()
        if command == "adapt":
            cmd_adapt()
        if command == "sim_all":
            cmd_sim_all()
        if command == "analyze":
            cmd_analyze()
