# Filtering out Clusters

```python
import belinda as bl
import polars as pl

g = bl.Graph("com-amazon.ungraph.txt") # the edgelist graph
c = bl.read_membership(g, "com-amazon.leiden.txt") # the membership as given by Leiden
c_wo_trees = c.filter(pl.col('n') != pl.col('m') + 1) # trees: those clusters with n = m + 1
bl.write_membership(g, c_wo_trees, "com-amazon.leiden.wotrees.txt") # write the new membership

c_largish = c.filter(pl.col('n') > 10) # clusters with more than 10 nodes
bl.write_membership(g, c_largish, "com-amazon.leiden.largish.txt")
```

Now let's check that these files indeed do exist:
```python
>>!wc -l com-amazon.leiden.wotrees.txt
  230845 c_wo_trees.leiden.tsv
```