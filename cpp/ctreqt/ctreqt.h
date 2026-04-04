#ifndef CTREQT_H
#define CTREQT_H

/**
 * C Traceability Library (ctreqt)
 *
 * Provides macros for marking tests as tracing to Feature Requirements.
 *
 * Example:
 * ```c
 * CTREQT_TRACE_TO(test_feature, "FR-EXAMPLE-001");
 * void test_feature(void) {
 *     // test code
 * }
 * ```
 */

#include <stdio.h>
#include <string.h>
#include <regex.h>

/**
 * Macro to mark a test function as tracing to an FR.
 */
#define CTREQT_TRACE_TO(func_name, fr_id) \
    static const char* _ctreqt_##func_name##_fr = fr_id; \
    __attribute__((constructor)) void _ctreqt_##func_name##_register(void) { \
        ctreqt_register_trace(#func_name, fr_id); \
    }

/**
 * Register a trace entry.
 */
void ctreqt_register_trace(const char* test_name, const char* fr_id);

/**
 * Validate an FR ID format.
 * Returns 1 if valid, 0 otherwise.
 */
int ctreqt_validate_fr_id(const char* fr_id);

/**
 * Get all recorded traces.
 * Returns NULL-terminated array of strings (test_name:fr_id format).
 */
char** ctreqt_get_traces(void);

/**
 * Reset all traces.
 */
void ctreqt_reset_traces(void);

#endif /* CTREQT_H */
