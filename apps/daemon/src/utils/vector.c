#include <stdio.h>
#include <stdbool.h>

#include "vector.h"

#define BYTE_PTR char *

void *
_vec_create(size_t init_cap, size_t stride)
{
    size_t header_size = VECTOR_FIELDS * sizeof(size_t);
    size_t arr_size = init_cap * stride;
    size_t *arr = (size_t *)malloc(header_size + arr_size);
    arr[CAPACITY] = init_cap;
    arr[LENGTH] = 0;
    arr[STRIDE] = stride;
    return (void *)(arr + VECTOR_FIELDS);
}

void *
_vec_dup(void *arr)
{
    size_t header_size = VECTOR_FIELDS * sizeof(size_t);
    size_t arr_size = vec_capacity(arr) * vec_stride(arr);
    size_t total_size = header_size + arr_size;

    size_t *tmp = (size_t *)malloc(total_size);
    memcpy(tmp, (size_t *)(arr)-VECTOR_FIELDS, total_size);
    return tmp + VECTOR_FIELDS;
}

void
_vec_free(void *arr)
{
    free((size_t *)(arr)-VECTOR_FIELDS);
}

void
_vec_scoped(void *arr)
{
    _vec_free(*((void **)arr));
}

size_t
_vector_field_get(void *arr, size_t field)
{
    return ((size_t *)(arr)-VECTOR_FIELDS)[field];
}

void
_vector_field_set(void *arr, size_t field, size_t value)
{
    ((size_t *)(arr)-VECTOR_FIELDS)[field] = value;
}

void *
_vector_resize(void *arr)
{
    size_t capacity = vec_capacity(arr);
    size_t new_capacity = capacity > 0 ? VECTOR_RESIZE_FACTOR * vec_capacity(arr) : VECTOR_DEFAULT_CAPACITY;

    void *new_arr = _vec_create(new_capacity, vec_stride(arr));
    memcpy(new_arr, arr, vec_length(arr) * vec_stride(arr));
    _vector_field_set(new_arr, LENGTH, vec_length(arr));
    _vec_free(arr);
    return new_arr;
}

void *
_vec_push(void *arr, void *xptr)
{
    if (vec_length(arr) >= vec_capacity(arr))
        arr = _vector_resize(arr);

    memcpy((BYTE_PTR)arr + vec_length(arr) * vec_stride(arr), xptr, vec_stride(arr));
    _vector_field_set(arr, LENGTH, vec_length(arr) + 1);
    return arr;
}

int
_vec_pop(void *arr, void *dest)
{
    size_t len = vec_length(arr);
    if (len == 0)
        return 1;

    if (dest != NULL)
        memcpy(dest, (BYTE_PTR)arr + (vec_length(arr) - 1) * vec_stride(arr), vec_stride(arr));

    _vector_field_set(arr, LENGTH, vec_length(arr) - 1);
    return 0;
}

int
_vec_remove(void *arr, size_t pos, void *dest)
{
    size_t len = vec_length(arr);
    if (pos >= len)
        return 1;

    bool is_last = pos + 1 == len;
    if (is_last)
    {
        _vec_pop(arr, dest);
        return 0;
    }

    if (dest != NULL)
        memcpy(dest, (BYTE_PTR)arr + pos * vec_stride(arr), vec_stride(arr));

    memmove((BYTE_PTR)arr + (pos * vec_stride(arr)), (BYTE_PTR)arr + ((pos + 1) * vec_stride(arr)),
            vec_stride(arr) * (len - 1 - pos));

    _vector_field_set(arr, LENGTH, vec_length(arr) - 1);

    return 0;
}
