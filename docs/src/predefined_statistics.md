# Predefined Statistics

Given a graph `g`, Belinda provides these "quality measures" for clusters dependent on
the graph `g`.
Note that statistics such as `mcd` are already provided in the columns for the data frame.

## `g.n`, `g.m`

These are shorthands, `g.n` is the number of nodes, and `g.m` is the number of edges
for the entire graph.

## `g.modularity(r=1)`

"Vanilla" modularity measure of a cluster with resolution defaulting to 1, defined somewhat as:

```python
# degree volume
vol = (col("m") * 2 + col("c")).alias("vol")

def modularity(self, r=1):
    big_l = self.m
    return (col("m") / big_l - r * (vol / (2 * big_l)) ** 2).alias("modularity")
```

### Example

```python
>>> c.head(5) # assuming clustering `c` already loaded
shape: (5, 6)
┌────────┬───────────────┬─────┬─────┬─────┬─────┐
│ label  ┆ nodes         ┆ n   ┆ m   ┆ c   ┆ mcd │
│ ---    ┆ ---           ┆ --- ┆ --- ┆ --- ┆ --- │
│ i64    ┆ binary        ┆ u64 ┆ u64 ┆ u64 ┆ u64 │
╞════════╪═══════════════╪═════╪═════╪═════╪═════╡
│ 34680  ┆ [binary data] ┆ 4   ┆ 3   ┆ 11  ┆ 1   │
├╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┤
│ 298616 ┆ [binary data] ┆ 1   ┆ 0   ┆ 2   ┆ 0   │
├╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┤
│ 45248  ┆ [binary data] ┆ 3   ┆ 2   ┆ 10  ┆ 1   │
├╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┤
│ 297168 ┆ [binary data] ┆ 1   ┆ 0   ┆ 2   ┆ 0   │
├╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┤
│ 294432 ┆ [binary data] ┆ 1   ┆ 0   ┆ 5   ┆ 0   │
└────────┴───────────────┴─────┴─────┴─────┴─────┘

>>> c.head(5).with_column(g.modularity())
shape: (5, 7)
┌────────┬───────────────┬─────┬─────┬─────┬─────┬─────────────┐
│ label  ┆ nodes         ┆ n   ┆ m   ┆ c   ┆ mcd ┆ modularity  │
│ ---    ┆ ---           ┆ --- ┆ --- ┆ --- ┆ --- ┆ ---         │
│ i64    ┆ binary        ┆ u64 ┆ u64 ┆ u64 ┆ u64 ┆ f64         │
╞════════╪═══════════════╪═════╪═════╪═════╪═════╪═════════════╡
│ 34680  ┆ [binary data] ┆ 4   ┆ 3   ┆ 11  ┆ 1   ┆ 0.000003    │
├╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ 298616 ┆ [binary data] ┆ 1   ┆ 0   ┆ 2   ┆ 0   ┆ -1.1665e-12 │
├╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ 45248  ┆ [binary data] ┆ 3   ┆ 2   ┆ 10  ┆ 1   ┆ 0.000002    │
├╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ 297168 ┆ [binary data] ┆ 1   ┆ 0   ┆ 2   ┆ 0   ┆ -1.1665e-12 │
├╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ 294432 ┆ [binary data] ┆ 1   ┆ 0   ┆ 5   ┆ 0   ┆ -7.2908e-12 │
└────────┴───────────────┴─────┴─────┴─────┴─────┴─────────────┘
```

## `g.conductance()`

Intra-cluster conductance, defined somewhat as:

```python
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
```

## `g.cpm(r)`

Constant Potts model with resolution value `r`.