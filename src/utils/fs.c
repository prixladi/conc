#include <stdbool.h>
#include <dirent.h>
#include <stdlib.h>

#include "fs.h"

bool dir_exists(char *path)
{
    DIR *project_dir = opendir(path);
    if (project_dir == NULL)
        return false;

    closedir(project_dir);
    return true;
}