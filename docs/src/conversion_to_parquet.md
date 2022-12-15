# Conversion to Parquet

Since Polars data frames are Apache Arrow based, conversion to Parquet is quite easy and lossless, useful for storage.

```python
import belinda as bl
import polars as pl

g = bl.Graph("com-amazon.ungraph.txt")
c = bl.read_membership(g, "com-amazon.leiden.txt")

c.write_parquet("com-amazon.leiden.parquet")
```