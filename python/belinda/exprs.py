from polars import col, when
from polars import Expr
from .belinda import *

def cpm(r):
    return (col("m") - r * col("n") * (col("n") - 1) / 2).alias("cpm")

vol = (col("m") * 2 + col("c")).alias("vol")

def modularity(self, r = 1):
    big_l = self.m
    return (col("m") / big_l - r * (vol / (2 * big_l)) ** 2).alias("modularity")

def vol1(self):
    complement = 2 * self.m - vol
    return when(vol > complement).then(complement).otherwise(vol).alias("vol1")

def conductance(self):
    return when(col("n") > 1).then((col("c") / self.vol1())).otherwise(None).alias("conductance")

setattr(Graph, "modularity", modularity)
setattr(Graph, "cpm", lambda self, r: cpm(r))
setattr(Graph, "intra_edges", lambda self, exprs: exprs.map(lambda x: self.covered_edges(x)))
setattr(Graph, "conductance", conductance)
setattr(Graph, "vol1", vol1)
setattr(Expr, "popcnt", lambda self: self.map(popcnt))
setattr(Expr, "union", lambda self: self.map(lambda x: union(x)))