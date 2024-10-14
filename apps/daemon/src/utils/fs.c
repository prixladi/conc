#include <stdbool.h>
#include <stdlib.h>
#include <dirent.h>
#include <stdlib.h>
#include <string.h>

#include "string.h"
#include "fs.h"

bool
dir_exists(char *path)
{
    DIR *dir = opendir(path);
    if (dir == NULL)
        return false;

    closedir(dir);
    return true;
}

char *
get_file_content(FILE *fp)
{
    fseek(fp, 0, SEEK_END);
    long size = ftell(fp);
    fseek(fp, 0, SEEK_SET);

    char *content = malloc(size * sizeof(char) + 1);
    fread(content, sizeof(char), size, fp);

    content[size] = '\0';

    return content;
}

bool
is_path_absolute(char *path)
{
    return strlen(path) > 0 && path[0] == '/';
}

char *
paths_join(char *p1, char *p2)
{
    if (p1 == NULL || strlen(p1) == 0)
        return p2 == NULL ? NULL : str_dup(p2);

    if (p2 == NULL || strlen(p2) == 0)
        return p1 == NULL ? NULL : str_dup(p1);

    if (p1[strlen(p1) - 1] == '/')
        return str_concat(p1, p2);
    return str_printf("%s/%s", p1, p2);
}
