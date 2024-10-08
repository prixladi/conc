#ifndef THREAD_POOL__H
#define THREAD_POOL__H

#include <pthread.h>
#include <stdbool.h>

#define THREAD_POOL_UNLIMITED_CAPACITY 0

struct thread_pool;

struct thread_pool *thread_pool_create(int size, int capacity, char *name);
int thread_pool_start(struct thread_pool *pool);

int thread_pool_queue_job(struct thread_pool *pool, char *name, void *(*start_job)(void *), void *job_args);

int thread_pool_stop_and_wait(struct thread_pool *pool);
int thread_pool_pause_and_wait(struct thread_pool *pool);
int thread_pool_free(struct thread_pool *pool);

#endif
