import pytest
from belinda import *
import polars as pl
g = Graph("../belinda_data/cen.bincode.lz4")
# print(g.summary())
c = read_json(g, "resources/test.json")
print(c)
# nodes = g.nodes()
# print(nodeset_to_list(g, c.get_column("nodes")))
# print(cc_labels(g, nodes.get_column("node")))
# print(cc_size(g, cc_labels(g, nodes.get_column("node"))))
# print(g.annotate_cc(nodes))