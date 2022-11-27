from polars import col
from polars import Expr
from .belinda import *

def cpm(r):
    return (col("m") - r * col("n") * (col("n") - 1) / 2).alias("cpm")

vol = (col("m") * 2 + col("c")).alias("vol")

def modularity(self, r = 1):
    big_l = self.m
    return (col("m") / big_l - r * (vol / (2 * big_l)) ** 2).alias("modularity")

setattr(Graph, "modularity", modularity)
setattr(Graph, "cpm", lambda self, r: cpm(r))
setattr(Graph, "intra_edges", lambda self, exprs: exprs.map(lambda x: self.covered_edges(x)))
setattr(Expr, "popcnt", lambda self: self.map(popcnt))
setattr(Expr, "union", lambda self: self.map(lambda x: union(x)))