#ifndef STRING__H
#define STRING__H

char *str_dup(const char *str);
char *str_printf(const char *format, ...) __attribute__((format(printf, 1, 2)));
char *int_to_str(int i);
char *unsigned_long_to_str(unsigned long i);

char *_str_concat(const char *fst, ...);
#define str_concat(...) _str_concat(__VA_ARGS__, NULL)

#endif
