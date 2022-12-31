# Introduction

Belinda is a high performance data science library for graph clusterings, developed
as an add-on to the [Polars](https://pola-rs.github.io/polars-book/user-guide/introduction.html) dataframes library.

 - Reasonably scalable, high performance
 - Written in Rust, designed for Python
 - Bulit-in support for community detection outputs (e.g., Leiden, VieClust, etc.)
 - Designed for graph clusterings, but will generalize beyond

Belinda's data model maps each cluster to a row inside a dataframe. This data model allows manipulating a clustering
just like how one manipulates a dataframe. Moreover, calculating statistics, writing edited clusterings to disk,
and some graph analytics (e.g., getting high degree nodes from a graph, and then seeing which clusters they belong to)
are all made easy with Belinda.

> This library is a heavy work in progress, and the API *will* change substantially (to generalize
> beyond graph clusterings). Currently,
> it is being used in-house, and user feedback is heavily used to influence future API decisions.

## Installation

Try the following

```bash
pip3 install --pre belinda # --pre is important. Belinda updates frequently
```

## Five-minute pitch

Here is an example showing how Belinda explores a clustering:
```python
# interative prompt
>>> g.summary() # `g` a background graph we already loaded
shape: (1, 4)
┌────────┬────────┬────────────────┬───────────────────┐
│ n      ┆ m      ┆ num_components ┆ largest_component │
│ ---    ┆ ---    ┆ ---            ┆ ---               │
│ u32    ┆ u64    ┆ u32            ┆ u32               │
╞════════╪════════╪════════════════╪═══════════════════╡
│ 334863 ┆ 925872 ┆ 1              ┆ 334863            │
└────────┴────────┴────────────────┴───────────────────┘

>>> c # a data frame of a clustering imported by Belinda
shape: (85036, 6)
┌────────┬───────────────┬─────┬─────┬─────┬─────┐
│ label  ┆ nodes         ┆ n   ┆ m   ┆ c   ┆ mcd │
│ ---    ┆ ---           ┆ --- ┆ --- ┆ --- ┆ --- │
│ i64    ┆ binary        ┆ u64 ┆ u64 ┆ u64 ┆ u64 │
╞════════╪═══════════════╪═════╪═════╪═════╪═════╡
│ 0      ┆ [binary data] ┆ 26  ┆ 115 ┆ 128 ┆ 5   │
├╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┤
│ 8      ┆ [binary data] ┆ 25  ┆ 110 ┆ 537 ┆ 5   │
├╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┤
│ 7      ┆ [binary data] ┆ 25  ┆ 106 ┆ 83  ┆ 5   │
├╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┤
│ 6      ┆ [binary data] ┆ 25  ┆ 107 ┆ 85  ┆ 5   │
├╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┤
│ ...    ┆ ...           ┆ ... ┆ ... ┆ ... ┆ ... │
├╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┤

>>> bl.peek(g, c)
shape: (1, 4)
┌────────────┬───────────────┬───────────────┬──────────────────┐
│ n_clusters ┆ node_coverage ┆ edge_coverage ┆ n                │
│ ---        ┆ ---           ┆ ---           ┆ ---              │
│ u32        ┆ f64           ┆ f64           ┆ list[f64]        │
╞════════════╪═══════════════╪═══════════════╪══════════════════╡
│ 85036      ┆ 1.0           ┆ 0.5739        ┆ [1.0, 3.0, 26.0] │
└────────────┴───────────────┴───────────────┴──────────────────┘

>>> bl.peek(g, c.filter(pl.col('n') > 1), statistics=[g.modularity()])
shape: (1, 4)
┌────────────┬───────────────┬───────────────┬────────────────────────────────┐
│ n_clusters ┆ node_coverage ┆ edge_coverage ┆ modularity                     │
│ ---        ┆ ---           ┆ ---           ┆ ---                            │
│ u32        ┆ f64           ┆ f64           ┆ list[f64]                      │
╞════════════╪═══════════════╪═══════════════╪════════════════════════════════╡
│ 60609      ┆ 0.927054      ┆ 0.5739        ┆ [0.000001, 0.000004, 0.000124] │
└────────────┴───────────────┴───────────────┴────────────────────────────────┘
```