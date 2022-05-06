#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

#define SEGMENT_LEN_BITS 10 // => SEGMENT_LEN := 1024 pointers

void *init_stack() {
    int segment_len = (1 << SEGMENT_LEN_BITS) * sizeof(void*);
    // printf("STACK LEN: %d\n", segment_len);
    void* stack_start = aligned_alloc(segment_len, segment_len);
    *((void**)stack_start) = NULL;
    return stack_start;
}

void *stack_alloc(void *sp) {
    // static int count = 0;
    int segment_len = (1 << SEGMENT_LEN_BITS) * sizeof(void*);
    /*
    if (count >= 100) {
        printf("CURRENT STACK OFFSET: %d\n", (((uint64_t)sp) & (segment_len - 1)) / sizeof(void*));
        count = 0;
    } else {
        count++;
    }
    */
    if ((((uint64_t)sp) & (segment_len - 1)) >= segment_len - 1) {
        printf("STACK OVERFLOW!\n");
        exit(1);
    }
    return sp + sizeof(void*);
}
