#!/bin/bash

if [ $# -ne 1 ]; then
    echo "Please provide only one parameter"
else
    arena $1 -o out_arc && arena $1 --tgc -o out_tgc &&
    arena $1 --heap-profiling -p 10000 -o out_arc_prof && arena $1 --heap-profiling -p 10000 --tgc -o out_tgc_prof &&
    sudo perf record -g -o out_arc.data ./out_arc &> /dev/null && sudo perf report -i out_arc.data > arc_data.txt &&
    sudo perf record -g -o out_tgc.data ./out_tgc &> /dev/null && sudo perf report -i out_tgc.data > tgc_data.txt &&
    echo "ARC:" &&
    cat arc_data.txt | grep "stack_alloc\|type_alloc\|type_free\|arc_\|tgc_garbage_collection\|tgc_type_alloc" | grep "\[" &&
    echo "" &&
    sudo perf stat ./out_arc |& grep "task-clock\|branch\|user\|sys" &&
    echo "" &&
    ./out_arc_prof > /dev/null && python3 tools/compute_application_metrics.py &&
    echo "" &&
    echo "TGC:" &&
    cat tgc_data.txt | grep "stack_alloc\|type_alloc\|type_free\|arc_\|tgc_garbage_collection\|tgc_type_alloc\|copy_object" | grep "\[" &&
    echo "" &&
    sudo perf stat ./out_tgc |& grep "task-clock\|branch\|user\|sys" &&
    echo "" &&
    ./out_tgc_prof > /dev/null && python3 tools/compute_application_metrics.py
fi
