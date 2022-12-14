from belinda import *
import numpy as np
import pytest

@pytest.fixture
def simple_graph():
    return Graph("resources/discont_graph.txt")

def test_graph_has_sane_information(simple_graph):
    summary = simple_graph.summary()
    assert summary["n"].view()[0] == 6
    assert summary["m"].view()[0] == 5

def test_clustering_differences(simple_graph):
    c1 = read_membership(simple_graph, "resources/discont_graph.clus.txt", mode=SingletonMode.Ignore)
    assert len(c1) == 1
    c2 = read_membership(simple_graph, "resources/discont_graph.clus.txt", mode=SingletonMode.AsIs)
    assert len(c2) == 2
    c3 = read_membership(simple_graph, "resources/discont_graph.clus.txt", mode=SingletonMode.AutoPopulate)
    assert len(c3) == 4

def test_autopopulate(simple_graph):
    c3 = read_membership(simple_graph, "resources/discont_graph.clus.txt", mode=SingletonMode.AutoPopulate)
    csize = c3.select([pl.col('nodes').set.union().set.popcnt().alias('cover_size')])["cover_size"].view()[0]
    sum_size = c3.with_column(pl.col('nodes').set.popcnt().sum().alias('sum_size'))["sum_size"].view()[0]
    cluster_sizes = c3.with_column(pl.col('nodes').set.popcnt().alias('cluster_size'))["cluster_size"].to_numpy()
    assert csize == simple_graph.n
    assert csize == sum_size
    assert np.all(cluster_sizes >= 1)