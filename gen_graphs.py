#!/usr/bin/env python3

import sys
import matplotlib.pyplot as plt
import os
import multiprocessing
import numpy as np

_this_file = os.path.realpath(__file__)
sys.path.append(os.path.join(os.path.dirname(_this_file), 'network_gen/'))
from network_gen import *

n_graphs = 20

# sim_tasks = [
#     ("BA", gen_ba_graph, [1000, 3, 10]),
#     ("BA", gen_ba_graph, [1000, 6, 10]),
#     ("BA", gen_ba_graph, [1000, 10, 15]),
#     ("ER", gen_er_graph, [1000, 0.1]),
#     ("ER", gen_er_graph, [1000, 0.9]),
#     ("WS", gen_ws_graph, [1000, 20, 0.1]),
#     ("WS", gen_ws_graph, [1000, 10, 0.1]),
#     ("WS", gen_ws_graph, [1000, 40, 0.1]),
#     ("ER", gen_er_graph, [1000, 0.3]),
#     ("WS", gen_ws_graph, [1000, 20, 0.3]),
#     ("ER", gen_er_graph, [1000, 0.02]),
#     ("ER", gen_er_graph, [1000, 0.5]),
#     ("WS", gen_ws_graph, [1000, 20, 0.02]),
#     ("CM-Poisson", gen_cm_poisson_graph, [1000, 20]),
#     ("CM-Powerlaw", gen_cm_powerlaw_graph, [1000, 3, 4])
# ]
sim_tasks = [
    ("CM-Powerlaw", gen_cm_powerlaw_graph, [1000, 2, 6]),
]

def get_name(graph):
    param_str = "_".join([str(p) for p in graph[2]])
    return f"{graph[0]}_{param_str}"


def gen_graphs(directory):
    # for every graph generate a subdirectory with the graph type and parameters in the name then generate n_graphs graphs
    os.makedirs(directory, exist_ok=True)
    for graph in sim_tasks:
        name = get_name(graph)
        graph_dir = os.path.join(directory, name)
        os.makedirs(graph_dir, exist_ok=True)
        pool = multiprocessing.Pool()
        for i in range(n_graphs):
            pool.apply_async(graph[1], graph[2] + [os.path.join(graph_dir, f"{name}-{i}.net")])
        pool.close()
        pool.join()


def validate_graph(graph_path):
    import networkx as nx
    G = nx.read_pajek(graph_path)
    G = nx.Graph(G)
    has_self_loops = False
    for u, v in G.edges():
        if u == v:
            print(f"Graph {graph_path} has self-loops")
            return False
    n_components = nx.number_connected_components(G)
    if n_components != 1:
        print(f"Graph {graph_path} has {n_components} components")
        return False
    n_edges = G.number_of_edges()
    if n_edges < 100:
        print(f"Graph {graph_path} has less than 100 edges")
        return False
    return True


def validate_graphs(directory):
    print("Validating graphs...")
    pool = multiprocessing.Pool()
    # validate all .net file in directory and subdirectories
    for root, _, files in os.walk(directory):
        for file in files:
            if file.endswith(".net"):
                pool.apply_async(validate_graph, [os.path.join(root, file)])
    pool.close()
    pool.join()


if __name__ == "__main__":
    if sys.argv[1] == "gen":
        gen_graphs(sys.argv[2])
    elif sys.argv[1] == "validate":
        validate_graph(sys.argv[2])
