# Singleton Handling

Should there be nodes that don't belong to any cluster? This is a question treated differently by different clustering algorithms. For example, Leiden assigns every node to a cluster. IKC only outputs clusters that are deemed "valid". Some people prefer to
simply ignore singleton clusters when doing analyses. Belinda tries to be flexible by letting the user decide
which philosophy to take when reading in the clusters. In other words, users can specify a `mode` as to read
the clusters. For example, the following statement will read clusters but ignore all singletons:

```python
bl.read_membership(g, cluster_path, mode = SingletonMode.Ignore)
```

## Singleton Modes

 - `bl.SingletonMode.AsIs`: read the input clustering "as-is" (see also [Dummy Node Tolerance](#dummy-node-tolerance)). Keep whatever the input has. This is the default.
 - `bl.SingletonMode.Ignore`: remove all singleton clusters
 - `bl.SingletonMode.AutoPopulate`: after reading the input clustering as-is, checks if there are nodes that are not assigned to any cluster. If so, for each node create a singleton cluster to house it. The new cluster will have a `NULL` label.

## Dummy Node Tolerance

Some clustering methods expect continuous node ids from the input graphs.
That is, if the input file has nodeset `{0, 3}`, then the clustering method
will actually create four nodes (`{0, 1, 2, 3}`) in total. These padded nodes are called "dummy nodes". First, Belinda does not create dummy nodes unlike some other software. Second, Belinda, when parsing clusters, actually actively *removes* these dummy nodes when seeing them.