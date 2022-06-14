#!/usr/bin/env python3

import pandas as pd
import matplotlib.pyplot as plt

df = pd.read_json("stack_profiling.json")
df.plot(kind = 'scatter', x='stack_count', y='stack_offset')
plt.show()
