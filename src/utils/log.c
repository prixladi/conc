
#include <stdarg.h>
#include <stdlib.h>

#include "log.h"

static volatile enum log_level min_level = WARN;

void log_init(enum log_level level)
{
    min_level = level;
}

void log_critical(const char *format, ...)
{
    va_list args;
    va_start(args, format);

    char *fmt = str_concat("[CRT] ", format, NULL);
    vfprintf(stderr, fmt, args);
    free(fmt);

    va_end(args);
}

void log_error(const char *format, ...)
{
    if (min_level > ERROR)
        return;

    va_list args;
    va_start(args, format);

    char *fmt = str_concat("[ERR] ", format, NULL);
    vfprintf(stderr, fmt, args);
    free(fmt);

    va_end(args);
}

void log_warn(const char *format, ...)
{
    if (min_level > WARN)
        return;

    va_list args;
    va_start(args, format);

    char *fmt = str_concat("[WRN] ", format, NULL);
    vfprintf(stderr, fmt, args);
    free(fmt);

    va_end(args);
}

void log_info(const char *format, ...)
{
    if (min_level > INFO)
        return;

    va_list args;
    va_start(args, format);

    char *fmt = str_concat("[INF] ", format, NULL);
    vprintf(fmt, args);
    free(fmt);

    va_end(args);
}

void log_debug(const char *format, ...)
{
    if (min_level > DEBUG)
        return;

    va_list args;
    va_start(args, format);

    char *fmt = str_concat("[DBG] ", format, NULL);
    vprintf(fmt, args);
    free(fmt);

    va_end(args);
}
