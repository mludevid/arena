#!/usr/bin/env python3

# before running this script you should run:
# COMPILE PROGRAM WITHOUT PROFILING
# $ ./tools/time_out.sh
# COMPILE PROGRAM WITH PROFILING
# $ ./out

import pandas as pd
import matplotlib.pyplot as plt

MILLION = 1000000

# Read time without profiling overhead:
f = open("time.time", "r")
lines = f.readlines()
user_time = float(lines[1].split(" ")[1])
sys_time = float(lines[2].split(" ")[1])
total_time = (user_time + sys_time) * MILLION
total_time_secs = (user_time + sys_time)

# Read log to create other metrics
df = pd.read_json("heap_profiling.json")
describe = df[['currently_allocated_bytes', 'stack_offset']].describe()
average_currently_allocated_bytes = describe['currently_allocated_bytes'].loc['mean']
average_stack_offset = describe['stack_offset'].loc['mean']
# print(describe.to_string())

# print("\nTotals:")
last_row = df.tail(1).to_dict()
total_pause_time = list(last_row["total_pause_ticks"].values())[0]
gc_overhead = total_pause_time / total_time
throughput = 1 - (gc_overhead)
total_pause_time_secs = list(last_row["total_pause_secs"].values())[0]
total_allocated_objects = list(last_row["total_allocated_objects"].values())[0]
total_allocated_bytes = list(last_row["total_allocated_bytes"].values())[0]
total_allocation_time = list(last_row["total_alloc_pause_ticks"].values())[0]
average_allocation_time = total_allocation_time / total_allocated_objects
# for key in last_row:
#     print(key + ": " + str(list(last_row[key].values())[0]))
# print("")

print("TOTAL TIME: " + str(total_time))
print("TOTAL PAUSE TIME: " + str(total_pause_time))
print("THROUGHPUT: " + str(throughput))
print("GC OVERHEAD: " + str(gc_overhead))
print("TOTAL ALLOCATED OBJECTS: " + str(total_allocated_objects))
print("TOTAL ALLOCATED BYTES: " + str(total_allocated_bytes))
print("AVERAGE OBJECT SIZE: " + str(total_allocated_bytes / total_allocated_objects))
print("AVERAGE CURRENTLY_ALLOCATED_BYTES: " + str(average_currently_allocated_bytes))
print("AVERAGE STACK_OFFSET: " + str(average_stack_offset))
print("AVERAGE ALLOCATION TIME: " + str(average_allocation_time))
