/* Trace region markers for dynamic instrumentation (outside upstream crypto). */
#ifndef ECC_TRACE_MARKERS_H
#define ECC_TRACE_MARKERS_H

#ifdef __cplusplus
extern "C" {
#endif

/* Noinline so symbols exist for Valgrind / nm resolution. */
__attribute__((noinline, used)) void ecc_trace_region_begin(void);
__attribute__((noinline, used)) void ecc_trace_region_end(void);

#ifdef __cplusplus
}
#endif

#endif
