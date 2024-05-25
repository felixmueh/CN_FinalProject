import network_gen.erdos_renyi as er
import network_gen.barbasi_albert as ba
import matplotlib.pyplot as plt
import networkx as nx
import os

__all__ = ["gen_ba_graph", "gen_er_graph", "gen_ws_graph", "gen_cm_poisson_graph", "gen_cm_powerlaw_graph"]

def build_rust_lib():
    current_dir = os.getcwd()
    os.chdir(os.path.dirname(os.path.realpath(__file__)))
    os.system("cargo build --release")
    os.chdir(current_dir)

build_rust_lib()
exec_path = os.path.join(os.path.dirname(os.path.realpath(__file__)), "target/release/network-gen")

# method to store each graph in a .net file
def store_graph(g, filename):
    with open(filename, 'w') as f:
        f.write("*Vertices " + str(len(g.nodes())) + "\n")
        for node in g.nodes():
            f.write(str(node) + " \"" + str(node) + "\"" + "\n")
        f.write("*Edges\n")
        for edge in g.edges():
            f.write(str(edge[0]) + " " + str(edge[1]) + "\n")

def run_rust_lib(*args):
    os.system(f"{exec_path} {' '.join(args)}")

def gen_ba_graph(n, m, m_0, filepath):
    g = ba.barabasi_albert(n, m, m_0)
    store_graph(g, filepath)

def gen_er_graph(n, p, filepath):
    g = er.generate_erdos_renyi(n, p)
    store_graph(g, filepath)

def gen_ws_graph(n, k, p, filepath):
    run_rust_lib("WS", "-n", str(n), "-k", str(k), "-p", str(p), "-o", filepath)

def gen_cm_poisson_graph(n, lam, filepath):
    run_rust_lib("CM-Poisson", "-n", str(n), "-lambda", str(lam), "-o", filepath)

def gen_cm_powerlaw_graph(n, gamma, k_min, filepath):
    run_rust_lib("CM-Powerlaw", "-n", str(n), "-gamma", str(gamma), "-k-min", str(k_min), "-o", filepath)
