#include <string.h>
#include <stdlib.h>
#include <stdio.h>
#include <stdarg.h>

#include "string.h"

char *
str_dup(const char *str)
{
    if (!str)
        return NULL;

    size_t len = strlen(str);
    char *new_str = (char *)malloc(len + 1);
    new_str[len] = '\0';

    memcpy(new_str, str, len);
    return new_str;
}

char *
str_printf(const char *format, ...)
{
    va_list args;
    va_start(args, format);

    int len = vsnprintf(NULL, 0, format, args);
    char *buff = malloc(sizeof(char *) * len + 1);

    va_end(args);
    va_start(args, format);

    vsnprintf(buff, len + 1, format, args);

    va_end(args);

    return buff;
}

char *
int_to_str(int i)
{
    return str_printf("%d", i);
}

char *
unsigned_long_to_str(unsigned long i)
{
    return str_printf("%lu", i);
}

char *
_str_concat(const char *fst, ...)
{
    va_list strings;

    int total_len = strlen(fst);
    const char *current = fst;

    va_start(strings, fst);
    do
    {
        total_len += strlen(current);
    } while ((current = va_arg(strings, const char *)));
    va_end(strings);

    char *result = malloc(total_len + 1);
    result[0] = '\0';
    current = fst;

    va_start(strings, fst);
    do
    {
        strcat(result, current);
    } while ((current = va_arg(strings, char *)));
    va_end(strings);

    return result;
}
