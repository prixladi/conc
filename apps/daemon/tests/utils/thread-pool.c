#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <unistd.h>

#include "../expect.h"

#include "../../src/utils/thread-pool.c"

void *
test_job(void *data)
{
    int *sleep_duration_s = data;
    sleep(*sleep_duration_s);
    return NULL;
}

char *
test__thread_pool_create()
{
    struct thread_pool *pool = thread_pool_create(2, 5, "Test");

    expect(pool->concurrency == 2, "Expected pool concurrency to be 2");
    expect(pool->job_queue_capacity == 5, "Expected pool job_queue_capacity to be 5");
    expect(pool->job_queue_size == 0, "Expected pool job_queue_size to be 0");
    expect(pool->state == IDLE, "Expected pool status to be idle");

    expect(thread_pool_free(pool) == 0, "Expected pool to be freed successfully");

    return NULL;
}

char *
test__thread_pool_queue_job__idle_pool()
{
    struct thread_pool *pool = thread_pool_create(1, 5, "Test");

    int sleep_duration_s = 0;

    expect(thread_pool_queue_job(pool, "test_job1", test_job, &sleep_duration_s) == 0,
           "Expected first job to be queued successfully");
    expect(thread_pool_queue_job(pool, "test_job2", test_job, &sleep_duration_s) == 0,
           "Expected second job to be queued successfully");

    expect(pool->state == IDLE, "Expected pool status to be idle");
    expect(pool->job_queue_size == 2, "Expected pool job_queue_size to be 2");

    expect(thread_pool_free(pool) == 0, "Expected pool to be freed successfully");

    return NULL;
}

char *
test__thread_pool_queue_job__idle_pool_over_capacity()
{
    struct thread_pool *pool = thread_pool_create(1, 5, "Test");

    int sleep_duration_s = 0;

    expect(thread_pool_queue_job(pool, "test_job1", test_job, &sleep_duration_s) == 0,
           "Expected first job to be queued successfully");
    expect(thread_pool_queue_job(pool, "test_job2", test_job, &sleep_duration_s) == 0,
           "Expected second job to be queued successfully");
    expect(thread_pool_queue_job(pool, "test_job3", test_job, &sleep_duration_s) == 0,
           "Expected third job to be queued successfully");
    expect(thread_pool_queue_job(pool, "test_job4", test_job, &sleep_duration_s) == 0,
           "Expected fourth job to be queued successfully");
    expect(thread_pool_queue_job(pool, "test_job5", test_job, &sleep_duration_s) == 0,
           "Expected fifth job to be queued successfully");

    expect(pool->job_queue_size == 5, "Expected pool job_queue_size to be 5");

    expect(thread_pool_queue_job(pool, "test_job6", test_job, &sleep_duration_s) == 1,
           "Expected sixth job to not be queued successfully");

    expect(pool->job_queue_size == 5, "Expected pool job_queue_size to be 5");

    expect(thread_pool_queue_job(pool, "test_job7", test_job, &sleep_duration_s) == 1,
           "Expected seventh job to not be queued successfully");

    expect(thread_pool_free(pool) == 0, "Expected pool to be freed successfully");

    return NULL;
}

char *
test__thread_pool_start__empty()
{
    struct thread_pool *pool = thread_pool_create(1, 5, "Test");

    expect(thread_pool_start(pool) == 0, "Expected pool to start successfully");
    expect(pool->state == RUNNING, "Expected pool status to be running");

    expect(thread_pool_finish_and_stop(pool) == 0, "Expected pool to be finish and stop successfully");

    expect(thread_pool_free(pool) == 0, "Expected pool to be freed successfully");

    return NULL;
}

char *
test__thread_pool_start__empty_running()
{
    struct thread_pool *pool = thread_pool_create(1, 5, "Test");

    expect(thread_pool_start(pool) == 0, "Expected pool to start successfully");
    expect(pool->state == RUNNING, "Expected pool status to be running");

    expect(thread_pool_start(pool) == 1, "Expected to not be able to start if it is already running");
    expect(pool->state == RUNNING, "Expected pool status to be running");

    expect(thread_pool_finish_and_stop(pool) == 0, "Expected pool to be finish and stop successfully");

    expect(thread_pool_free(pool) == 0, "Expected pool to be freed successfully");

    return NULL;
}

