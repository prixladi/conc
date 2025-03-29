#include <stdio.h>
#include <string.h>

#include "../expect.h"

#include "../../src/utils/vector.c"

static int *alloc_int(int i);

char *
test__vec_create()
{
    int *vec = vec_create(int);

    expect(vec_length(vec) == 0, "Expected empty vector length");
    expect(vec_capacity(vec) == VECTOR_DEFAULT_CAPACITY, "Expected empty vector capacity");

    vec_free(vec);
    return NULL;
}

char *
test__vec_create_prealloc()
{
    int *vec = vec_create_prealloc(int, 8);

    expect(vec_length(vec) == 0, "Expected empty vector length");
    expect(vec_capacity(vec) == 8, "Expected preallocated vector capacity");

    vec_free(vec);
    return NULL;
}

char *
test__vec_push()
{
    int *vec = vec_create(int);

    vec_push(vec, 1);
    vec_push(vec, 2);
    vec_push(vec, 3);
    vec_push(vec, 4);
    vec_push(vec, 5);
    vec_push(vec, 6);
    vec_push(vec, 7);

    expect(vec_length(vec) == 7, "Expected vector of length 7");
    expect(vec_capacity(vec) >= 7, "Expected vector of capacity greater or equal to 7");

    vec_free(vec);
    return NULL;
}

char *
test__vec_push_preallocated()
{
    int *vec = vec_create_prealloc(int, 8);

    vec_push(vec, 1);
    vec_push(vec, 2);
    vec_push(vec, 3);
    vec_push(vec, 4);

    expect(vec_length(vec) == 4, "Expected vector of length 4");
    expect(vec_capacity(vec) == 8, "Expected preallocated vector capacity");

    vec_free(vec);
    return NULL;
}

char *
test__vec_access()
{
    int *vec = vec_create(int);

    vec_push(vec, 1);
    vec_push(vec, 2);
    vec_push(vec, 3);
    vec_push(vec, 4);
    vec_push(vec, 5);
    vec_push(vec, 6);
    vec_push(vec, 7);

    expect(vec[2] == 3, "Expected third element to be 3");
    expect(vec[5] == 6, "Expected sixth element to be 6");

    vec_free(vec);
    return NULL;
}

char *
test__vec_pop()
{
    int *vec = vec_create(int);

    vec_push(vec, 1);
    vec_push(vec, 2);
    vec_push(vec, 3);
    vec_push(vec, 4);
    vec_push(vec, 5);
    vec_push(vec, 6);
    vec_push(vec, 7);

    int out = 0;
    vec_pop(vec, &out);

    expect(out == 7, "Expected popped element to be 7");
    expect(vec_length(vec) == 6, "Expected vector of length 6");

    vec_pop(vec, NULL);
    vec_pop(vec, NULL);
    vec_pop(vec, NULL);
    vec_pop(vec, NULL);
    vec_pop(vec, NULL);
    vec_pop(vec, &out);

    expect(out == 1, "Expected popped element to be 1");
    expect(vec_length(vec) == 0, "Expected vector of length 0");

    vec_free(vec);
    return NULL;
}

char *
test__vec_remove()
{
    int *vec = vec_create(int);

    vec_push(vec, 1);
    vec_push(vec, 2);
    vec_push(vec, 3);
    vec_push(vec, 4);
    vec_push(vec, 5);
    vec_push(vec, 6);
    vec_push(vec, 7);

    int out = 0;
    vec_remove(vec, 1, &out);

    expect(out == 2, "Expected removed element to be 2");
    expect(vec_length(vec) == 6, "Expected vector of length 6");

    vec_remove(vec, 0, &out);
    vec_remove(vec, 0, &out);
    vec_remove(vec, 0, &out);
    vec_remove(vec, 0, &out);
    vec_remove(vec, 0, &out);
    vec_remove(vec, 0, &out);

    expect(out == 7, "Expected popped element to be 7");
    expect(vec_length(vec) == 0, "Expected vector of length 0");

    vec_free(vec);
    return NULL;
}

