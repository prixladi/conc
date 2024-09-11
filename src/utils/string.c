#include <string.h>
#include <stdlib.h>
#include <stdio.h>
#include <stdarg.h>

char *str_dup(const char *str)
{
    if (!str)
        return NULL;

    size_t len = strlen(str);
    char *new_str = (char *)malloc(len + 1);
    new_str[len] = '\0';

    memcpy(new_str, str, len);
    return new_str;
}

char *str_concat(const char *fst, ...)
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

char *int_to_string(int i)
{
    int length = snprintf(NULL, 0, "%d", i);
    char *str = malloc(length + 1);
    snprintf(str, length + 1, "%d", i);
    return str;
}
