# Filtering out Clusters

```python
import belinda as bl
import polars as pl

# the edgelist graph
g = bl.Graph("com-amazon.ungraph.txt")
# the membership as given by Leiden
c = bl.read_membership(g, "com-amazon.leiden.txt")
c_wo_trees = c.filter(pl.col('n') != pl.col('m') + 1)
# `c_wo_trees` is a new set of clusters without "trees", clusters with n = m + 1

# write the new membership
bl.write_membership(g, c_wo_trees, "com-amazon.leiden.wotrees.txt")

c_largish = c.filter(pl.col('n') > 10) # only take clusters with more than 10 nodes
bl.write_membership(g, c_largish, "com-amazon.leiden.largish.txt")
```

Now let's check that these files indeed do exist:
```python
>>!wc -l com-amazon.leiden.wotrees.txt
  230845 c_wo_trees.leiden.tsv
```