char *
test__vec_dup()
{
    int *vec = vec_create(int);

    vec_push(vec, 1);
    vec_push(vec, 2);
    vec_push(vec, 3);
    vec_push(vec, 4);
    vec_push(vec, 5);
    vec_push(vec, 6);
    vec_push(vec, 7);

    int *vec2 = vec_dup(vec);

    expect(vec_length(vec) == vec_length(vec2), "Expected duplicated vectors to have the same length");
    expect(vec_capacity(vec) == vec_capacity(vec2), "Expected duplicated vectors to have the same capacity");
    expect(vec_stride(vec) == vec_stride(vec2), "Expected duplicated vectors to have the same stride");

    vec_push(vec2, 8);

    expect(vec_length(vec) == 7, "Expected original vector length be still the same after push to duplicate");
    expect(vec_length(vec2) == 8, "Expected duplicate vector to have length 8");

    vec_free(vec);
    vec_free(vec2);
    return NULL;
}

int test__vec_for_each_counter = 0;
void
test__vec_for_each_callback(int x)
{
    test__vec_for_each_counter += x;
}

char *
test__vec_for_each()
{
    int *vec = vec_create(int);

    vec_push(vec, 1);
    vec_push(vec, 2);
    vec_push(vec, 3);
    vec_push(vec, 4);
    vec_push(vec, 5);
    vec_push(vec, 6);
    vec_push(vec, 7);

    vec_for_each(vec, test__vec_for_each_callback);

    expect(test__vec_for_each_counter == 1 + 2 + 3 + 4 + 5 + 6 + 7, "Expected counter to be sum of all elements");

    vec_free(vec);
    return NULL;
}

char *
test__vec_methods_with_pointers()
{
    int **vec = vec_create(char *);

    vec_push(vec, alloc_int(1));
    vec_push(vec, alloc_int(2));
    vec_push(vec, alloc_int(3));
    vec_push(vec, alloc_int(4));
    vec_push(vec, alloc_int(5));
    vec_push(vec, alloc_int(6));
    vec_push(vec, alloc_int(7));

    expect(vec_length(vec) == 7, "Expected vector of length 7");
    expect(*vec[2] == 3, "Expected correct second element");

    int *out = NULL;

    vec_pop(vec, &out);

    expect(vec_length(vec) == 6, "Expected vector of length 6");
    expect(*out == 7, "Expected correct popped element");

    free(out);

    vec_remove(vec, 3, &out);

    expect(vec_length(vec) == 5, "Expected vector of length 5");
    expect(*out == 4, "Expected correct removed element");

    free(out);

    vec_for_each(vec, free);
    vec_free(vec);
    return NULL;
}

struct test_vector_struct
{
    char *name;
    int num;
};

char *
test__vec_methods_with_struct()
{
    struct test_vector_struct *vec = vec_create(struct test_vector_struct);

    struct test_vector_struct first = { .name = "first", .num = 1 };
    struct test_vector_struct second = { .name = "second", .num = 2 };
    struct test_vector_struct third = { .name = "third", .num = 3 };
    struct test_vector_struct fourth = { .name = "fourth", .num = 4 };
    struct test_vector_struct fifth = { .name = "fifth", .num = 5 };

    vec_push(vec, first);
    vec_push(vec, second);
    vec_push(vec, third);
    vec_push(vec, fourth);
    vec_push(vec, fifth);

    expect(vec_length(vec) == 5, "Expected vector of length 5");

    struct test_vector_struct out;

    vec_pop(vec, &out);

    expect(out.num == 5, "Expected correct popped struct");

    vec_remove(vec, 1, &out);

    expect(out.num == 2, "Expected correct removed struct");

    vec_free(vec);
    return NULL;
}


char *
test__vec_methods_with_structs()
{
    struct test_vector_struct *vec = vec_create(struct test_vector_struct);

    struct test_vector_struct first = { .name = "first", .num = 1 };
    struct test_vector_struct second = { .name = "second", .num = 2 };
    struct test_vector_struct third = { .name = "third", .num = 3 };
    struct test_vector_struct fourth = { .name = "fourth", .num = 4 };
    struct test_vector_struct fifth = { .name = "fifth", .num = 5 };

    vec_push(vec, first);
    vec_push(vec, second);
    vec_push(vec, third);
    vec_push(vec, fourth);
    vec_push(vec, fifth);

    expect(vec_length(vec) == 5, "Expected vector of length 5");

    struct test_vector_struct out;

    vec_pop(vec, &out);

    expect(out.num == 5, "Expected correct popped struct");

    vec_remove(vec, 1, &out);

    expect(out.num == 2, "Expected correct removed struct");

    vec_free(vec);
    return NULL;
}

static int *
alloc_int(int i)
{
    int *val = malloc(sizeof(int));
    *val = i;
    return val;
}
