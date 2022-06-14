#include <stdint.h>

// *******************
// ****** STACK ******
// *******************

#ifdef PROFILING_STACK
#define INIT_STACK_PROFILING() init_stack_profiling()
#define STACK_ALLOC_PROFILING(sp, segment_len, frequency) stack_alloc_profiling(sp, segment_len, frequency)
#define CLOSE_STACK_PROFILING() close_stack_profiling()
#else
#define INIT_STACK_PROFILING()
#define STACK_ALLOC_PROFILING(sp, segment_len, frequency)
#define CLOSE_STACK_PROFILING()
#endif

void init_stack_profiling();
void stack_alloc_profiling(void *sp, uint64_t segment_len, uint64_t frequency);
void close_stack_profiling();
