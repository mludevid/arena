#!/usr/bin/env python3

import os
import sys
from pathlib import Path
import pandas as pd
import matplotlib.pyplot as plt

if len(sys.argv) != 2:
    print("Please provide exactly one result to visualize")
    exit(1)

path = Path(sys.argv[1])
if not path.is_file():
    print("Provided file does not exist")
    exit()

folder = path.parent
png_folder = folder.joinpath('results')

if not png_folder.is_dir():
    os.mkdir(png_folder)

df = pd.read_csv(sys.argv[1])

df['stack_alloc_time'] = (df['stack_alloc'] / 100) * df['total_time']
df['gc_overhead'] = df['arc_drop_ptr'] + df['arc_ptr_access'] + df['type_alloc'] + df['arc_free_obj'] + df['type_free']
df['throughput'] = 100 - df['gc_overhead']
df['gc_time'] = (df['gc_overhead'] / 100) * df['total_time']
df['throughput_time'] = (df['throughput'] / 100) * df['total_time']

df_arc = df[df['gc'] == 'arc']
df_tgc = df[df['gc'] == 'tgc']

# x-axis, y-axis, x-label, y-label, logx, logy
plots = [
    ('average_stack_offset', 'total_time', 'average stack size', 'total time [ms]', True, False),
    ('average_stack_offset', 'total_time_std', 'average stack size', 'total time', True, False),
    ('average_stack_offset', 'stack_alloc_time', 'average stack size', 'total stack allocation time [ms]', True, False),
    ('average_stack_offset', 'stack_alloc_time_std', 'average stack size', 'total stack allocation time', True, False),
    ('average_stack_offset', 'gc_time', 'average stack size', 'total gc time [ms]', True, False),
    ('average_stack_offset', 'gc_time_std', 'average stack size', 'total gc time', True, False),
    ('average_stack_offset', 'gc_overhead', 'average stack size', 'gc overhead [%]', True, False),
    ('average_stack_offset', 'throughput_time', 'average stack size', 'total time outside gc [ms]', True, False),
    ('average_stack_offset', 'throughput_time_std', 'average stack size', 'total time outside gc', True, False),

    ('average_object_size', 'total_time', 'average object size', 'total time [ms]', False, False),
    ('average_object_size', 'total_time_std', 'average object size', 'total time', False, False),
    ('average_object_size', 'stack_alloc_time', 'average object size', 'total stack allocation time [ms]', False, False),
    ('average_object_size', 'stack_alloc_time_std', 'average object size', 'total stack allocation time', False, False),
    ('average_object_size', 'gc_time', 'average object size', 'total gc time [ms]', False, False),
    ('average_object_size', 'gc_time_std', 'average object size', 'total gc time', False, False),
    ('average_object_size', 'gc_overhead', 'average object size', 'gc overhead [%]', False, False),
    ('average_object_size', 'throughput_time', 'average object size', 'total time outside gc [ms]', False, False),
    ('average_object_size', 'throughput_time_std', 'average object size', 'total time outside gc', False, False),

    ('average_currently_allocated_bytes', 'total_time', 'average allocated bytes', 'total time [ms]', False, False),
    ('average_currently_allocated_bytes', 'total_time_std', 'average allocated bytes', 'total time', False, False),
    ('average_currently_allocated_bytes', 'stack_alloc_time', 'average allocated bytes', 'total stack allocation time [ms]', False, False),
    ('average_currently_allocated_bytes', 'stack_alloc_time_std', 'average allocated bytes', 'total stack allocation time', False, False),
    ('average_currently_allocated_bytes', 'gc_time', 'average allocated bytes', 'total gc time [ms]', False, False),
    ('average_currently_allocated_bytes', 'gc_time_std', 'average allocated bytes', 'total gc time', False, False),
    ('average_currently_allocated_bytes', 'gc_overhead', 'average allocated bytes', 'gc overhead [%]', False, False),
    ('average_currently_allocated_bytes', 'throughput_time', 'average allocated bytes', 'total time outside gc [ms]', False, False),
    ('average_currently_allocated_bytes', 'throughput_time_std', 'average allocated bytes', 'total time outside gc', False, False),

    ('max_currently_allocated_bytes', 'total_time', 'max allocated bytes', 'total time [ms]', False, False),
    ('max_currently_allocated_bytes', 'total_time_std', 'max allocated bytes', 'total time', False, False),
    ('max_currently_allocated_bytes', 'stack_alloc_time', 'max allocated bytes', 'total stack allocation time [ms]', False, False),
    ('max_currently_allocated_bytes', 'stack_alloc_time_std', 'max allocated bytes', 'total stack allocation time', False, False),
    ('max_currently_allocated_bytes', 'gc_time', 'max allocated bytes', 'total gc time [ms]', False, False),
    ('max_currently_allocated_bytes', 'gc_time_std', 'max allocated bytes', 'total gc time', False, False),
    ('max_currently_allocated_bytes', 'gc_overhead', 'max allocated bytes', 'gc overhead [%]', False, False),
    ('max_currently_allocated_bytes', 'throughput_time', 'max allocated bytes', 'total time outside gc [ms]', False, False),
    ('max_currently_allocated_bytes', 'throughput_time_std', 'max allocated bytes', 'total time outside gc', False, False),
]

for (i, (x_data, y_data, x_label, y_label, log_x, log_y)) in enumerate(plots):
    df_arc_groupby = df_arc.groupby(x_data).mean().reset_index(level=0)
    df_tgc_groupby = df_tgc.groupby(x_data).mean().reset_index(level=0)

    arc_first_time = df_arc_groupby.iloc[0]['total_time']
    df_arc_groupby['total_time_std'] = df_arc_groupby['total_time'] / arc_first_time
    tgc_first_time = df_tgc_groupby.iloc[0]['total_time']
    df_tgc_groupby['total_time_std'] = df_tgc_groupby['total_time'] / tgc_first_time

    arc_first_time = df_arc_groupby.iloc[0]['stack_alloc_time']
    df_arc_groupby['stack_alloc_time_std'] = df_arc_groupby['stack_alloc_time'] / arc_first_time
    tgc_first_time = df_tgc_groupby.iloc[0]['stack_alloc_time']
    df_tgc_groupby['stack_alloc_time_std'] = df_tgc_groupby['stack_alloc_time'] / tgc_first_time

    arc_first_time = df_arc_groupby.iloc[0]['gc_time']
    df_arc_groupby['gc_time_std'] = df_arc_groupby['gc_time'] / arc_first_time
    tgc_first_time = df_tgc_groupby.iloc[0]['gc_time']
    df_tgc_groupby['gc_time_std'] = df_tgc_groupby['gc_time'] / tgc_first_time

    arc_first_time = df_arc_groupby.iloc[0]['throughput_time']
    df_arc_groupby['throughput_time_std'] = df_arc_groupby['throughput_time'] / arc_first_time
    tgc_first_time = df_tgc_groupby.iloc[0]['throughput_time']
    df_tgc_groupby['throughput_time_std'] = df_tgc_groupby['throughput_time'] / tgc_first_time

    ax = df_arc_groupby.plot(x=x_data, y=y_data, label='Reference Counting', logx=log_x, logy=log_y, xlabel=x_label, ylabel=y_label, marker='.')
    df_tgc_groupby.plot(ax=ax, x=x_data, y=y_data, label='Tracing GC', logx=log_x, logy=log_y, xlabel=x_label, ylabel=y_label, marker='.')
    plt.savefig(png_folder.joinpath('result' + str(i).zfill(3) + '.svg'), bbox_inches='tight')
    plt.close()

# plt.show()
