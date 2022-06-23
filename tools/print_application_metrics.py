#!/usr/bin/env python3

import pandas as pd
import matplotlib.pyplot as plt

df = pd.read_json("heap_profiling.json")
print("Describe:")
print(df[['currently_allocated_bytes', 'stack_offset']].describe().to_string())
print("\nTotals:")
last_row = df.tail(1).to_dict()
for key in last_row:
    print(key + ": " + str(list(last_row[key].values())[0]))
