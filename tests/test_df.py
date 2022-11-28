import pytest
from belinda import *
import polars as pl
g = Graph("resources/com-dblp.bincode.lz4")
# c = #pl.read_csv("resources/clus.str.txt", has_header=False, sep='\t')
print(g.m * 2 / g.n)
modes = [SingletonMode.Ignore, SingletonMode.AsIs, SingletonMode.AutoPopulate]
df = read_json(g, "resources/clus.json", modes[1])
print(df)
print(g.n, modes[1], df.select([g.conductance().max()]))