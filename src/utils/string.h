#ifndef UTILS__H
#define UTILS__H

char *str_dup(const char *str);

char *int_to_str(int i);
char *unsigned_long_to_str(unsigned long i);

char *_str_concat(const char *fst, ...);
#define STR_CONCAT(...) _str_concat(__VA_ARGS__, NULL)

#endif