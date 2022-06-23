#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "profiling.h"

// *******************
// ****** STACK ******
// *******************

#define SEGMENT_LEN_BITS 5 // 10 // => SEGMENT_LEN := 1024 pointers

uint64_t PROFILING_FREQUENCY;

void *alloc_new_segment(void *previous_segment) {
    uint32_t segment_len = (1 << SEGMENT_LEN_BITS) * sizeof(void*);
    void* stack_start = aligned_alloc(segment_len, segment_len);
    *((void**)stack_start) = previous_segment;
    *((void**)(stack_start + segment_len - sizeof(void*))) = NULL;
    return stack_start;
}

void *init_stack(uint64_t profiling_frequency) {
    PROFILING_FREQUENCY = profiling_frequency;
    INIT_STACK_PROFILING();
    return alloc_new_segment(NULL);
}

void *stack_alloc(void *sp) {
    uint64_t segment_len = (1 << SEGMENT_LEN_BITS) * sizeof(void*);
    void *ret;
    if ((((uint64_t)sp) & (segment_len - 1)) == segment_len - 2 * sizeof(void*)) {
        if (*((void**)(sp + sizeof(void*))) == NULL) {
            ret = alloc_new_segment(sp) + sizeof(void*);
            *((void**)(sp + sizeof(void*))) = ret;
        } else {
            ret = *((void**)(sp + sizeof(void*)));
        }
    } else {
        ret = sp + sizeof(void*);
    }
    STACK_ALLOC_PROFILING(ret, segment_len, PROFILING_FREQUENCY);
    return ret;
}

void close_stack() {
    CLOSE_STACK_PROFILING();
}

// *************************
// ****** SPILL / ARC ******
// *************************

void init_heap() {
    INIT_HEAP_PROFILING();
}

void *type_alloc(uint64_t size, void *sp) {
    HEAP_EVENT_START_PROFILING();

    void* ptr = malloc(size);

    HEAP_ALLOC_PTR_PROFILING(ptr);
    HEAP_EVENT_END_PROFILING(TYPE_ALLOC, sp, SEGMENT_LEN_BITS, PROFILING_FREQUENCY);
    return ptr;
}

void type_free(void *ptr, void *sp) {
    HEAP_FREE_PTR_PROFILING(ptr);

    free(ptr);
}

void close_heap() {
    CLOSE_HEAP_PROFILING();
}

// *****************
// ****** ARC ******
// *****************

// uint64_t ptr_access_count = 0;
void arc_ptr_access(void *ptr, void *sp) {
    /*
    ptr_access_count += 1;
    if (ptr_access_count % 10000000 == 0) {
        printf("PTR ACCESS COUNT: %ld\n", ptr_access_count);
    }
    */

    HEAP_EVENT_START_PROFILING();
    uint32_t *header = (uint32_t *)ptr;
    *header = *header + 1;
    HEAP_EVENT_END_PROFILING(PTR_ACCESS, sp, SEGMENT_LEN_BITS, PROFILING_FREQUENCY);
}

void arc_drop_ptr(void *ptr, void *sp) {
    /*
    ptr_access_count += 1;
    if (ptr_access_count % 10000000 == 0) {
        printf("PTR ACCESS COUNT: %ld\n", ptr_access_count);
    }
    */

    HEAP_EVENT_START_PROFILING();
    uint32_t *header = (uint32_t *)ptr;
    *header = *header - 1;
    HEAP_EVENT_END_PROFILING(PTR_DROP, sp, SEGMENT_LEN_BITS, PROFILING_FREQUENCY);

    if (*header == 0) {
        HEAP_EVENT_START_PROFILING();

        uint32_t pointer_count = *(header + 1) >> 16;
        for (uint32_t offset = 0; offset < pointer_count; offset++) {
            void* obj_ptr = *(((void**)(header + 2)) + offset);
            if (obj_ptr != NULL) {
                arc_drop_ptr(obj_ptr, sp);
            }
        }

        // free this object
        type_free(ptr, sp);

        HEAP_EVENT_END_PROFILING(TYPE_FREE, sp, SEGMENT_LEN_BITS, PROFILING_FREQUENCY);
    }
}

// *****************
// ****** TGC ******
// *****************

// #define NURSERY_LEN_BITS 15 // => NURSERY_LEN := 32768 bytes
#define NURSERY_LEN_BITS 13

