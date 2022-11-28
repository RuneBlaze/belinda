import pytest
from belinda import *
import polars as pl
g = Graph("resources/com-dblp.bincode.lz4")
c = read_assignment(g, "resources/clus.int.txt", mode=SingletonMode.Ignore)
print(g.nodes(c))