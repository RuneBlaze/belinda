# Basic EDA

Let's first load everything:

```python
>>> import polars as pl
>>> import belinda as bl
>>> g = bl.Graph("com-amazon.ungraph.txt")
>>> g.summary()
shape: (1, 4)
┌────────┬────────┬────────────────┬───────────────────┐
│ n      ┆ m      ┆ num_components ┆ largest_component │
│ ---    ┆ ---    ┆ ---            ┆ ---               │
│ u32    ┆ u64    ┆ u32            ┆ u32               │
╞════════╪════════╪════════════════╪═══════════════════╡
│ 334863 ┆ 925872 ┆ 1              ┆ 334863            │
└────────┴────────┴────────────────┴───────────────────┘

>>> g.nodes() # a table of all nodes
shape: (334863, 2)
┌────────┬────────┐
│ node   ┆ degree │
│ ---    ┆ ---    │
│ u32    ┆ u32    │
╞════════╪════════╡
│ 1      ┆ 8      │
├╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┤
│ 88160  ┆ 7      │
├╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┤
│ 118052 ┆ 18     │
├╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┤
│ 161555 ┆ 31     │
├╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┤
│ ...    ┆ ...    │
├╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┤
```

## Loading the Leiden(r=0.2) clustering

(These data were obtained from the [Quick Start](./quick_start.md) page)

```python
>>> c = bl.read_membership(g, "com-amazon.leiden.txt").sort(pl.col('n'), reverse=True)
>>> c
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
```

Let's further filter out singletons:

```python
>>> c = c.filter(pl.col('n') > 1)
>>> c
shape: (60609, 6)
┌───────┬───────────────┬─────┬─────┬─────┬─────┐
│ label ┆ nodes         ┆ n   ┆ m   ┆ c   ┆ mcd │
│ ---   ┆ ---           ┆ --- ┆ --- ┆ --- ┆ --- │
│ i64   ┆ binary        ┆ u64 ┆ u64 ┆ u64 ┆ u64 │
╞═══════╪═══════════════╪═════╪═════╪═════╪═════╡
│ 0     ┆ [binary data] ┆ 26  ┆ 115 ┆ 128 ┆ 5   │
├╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┤
│ 8     ┆ [binary data] ┆ 25  ┆ 110 ┆ 537 ┆ 5   │
├╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┤
│ 7     ┆ [binary data] ┆ 25  ┆ 106 ┆ 83  ┆ 5   │
├╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┤
│ 6     ┆ [binary data] ┆ 25  ┆ 107 ┆ 85  ┆ 5   │
├╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┤
│ ...   ┆ ...           ┆ ... ┆ ... ┆ ... ┆ ... │
```

Now, without singletons, what is the coverage? Belinda provides the `peek`
function to quickly get an overview of the clustering:

```python
>>> bl.peek(g, c)
shape: (1, 4)
┌────────────┬───────────────┬───────────────┬──────────────────┐
│ n_clusters ┆ node_coverage ┆ edge_coverage ┆ n                │
│ ---        ┆ ---           ┆ ---           ┆ ---              │
│ u32        ┆ f64           ┆ f64           ┆ list[f64]        │
╞════════════╪═══════════════╪═══════════════╪══════════════════╡
│ 60609      ┆ 0.927054      ┆ 0.5739        ┆ [2.0, 4.0, 26.0] │
└────────────┴───────────────┴───────────────┴──────────────────┘
# the list[f64] is the distribution of cluster sizes (n)
# given in mininum, median, and maximum
```

Let's just say that we are interested in the average degree inside each cluster,
and would also like to know the distribution of the average degree:

Notice that mathematically average degree is \\(2 m / n\\). The rest is easy:

```python
>>> bl.peek(g, c, statistics=[(pl.col('m') * 2 / pl.col('n')).alias("avg_degree")])
shape: (1, 4)
┌────────────┬───────────────┬───────────────┬──────────────────┐
│ n_clusters ┆ node_coverage ┆ edge_coverage ┆ avg_degree       │
│ ---        ┆ ---           ┆ ---           ┆ ---              │
│ u32        ┆ f64           ┆ f64           ┆ list[f64]        │
╞════════════╪═══════════════╪═══════════════╪══════════════════╡
│ 60609      ┆ 0.927054      ┆ 0.5739        ┆ [1.0, 2.0, 8.88] │
└────────────┴───────────────┴───────────────┴──────────────────┘
```

