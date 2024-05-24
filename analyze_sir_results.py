#!/usr/bin/env python3

import sys
from pathlib import Path
import os

import numpy as np

from gen_graphs import *


def _align_str(s, length=20):
    return s + ' ' * (length - len(s))


def _make_mean_std_str(mean, std):
    basic_str = f"{mean:.2f} +- {std:.2f}"
    return _align_str(basic_str)


def _make_std_of_mean_str(std_of_mean):
    basic_str = f"    {std_of_mean:.2f}"
    return _align_str(basic_str)


def analyze_sir_results(folder_path: Path, verbose=True):
    all_data = None
    if verbose:
        print(f"{'File':<30} {'Peak':<20} {'Peak Time':<20} {'T End':<20} {'N Sus End':<20}")
        print('-' * 110)
    for f in os.listdir(folder_path):
        if not os.path.isfile(folder_path / f):
            continue
        data = np.genfromtxt(folder_path / f, delimiter=',', skip_header=1)
        all_data = np.concatenate([all_data, data]) if all_data is not None else data
        if verbose:
            means = np.mean(data, axis=0)
            stds = np.std(data, axis=0)
            print(f"{f:<30} {_make_mean_std_str(means[0], stds[0])} {_make_mean_std_str(means[1], stds[1])} {_make_mean_std_str(means[2], stds[2])} {_make_mean_std_str(means[3], stds[3])}")

    if verbose:
        print('-' * 110)
    means = np.mean(all_data, axis=0)
    stds = np.std(all_data, axis=0)
    std_of_means = stds / np.sqrt(all_data.shape[0])

    if verbose:
        print(f"{'Overall':<30} {_make_mean_std_str(means[0], stds[0])} {_make_mean_std_str(means[1], stds[1])} {_make_mean_std_str(means[2], stds[2])} {_make_mean_std_str(means[3], stds[3])}")
        print(f"{'Standard Error of Mean':<30} {_make_std_of_mean_str(std_of_means[0])} {_make_std_of_mean_str(std_of_means[1])} {_make_std_of_mean_str(std_of_means[2])} {_make_std_of_mean_str(std_of_means[3])}")
    return {
        "peak": (means[0], stds[0], std_of_means[0]),
        "peak_time": (means[1], stds[1], std_of_means[1]),
        "t_end": (means[2], stds[2], std_of_means[2]),
        "n_sus_end": (means[3], stds[3], std_of_means[3])
    }


if __name__ == '__main__':
    if len(sys.argv) != 2:
        print("Usage: python analyze_sir_results.py <folder_path>")
        sys.exit(1)

    folder_path = Path(sys.argv[1])
    analyze_sir_results(folder_path, verbose=True)
