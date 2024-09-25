#include <pthread.h>
#include <time.h>

#include "time.h"

#define ms_to_ns(x) x * 1000 * 1000
#define us_to_ns(x) x * 1000

void sleep_ms(int ms)
{
    sleep_ns(ms_to_ns(ms));
}

void sleep_us(int us)
{
    sleep_ns(us_to_ns(us));
}

void sleep_ns(int ns)
{
    struct timespec t = {
        .tv_sec = 0,
        .tv_nsec = ns};
    struct timespec r;

    nanosleep(&t, &r);
}