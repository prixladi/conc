#ifndef LOG__H
#define LOG__H

#include <stdio.h>

#define LOG_ERROR(...) fprintf(stderr, "[ERR] " __VA_ARGS__)

#define LOG_INFO(...) printf("[OK] " __VA_ARGS__)

#ifdef __DEBUG__
#define LOG_DEBUG(...) printf("[DBG] "__VA_ARGS__)
#else
#define LOG_DEBUG(...)
#endif

#endif