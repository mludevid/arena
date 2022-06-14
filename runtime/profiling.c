#include <stdlib.h>
#include <stdint.h>
#include <stdio.h>
#include <time.h>

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
