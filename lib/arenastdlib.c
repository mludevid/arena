#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

#define SEGMENT_LEN_BITS 10 // => SEGMENT_LEN := 1024 pointers

// int max_stack;

void *init_stack() {
    // max_stack = 0;
    int segment_len = (1 << SEGMENT_LEN_BITS) * sizeof(void*);
    // printf("STACK LEN: %d\n", segment_len);
    void* stack_start = aligned_alloc(segment_len, segment_len);
    *((void**)stack_start) = NULL;
    return stack_start;
}

void *stack_alloc(void *sp) {
    int segment_len = (1 << SEGMENT_LEN_BITS) * sizeof(void*);
    /*
    if ((((uint64_t)sp) & (segment_len - 1)) / sizeof(void*) > max_stack) {
        max_stack = (((uint64_t)sp) & (segment_len - 1)) / sizeof(void*);
        printf("NEW MAX STACK RECORD: %d\n", max_stack);
    }
    */
    if ((((uint64_t)sp) & (segment_len - 1)) == segment_len - sizeof(void*)) {
        printf("STACK OVERFLOW!\n");
        exit(1);
    }
    return sp + sizeof(void*);
}

void *type_alloc(uint64_t size) {
    return malloc(size);
}
