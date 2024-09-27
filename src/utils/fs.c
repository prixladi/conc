#include <stdbool.h>
#include <stdlib.h>
#include <dirent.h>
#include <stdlib.h>

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
