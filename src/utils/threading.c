#include <pthread.h>
#include <time.h>

#include "threading.h"

#define ms_to_ns(x) x * 1000 * 1000

void sleep_ns(int ns)
{
    struct timespec t = {
        .tv_sec = 0,
        .tv_nsec = ns};
    struct timespec r;

    nanosleep(&t, &r);
}

void sleep_ms(int ms)
{
    sleep_ns(ms_to_ns(ms));
}