char *
test__thread_pool_free__idle()
{
    struct thread_pool *pool = thread_pool_create(1, 5, "Test");
    expect(thread_pool_free(pool) == 0, "Expected pool to be freed successfully");

    return NULL;
}

char *
test__thread_pool_free__running()
{
    struct thread_pool *pool = thread_pool_create(1, 5, "Test");
    expect(thread_pool_start(pool) == 0, "Expected pool to start successfully");
    expect(pool->state == RUNNING, "Expected pool status to be running");

    expect(thread_pool_free(pool) == 1, "Expected pool to be unable to be freed when running");

    return NULL;
}

char *
test__thread_pool_free__idle_after_stop()
{
    struct thread_pool *pool = thread_pool_create(1, 5, "Test");
    expect(thread_pool_start(pool) == 0, "Expected pool to start successfully");
    expect(pool->state == RUNNING, "Expected pool status to be running");

    expect(thread_pool_free(pool) == 1, "Expected pool to be unable to be freed when running");

    expect(thread_pool_finish_and_stop(pool) == 0, "Expected pool to be finish and stop successfully");

    expect(thread_pool_free(pool) == 0, "Expected pool to be freed successfully");

    return NULL;
}

char *
test__thread_pool__full_service1()
{
    struct thread_pool *pool = thread_pool_create(3, 15, "Test");

    int sleep_duration_s = 2;

    expect(thread_pool_queue_job(pool, "test_job1", test_job, &sleep_duration_s) == 0,
           "Expected first job to be queued successfully");
    expect(thread_pool_queue_job(pool, "test_job2", test_job, &sleep_duration_s) == 0,
           "Expected second job to be queued successfully");
    expect(thread_pool_queue_job(pool, "test_job3", test_job, &sleep_duration_s) == 0,
           "Expected third job to be queued successfully");
    expect(thread_pool_queue_job(pool, "test_job4", test_job, &sleep_duration_s) == 0,
           "Expected fourth job to be queued successfully");
    expect(thread_pool_queue_job(pool, "test_job5", test_job, &sleep_duration_s) == 0,
           "Expected fifth job to be queued successfully");

    expect(thread_pool_start(pool) == 0, "Expected pool to start successfully");
    expect(pool->state == RUNNING, "Expected pool status to be running");

    sleep(1);
    expect(pool->job_queue_size == 2, "Expected pool job_queue_size to be 2");

    expect(thread_pool_free(pool) == 1, "Expected pool to be unable to be freed when running");

    expect(thread_pool_finish_and_stop(pool) == 0, "Expected pool to be finish and stop successfully");
    expect(pool->job_queue_size == 0, "Expected pool job_queue_size to be 0 after finish");

    expect(thread_pool_free(pool) == 0, "Expected pool to be freed successfully");

    return NULL;
}

char *
test__thread_pool__full_service2()
{
    struct thread_pool *pool = thread_pool_create(8, 15, "Test");

    int sleep_duration_s = 2;

    expect(thread_pool_queue_job(pool, "test_job1", test_job, &sleep_duration_s) == 0,
           "Expected first job to be queued successfully");
    expect(thread_pool_queue_job(pool, "test_job2", test_job, &sleep_duration_s) == 0,
           "Expected second job to be queued successfully");
    expect(thread_pool_queue_job(pool, "test_job3", test_job, &sleep_duration_s) == 0,
           "Expected third job to be queued successfully");
    expect(thread_pool_queue_job(pool, "test_job4", test_job, &sleep_duration_s) == 0,
           "Expected fourth job to be queued successfully");
    expect(thread_pool_queue_job(pool, "test_job5", test_job, &sleep_duration_s) == 0,
           "Expected fifth job to be queued successfully");

    expect(thread_pool_start(pool) == 0, "Expected pool to start successfully");
    expect(pool->state == RUNNING, "Expected pool status to be running");

    sleep(1);
    expect(pool->job_queue_size == 0, "Expected pool job_queue_size to be 0");

    expect(thread_pool_free(pool) == 1, "Expected pool to be unable to be freed when running");

    expect(thread_pool_finish_and_stop(pool) == 0, "Expected pool to be finish and stop successfully");
    expect(pool->job_queue_size == 0, "Expected pool job_queue_size to be 0 after finish");

    expect(thread_pool_free(pool) == 0, "Expected pool to be freed successfully");

    return NULL;
}

