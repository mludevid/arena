#include <stdint.h>

// *******************
// ****** STACK ******
// *******************

#ifdef PROFILING_STACK
#define INIT_STACK_PROFILING()                              init_stack_profiling()
#define STACK_ALLOC_PROFILING(sp, segment_len, frequency)   stack_alloc_profiling(sp, segment_len, frequency)
#define CLOSE_STACK_PROFILING()                             close_stack_profiling()
#else
#define INIT_STACK_PROFILING()
#define STACK_ALLOC_PROFILING(sp, segment_len, frequency)
#define CLOSE_STACK_PROFILING()
#endif

void init_stack_profiling();
void stack_alloc_profiling(void *sp, uint64_t segment_len, uint64_t frequency);
void close_stack_profiling();

// ******************
// ****** HEAP ******
// ******************

#ifdef PROFILING_HEAP
#define INIT_HEAP_PROFILING()                                               init_heap_profiling()
#define HEAP_ALLOC_PTR_PROFILING(ptr)                                       heap_alloc_ptr_profiling(ptr)
#define HEAP_ALLOC_BYTES_PROFILING(len)                                     heap_alloc_bytes_profiling(len)
#define HEAP_FREE_PTR_PROFILING(ptr)                                        heap_free_ptr_profiling(ptr)
#define HEAP_FREE_BYTES_PROFILING(len)                                      heap_free_bytes_profiling(len)
#define HEAP_EVENT_START_PROFILING()                                        heap_event_start_profiling()
#define HEAP_EVENT_END_PROFILING(type, sp, segment_len_bits, frequency)     heap_event_end_profiling(type, sp, segment_len_bits, frequency)
#define CLOSE_HEAP_PROFILING()                                              close_heap_profiling()
#else
#define INIT_HEAP_PROFILING()
#define HEAP_ALLOC_PTR_PROFILING(ptr)
#define HEAP_ALLOC_BYTES_PROFILING(len)
#define HEAP_FREE_PTR_PROFILING(ptr)
#define HEAP_FREE_BYTES_PROFILING(len)
#define HEAP_EVENT_START_PROFILING()
#define HEAP_EVENT_END_PROFILING(type, sp, segment_len_bits, frequency)
#define CLOSE_HEAP_PROFILING()
#endif

#define TYPE_ALLOC 0
#define TYPE_FREE 1
#define PTR_ACCESS 2
#define PTR_DROP 3
#define TGC 4

void init_heap_profiling();
void heap_alloc_ptr_profiling(void* ptr);
void heap_alloc_bytes_profiling(uint64_t len);
void heap_free_ptr_profiling(void *ptr);
void heap_free_bytes_profiling(uint64_t len);
void heap_event_start_profiling();
void heap_event_end_profiling(uint64_t type, void *sp, int segment_len_bits, uint64_t frequency);
void close_heap_profiling();
