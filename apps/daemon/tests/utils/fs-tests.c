#include <stdio.h>
#include <string.h>
#include <stdlib.h>

#include "../expect.h"

#include "../../src/utils/fs.h"

char *
test__is_path_absolute__root()
{
    bool is_absolute = is_path_absolute("/");

    expect(is_absolute, "Expected path to be absolute");

    return NULL;
}

char *
test__is_path_absolute__usr_dir()
{
    bool is_absolute = is_path_absolute("/usr");

    expect(is_absolute, "Expected path to be absolute");

    return NULL;
}

char *
test__is_path_absolute__relative()
{
    bool is_absolute = is_path_absolute("dir");

    expect(!is_absolute, "Expected path to be relative");

    return NULL;
}

char *
test__is_path_absolute__relative_with_dot()
{
    bool is_absolute = is_path_absolute("./dir");

    expect(!is_absolute, "Expected path to be relative");

    return NULL;
}