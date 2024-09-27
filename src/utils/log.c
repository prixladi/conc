
#include <stdarg.h>
#include <stdlib.h>

#include "log.h"

static volatile enum log_level min_level = WARN;

void
log_init(enum log_level level)
{
	min_level = level;
}

void
log_critical(const char *format, ...)
{
	va_list args;
	va_start(args, format);

	char *fmt = STR_CONCAT("[CRT] ", format);
	vfprintf(stderr, fmt, args);
	free(fmt);

	va_end(args);
}

void
log_error(const char *format, ...)
{
	if (min_level > ERROR)
		return;

	va_list args;
	va_start(args, format);

	char *fmt = STR_CONCAT("[ERR] ", format);
	vfprintf(stderr, fmt, args);
	free(fmt);

	va_end(args);
}

void
log_warn(const char *format, ...)
{
	if (min_level > WARN)
		return;

	va_list args;
	va_start(args, format);

	char *fmt = STR_CONCAT("[WRN] ", format);
	vfprintf(stderr, fmt, args);
	free(fmt);

	va_end(args);
}

void
log_info(const char *format, ...)
{
	if (min_level > INFO)
		return;

	va_list args;
	va_start(args, format);

	char *fmt = STR_CONCAT("[INF] ", format);
	vprintf(fmt, args);
	free(fmt);

	va_end(args);
}

void
log_debug(const char *format, ...)
{
	if (min_level > DEBUG)
		return;

	va_list args;
	va_start(args, format);

	char *fmt = STR_CONCAT("[DBG] ", format);
	vprintf(fmt, args);
	free(fmt);

	va_end(args);
}

void
log_trace(const char *trace_name, const char *format, ...)
{
	if (min_level > TRACE)
		return;

	va_list args;
	va_start(args, format);

	char *fmt = STR_CONCAT("[TRC] (", trace_name, ") ", format);
	vprintf(fmt, args);
	free(fmt);

	va_end(args);
}
