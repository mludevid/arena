#!/usr/bin/env python3

import os

BENCHMARKING_REPETITION = 10

commands = [
    (1, 'echo STARTING...'),

    (1, 'rm -f benchmarks/nqueens_stack/results.csv'),
    (1, 'cp ./tools/results_template.csv benchmarks/nqueens_stack/results.csv'),
    (BENCHMARKING_REPETITION, './tools/store_comparison_metrics.py benchmarks/nqueens_stack/NQueenProblem.arena >> benchmarks/nqueens_stack/results.csv'),
    (BENCHMARKING_REPETITION, './tools/store_comparison_metrics.py benchmarks/nqueens_stack/NQueenProblem_10.arena >> benchmarks/nqueens_stack/results.csv'),
    (BENCHMARKING_REPETITION, './tools/store_comparison_metrics.py benchmarks/nqueens_stack/NQueenProblem_100.arena >> benchmarks/nqueens_stack/results.csv'),
    (BENCHMARKING_REPETITION, './tools/store_comparison_metrics.py benchmarks/nqueens_stack/NQueenProblem_500.arena >> benchmarks/nqueens_stack/results.csv'),
    (BENCHMARKING_REPETITION, './tools/store_comparison_metrics.py benchmarks/nqueens_stack/NQueenProblem_1000.arena >> benchmarks/nqueens_stack/results.csv'),
    (BENCHMARKING_REPETITION, './tools/store_comparison_metrics.py benchmarks/nqueens_stack/NQueenProblem_5000.arena >> benchmarks/nqueens_stack/results.csv'),
    (BENCHMARKING_REPETITION, './tools/store_comparison_metrics.py benchmarks/nqueens_stack/NQueenProblem_10000.arena >> benchmarks/nqueens_stack/results.csv'),
    (BENCHMARKING_REPETITION, './tools/store_comparison_metrics.py benchmarks/nqueens_stack/NQueenProblem_50000.arena >> benchmarks/nqueens_stack/results.csv'),
    (BENCHMARKING_REPETITION, './tools/store_comparison_metrics.py benchmarks/nqueens_stack/NQueenProblem_100000.arena >> benchmarks/nqueens_stack/results.csv'),
    (1, './tools/visualize_comparison_results.py benchmarks/nqueens_stack/results.csv'),

    (1, 'rm -f benchmarks/nqueens_objects/results.csv'),
    (1, 'cp ./tools/results_template.csv benchmarks/nqueens_objects/results.csv'),
    (BENCHMARKING_REPETITION, './tools/store_comparison_metrics.py benchmarks/nqueens_objects/NQueenProblem.arena >> benchmarks/nqueens_objects/results.csv'),
    (BENCHMARKING_REPETITION, './tools/store_comparison_metrics.py benchmarks/nqueens_objects/NQueenProblem_8.arena >> benchmarks/nqueens_objects/results.csv'),
    (BENCHMARKING_REPETITION, './tools/store_comparison_metrics.py benchmarks/nqueens_objects/NQueenProblem_16.arena >> benchmarks/nqueens_objects/results.csv'),
    (BENCHMARKING_REPETITION, './tools/store_comparison_metrics.py benchmarks/nqueens_objects/NQueenProblem_24.arena >> benchmarks/nqueens_objects/results.csv'),
    (BENCHMARKING_REPETITION, './tools/store_comparison_metrics.py benchmarks/nqueens_objects/NQueenProblem_32.arena >> benchmarks/nqueens_objects/results.csv'),
    (BENCHMARKING_REPETITION, './tools/store_comparison_metrics.py benchmarks/nqueens_objects/NQueenProblem_40.arena >> benchmarks/nqueens_objects/results.csv'),
    (1, './tools/visualize_comparison_results.py benchmarks/nqueens_objects/results.csv'),

    (1, 'rm -f benchmarks/nqueens_heap/results.csv'),
    (1, 'cp ./tools/results_template.csv benchmarks/nqueens_heap/results.csv'),
    (BENCHMARKING_REPETITION, './tools/store_comparison_metrics.py benchmarks/nqueens_heap/NQueenProblem.arena >> benchmarks/nqueens_heap/results.csv'),
    (BENCHMARKING_REPETITION, './tools/store_comparison_metrics.py benchmarks/nqueens_heap/NQueenProblem_100.arena >> benchmarks/nqueens_heap/results.csv'),
    (BENCHMARKING_REPETITION, './tools/store_comparison_metrics.py benchmarks/nqueens_heap/NQueenProblem_200.arena >> benchmarks/nqueens_heap/results.csv'),
    (BENCHMARKING_REPETITION, './tools/store_comparison_metrics.py benchmarks/nqueens_heap/NQueenProblem_300.arena >> benchmarks/nqueens_heap/results.csv'),
    (BENCHMARKING_REPETITION, './tools/store_comparison_metrics.py benchmarks/nqueens_heap/NQueenProblem_400.arena >> benchmarks/nqueens_heap/results.csv'),
    (BENCHMARKING_REPETITION, './tools/store_comparison_metrics.py benchmarks/nqueens_heap/NQueenProblem_500.arena >> benchmarks/nqueens_heap/results.csv'),
    (BENCHMARKING_REPETITION, './tools/store_comparison_metrics.py benchmarks/nqueens_heap/NQueenProblem_600.arena >> benchmarks/nqueens_heap/results.csv'),
    (BENCHMARKING_REPETITION, './tools/store_comparison_metrics.py benchmarks/nqueens_heap/NQueenProblem_700.arena >> benchmarks/nqueens_heap/results.csv'),
    (BENCHMARKING_REPETITION, './tools/store_comparison_metrics.py benchmarks/nqueens_heap/NQueenProblem_800.arena >> benchmarks/nqueens_heap/results.csv'),
    (BENCHMARKING_REPETITION, './tools/store_comparison_metrics.py benchmarks/nqueens_heap/NQueenProblem_900.arena >> benchmarks/nqueens_heap/results.csv'),
    (BENCHMARKING_REPETITION, './tools/store_comparison_metrics.py benchmarks/nqueens_heap/NQueenProblem_1000.arena >> benchmarks/nqueens_heap/results.csv'),
    (BENCHMARKING_REPETITION, './tools/store_comparison_metrics.py benchmarks/nqueens_heap/NQueenProblem_1100.arena >> benchmarks/nqueens_heap/results.csv'),
    (1, './tools/visualize_comparison_results.py benchmarks/nqueens_heap/results.csv'),

    (1, 'rm -f benchmarks/sorting_stack/results.csv'),
    (1, 'cp ./tools/results_template.csv benchmarks/sorting_stack/results.csv'),
    (BENCHMARKING_REPETITION, './tools/store_comparison_metrics.py benchmarks/sorting_stack/Sorting.arena >> benchmarks/sorting_stack/results.csv'),
    (BENCHMARKING_REPETITION, './tools/store_comparison_metrics.py benchmarks/sorting_stack/Sorting_10.arena >> benchmarks/sorting_stack/results.csv'),
    (BENCHMARKING_REPETITION, './tools/store_comparison_metrics.py benchmarks/sorting_stack/Sorting_100.arena >> benchmarks/sorting_stack/results.csv'),
    (BENCHMARKING_REPETITION, './tools/store_comparison_metrics.py benchmarks/sorting_stack/Sorting_500.arena >> benchmarks/sorting_stack/results.csv'),
    (BENCHMARKING_REPETITION, './tools/store_comparison_metrics.py benchmarks/sorting_stack/Sorting_1000.arena >> benchmarks/sorting_stack/results.csv'),
    (BENCHMARKING_REPETITION, './tools/store_comparison_metrics.py benchmarks/sorting_stack/Sorting_5000.arena >> benchmarks/sorting_stack/results.csv'),
    (BENCHMARKING_REPETITION, './tools/store_comparison_metrics.py benchmarks/sorting_stack/Sorting_10000.arena >> benchmarks/sorting_stack/results.csv'),
    (BENCHMARKING_REPETITION, './tools/store_comparison_metrics.py benchmarks/sorting_stack/Sorting_50000.arena >> benchmarks/sorting_stack/results.csv'),
    (BENCHMARKING_REPETITION, './tools/store_comparison_metrics.py benchmarks/sorting_stack/Sorting_100000.arena >> benchmarks/sorting_stack/results.csv'),
    (1, './tools/visualize_comparison_results.py benchmarks/sorting_stack/results.csv'),

    (1, 'rm -f benchmarks/sorting_objects/results.csv'),
    (1, 'cp ./tools/results_template.csv benchmarks/sorting_objects/results.csv'),
    (BENCHMARKING_REPETITION, './tools/store_comparison_metrics.py benchmarks/sorting_objects/Sorting.arena >> benchmarks/sorting_objects/results.csv'),
    (BENCHMARKING_REPETITION, './tools/store_comparison_metrics.py benchmarks/sorting_objects/Sorting_8.arena >> benchmarks/sorting_objects/results.csv'),
    (BENCHMARKING_REPETITION, './tools/store_comparison_metrics.py benchmarks/sorting_objects/Sorting_16.arena >> benchmarks/sorting_objects/results.csv'),
    (BENCHMARKING_REPETITION, './tools/store_comparison_metrics.py benchmarks/sorting_objects/Sorting_24.arena >> benchmarks/sorting_objects/results.csv'),
    (BENCHMARKING_REPETITION, './tools/store_comparison_metrics.py benchmarks/sorting_objects/Sorting_32.arena >> benchmarks/sorting_objects/results.csv'),
    (BENCHMARKING_REPETITION, './tools/store_comparison_metrics.py benchmarks/sorting_objects/Sorting_40.arena >> benchmarks/sorting_objects/results.csv'),
    (1, './tools/visualize_comparison_results.py benchmarks/sorting_objects/results.csv'),

    (1, 'rm -f benchmarks/sorting_heap/results.csv'),
    (1, 'cp ./tools/results_template.csv benchmarks/sorting_heap/results.csv'),
    (BENCHMARKING_REPETITION, './tools/store_comparison_metrics.py benchmarks/sorting_heap/Sorting.arena >> benchmarks/sorting_heap/results.csv'),
    (BENCHMARKING_REPETITION, './tools/store_comparison_metrics.py benchmarks/sorting_heap/Sorting_100.arena >> benchmarks/sorting_heap/results.csv'),
    (BENCHMARKING_REPETITION, './tools/store_comparison_metrics.py benchmarks/sorting_heap/Sorting_200.arena >> benchmarks/sorting_heap/results.csv'),
    (BENCHMARKING_REPETITION, './tools/store_comparison_metrics.py benchmarks/sorting_heap/Sorting_300.arena >> benchmarks/sorting_heap/results.csv'),
    (BENCHMARKING_REPETITION, './tools/store_comparison_metrics.py benchmarks/sorting_heap/Sorting_400.arena >> benchmarks/sorting_heap/results.csv'),
    (BENCHMARKING_REPETITION, './tools/store_comparison_metrics.py benchmarks/sorting_heap/Sorting_500.arena >> benchmarks/sorting_heap/results.csv'),
    (BENCHMARKING_REPETITION, './tools/store_comparison_metrics.py benchmarks/sorting_heap/Sorting_600.arena >> benchmarks/sorting_heap/results.csv'),
    (BENCHMARKING_REPETITION, './tools/store_comparison_metrics.py benchmarks/sorting_heap/Sorting_700.arena >> benchmarks/sorting_heap/results.csv'),
    (BENCHMARKING_REPETITION, './tools/store_comparison_metrics.py benchmarks/sorting_heap/Sorting_800.arena >> benchmarks/sorting_heap/results.csv'),
    (1, './tools/visualize_comparison_results.py benchmarks/sorting_heap/results.csv'),
]

print('About to run:')
for (times, command) in commands:
    for _ in range(times):
        print('  $ ' + command)

input("Press ENTER to start")

for i, (times, command) in enumerate(commands):
    print(str(i+1) + ' / ' + str(len(commands)))
    for time in range(times):
        print('  ' + str(time + 1) + ' / ' + str(times))
        stream = os.popen(command)
        print(stream.read())
