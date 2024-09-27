#ifndef LOG__H
#define LOG__H

#include <stdio.h>

#include "string.h"

enum log_level
{
    TRACE,
    DEBUG,
    INFO,
    WARN,
    ERROR,
    CRITICAL
};

void log_init(enum log_level level);

void log_critical(const char *format, ...);
void log_error(const char *format, ...);
void log_warn(const char *format, ...);
void log_info(const char *format, ...);
void log_debug(const char *format, ...);
void log_trace(const char *trace_name, const char *format, ...);

#endif