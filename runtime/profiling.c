#include <stdlib.h>
#include <stdint.h>
#include <stdio.h>
#include <time.h>
#include <malloc.h>
#include "profiling.h"

// *******************
// ****** STACK ******
// *******************

FILE *fp_stack;

void init_stack_profiling() {
    fp_stack = fopen("stack_profiling.json", "w");

    if (fp_stack == NULL) {
        printf("stack profiling log could not be opened\n");
        exit(1);
    }

    fprintf(fp_stack, "[\n{\"ticks\": 0, \"seconds\": 0.000000, \"stack_count\": 0, \"stack_offset\": 0}");
}

uint64_t get_stack_len(void *sp, uint64_t segment_len) {
    uint64_t current_offset = (((uint64_t)sp) & (segment_len - 1));
    uint64_t depth = 0;
    void *sp_in_previous_segment;
    void **previous_segment = (void**)(((uint64_t)sp) & ~(segment_len - 1));
    while (*previous_segment != NULL) {
        depth += 1;
        sp_in_previous_segment = *previous_segment;
        previous_segment = (void**)(((uint64_t)sp_in_previous_segment) & ~(segment_len - 1));
    }
    return (current_offset + depth * (segment_len - 2 * sizeof(void*))) / sizeof(void*);
}

uint64_t stack_profiling_count = 1;
void stack_alloc_profiling(void *sp, uint64_t segment_len, uint64_t frequency) {
    if (stack_profiling_count % frequency == 0) {
        uint64_t total_offset = get_stack_len(sp, segment_len);
        clock_t ticks = clock();
        fprintf(fp_stack, ",\n{\"ticks\": %ld, \"seconds\": %f, \"stack_count\": %ld, \"stack_offset\": %ld}", ticks, ((double)ticks) / CLOCKS_PER_SEC, stack_profiling_count, total_offset);
    }
    stack_profiling_count += 1;
}

void close_stack_profiling() {
    fprintf(fp_stack, "\n]\n");
    fclose(fp_stack);
}

// ******************
// ****** HEAP ******
// ******************

FILE *fp_heap;
uint64_t heap_profiling_count = 1;
uint64_t currently_allocated_bytes = 0;
uint64_t total_allocated_bytes = 0;
uint64_t total_allocated_objects = 0;

void init_heap_profiling() {
    fp_heap = fopen("heap_profiling.json", "w");

    if (fp_heap == NULL) {
        printf("heap profiling log could not be opened\n");
        exit(1);
    }

    setvbuf(fp_heap, NULL, _IOFBF, 16384);
    fprintf(fp_heap, "[\n{\"ticks\": 0, \"seconds\": 0.000000, \"total_pause_ticks\": 0, \"total_pause_secs\": 0, \"type\": -1, \"duration_ticks\": 0, \"total_alloc_pause_ticks\": 0, \"total_free_pause_ticks\": 0, \"total_ptr_access_pause_ticks\": 0, \"total_ptr_drop_pause_ticks\": 0, \"total_tgc_pause_ticks\": 0, \"total_allocated_bytes\": 0, \"total_allocated_objects\": 0, \"currently_allocated_bytes\": 0, \"allocated_bytes_diff\": 0, \"stack_offset\": 0}");
}

void heap_alloc_ptr_profiling(void* ptr) {
    currently_allocated_bytes += malloc_usable_size(ptr);
    total_allocated_bytes += malloc_usable_size(ptr);
    total_allocated_objects += 1;
}

void heap_alloc_bytes_profiling(uint64_t len) {
    currently_allocated_bytes += len;
    total_allocated_bytes += len;
    total_allocated_objects += 1;
}

void heap_free_ptr_profiling(void *ptr) {
    currently_allocated_bytes -= malloc_usable_size(ptr);
}

void heap_free_bytes_profiling(uint64_t len) {
    currently_allocated_bytes -= len;
}

clock_t heap_event_start;
uint64_t allocated_bytes_event_start;
void heap_event_start_profiling() {
    heap_event_start = clock();
    allocated_bytes_event_start = currently_allocated_bytes;
}

uint64_t total_pause_ticks;
uint64_t total_alloc_pause_ticks;
uint64_t total_free_pause_ticks;
uint64_t total_ptr_access_pause_ticks;
uint64_t total_ptr_drop_pause_ticks;
uint64_t total_tgc_pause_ticks;
void heap_event_end_profiling(uint64_t type, void *sp, int segment_len_bits, uint64_t frequency) {
    clock_t ticks = clock();
    clock_t duration = ticks - heap_event_start;
    total_pause_ticks += duration;
    if (type == TYPE_ALLOC) {
        total_alloc_pause_ticks += duration;
    } else if (type == TYPE_FREE) {
        total_free_pause_ticks += duration;
    } else if (type == PTR_ACCESS) {
        total_ptr_access_pause_ticks += duration;
    } else if (type == PTR_DROP) {
        total_ptr_drop_pause_ticks += duration;
    } else if (type == TGC) {
        total_tgc_pause_ticks += duration;
    }

    if (type == TGC || heap_profiling_count % frequency == 0) {
        uint32_t segment_len = (1 << segment_len_bits) * sizeof(void*);
        uint64_t total_offset = get_stack_len(sp, segment_len);

        uint64_t allocated_bytes_diff = currently_allocated_bytes - allocated_bytes_event_start;
        // CURRENT TIME INFO:
        fprintf(fp_heap, ",\n{\"ticks\": %ld, \"seconds\": %f, ", ticks, ((double)ticks) / CLOCKS_PER_SEC);
        // CURRENT PAUSE INFO:
        fprintf(fp_heap, "\"total_pause_ticks\": %ld, \"total_pause_secs\": %f, ", total_pause_ticks, ((double)total_pause_ticks) / CLOCKS_PER_SEC);
        // EVENT INFO:
        fprintf(fp_heap, "\"type\": %ld, \"duration_ticks\": %ld, ", type, duration);
        // CURRENT ACCUMULATED PAUSE INFO:
        fprintf(fp_heap, "\"total_alloc_pause_ticks\": %ld, ", total_alloc_pause_ticks);
        fprintf(fp_heap, "\"total_free_pause_ticks\": %ld, ", total_free_pause_ticks);
        fprintf(fp_heap, "\"total_ptr_access_pause_ticks\": %ld, ", total_ptr_access_pause_ticks);
        fprintf(fp_heap, "\"total_ptr_drop_pause_ticks\": %ld, ", total_ptr_drop_pause_ticks);
        fprintf(fp_heap, "\"total_tgc_pause_ticks\": %ld, ", total_tgc_pause_ticks);
        // HEAP INFO:
        fprintf(fp_heap, "\"total_allocated_bytes\": %ld, \"total_allocated_objects\": %ld, \"currently_allocated_bytes\": %ld, \"allocated_bytes_diff\": %ld, ", total_allocated_bytes, total_allocated_objects, currently_allocated_bytes, allocated_bytes_diff);
        // ADDITIONAL INFO:
        fprintf(fp_heap, "\"stack_offset\": %ld}", total_offset);
    }
    heap_profiling_count += 1;
}

void close_heap_profiling() {
    fprintf(fp_heap, "\n]\n");
    fclose(fp_heap);
}
