#!/usr/bin/env python3

from pathlib import Path
import os
import sys

this_dir = Path(__file__).resolve().parent
exec_path_adapt = Path(__file__).resolve().parent / "graph-sd-adaptor/target/release/graph-sd-adaptor"


def build_path_adaptor_lib():
    print("Runner: building graph-sd-adaptor...")
    current_dir = os.getcwd()
    os.chdir(this_dir / "graph-sd-adaptor")
    os.system("cargo build --release")
    os.chdir(current_dir)


def call_graph_sd_adaptor(in_file, out_file, *args):
    print(f"Runner: calling graph-sd-adaptor with args: {args}")
    os.system(f"{exec_path_adapt} {in_file} {out_file} {' '.join(map(str, args))}")


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print(f"Usage: {sys.argv[0]} [commands]")
        sys.exit(1)
    for command in sys.argv[1:]:
        if command == "adopt":
            build_path_adaptor_lib()
            call_graph_sd_adaptor("example_graphs", "adopted_example_graphs", "max_degree", 3)
