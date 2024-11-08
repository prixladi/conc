#include <stdio.h>

#include "utils/vector-tests.h"

#define run(test, name) \
    do \
    { \
        char *error = test(); \
        if (error) \
        { \
            printf("Error '%s' - %s\n", name, error); \
            return 1; \
        } \
        else \
        { \
            printf("Success '%s'\n", name); \
        } \
    } while (0)

int
main()
{
    run(test__vec_create, "vector create");
    run(test__vec_create_prealloc, "vector create prealloc");
    run(test__vec_push_basic, "vector push basic");
    run(test__vec_push_basic_preallocated, "vector push preallocated");
    run(test__vec_pop, "vector pop");
    run(test__vec_remove, "vector remove");
    run(test__vec_dup, "vector duplicate");
    run(test__vec_for_each, "vector for each");

    return 0;
}
