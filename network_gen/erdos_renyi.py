# Import Required modules 
import networkx as nx 
import matplotlib.pyplot as plt 
import random 


# Distribution graph for Erdos_Renyi model 
def distribution_graph(g): 
	print(nx.degree(g)) 
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

		
def generate_erdos_renyi(N, P): 
    if P >= 1: 
        print("Probability P should be less than 1") 
        return nx.empty_graph(N)
    
    if P <= 0: 
        print("Probability P should be greater than 0") 
        return nx.empty_graph(N)

    if N <= 0: 
        print("Number of nodes should be greater than 0") 
        return nx.empty_graph(N)
    
    if N == 1: 
        print("Only one node, No edges to form") 
        return nx.empty_graph(N)
     
    # Create an empty graph object 
    g = nx.Graph() 

    # Adding nodes 
    g.add_nodes_from(range(1, N + 1)) 

    # Add edges to the graph randomly. 
    for i in g.nodes(): 
        for j in g.nodes(): 

            if (i < j): 
                if (g.degree(i) < 19 and g.degree(j) < 19):

                    # Take random number R. 
                    R = random.random() 
                    
                    # Check if R<P add the edge to the graph else ignore. 
                    if (R < P): 
                        g.add_edge(i, j) 
    pos = nx.circular_layout(g) 
    
    return g

