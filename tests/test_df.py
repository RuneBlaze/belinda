import pytest
from belinda import *
import polars as pl
g = Graph("../belinda_data/cen.bincode.lz4")
# print(g.summary())
c = read_assignment(g, "../belinda_data/cen.tsv.leiden_cpm_0.01.aoc")
# nodes = g.nodes()
# print(nodeset_to_list(g, c.get_column("nodes")))
# print(cc_labels(g, nodes.get_column("node")))
# print(cc_size(g, cc_labels(g, nodes.get_column("node"))))
# print(g.annotate_cc(nodes))