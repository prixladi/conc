#ifndef FS__H
#define FS__H

#include <stdbool.h>
#include <stdio.h>

bool dir_exists(char *path);
char *get_file_content(FILE *fp);

#endif
