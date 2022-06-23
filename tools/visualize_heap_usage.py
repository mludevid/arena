#!/usr/bin/env python3

import pandas as pd
import matplotlib.pyplot as plt

df_arc = pd.read_json("heap_profiling_arc.json")
df_tgc = pd.read_json("heap_profiling_tgc.json")
df = df_arc.merge(df_tgc, how='outer', on="total_allocated_objects", suffixes=("_arc", "_tgc"))
df.plot(kind = 'line', x='total_allocated_objects', y=['currently_allocated_bytes_arc', 'currently_allocated_bytes_tgc'])
plt.show()
