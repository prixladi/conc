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

void log_critical(const char *format, ...) __attribute__((format(printf, 1, 2)));
void log_error(const char *format, ...) __attribute__((format(printf, 1, 2)));
void log_warn(const char *format, ...) __attribute__((format(printf, 1, 2)));
void log_info(const char *format, ...) __attribute__((format(printf, 1, 2)));
void log_debug(const char *format, ...) __attribute__((format(printf, 1, 2)));
void log_trace(const char *trace_name, const char *format, ...) __attribute__((format(printf, 2, 3)));

#endif
