#!/usr/bin/env python3

import matplotlib.pyplot as plt
import numpy as np

import sys

filename = sys.argv[1]

data = np.genfromtxt(filename, skip_header=1, delimiter=",")
labels = np.genfromtxt(filename, max_rows=1, dtype=str, delimiter=",")

plt.plot(data)
plt.legend(labels)
plt.show()
