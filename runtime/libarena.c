#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "profiling.h"

// *******************
// ****** STACK ******
// *******************

#define SEGMENT_LEN_BITS 5 // 10 // => SEGMENT_LEN := 1024 pointers

void *alloc_new_segment(void *previous_segment) {
    uint32_t segment_len = (1 << SEGMENT_LEN_BITS) * sizeof(void*);
    void* stack_start = aligned_alloc(segment_len, segment_len);
    *((void**)stack_start) = previous_segment;
    *((void**)(stack_start + segment_len - sizeof(void*))) = NULL;
    return stack_start;
}

void *init_stack() {
    INIT_STACK_PROFILING();
    return alloc_new_segment(NULL);
}

void *stack_alloc(void *sp, uint64_t profiling_frequency) {
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
    STACK_ALLOC_PROFILING(ret, segment_len, profiling_frequency);
    return ret;
}

void close_stack() {
    CLOSE_STACK_PROFILING();
}

// *****************
// ****** ARC ******
// *****************

// uint32_t allocated_objects = 0;

void *type_alloc(uint64_t size) {
    // printf("ALLOC\n");
    // allocated_objects += 1;
    return malloc(size);
}

void type_free(void *ptr) {
    // printf("FREE %x\n", ptr);
    // allocated_objects -= 1;
    free(ptr);
}

void close_heap() {
    /*
    if (allocated_objects > 0) {
        printf("%d allocated objects leaked\n", allocated_objects);
    }
    */
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
        for (uint32_t offset = 0; offset < pointer_count; offset++) {
            void* obj_ptr = *(((void**)(header + 2)) + offset);
            if (obj_ptr != NULL) {
                arc_drop_ptr(obj_ptr);
            }
        }

        // free this object
        type_free(ptr);
    }
}

// *****************
// ****** TGC ******
// *****************

#define NURSERY_LEN_BITS 15 // => NURSERY_LEN := 32768 bytes

void *nursery_active;
void *nursery_copy;
void *nursery_active_end;
void *nursery_copy_end;
void *nursery_pointer;

void tgc_init_heap() {
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

void tgc_garbage_collection(void *sp) {
    // printf("GARBAGE COLLECTION!\n");
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
}

void tgc_close_heap() {
    tgc_garbage_collection(NULL);
    if (nursery_active != nursery_pointer) {
        printf("NURSERY IS NOT EMPTY\n");
    }
}

void *tgc_type_alloc(uint64_t size, void *sp) {
    uint64_t padded_len = ((size + 7) / 8) * 8;
    if (nursery_pointer + padded_len >= nursery_active_end) {
        tgc_garbage_collection(sp);
    }
    void *ret = nursery_pointer;
    nursery_pointer = nursery_pointer + padded_len;
    return ret;
}
