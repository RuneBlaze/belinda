import pytest
from belinda import *
import polars as pl
g = Graph("resources/com-dblp.bincode.lz4")
c = read_assignment(g, "resources/clus.str.txt", mode=SingletonMode.Ignore)
print(one_liner(g, c, overlap=False, statistics=[g.modularity(), g.cpm(1), g.conductance(), pl.col('mcd'), pl.col('n')]))
write_assignment(g, c, "resources/clus.str.mod.txt")