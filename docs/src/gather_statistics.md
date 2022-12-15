# Gathering Statistics

The Constants Potts Model (CPM) is an optimization problem for community detection (a form of clustering).
The maximization problem CPM defines is *additive* for all clusters, meaning that
each cluster has its own independent score, and the total score of the entire clustering
is the sum of the score of its clusters. The optimization problem asks to assign clusters
to maximize the total score.

The quality/score of a single cluster \\(S_i\\) where \\(S_i\\) has \\(n\\) nodes and \\(m\\)
edges (internal nodes and edges) is defined by the following function, parameterized
by a resolution value \\(\gamma \in (0, 1]\\):

\\[
    Q(S_i) = m - \gamma \binom{n}{2}
\\]

And given a clustering \\(\\{S_i\\}\\), the quality of this clustering is defined as:

\\[
    Q(\\{S_i\\}) = \sum_{i=1} Q(S_i)
\\]

A naive question to ask this point is: given a clustering, what is the CPM quality on each of its clusters,
and what is the CPM quality of the entire clustering?

## Calculating CPM in a composable way

Assuming that we still have the `c` cluster data frame obtained from [Quick Start](./quick_start.md) (if stored as Parquet,
it is easy to load it back) and the `g` graph. Let's see what we can do to calculate the CPM score for each cluster,
and of course, after we have the CPM score of each cluster, we just sum them up to get the total CPM score of the clustering.

Let's look at our clustering again:

```python
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

So let's take a step back: what is the CPM score of cluster `0`? Using the definition:

```python
>>> gamma = 0.2 # this cluster is obtained with resolution = 0.2
>>> 115 - 26 * (26 - 1) * 0.5 * gamma # a rather stupid way to write choose2
50.0
```

So cluster `0` has CPM score `50.0`, which seems right from the given definition. Now we are ready to take on the entire clustering:
```python
>>> c_cpm = c.with_column((pl.col("m") - pl.col("n") * (pl.col("n") - 1) * 0.5 * gamma).alias("cpm"))
>>> c_cpm
shape: (85036, 7)
┌────────┬───────────────┬─────┬─────┬─────┬─────┬──────┐
│ label  ┆ nodes         ┆ n   ┆ m   ┆ c   ┆ mcd ┆ cpm  │
│ ---    ┆ ---           ┆ --- ┆ --- ┆ --- ┆ --- ┆ ---  │
│ i64    ┆ binary        ┆ u64 ┆ u64 ┆ u64 ┆ u64 ┆ f64  │
╞════════╪═══════════════╪═════╪═════╪═════╪═════╪══════╡
│ 0      ┆ [binary data] ┆ 26  ┆ 115 ┆ 128 ┆ 5   ┆ 50.0 │
├╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌╌┤
│ 8      ┆ [binary data] ┆ 25  ┆ 110 ┆ 537 ┆ 5   ┆ 50.0 │
├╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌╌┤
│ 7      ┆ [binary data] ┆ 25  ┆ 106 ┆ 83  ┆ 5   ┆ 46.0 │
├╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌╌┤
│ 6      ┆ [binary data] ┆ 25  ┆ 107 ┆ 85  ┆ 5   ┆ 47.0 │
├╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌╌┤
│ ...    ┆ ...           ┆ ... ┆ ... ┆ ... ┆ ... ┆ ...  │
├╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌╌┤
│ 295297 ┆ [binary data] ┆ 1   ┆ 0   ┆ 1   ┆ 0   ┆ 0.0  │
├╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌╌┤
```

Note that even though a bit roundabout, we did manage to use the `n` and `m` column directly
to generate the `cpm` column, and our calculation follows straight from the definition.

In fact, if we want, we can define the following expression ourselves:

```python
from polars import col
def cpm(r):
    return (col("m") - r * col("n") * (col("n") - 1) / 2).alias("cpm")
```

```python
>>> c.with_column(cpm(gamma))
shape: (85036, 7)
┌────────┬───────────────┬─────┬─────┬─────┬─────┬──────┐
│ label  ┆ nodes         ┆ n   ┆ m   ┆ c   ┆ mcd ┆ cpm  │
│ ---    ┆ ---           ┆ --- ┆ --- ┆ --- ┆ --- ┆ ---  │
│ i64    ┆ binary        ┆ u64 ┆ u64 ┆ u64 ┆ u64 ┆ f64  │
╞════════╪═══════════════╪═════╪═════╪═════╪═════╪══════╡
│ 0      ┆ [binary data] ┆ 26  ┆ 115 ┆ 128 ┆ 5   ┆ 50.0 │
├╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌╌┤
│ 8      ┆ [binary data] ┆ 25  ┆ 110 ┆ 537 ┆ 5   ┆ 50.0 │
├╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌╌┤
│ 7      ┆ [binary data] ┆ 25  ┆ 106 ┆ 83  ┆ 5   ┆ 46.0 │
├╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌╌┤
│ 6      ┆ [binary data] ┆ 25  ┆ 107 ┆ 85  ┆ 5   ┆ 47.0 │
├╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌╌┤
```

Note how composable calculating CPM felt like: one can abstract the mathematical
expression into Polars expressions, and then abstract them into a call like `cpm(gamma)`.

Now let's check: are there any clusters that have negative CPM scores?

```python
>>> c.with_column(cpm(gamma)).filter(pl.col('cpm') < 0)
shape: (0, 7)
┌───────┬────────┬─────┬─────┬─────┬─────┬─────┐
│ label ┆ nodes  ┆ n   ┆ m   ┆ c   ┆ mcd ┆ cpm │
│ ---   ┆ ---    ┆ --- ┆ --- ┆ --- ┆ --- ┆ --- │
│ i64   ┆ binary ┆ u64 ┆ u64 ┆ u64 ┆ u64 ┆ f64 │
╞═══════╪════════╪═════╪═════╪═════╪═════╪═════╡
└───────┴────────┴─────┴─────┴─────┴─────┴─────┘
```

which looks right: clusters should never have negative CPM scores.

Now we are ready to calculate the optimization score:

```python
>>> c.with_column(cpm(gamma)).select(pl.col('cpm').sum())
shape: (1, 1)
┌──────────┐
│ cpm      │
│ ---      │
│ f64      │
╞══════════╡
│ 343848.0 │
└──────────┘
```

Although we don't know how to interpret the optimization score (after all, it is just a score), it is good to
know that it can be derived in a composable way.