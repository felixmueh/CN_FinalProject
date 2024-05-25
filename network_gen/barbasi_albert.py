# BA algo functions
# Import Required modules 
import networkx as nx 
import matplotlib.pyplot as plt 
import random 
import numpy as np

# Distribution graph for Erdos_Renyi model 
def distribution_graph(g): 
	#print(nx.degree(g)) 
	all_node_degree = list(dict((nx.degree(g))).values()) 

	unique_degree = list(set(all_node_degree)) 
	unique_degree.sort() 
	nodes_with_degree = [] 
	for i in unique_degree: 
		nodes_with_degree.append(all_node_degree.count(i)) 

	plt.plot(unique_degree, nodes_with_degree) 
	plt.xlabel("Degrees") 
	plt.ylabel("No. of nodes") 
	plt.title("Degree distribution") 
	plt.show() 

def barabasi_albert(N, m, m_0):
    if m >= m_0:
        print("m should be less than m_0")
        return nx.empty_graph(N)

    if m < 1:
        print("m should be greater than 0")
        return nx.empty_graph(N)

    if N <= m_0:
        print("N should be greater than m_0")
        return nx.empty_graph(N)
    
    G = nx.Graph()
    G = nx.wheel_graph(m_0)

    # Add nodes iteratively
    for i in range(m_0, N):
        degrees = dict(G.degree())
        # print(f"Degrees: {degrees}")
        total_degree = sum(degrees.values())
        # print(f"Total degrees: {total_degree}")

        sum_degrees = 0
        probabilities = {}
        for node, degree in degrees.items():   
            sum_degrees += degree 
            probabilities[node] = sum_degrees
        
        # print(probabilities)

        # Select m nodes to connect to
        targets = []
        while len(targets) < m:
            random_n = random.random()
            r = random_n * total_degree
            # print(random_n, r)
            for node, prob in probabilities.items():
                if r <= prob:
                    if degrees[node] < 20 and node not in targets:
                        G.add_edge(i, node)
                        targets.append(node)   
                        break

    return G
