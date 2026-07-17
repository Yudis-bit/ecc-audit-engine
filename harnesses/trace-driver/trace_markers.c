#include "trace_markers.h"

__attribute__((noinline, used)) void ecc_trace_region_begin(void) {
    __asm__ __volatile__("" ::: "memory");
}

__attribute__((noinline, used)) void ecc_trace_region_end(void) {
    __asm__ __volatile__("" ::: "memory");
}
