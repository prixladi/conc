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

void *_vector_create(size_t length, size_t stride);
void *_vector_dup(void *arr);
void _vector_free(void *arr);

size_t _vector_field_get(void *arr, size_t field);
void _vector_field_set(void *arr, size_t field, size_t value);

void *_vector_resize(void *arr);

void *_vector_push(void *arr, void *xptr);
int _vector_pop(void *arr, void *dest);
int _vector_remove(void *arr, size_t pos, void *dest);

#define VECTOR_DEFAULT_CAPACITY 1
#define VECTOR_RESIZE_FACTOR 2

#define vector_create(type) _vector_create(VECTOR_DEFAULT_CAPACITY, sizeof(type))
#define vector_create_prealloc(type, capacity) _vector_create(capacity, sizeof(type))
#define vector_dup(arr) _vector_dup(arr)

#define vector_free(arr) _vector_free(arr)

#define vector_push(arr, x) \
    do \
    { \
        arr = _vector_push(arr, &x); \
    } while (0)

#define vector_push_rval(arr, x) \
    do \
    { \
        __auto_type temp = x; \
        arr = _vector_push(arr, &temp); \
    } while (0)
#define vector_pop(arr, xptr) _vector_pop(arr, xptr)
#define vector_remove(arr, pos, xptr) _vector_remove(arr, pos, xptr)

#define vector_capacity(arr) _vector_field_get(arr, CAPACITY)
#define vector_length(arr) _vector_field_get(arr, LENGTH)
#define vector_stride(arr) _vector_field_get(arr, STRIDE)

#define vector_for_each(arr, callback) \
    do \
    { \
        for (size_t i = 0; i < vector_length(arr); i++) \
            callback(arr[i]); \
    } while (0)

#endif