char *
test__thread_pool__full_service3()
{
    struct thread_pool *pool = thread_pool_create(3, 15, "Test");

    int sleep_duration_s = 2;

    expect(thread_pool_queue_job(pool, "test_job1", test_job, &sleep_duration_s) == 0,
           "Expected first job to be queued successfully");
    expect(thread_pool_queue_job(pool, "test_job2", test_job, &sleep_duration_s) == 0,
           "Expected second job to be queued successfully");
    expect(thread_pool_queue_job(pool, "test_job3", test_job, &sleep_duration_s) == 0,
           "Expected third job to be queued successfully");
    expect(thread_pool_queue_job(pool, "test_job4", test_job, &sleep_duration_s) == 0,
           "Expected fourth job to be queued successfully");
    expect(thread_pool_queue_job(pool, "test_job5", test_job, &sleep_duration_s) == 0,
           "Expected fifth job to be queued successfully");

    expect(thread_pool_start(pool) == 0, "Expected pool to start successfully");
    expect(pool->state == RUNNING, "Expected pool status to be running");

    sleep(1);
    expect(pool->job_queue_size == 2, "Expected pool job_queue_size to be 2");

    expect(thread_pool_free(pool) == 1, "Expected pool to be unable to be freed when running");

    expect(thread_pool_wait_and_pause(pool) == 0, "Expected pool to be wait and pause successfully");
    expect(pool->job_queue_size == 2, "Expected pool job_queue_size to be 2 after pause");

    expect(thread_pool_free(pool) == 0, "Expected pool to be freed successfully");

    return NULL;
}

char *
test__thread_pool__full_service4()
{
    struct thread_pool *pool = thread_pool_create(3, 15, "Test");

    int sleep_duration_s = 2;

    expect(thread_pool_queue_job(pool, "test_job1", test_job, &sleep_duration_s) == 0,
           "Expected first job to be queued successfully");
    expect(thread_pool_queue_job(pool, "test_job2", test_job, &sleep_duration_s) == 0,
           "Expected second job to be queued successfully");
    expect(thread_pool_queue_job(pool, "test_job3", test_job, &sleep_duration_s) == 0,
           "Expected third job to be queued successfully");
    expect(thread_pool_queue_job(pool, "test_job4", test_job, &sleep_duration_s) == 0,
           "Expected fourth job to be queued successfully");
    expect(thread_pool_queue_job(pool, "test_job5", test_job, &sleep_duration_s) == 0,
           "Expected fifth job to be queued successfully");

    expect(thread_pool_start(pool) == 0, "Expected pool to start successfully");
    expect(pool->state == RUNNING, "Expected pool status to be running");

    sleep(1);
    expect(pool->job_queue_size == 2, "Expected pool job_queue_size to be 2");

    expect(thread_pool_free(pool) == 1, "Expected pool to be unable to be freed when running");

    expect(thread_pool_wait_and_pause(pool) == 0, "Expected pool to be wait and pause successfully");
    expect(pool->job_queue_size == 2, "Expected pool job_queue_size to be 2 after pause");

    expect(thread_pool_start(pool) == 0, "Expected pool to start successfully");
    expect(pool->state == RUNNING, "Expected pool status to be running");

    sleep(1);
    expect(pool->job_queue_size == 0, "Expected pool job_queue_size to be 0");

    expect(thread_pool_free(pool) == 1, "Expected pool to be unable to be freed when running");
    expect(thread_pool_finish_and_stop(pool) == 0, "Expected pool to be finish and stop successfully");
    expect(thread_pool_free(pool) == 0, "Expected pool to be freed successfully");

    return NULL;
}

