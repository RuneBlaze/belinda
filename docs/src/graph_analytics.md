# Graph Analytics

Belinda is not designed to be the full-fledged solution
to undirected graph analytics. Here we show the limited features provided.

Assume that `g` is a `bl.Graph`.

## `g.summary()`

Convenience method producing a dataframe of some summary statistics.

```python
>>> g.summary()
shape: (1, 4)
┌────────┬────────┬────────────────┬───────────────────┐
│ n      ┆ m      ┆ num_components ┆ largest_component │
│ ---    ┆ ---    ┆ ---            ┆ ---               │
│ u32    ┆ u64    ┆ u32            ┆ u32               │
╞════════╪════════╪════════════════╪═══════════════════╡
│ 334863 ┆ 925872 ┆ 1              ┆ 334863            │
└────────┴────────┴────────────────┴───────────────────┘
```

## `g.nodes(clustering=None, verbose=False)`

> This feature is experimental, and the API may change.

A table of nodes, depending on the arguments can achieve various things:

```
>>> g.nodes() # a table of nodes with degrees
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
```

Setting `verbose=True` will also print out the adjacency list:

```
>>> g.nodes(verbose=True)
shape: (334863, 3)
┌────────┬────────┬─────────────────────────────┐
│ node   ┆ degree ┆ adj                         │
│ ---    ┆ ---    ┆ ---                         │
│ u32    ┆ u32    ┆ list[u32]                   │
╞════════╪════════╪═════════════════════════════╡
│ 1      ┆ 8      ┆ [88160, 118052, ... 500600] │
├╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ 88160  ┆ 7      ┆ [1, 161555, ... 102091]     │
├╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ 118052 ┆ 18     ┆ [1, 161555, ... 479787]     │
├╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ 161555 ┆ 31     ┆ [1, 88160, ... 470778]      │
├╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ ...    ┆ ...    ┆ ...                         │
├╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ 548085 ┆ 1      ┆ [548091]                    │
```

Given a clustering, the final result includes the labels for each node.
This is an unstable API, and to support overlapping clusters
each node can be assigned to multiple clusters.

```
>>> g.nodes(c)
shape: (334863, 3)
┌────────┬────────┬───────────┐
│ node   ┆ degree ┆ labels    │
│ ---    ┆ ---    ┆ ---       │
│ u32    ┆ u32    ┆ list[u32] │
╞════════╪════════╪═══════════╡
│ 1      ┆ 8      ┆ [18951]   │
├╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌┤
│ 88160  ┆ 7      ┆ [18951]   │
├╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌┤
│ 118052 ┆ 18     ┆ [35215]   │
├╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌┤
│ 161555 ┆ 31     ┆ [18951]   │
├╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌┤
│ ...    ┆ ...    ┆ ...       │
├╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌┤
│ 548085 ┆ 1      ┆ [295065]  │
├╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌┤
```