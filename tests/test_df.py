import pytest
from belinda import *
import polars as pl
g = Graph("resources/com-dblp.bincode.lz4")
c = read_clusters(g, "resources/clus.txt")

out = c.select(
    [
        g.intra_edges(pl.col("nodes")).union().popcnt() / g.m,
        cpm(0.5),
    ]
)
print(out)