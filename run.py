#!/usr/bin/env python3

from pathlib import Path
import os
import sys

from collections import defaultdict
from multiprocessing import Pool

from gen_graphs import gen_graphs, gen_ws_graph
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
SINGLE_DIR = Path("graphs_single")
SINGLE_DIR_NAME = "single"
SINGLE_NO_ADAPT_DIR = Path("graphs-sd-single-no-adapt")
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
    os.system(f"{exec_path_adapt} folder {in_file} {out_file} {' '.join(map(str, args))}")

def call_graph_sd_adaptor_single(in_file, out_file, nadaptions, *args):
    print(f"Runner: calling graph-sd-adaptor with args: {args}")
    os.system(f"{exec_path_adapt} file {nadaptions} {in_file} {out_file} {' '.join(map(str, args))}")

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

def adapt_single(graph_file):
    print("Here")
    graph_file_no_ext = graph_file[:-4]
    # adapt every graph 20 times write to
    for f_name, f_args in ADAPTORS:
        for cmax in CMAXs:
            name, args = f_name(cmax), f_args(cmax)
            new_dir = GRAPHS_DIR / name / graph_file_no_ext
            if os.path.exists(new_dir):
                print(f"Runner: {new_dir} already exists, skipping...")
                continue
            os.makedirs(new_dir, exist_ok=True)
            new_folder = GRAPHS_DIR / name / graph_file_no_ext
            print(f"Runner: adapting {graph_file} to {new_folder}")
            call_graph_sd_adaptor_single(SINGLE_DIR / graph_file, new_folder, 20, *args)

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


    print("Runner: adapting single graphs")
    p = Pool(10)
    for graph_file in os.listdir(SINGLE_DIR):
        if not graph_file.endswith(".net"):
            continue
        p.apply_async(adapt_single, args=(graph_file,))
    p.close()
    p.join()

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

    # implicit sd sim for all basic graphs and all single degree graphs
    for base_graph_dir in ["basic", "single"]:
        for graph_dir in os.listdir(GRAPHS_DIR / base_graph_dir):
            if not os.path.isdir(GRAPHS_DIR / base_graph_dir / graph_dir):
                continue
            for cmax in CMAXs:
                results_dir_graph = RESULTS_DIR / r_folder_name / get_sd_sim_results_name(cmax) / graph_dir
                total_graph_dir = GRAPHS_DIR / base_graph_dir / graph_dir
                if os.path.exists(results_dir_graph):
                    print(f"Runner: {results_dir_graph} already exists, skipping...")
                    continue
                call_sir_sim("sir_expr_sd", total_graph_dir, results_dir_graph, beta, mu, prop_init, 20, cmax)


def overview_results(results):
    graph_results = defaultdict(lambda: defaultdict(dict))
    for (sd_type, cmax), graph_results_type in results.items():
        for graph, graph_result in graph_results_type.items():
            graph_results[graph][sd_type][cmax] = graph_result

    with open("results_summary.txt", "w") as f:
        f.write("Results overview:\n")
        f.write("Format is: name:  value +- std (std of mean)\n\n")
        for graph, graph_results_by_type in graph_results.items():
            f.write(f"Results for {graph}:\n")
            column_size = 35
            f.write(f"{'':<35}{'Peak':<35}{'Peak Time':<35}{'T End':<35}{'N Sus End':<35}\n")
            # f.write(make_row(graph_results["basic"], "basic") + "\n")
            for name, results in graph_results_by_type.items():
                for n, results in results.items():
                    f.write(make_row(results, name + f"_{n}") + "\n")
            f.write("\n")