Notice that `peek` accepts a named argument called `statistics`, which accepts
a list of Polars expressions, that it will automatically broadcast to each cluster
and calculate the statistics for. Let's try something more extravagant.

```python
>>> avg_degree = (pl.col('m') * 2 / pl.col('n')).alias("avg_degree")
>>> choose2 = lambda x: x * (x - 1) / 2 # \binom{x}{2}
>>> edge_density = (pl.col('m')/choose2(pl.col('n'))).alias("edge_density")
>>> cpm = lambda r: (pl.col('m') - choose2(pl.col('n')) * r).alias("cpm")
>>> bl.peek(g, c, statistics=[avg_degree, edge_density, cpm(0.2)])
shape: (1, 6)
┌────────────┬───────────────┬───────────────┬──────────────────┬───────────────────────────┬──────────────────┐
│ n_clusters ┆ node_coverage ┆ edge_coverage ┆ avg_degree       ┆ edge_density              ┆ cpm              │
│ ---        ┆ ---           ┆ ---           ┆ ---              ┆ ---                       ┆ ---              │
│ u32        ┆ f64           ┆ f64           ┆ list[f64]        ┆ list[f64]                 ┆ list[f64]        │
╞════════════╪═══════════════╪═══════════════╪══════════════════╪═══════════════════════════╪══════════════════╡
│ 60609      ┆ 0.927054      ┆ 0.5739        ┆ [1.0, 2.0, 8.88] ┆ [0.304762, 0.666667, 1.0] ┆ [0.8, 2.8, 51.0] │
└────────────┴───────────────┴───────────────┴──────────────────┴───────────────────────────┴──────────────────┘
```

The theme here is *composable*: it is easy to define custom statistics, and
it is easy to cherry-pick the statistics that you actually want.

Let's take it further: what about statistics on clusters only of at least size 5?

```python
>>> bl.peek(g, c.filter(pl.col('n') >= 5))
shape: (1, 4)
┌────────────┬───────────────┬───────────────┬──────────────────┐
│ n_clusters ┆ node_coverage ┆ edge_coverage ┆ n                │
│ ---        ┆ ---           ┆ ---           ┆ ---              │
│ u32        ┆ f64           ┆ f64           ┆ list[f64]        │
╞════════════╪═══════════════╪═══════════════╪══════════════════╡
│ 28512      ┆ 0.642454      ┆ 0.494698      ┆ [5.0, 6.0, 26.0] │
└────────────┴───────────────┴───────────────┴──────────────────┘
```

What about statistics on the top 10 clusters by size?
```python
>>> bl.peek(g, c.sort(pl.col('n'), reverse=True).head(10))
shape: (1, 4)
┌────────────┬───────────────┬───────────────┬────────────────────┐
│ n_clusters ┆ node_coverage ┆ edge_coverage ┆ n                  │
│ ---        ┆ ---           ┆ ---           ┆ ---                │
│ u32        ┆ f64           ┆ f64           ┆ list[f64]          │
╞════════════╪═══════════════╪═══════════════╪════════════════════╡
│ 10         ┆ 0.00075       ┆ 0.001163      ┆ [25.0, 25.0, 26.0] │
└────────────┴───────────────┴───────────────┴────────────────────┘
```

## Behind the scenes

The `peek` implementation is relatively magic-free:

```python
def peek(graph, clustering, overlap=False, statistics=[pl.col('n')]):
    return clustering.select(
        [
            pl.col('nodes').count().alias('n_clusters'),
            graph.node_coverage(overlap),
            graph.edge_coverage(overlap),
            *[
                pl.concat_list([s.quantile(0), s.quantile(0.5), s.quantile(1)])
                for s in statistics
            ],
        ]
    )
```