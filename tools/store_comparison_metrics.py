#!/usr/bin/env python3

import os
import sys
from locale import atof, setlocale, LC_NUMERIC

if len(sys.argv) != 2:
    print("Please provide exactly one program to benchmark")
    exit(1)

setlocale(LC_NUMERIC, '')

stream = os.popen('./tools/benchmark.sh ' + sys.argv[1])
output = stream.read()
lines = output.split("\n")

gc = 'arc'
arc_drop_ptr = 0
arc_ptr_access = 0
stack_alloc = 0
type_alloc = 0
arc_free_obj = 0
tgc_garbage_collection = 0
type_free = 0
total_time = 0
total_allocated_objects = 0
total_allocated_bytes = 0
average_object_size = 0
average_currently_allocated_bytes = 0
max_currently_allocated_bytes = 0
average_stack_offset = 0

if lines[0] != "ARC:":
    print("Unexpected output")
    print(output)
    exit(1)

i = 1
while len(lines) > i and lines[i] != "TGC:":
    if "arc_drop_ptr" in lines[i]:
        arc_drop_ptr = float(lines[i].split("%")[0].split(" ")[-1])
    elif "arc_ptr_access" in lines[i]:
        arc_ptr_access = float(lines[i].split("%")[0].split(" ")[-1])
    elif "stack_alloc" in lines[i]:
        stack_alloc = float(lines[i].split("%")[0].split(" ")[-1])
    elif "type_alloc" in lines[i]:
        type_alloc = float(lines[i].split("%")[0].split(" ")[-1])
    elif "arc_free_obj" in lines[i]:
        arc_free_obj = float(lines[i].split("%")[0].split(" ")[-1])
    elif "type_free" in lines[i]:
        type_free = float(lines[i].split("%")[0].split(" ")[-1])
    elif "tgc_garbage_collection" in lines[i]:
        tgc_garbage_collection = float(lines[i].split("%")[0].split(" ")[-1])
    elif "task-clock" in lines[i]:
        total_time = atof(lines[i].split(" msec")[0].split(" ")[-1])
    elif "TOTAL ALLOCATED OBJECTS" in lines[i]:
        total_allocated_objects = atof(lines[i].split(": ")[1])
    elif "TOTAL ALLOCATED BYTES" in lines[i]:
        total_allocated_bytes = atof(lines[i].split(": ")[1])
    elif "AVERAGE OBJECT SIZE" in lines[i]:
        average_object_size = atof(lines[i].split(": ")[1])
    elif "AVERAGE CURRENTLY_ALLOCATED_BYTES" in lines[i]:
        average_currently_allocated_bytes = atof(lines[i].split(": ")[1])
    elif "MAX CURRENTLY_ALLOCATED_BYTES" in lines[i]:
        max_currently_allocated_bytes = atof(lines[i].split(": ")[1])
    elif "AVERAGE STACK_OFFSET" in lines[i]:
        average_stack_offset = atof(lines[i].split(": ")[1])
    i += 1

print(f"{gc},{arc_drop_ptr:.2f},{arc_ptr_access:.2f},{stack_alloc:.2f},{type_alloc:.2f},{arc_free_obj:.2f}," +
        f"{tgc_garbage_collection:.2f},{type_free:.2f},{total_time:.2f},{total_allocated_objects:.2f}," +
        f"{total_allocated_bytes:.2f},{average_object_size:.2f},{average_currently_allocated_bytes:.2f},{max_currently_allocated_bytes:.2f},"+
        f"{average_stack_offset:.2f}")

gc = 'tgc'
arc_drop_ptr = 0
arc_ptr_access = 0
stack_alloc = 0
type_alloc = 0
arc_free_obj = 0
tgc_garbage_collection = 0
type_free = 0
total_time = 0
total_allocated_objects = 0
total_allocated_bytes = 0
average_object_size = 0
average_currently_allocated_bytes = 0
max_currently_allocated_bytes = 0
average_stack_offset = 0

while len(lines) > i:
    if "arc_drop_ptr" in lines[i]:
        arc_drop_ptr = float(lines[i].split("%")[0].split(" ")[-1])
    elif "arc_ptr_access" in lines[i]:
        arc_ptr_access = float(lines[i].split("%")[0].split(" ")[-1])
    elif "stack_alloc" in lines[i]:
        stack_alloc = float(lines[i].split("%")[0].split(" ")[-1])
    elif "type_alloc" in lines[i]:
        type_alloc = float(lines[i].split("%")[0].split(" ")[-1])
    elif "arc_free_obj" in lines[i]:
        arc_free_obj = float(lines[i].split("%")[0].split(" ")[-1])
    elif "type_free" in lines[i]:
        type_free = float(lines[i].split("%")[0].split(" ")[-1])
    elif "tgc_garbage_collection" in lines[i]:
        tgc_garbage_collection = float(lines[i].split("%")[0].split(" ")[-1])
    elif "copy_object" in lines[i]:
        copy_object = float(lines[i].split("%")[0].split(" ")[-1])
    elif "task-clock" in lines[i]:
        total_time = atof(lines[i].split(" msec")[0].split(" ")[-1])
    elif "TOTAL ALLOCATED OBJECTS" in lines[i]:
        total_allocated_objects = atof(lines[i].split(": ")[1])
    elif "TOTAL ALLOCATED BYTES" in lines[i]:
        total_allocated_bytes = atof(lines[i].split(": ")[1])
    elif "AVERAGE OBJECT SIZE" in lines[i]:
        average_object_size = atof(lines[i].split(": ")[1])
    elif "AVERAGE CURRENTLY_ALLOCATED_BYTES" in lines[i]:
        average_currently_allocated_bytes = atof(lines[i].split(": ")[1])
    elif "MAX CURRENTLY_ALLOCATED_BYTES" in lines[i]:
        max_currently_allocated_bytes = atof(lines[i].split(": ")[1])
    elif "AVERAGE STACK_OFFSET" in lines[i]:
        average_stack_offset = atof(lines[i].split(": ")[1])
    i += 1

if copy_object > type_alloc:
    # When there are very deep objects allocated on the heap the function copy_object
    # gets called recursively in libarena.c very often leading to perf missing out
    # on adding the time spend in copy_object to tgc_garbage_collection and consequently to type_alloc
    type_alloc = copy_object
    tgc_garbage_collection = copy_object

print(f"{gc},{arc_drop_ptr:.2f},{arc_ptr_access:.2f},{stack_alloc:.2f},{type_alloc:.2f},{arc_free_obj:.2f}," +
        f"{tgc_garbage_collection:.2f},{type_free:.2f},{total_time:.2f},{total_allocated_objects:.2f}," +
        f"{total_allocated_bytes:.2f},{average_object_size:.2f},{average_currently_allocated_bytes:.2f},{max_currently_allocated_bytes:.2f},"+
        f"{average_stack_offset:.2f}")
