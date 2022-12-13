# Clustering Formats

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