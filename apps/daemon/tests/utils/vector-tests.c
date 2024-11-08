#include <stdio.h>

#include "../expect.h"

#include "../../src/utils/vector.h"

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
test__vec_push_basic()
{
    int *vec = vec_create(int);

    vec_push_rval(vec, 1);
    vec_push_rval(vec, 2);
    vec_push_rval(vec, 3);
    vec_push_rval(vec, 4);
    vec_push_rval(vec, 5);
    vec_push_rval(vec, 6);
    vec_push_rval(vec, 7);

    expect(vec_length(vec) == 7, "Expected vector of length 7");
    expect(vec_capacity(vec) >= 7, "Expected vector of capacity greater or equal to 7");

    vec_free(vec);
    return NULL;
}

char *
test__vec_push_basic_preallocated()
{
    int *vec = vec_create_prealloc(int, 8);

    vec_push_rval(vec, 1);
    vec_push_rval(vec, 2);
    vec_push_rval(vec, 3);
    vec_push_rval(vec, 4);

    expect(vec_length(vec) == 4, "Expected vector of length 4");
    expect(vec_capacity(vec) == 8, "Expected preallocated vector capacity");

    vec_free(vec);
    return NULL;
}

char *
test__vec_pop()
{
    int *vec = vec_create(int);

    vec_push_rval(vec, 1);
    vec_push_rval(vec, 2);
    vec_push_rval(vec, 3);
    vec_push_rval(vec, 4);
    vec_push_rval(vec, 5);
    vec_push_rval(vec, 6);
    vec_push_rval(vec, 7);

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

    vec_push_rval(vec, 1);
    vec_push_rval(vec, 2);
    vec_push_rval(vec, 3);
    vec_push_rval(vec, 4);
    vec_push_rval(vec, 5);
    vec_push_rval(vec, 6);
    vec_push_rval(vec, 7);

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

    vec_push_rval(vec, 1);
    vec_push_rval(vec, 2);
    vec_push_rval(vec, 3);
    vec_push_rval(vec, 4);
    vec_push_rval(vec, 5);
    vec_push_rval(vec, 6);
    vec_push_rval(vec, 7);

    int *vec2 = vec_dup(vec);

    expect(vec_length(vec) == vec_length(vec2), "Expected duplicated vectors to have the same length");
    expect(vec_capacity(vec) == vec_capacity(vec2), "Expected duplicated vectors to have the same capacity");
    expect(vec_stride(vec) == vec_stride(vec2), "Expected duplicated vectors to have the same stride");

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

    vec_push_rval(vec, 1);
    vec_push_rval(vec, 2);
    vec_push_rval(vec, 3);
    vec_push_rval(vec, 4);
    vec_push_rval(vec, 5);
    vec_push_rval(vec, 6);
    vec_push_rval(vec, 7);

    vec_for_each(vec, test__vec_for_each_callback);

    expect(test__vec_for_each_counter == 1 + 2 + 3 + 4 + 5 + 6 + 7, "Expected counter to be sum of all elements");

    vec_free(vec);
    return NULL;
}
