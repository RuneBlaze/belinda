import pytest
from belinda import *
import polars as pl
g = Graph("resources/com-dblp.bincode.lz4")
print(g.summary())
# c = read_assignment(g, "resources/clus.int.txt", mode=SingletonMode.Ignore)
# nodes = g.nodes()
# print(nodeset_to_list(g, c.get_column("nodes")))
# print(cc_labels(g, nodes.get_column("node")))
# print(cc_size(g, cc_labels(g, nodes.get_column("node"))))
# print(g.annotate_cc(nodes))