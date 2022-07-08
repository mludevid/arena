#!/usr/bin/env python3

import pandas as pd
import locale
from locale import setlocale, LC_NUMERIC

setlocale(LC_NUMERIC, '')

# Read log to create other metrics
df = pd.read_json("heap_profiling.json")
describe = df[['currently_allocated_bytes', 'stack_offset']].describe()
average_currently_allocated_bytes = describe['currently_allocated_bytes'].loc['mean']
max_currently_allocated_bytes = describe['currently_allocated_bytes'].loc['max']
average_stack_offset = describe['stack_offset'].loc['mean']

last_row = df.tail(1).to_dict()
total_allocated_objects = list(last_row["total_allocated_objects"].values())[0]
total_allocated_bytes = list(last_row["total_allocated_bytes"].values())[0]

print("TOTAL ALLOCATED OBJECTS: " + locale.str(total_allocated_objects))
print("TOTAL ALLOCATED BYTES: " + locale.str(total_allocated_bytes))
print("AVERAGE OBJECT SIZE: " + locale.str(total_allocated_bytes / total_allocated_objects))
print("AVERAGE CURRENTLY_ALLOCATED_BYTES: " + locale.str(average_currently_allocated_bytes))
print("MAX CURRENTLY_ALLOCATED_BYTES: " + locale.str(max_currently_allocated_bytes))
print("AVERAGE STACK_OFFSET: " + locale.str(average_stack_offset))
