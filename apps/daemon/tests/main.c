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
    printf("Vector tests:\n");
    run(test__vec_create, "vector create");
    run(test__vec_create_prealloc, "vector create prealloc");
    run(test__vec_push, "vector push basic");
    run(test__vec_push_preallocated, "vector push preallocated");
    run(test__vec_access, "vector access");
    run(test__vec_pop, "vector pop");
    run(test__vec_remove, "vector remove");
    run(test__vec_dup, "vector duplicate");
    run(test__vec_for_each, "vector for each");
    run(test__vec_methods_with_pointers, "vector methods with pointers");
    run(test__vec_methods_with_structs, "vector methods with structs");
    printf("\n");

    printf("String tests:\n");

    return 0;
}
