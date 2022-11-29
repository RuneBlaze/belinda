from polars import col, when
from polars import Expr
import polars as pl
from .belinda import *


def cpm(r):
    return (col("m") - r * col("n") * (col("n") - 1) / 2).alias("cpm")


vol = (col("m") * 2 + col("c")).alias("vol")


def modularity(self, r=1):
    big_l = self.m
    return (col("m") / big_l - r * (vol / (2 * big_l)) ** 2).alias("modularity")


def vol1(self):
    complement = 2 * self.m - vol
    return when(vol > complement).then(complement).otherwise(vol).alias("vol1")


def conductance(self):
    return (
        when(col("n") > 1)
        .then((col("c") / self.vol1()))
        .otherwise(None)
        .alias("conductance")
    )


def node_coverage(self, overlap=False):
    a = "node_coverage"
    if overlap:
        return (pl.col("nodes").set.union().set.popcnt() / self.n).alias(a)
    else:
        return (pl.col("n").sum() / self.n).alias(a)


def edge_coverage(self, overlap=False):
    a = "edge_coverage"
    if overlap:
        return (
            self.intra_edges(pl.col("nodes")).set.union().set.popcnt() / self.m
        ).alias(a)
    else:
        return (pl.col("m").sum() / self.m).alias(a)


def verbose_statistics(graph, clustering, overlap=False, statistics=[]):
    return clustering.select(
        [graph.node_coverage(overlap), graph.edge_coverage(overlap), *statistics]
    )


def summary_statistics(graph, clustering, overlap=False, statistics=[]):
    return verbose_statistics(graph, clustering, overlap, statistics).describe()


def one_liner(graph, clustering, overlap=False, statistics=[]):
    return clustering.select(
        [
            graph.node_coverage(overlap),
            graph.edge_coverage(overlap),
            *[
                pl.concat_list([s.quantile(0), s.quantile(0.5), s.quantile(1)])
                for s in statistics
            ],
        ]
    )


def write_assignment(graph, clustering, filepath):
    df = graph.nodes(clustering)
    with open(filepath, "w+") as fh:
        for n, lbls in zip(df.get_column("node"), df.get_column("labels")):
            for l in lbls:
                fh.write(f"{n}\t{l}\n")


setattr(Graph, "modularity", modularity)
setattr(Graph, "cpm", lambda self, r: cpm(r))
setattr(
    Graph, "intra_edges", lambda self, exprs: exprs.map(lambda x: self.covered_edges(x))
)
setattr(Graph, "conductance", conductance)
setattr(Graph, "vol1", vol1)
setattr(Graph, "cc", lambda self, exprs: exprs.map(lambda x: cc_labels(self, x)))
setattr(Graph, "cc_size", lambda self, exprs: exprs.map(lambda x: cc_size(self, x)))
setattr(
    Graph,
    "annotate_cc",
    lambda self, df: df.with_column(self.cc(col("node")).alias("cc")).with_column(
        self.cc_size(col("cc")).alias("cc_size")
    ),
)
setattr(Graph, "node_coverage", node_coverage)
setattr(Graph, "edge_coverage", edge_coverage)
setattr(Graph, "summary", lambda self: pl.select([
    pl.lit(self.n).alias("n").cast(pl.UInt32),
    pl.lit(self.m).alias("m").cast(pl.UInt64),
    pl.lit(self.num_components()).alias("num_components").cast(pl.UInt32),
    pl.lit(self.largest_component()).alias("largest_component").cast(pl.UInt32),
]))

@pl.api.register_expr_namespace("set")
class EfficientSet:
    def __init__(self, expr: pl.Expr):
        self._expr = expr

    def popcnt(self):
        return self._expr.map(popcnt)

    def len(self):
        return self._expr.map(popcnt)

    def union(self):
        return self._expr.map(lambda x: union(x))
