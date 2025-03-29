#include <stdio.h>
#include <string.h>
#include <stdlib.h>

#include "../expect.h"

#include "../../src/utils/string.c"

char *
test__str_dup()
{
    char *str = "test string";

    char *duplicate = str_dup(str);

    expect(strcmp(str, duplicate) == 0, "Expected duplicated string to be equal to original");
    expect(str != duplicate, "Expected duplicated string to point to different memory than original");

    free(duplicate);

    return NULL;
}

char *
test__str_dup_null()
{
    char *str = NULL;

    char *duplicate = str_dup(str);

    expect(duplicate == NULL, "Expected duplicate to be null");

    free(str);

    return NULL;
}

char *
test__str_printf()
{
    char *printed = str_printf("number %d, char %c, string %s", 15, 'c', "str");

    expect(strcmp(printed, "number 15, char c, string str") == 0, "Expected printed string to be correct");

    free(printed);
    return NULL;
}

char *
test__str_printf__without_formatting()
{
    char *str = "test string";

    char *printed = str_printf(str);

    expect(strcmp(str, printed) == 0, "Expected printed string to be equal to original");
    expect(str != printed, "Expected printed string to point to different memory than original");

    free(printed);
    return NULL;
}

char *
test__int_to_str()
{
    char *str = int_to_str(66);

    expect(strcmp(str, "66") == 0, "Expected printed number to be correct");

    free(str);
    return NULL;
}

char *
test__int_to_str_negative()
{
    char *str = int_to_str(-6);

    expect(strcmp(str, "-6") == 0, "Expected printed number to be correct");

    free(str);
    return NULL;
}
