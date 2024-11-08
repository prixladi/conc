#ifndef VECTOR__H
#define VECTOR__H

#include <stdlib.h>
#include <string.h>

enum
{
    CAPACITY,
    LENGTH,
    STRIDE,
    VECTOR_FIELDS
};

void *_vec_create(size_t length, size_t stride);
void *_vec_dup(void *arr);
void _vec_free(void *arr);
void _vec_scoped(void *arr);

size_t _vector_field_get(void *arr, size_t field);
void _vector_field_set(void *arr, size_t field, size_t value);

void *_vector_resize(void *arr);

void *_vec_push(void *arr, void *xptr);
int _vec_pop(void *arr, void *dest);
int _vec_remove(void *arr, size_t pos, void *dest);

#define VECTOR_DEFAULT_CAPACITY 1
#define VECTOR_RESIZE_FACTOR 2

#define vec_create(type) _vec_create(VECTOR_DEFAULT_CAPACITY, sizeof(type))
#define vec_create_prealloc(type, capacity) _vec_create(capacity, sizeof(type))
#define vec_dup(arr) _vec_dup(arr)

#define vec_free(arr) _vec_free(arr)
#define vec_scoped __attribute__((__cleanup__(_vec_scoped)))

#define vec_push(arr, x) \
    do \
    { \
        __auto_type temp = x; \
        arr = _vec_push(arr, &temp); \
    } while (0)

#define vec_pop(arr, xptr) _vec_pop(arr, xptr)
#define vec_remove(arr, pos, xptr) _vec_remove(arr, pos, xptr)

#define vec_capacity(arr) _vector_field_get(arr, CAPACITY)
#define vec_length(arr) _vector_field_get(arr, LENGTH)
#define vec_stride(arr) _vector_field_get(arr, STRIDE)

#define vec_for_each(arr, callback) \
    do \
    { \
        for (size_t i = 0; i < vec_length(arr); i++) \
            callback(arr[i]); \
    } while (0)

#endif