void *nursery_active;
void *nursery_copy;
void *nursery_active_end;
void *nursery_copy_end;
void *nursery_pointer;

void tgc_init_heap() {
    INIT_HEAP_PROFILING();

    uint32_t nursery_len = (1 << NURSERY_LEN_BITS);
    nursery_active = malloc(nursery_len);
    nursery_copy = malloc(nursery_len);
    nursery_active_end= nursery_active + nursery_len;
    nursery_copy_end = nursery_copy + nursery_len;
    nursery_pointer = nursery_active;
}

// Copies the object to the copy heap if that hasn't happened yet and returns
// the new address
void *copy_object(void *obj) {
    if (*((uint32_t*)obj) != 0xFFFFFFFF) {
        void *ptr = nursery_pointer;
        // Copy object to new heap
        uint32_t obj_len = *((uint32_t*)obj);
        uint64_t padded_obj_len = ((obj_len + 7) / 8) * 8;
        memcpy(ptr, obj, padded_obj_len);
        nursery_pointer = nursery_pointer + padded_obj_len;

        // Overwrite current location with forwarding pointer
        *((uint32_t*)obj) = 0xFFFFFFFF;
        uint32_t offset = (uint32_t)(ptr - nursery_copy);
        *((uint32_t*)(obj + sizeof(uint32_t))) = offset;


        // Recursive call to pointers in object
        uint32_t obj_header = *((uint32_t*)(ptr + sizeof(uint32_t)));
        uint32_t rec_count = obj_header >> 16;
        for (uint32_t pointer = 0; pointer < rec_count; pointer++) {
            void* obj_ptr = ptr + 2 * sizeof(uint32_t) + pointer * sizeof(void*);
            if (*((void**)obj_ptr) != NULL) {
                *((void**)obj_ptr) = copy_object(*((void**)obj_ptr));
            }
        }

        return ptr;
    } else {
        // obj was already moved. return address stored.
        return nursery_copy + *((uint32_t*)(obj + sizeof(uint32_t)));
    }
}

void nursery_garbage_collection(void *current_sp) {
    uint32_t stack_segment_len = (1 << SEGMENT_LEN_BITS) * sizeof(void*);
    void *sp = current_sp;
    while ((((uint64_t)sp) & (stack_segment_len - 1)) != 0) {
        // Update adress in stack with new address of object
        *((void**)sp) = copy_object(*((void**)sp));
        sp = sp - sizeof(void*);
    }
    void *prev_sp = *((void**) sp);
    if (prev_sp != NULL) {
        nursery_garbage_collection(prev_sp);
    }
}

// Returns bytes freed
uint64_t tgc_garbage_collection(void *sp) {
    uint64_t len_before = nursery_pointer - nursery_active;

    nursery_pointer = nursery_copy;
    if (sp != NULL) {
        // SP == NULL for final cleanup otherwise it will be a valid pointer
        nursery_garbage_collection(sp);
    }
    void *tmp = nursery_active;
    nursery_active = nursery_copy;
    nursery_copy = tmp;
    tmp = nursery_active_end;
    nursery_active_end = nursery_copy_end;
    nursery_copy_end = tmp;

    uint64_t len_after = nursery_pointer - nursery_active;
    return len_before - len_after;
}

void tgc_close_heap() {
    tgc_garbage_collection(NULL);
    if (nursery_active != nursery_pointer) {
        printf("NURSERY IS NOT EMPTY\n");
    }
    CLOSE_HEAP_PROFILING();
}

void *tgc_type_alloc(uint64_t size, void *sp) {
    uint64_t padded_len = ((size + 7) / 8) * 8;
    if (nursery_pointer + padded_len >= nursery_active_end) {
        HEAP_EVENT_START_PROFILING();

        uint64_t bytes_freed = tgc_garbage_collection(sp);

        HEAP_FREE_BYTES_PROFILING(bytes_freed);
        HEAP_EVENT_END_PROFILING(TGC, sp, SEGMENT_LEN_BITS, PROFILING_FREQUENCY);
    }

    HEAP_EVENT_START_PROFILING();

    void *ret = nursery_pointer;
    nursery_pointer = nursery_pointer + padded_len;

    HEAP_ALLOC_BYTES_PROFILING(padded_len);
    HEAP_EVENT_END_PROFILING(TYPE_ALLOC, sp, SEGMENT_LEN_BITS, PROFILING_FREQUENCY);
    return ret;
}
