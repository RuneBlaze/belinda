# Introduction

Belinda allows using data frames to explore graph clusterings,
with particular attention paid to community detection. It is designed
for scalable exploratory data analysis and is an extension to [Polars](https://pola-rs.github.io/polars-book/user-guide/introduction.html).


Here is an example showing how Belinda visualizes a clustering:
```python
# interative prompt
>>> g.summary() # a graph loaded
shape: (1, 4)
┌────────┬────────┬────────────────┬───────────────────┐
│ n      ┆ m      ┆ num_components ┆ largest_component │
│ ---    ┆ ---    ┆ ---            ┆ ---               │
│ u32    ┆ u64    ┆ u32            ┆ u32               │
╞════════╪════════╪════════════════╪═══════════════════╡
│ 334863 ┆ 925872 ┆ 1              ┆ 334863            │
└────────┴────────┴────────────────┴───────────────────┘

>>> c # a data frame produced by Belinda
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

## Installation

Try the following

```bash
pip3 install --pre belinda # --pre is important. Belinda updates frequently
```