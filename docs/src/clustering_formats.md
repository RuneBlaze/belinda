# Clustering Formats and IO

## Membership format

A format where each line has a "membership assignment" in the form
`node_id<tab>cluster_id`, assigning a node to a specific cluster.

Example (note that I used spaces as tabs below):
```
0   1
1   1
2   1
3   2
4   2
5   2
```

The above clustering contains two clusters each of size three.

Used by Leiden and presumably other clustering methods. This is a very "raw" format. Notably,
it is very hard to individually annotate clusters to
have extra information (e.g. quality about that cluster).

### IO operations

 - `bl.read_membership(g, filename, sep = '\t', mode = bl.SingletonMode.AsIs)` reads the `sep` separated membership format.
 - `bl.write_membership(g, clus, filename)` writes the `clus` cluster data frame in membership format to `filename`.
 - `bl.read_membership_series(g, node_series, cluster_series, mode = bl.SingletonMode.AsIs)` takes the nodes (specified as `node_series`, a Polars `Series`) and the clusters correspondingly assigned (specified as `cluster_series`) and returns the cluster data frame. This is useful for parsing custom membership formats.
   - For example, `df = pl.read_csv("out.csv")` and then `bl.read_membership_series(g, df['node'], df['cluster'])` can be a good pairing


## JSON format

Designed by Belinda to be an easily consumable
and producible "sane" format for clustering.
This new-line delimited JSON format has each line
representing a cluster, and in its bare form like this:

```
{"label": 0, "nodes": [0, 1, 2]}
{"label": 1, "nodes": [3, 4, 5]}
```

Additional attributes are encouraged, for example, the following is also a valid cluster:
```
{"label": 0, "nodes": [0, 1, 2], "connectivity": 1}
```

The only hard requirements are:

 - Each object should have a "node" property of type `number[]` (but should in fact be integers)
 - Each object should have a "label" property of any type (that is internally consistent for the clustering)

Extra properties are supported and will be loaded into the data frame.

The JSON format is designed to be usable with tools such as `jq`.

### IO operations

  - `bl.read_json(g, filename, mode = bl.SingletonMode.AsIs)` reads the JSON format.
  - `bl.write_json(g, clus, filename)` writes the `clus` cluster data frame in JSON format to `filename`.