#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

#define SEGMENT_LEN_BITS 10 // => SEGMENT_LEN := 1024 pointers

uint32_t max_stack = 0;

void *init_stack() {
    uint32_t segment_len = (1 << SEGMENT_LEN_BITS) * sizeof(void*);
    // printf("STACK LEN: %d\n", segment_len);
    void* stack_start = aligned_alloc(segment_len, segment_len);
    *((void**)stack_start) = NULL;
    return stack_start;
}

void *stack_alloc(void *sp) {
    uint32_t segment_len = (1 << SEGMENT_LEN_BITS) * sizeof(void*);
    if ((((uint64_t)sp) & (segment_len - 1)) / sizeof(void*) > max_stack) {
        max_stack = (((uint64_t)sp) & (segment_len - 1)) / sizeof(void*);
    }
    if ((((uint64_t)sp) & (segment_len - 1)) == segment_len - sizeof(void*)) {
        printf("STACK OVERFLOW!\n");
        exit(1);
    }
    return sp + sizeof(void*);
}

void close_stack() {
    // printf("MAX STACK LEN: %d\n", max_stack);
}

uint32_t allocated_objects = 0;

void *type_alloc(uint64_t size) {
    // printf("ALLOC\n");
    allocated_objects += 1;
    return malloc(size);
}

void type_free(void *ptr) {
    // printf("FREE %x\n", ptr);
    allocated_objects -= 1;
    free(ptr);
}

void close_heap() {
    if (allocated_objects > 0) {
        printf("%d allocated objects leaked\n", allocated_objects);
    }
}

void arc_ptr_access(void *ptr) {
    // printf("PTR ACCESS\n");
    uint32_t *header = (uint32_t *)ptr;
    *header = *header + 1;
    // printf("ARC COUNTER INCREASED %x: %u\n", header, *header);
}

void arc_drop_ptr(void *ptr) {
    // printf("PTR DORP\n");
    uint32_t *header = (uint32_t *)ptr;
    *header = *header - 1;
    if (*header == 0) {
        uint32_t pointer_count = *(header + 1) >> 16;
        for (int offset = 0; offset < pointer_count; offset++) {
            void* obj_ptr = *(((void**)(header + 2)) + offset);
            if (obj_ptr != NULL) {
                arc_drop_ptr(obj_ptr);
            }
        }

        // free this object
        type_free(ptr);
    }
}
