#include <stdio.h>

#include "utils/vector-tests.h"
#include "utils/string-tests.h"
#include "utils/fs-tests.h"
#include "utils/thread-pool.h"

#include "../src/utils/log.h"

#define run(test, name) \
    do \
    { \
        printf(" • '%s'", name);\
        fflush(stdout);\
        char *error = test(); \
        if (error) \
        { \
            printf(" | Error %s\n", error); \
            return 1; \
        } \
        else \
        { \
            printf(" | Success\n"); \
        } \
    } while (0)

int
main()
{
    log_init(CRITICAL);

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
    run(test__str_dup, "str dup");
    run(test__str_dup_null, "str dup null");
    run(test__str_printf, "str printf");
    run(test__str_printf__without_formatting, "str printf without formatting");
    run(test__int_to_str, "int to str");
    run(test__int_to_str_negative, "int to str negative");
    printf("\n");

    printf("Fs tests:\n");
    run(test__is_path_absolute__root, "is path absolute (root)");
    run(test__is_path_absolute__usr_dir, "is path absolute (usr dir)");
    run(test__is_path_absolute__relative, "is path absolute (relative)");
    run(test__is_path_absolute__relative_with_dot, "is path absolute (relative with dot)");

    printf("Thread pool tests:\n");
    run(test__thread_pool_create, "thread pool create");
    run(test__thread_pool_queue_job__idle_pool, "thread pool queue job (idle pool)");
    run(test__thread_pool_queue_job__idle_pool_over_capacity, "thread pool queue job (idle pool over capacity)");
    run(test__thread_pool_start__empty, "thread pool start (empty)");
    run(test__thread_pool_start__empty_running, "thread pool start (empty and running)");
    run(test__thread_pool_free__idle, "thread pool free (idle)");
    run(test__thread_pool_free__running, "thread pool free (idle)");
    run(test__thread_pool_free__idle_after_stop, "thread pool free (idle after stop)");
    run(test__thread_pool__full_service1, "thread pool (full service 1)");
    run(test__thread_pool__full_service2, "thread pool (full service 2)");
    run(test__thread_pool__full_service3, "thread pool (full service 3)");
    run(test__thread_pool__full_service4, "thread pool (full service 4)");
    run(test__thread_pool__full_service5, "thread pool (full service 5)");
    run(test__thread_pool__full_service6, "thread pool (full service 6)");
    run(test__thread_pool__full_service7, "thread pool (full service 7)");
    run(test__thread_pool__full_service8, "thread pool (full service 8)");

    return 0;
}
