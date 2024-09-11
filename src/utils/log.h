#ifndef LOG__H
#define LOG__H

#include <stdio.h>

#define LOG_ERROR(...) fprintf(stderr, __VA_ARGS__)

#define LOG_INFO(...) printf(__VA_ARGS__)

#ifdef __DEBUG__
#define LOG_DEBUG(...) printf(__VA_ARGS__)
#else
#define LOG_DEBUG(...)
#endif

#endif