char *
test__thread_pool__full_service5()
{
    struct thread_pool *pool = thread_pool_create(4, 15, "Test");

    int sleep_duration_s = 2;

    expect(thread_pool_queue_job(pool, "test_job1", test_job, &sleep_duration_s) == 0,
           "Expected first job to be queued successfully");
    expect(thread_pool_queue_job(pool, "test_job2", test_job, &sleep_duration_s) == 0,
           "Expected second job to be queued successfully");
    expect(thread_pool_queue_job(pool, "test_job3", test_job, &sleep_duration_s) == 0,
           "Expected third job to be queued successfully");
    expect(thread_pool_queue_job(pool, "test_job4", test_job, &sleep_duration_s) == 0,
           "Expected fourth job to be queued successfully");
    expect(thread_pool_queue_job(pool, "test_job5", test_job, &sleep_duration_s) == 0,
           "Expected fifth job to be queued successfully");

    expect(thread_pool_start(pool) == 0, "Expected pool to start successfully");
    expect(pool->state == RUNNING, "Expected pool status to be running");

    sleep(1);
    expect(pool->job_queue_size == 1, "Expected pool job_queue_size to be 1");

    expect(thread_pool_free(pool) == 1, "Expected pool to be unable to be freed when running");

    expect(thread_pool_wait_and_pause(pool) == 0, "Expected pool to be wait and pause successfully");
    expect(pool->job_queue_size == 1, "Expected pool job_queue_size to be 1 after pause");

    expect(thread_pool_start(pool) == 0, "Expected pool to start successfully");
    expect(pool->state == RUNNING, "Expected pool status to be running");

    sleep(1);
    expect(pool->job_queue_size == 0, "Expected pool job_queue_size to be 0");

    expect(thread_pool_free(pool) == 1, "Expected pool to be unable to be freed when running");
    expect(thread_pool_finish_and_stop(pool) == 0, "Expected pool to be finish and stop successfully");
    expect(thread_pool_free(pool) == 0, "Expected pool to be freed successfully");

    return NULL;
}

char *
test__thread_pool__full_service6()
{
    struct thread_pool *pool = thread_pool_create(4, 15, "Test");

    int sleep_duration_s = 2;

    expect(thread_pool_queue_job(pool, "test_job1", test_job, &sleep_duration_s) == 0,
           "Expected first job to be queued successfully");
    expect(thread_pool_queue_job(pool, "test_job2", test_job, &sleep_duration_s) == 0,
           "Expected second job to be queued successfully");
    expect(thread_pool_queue_job(pool, "test_job3", test_job, &sleep_duration_s) == 0,
           "Expected third job to be queued successfully");
    expect(thread_pool_queue_job(pool, "test_job4", test_job, &sleep_duration_s) == 0,
           "Expected fourth job to be queued successfully");
    expect(thread_pool_queue_job(pool, "test_job5", test_job, &sleep_duration_s) == 0,
           "Expected fifth job to be queued successfully");

    expect(thread_pool_start(pool) == 0, "Expected pool to start successfully");
    expect(pool->state == RUNNING, "Expected pool status to be running");

    sleep(1);
    expect(pool->job_queue_size == 1, "Expected pool job_queue_size to be 1");

    expect(thread_pool_free(pool) == 1, "Expected pool to be unable to be freed when running");

    expect(thread_pool_wait_and_pause(pool) == 0, "Expected pool to be wait and pause successfully");
    expect(pool->job_queue_size == 1, "Expected pool job_queue_size to be 1 after pause");

    expect(thread_pool_queue_job(pool, "test_job6", test_job, &sleep_duration_s) == 0,
           "Expected fifth job to be queued successfully");
    expect(thread_pool_queue_job(pool, "test_job7", test_job, &sleep_duration_s) == 0,
           "Expected fifth job to be queued successfully");
    expect(thread_pool_queue_job(pool, "test_job8", test_job, &sleep_duration_s) == 0,
           "Expected fifth job to be queued successfully");
    expect(thread_pool_queue_job(pool, "test_job9", test_job, &sleep_duration_s) == 0,
           "Expected fifth job to be queued successfully");

    expect(thread_pool_start(pool) == 0, "Expected pool to start successfully");
    expect(pool->state == RUNNING, "Expected pool status to be running");

    sleep(1);
    expect(pool->job_queue_size == 1, "Expected pool job_queue_size to be 1");

    expect(thread_pool_finish_and_stop(pool) == 0, "Expected pool to be finish and stop successfully");
    expect(thread_pool_free(pool) == 0, "Expected pool to be freed successfully");

    return NULL;
}

