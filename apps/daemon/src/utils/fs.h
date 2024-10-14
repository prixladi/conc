#ifndef FS__H
#define FS__H

#include <stdbool.h>
#include <stdio.h>

bool dir_exists(char *path);
char *get_file_content(FILE *fp);
bool is_path_absolute(char *path);
char *paths_join(char *p1, char *p2);

#endif
