#include "ctreqt.h"
#include <stdlib.h>
#include <regex.h>

#define MAX_TRACES 1024

static struct {
    char* test_name;
    char* fr_id;
} traces[MAX_TRACES];

static int trace_count = 0;

void ctreqt_register_trace(const char* test_name, const char* fr_id) {
    if (trace_count >= MAX_TRACES) return;
    
    traces[trace_count].test_name = strdup(test_name);
    traces[trace_count].fr_id = strdup(fr_id);
    trace_count++;
    
    if (getenv("VERBOSE")) {
        printf("[TRACE] %s -> %s\n", test_name, fr_id);
    }
}

int ctreqt_validate_fr_id(const char* fr_id) {
    regex_t regex;
    int ret = regcomp(&regex, "^FR-[A-Z][A-Z0-9]*-[0-9]{3,}(-[A-Z0-9]+)?$", REG_EXTENDED);
    if (ret != 0) return 0;
    
    ret = regexec(&regex, fr_id, 0, NULL, 0);
    regfree(&regex);
    
    return ret == 0;
}

static char* trace_strings[MAX_TRACES + 1];

char** ctreqt_get_traces(void) {
    for (int i = 0; i < trace_count; i++) {
        size_t len = strlen(traces[i].test_name) + strlen(traces[i].fr_id) + 2;
        trace_strings[i] = malloc(len);
        snprintf(trace_strings[i], len, "%s:%s", traces[i].test_name, traces[i].fr_id);
    }
    trace_strings[trace_count] = NULL;
    return trace_strings;
}

void ctreqt_reset_traces(void) {
    for (int i = 0; i < trace_count; i++) {
        free(traces[i].test_name);
        free(traces[i].fr_id);
    }
    trace_count = 0;
}