def make_sd_efficacy_graphs(results, plot_dir, sd_types=None):
    if sd_types is None:
        sd_types = ["basic", "sd_sim", "simple_max_degree"]
    # Results is mapping from (sd_type, cmax) to a dictionary of results for different graphs
    # transpose results to map form graph to (sd_type, cmax)
    os.makedirs(plot_dir, exist_ok=True)
    graph_results = defaultdict(lambda: defaultdict(dict))
    for (sd_type, cmax), graph_results_type in results.items():
        if sd_type not in sd_types:
            continue
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
        if folder == BASIC_DIR_NAME or folder == SINGLE_DIR_NAME:
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
            res = analyze_sir_results(results_dir / folder / graph, verbose=False)
            if res is not None:
                results[f_type][graph] = res
    print("Results loaded, making plots ...")
    make_sd_efficacy_graphs(results, PLOT_DIR / get_result_dir_name(mu, beta, prop_init))
    overview_results(results)

def cmd_copy_single():
    """Moves the graphs from the single folder into their own subdirectories in the main graph directory to include them in the simulation."""
    for f in os.listdir(SINGLE_DIR):
        if not f.endswith(".net"):
            continue
        name = f[:-4]
        new_dir = GRAPHS_DIR / "single" / name
        os.makedirs(new_dir, exist_ok=True)
        os.system(f"cp {SINGLE_DIR / f} {new_dir / f}")
    for f in os.listdir(SINGLE_NO_ADAPT_DIR):
        if not f.endswith(".net"):
            continue
        name = f[:-4]
        new_dir = GRAPHS_DIR / "single_no_adapt" / name
        os.makedirs(new_dir, exist_ok=True)
        os.system(f"cp {SINGLE_NO_ADAPT_DIR / f} {new_dir / f}")

def cmd_cheat_prop_expr():
    build_path_adaptor_lib()
    build_sir_sim_lib()
    graph_dir = Path("cheat_graphs")
    results_dir = Path("cheat_results")
    os.makedirs(graph_dir / "basic", exist_ok=True)
    cheat_props = np.arange(0, 1.0, 0.1)
    for i in range(20):
        gen_ws_graph(1000, 20, 0.1, str(graph_dir / "basic" / f"ws-{i}.net"))
    for prop in cheat_props:
        call_graph_sd_adaptor(graph_dir / "basic", graph_dir / f"cheat_{prop}", "cheat", 4, prop)
        call_sir_sim("sir_expr", graph_dir / f"cheat_{prop}", results_dir / (get_result_dir_name(mu, beta, prop_init) + f"_cheat_prop={prop}"), beta, mu, prop_init, 20)


def cmd_cheat_prop_ana():
    results_dir = Path("cheat_results")
    results = {}
    for folder in os.listdir(results_dir):
        if not os.path.isdir(results_dir / folder):
            continue
        cheat_prop = float(folder.split("=")[-1])
        res = analyze_sir_results(results_dir / folder, verbose=False)
        results[cheat_prop] = res
    for metric in ["peak", "peak_time", "t_end", "n_sus_end"]:
        keys = sorted(results.keys())
        fig, ax = plt.subplots()
        fig.set_size_inches(10, 6)
        ax.set_xlim(0, 1)
        ax.set_xlabel("Cheat prop")
        ax.set_ylabel(metric)
        ax.plot(keys, [results[k][metric][0] for k in keys], label="mean")
        ax.fill_between(keys, [results[k][metric][0] - results[k][metric][1] for k in keys],
                        [results[k][metric][0] + results[k][metric][1] for k in keys], alpha=0.5, label="std")
        ax.legend()
        ax.set_title(f"Cheat prop {metric} for WS graph with 1000 nodes, 20 neighbors and 0.02 probability")
        fig.savefig(f"cheat_{metric}.png")


# TODO: clean existing directories?
if __name__ == "__main__":
    if len(sys.argv) < 2:
        print(f"Usage: {sys.argv[0]} [commands]")
        sys.exit(1)
    for command in sys.argv[1:]:
        if command == "gen":
            cmd_gen()
        elif command == "copy_single":
            cmd_copy_single()
        elif command == "adapt":
            cmd_adapt()
        elif command == "sim_all":
            cmd_sim_all()
        elif command == "analyze":
            cmd_analyze()
        elif command == "cheat_prop_expr":
            cmd_cheat_prop_expr()
        elif command == "cheat_prop_ana":
            cmd_cheat_prop_ana()
        else:
            print(f"Unknown command: {command}")
            sys.exit(1)