char *
test__thread_pool__full_service7()
{
    struct thread_pool *pool = thread_pool_create(3, 7, "Test");

    int sleep_duration_s = 2;

    expect(thread_pool_queue_job(pool, "test_job1", test_job, &sleep_duration_s) == 0,
           "Expected first job to be queued successfully");
    expect(thread_pool_queue_job(pool, "test_job2", test_job, &sleep_duration_s) == 0,
           "Expected second job to be queued successfully");
    expect(thread_pool_queue_job(pool, "test_job3", test_job, &sleep_duration_s) == 0,
           "Expected third job to be queued successfully");
    expect(thread_pool_queue_job(pool, "test_job4", test_job, &sleep_duration_s) == 0,
           "Expected fourth job to be queued successfully");
    expect(thread_pool_queue_job(pool, "test_job5", test_job, &sleep_duration_s) == 0,
           "Expected fifth job to be queued successfully");
    expect(thread_pool_queue_job(pool, "test_job6", test_job, &sleep_duration_s) == 0,
           "Expected sixth job to be queued successfully");
    expect(thread_pool_queue_job(pool, "test_job7", test_job, &sleep_duration_s) == 0,
           "Expected seventh job to be queued successfully");
    expect(thread_pool_queue_job(pool, "test_job8", test_job, &sleep_duration_s) == 1,
           "Expected eight job to not be queued successfully");

    expect(thread_pool_start(pool) == 0, "Expected pool to start successfully");
    expect(pool->state == RUNNING, "Expected pool status to be running");

    sleep(1);
    expect(pool->job_queue_size == 4, "Expected pool job_queue_size to be 4");

    expect(thread_pool_free(pool) == 1, "Expected pool to be unable to be freed when running");

    expect(thread_pool_wait_and_pause(pool) == 0, "Expected pool to be wait and pause successfully");
    expect(pool->job_queue_size == 4, "Expected pool job_queue_size to be 4 after pause");

    expect(thread_pool_start(pool) == 0, "Expected pool to start successfully");
    expect(pool->state == RUNNING, "Expected pool status to be running");

    sleep(1);
    expect(pool->job_queue_size == 1, "Expected pool job_queue_size to be 1");

    expect(thread_pool_free(pool) == 1, "Expected pool to be unable to be freed when running");
    expect(thread_pool_finish_and_stop(pool) == 0, "Expected pool to be finish and stop successfully");
    expect(thread_pool_free(pool) == 0, "Expected pool to be freed successfully");

    return NULL;
}

char *
test__thread_pool__full_service8()
{
    struct thread_pool *pool = thread_pool_create(3, 7, "Test");

    int sleep_duration_s = 2;

    expect(thread_pool_queue_job(pool, "test_job1", test_job, &sleep_duration_s) == 0,
           "Expected first job to be queued successfully");
    expect(thread_pool_queue_job(pool, "test_job2", test_job, &sleep_duration_s) == 0,
           "Expected second job to be queued successfully");
    expect(thread_pool_queue_job(pool, "test_job3", test_job, &sleep_duration_s) == 0,
           "Expected third job to be queued successfully");
    expect(thread_pool_queue_job(pool, "test_job4", test_job, &sleep_duration_s) == 0,
           "Expected fourth job to be queued successfully");
    expect(thread_pool_queue_job(pool, "test_job5", test_job, &sleep_duration_s) == 0,
           "Expected fifth job to be queued successfully");
    expect(thread_pool_queue_job(pool, "test_job6", test_job, &sleep_duration_s) == 0,
           "Expected sixth job to be queued successfully");
    expect(thread_pool_queue_job(pool, "test_job7", test_job, &sleep_duration_s) == 0,
           "Expected seventh job to be queued successfully");
    expect(thread_pool_queue_job(pool, "test_job8", test_job, &sleep_duration_s) == 1,
           "Expected eight job to not be queued successfully");

    expect(thread_pool_start(pool) == 0, "Expected pool to start successfully");
    expect(pool->state == RUNNING, "Expected pool status to be running");

    sleep(1);
    expect(pool->job_queue_size == 4, "Expected pool job_queue_size to be 4");

    expect(thread_pool_free(pool) == 1, "Expected pool to be unable to be freed when running");

    expect(thread_pool_wait_and_pause(pool) == 0, "Expected pool to be wait and pause successfully");
    expect(pool->job_queue_size == 4, "Expected pool job_queue_size to be 4 after pause");

    expect(thread_pool_start(pool) == 0, "Expected pool to start successfully");
    expect(pool->state == RUNNING, "Expected pool status to be running");

    sleep(1);
    expect(pool->job_queue_size == 1, "Expected pool job_queue_size to be 1");

    expect(thread_pool_free(pool) == 1, "Expected pool to be unable to be freed when running");
    expect(thread_pool_finish_and_stop(pool) == 0, "Expected pool to be finish and stop successfully");
    expect(thread_pool_free(pool) == 0, "Expected pool to be freed successfully");

    return NULL;
}
