import matplotlib.pyplot as plt
import math
import networkx as nx
import numpy as np
import random

# Save the graph with community information in Pajek format
def save_pajek_with_communities(G, filename):
    with open(filename, 'w') as f:
        # Write vertices with community information
        f.write(f"*Vertices {G.number_of_nodes()}\n")
        for node, data in G.nodes(data=True):
            community = data.get('community', 1)
            f.write(f"{node + 1} \"{node}\" ic {community}\n")
        
        # Write edges
        f.write("*Edges\n")
        for u, v, data in G.edges(data=True):            
            f.write(f"{u + 1} {v + 1}\n")


# # Caveman Graph
# Starts with fully connected cliques (caves) and then rewires edges to connect different cliques.
# Models populations with strong community structures, where disease spreads within and between communities.

ls= [50, 100, 150, 200] # Number of cliques
ks= [10, 50, 100, 150, 200] # Size of cliques

for l in ls:
    for k in ks:
        
        G = nx.caveman_graph(l, k)
        for start in range(0, l * k, k):
            G.remove_edge(start, start + 1)
            G.add_edge(start, (start - 1) % (l * k))
            
        # G = nx.connected_caveman_graph(l, k)

        # Add community information to each node
        communities = np.arange(l * k).reshape(l, k)
        for idx, community in enumerate(communities):
            for node in community:
                G.nodes[node]['community'] = idx + 1  # Pajek uses 1-based indexing

        # Save the graph with community information in Pajek format
        save_pajek_with_communities(G, f"connected_caveman_graph_l{l}_k{k}.net")

# # Caveman Graph modified
# The nodes conected to the others in the community are the ones connecting to other communities

ls= [50, 100, 150, 200] # Number of cliques
ks= [5, 25, 50, 75, 100] # Size of cliques

for l in ls:
    for k in ks:
        
        G = nx.caveman_graph(l, k)
        for start in range(0, l * k, k):
            G.remove_edge(start, start + 1)
            G.add_edge(start, (start - 1) % (l * k))           

        # Add community information to each node
        communities = np.arange(l * k).reshape(l, k)
        for idx, community in enumerate(communities):
            for node in community:
                G.nodes[node]['community'] = idx + 1  # Pajek uses 1-based indexing

        edges_to_remove=[]
        for start in range(0, l * k, k):
            for idx in range(start+1, start+k-1, 1):                
                nodes_to_add = nx.edges(G, idx)
                edges_to_remove = edges_to_remove + list(nodes_to_add)
                # print(nx.edges(G, idx))
        G.remove_edges_from(edges_to_remove)

        for start in range(0, l * k, k):
            for idx in range(start, start+k-1, 1):
                G.add_edge(idx, idx+1)

        # Save the graph with community information in Pajek format
        save_pajek_with_communities(G, f"not_fully_connected_caveman_graph_l{l}_k{k}.net")