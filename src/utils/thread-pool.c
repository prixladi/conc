#include <pthread.h>
#include <stdlib.h>
#include <stdbool.h>
#include <unistd.h>
#include <time.h>

#include "log.h"
#include "string.h"
#include "thread-pool.h"

#define DEFAULT_THREAD_POOL_NAME "default"
#define DEFAULT_THREAD_POOL_NAME_SUFFIX "_thread_pool"
#define DEFAULT_THREAD_POOL_JOB_NAME() int_to_str(rand())

#define MAX_THREAD_POOL_CAPACITY 1024

enum thread_pool_state
{
	IDLE,
	RUNNING,
	EXITING,
};

struct thread_pool_job
{
	char *name;
	void *(*run)(void *);
	void *args;

	struct thread_pool_job *next;
};

struct thread_pool
{
	char *name;
	int size;

	int job_queue_capacity;
	int job_queue_size;

	struct thread_pool_job *first_job;
	struct thread_pool_job *last_job;

	pthread_mutex_t *lock;
	pthread_cond_t *cond;

	enum thread_pool_state state;
	pthread_t *worker_threads;
};

static void *run_worker(void *arg);
static void thread_pool_job_free(struct thread_pool_job *job);
static void wait_for_workers(struct thread_pool *pool);

struct thread_pool *thread_pool_create(int size, int capacity, char *_name)
{
	char *name = STR_CONCAT(_name == NULL ? DEFAULT_THREAD_POOL_NAME : _name, DEFAULT_THREAD_POOL_NAME_SUFFIX);

	if (size < 1)
	{
		log_error("Unable to initialize thread_pool '%s' with size '%d'\n", name, size);
		return NULL;
	}

	pthread_mutex_t *lock = malloc(sizeof(pthread_mutex_t));
	if (pthread_mutex_init(lock, NULL) != 0)
	{
		free(lock);
		log_error("Unable to initialize thread_pool '%s' lock\n", name);
		return NULL;
	}

	pthread_cond_t *cond = malloc(sizeof(pthread_cond_t));
	if (pthread_cond_init(cond, NULL) != 0)
	{
		free(cond);
		log_error("Unable to initialize thread_pool '%s' cond\n", name);
		return NULL;
	}

	struct thread_pool *pool = calloc(1, sizeof(struct thread_pool));

	pool->name = name;
	pool->size = size;
	pool->job_queue_capacity = capacity;
	pool->job_queue_size = 0;
	pool->lock = lock;
	pool->cond = cond;

	return pool;
}

int thread_pool_start(struct thread_pool *pool)
{
	pthread_mutex_lock(pool->lock);

	if (pool->state != IDLE)
	{
		log_error("Thread pool '%s' is already running\n", pool->name);
		pthread_mutex_unlock(pool->lock);
		return 1;
	}

	pool->state = RUNNING;
	pool->worker_threads = malloc(sizeof(pthread_t) * pool->size);

	log_info("Starting thread_pool '%s'\n", pool->name);
	for (int i = 0; i < pool->size; i++)
	{
		log_trace(pool->name, "Starting worker - '%d'\n", i);
		pthread_t thr;
		pthread_create(&thr, NULL, run_worker, (void *)pool);
		pool->worker_threads[i] = thr;
	}

	pthread_mutex_unlock(pool->lock);

	return 0;
}

int thread_pool_queue_job(struct thread_pool *pool, char *name, void *(*run)(void *), void *args)
{
	struct thread_pool_job *job = calloc(1, sizeof(struct thread_pool_job));
	job->name = name == NULL ? DEFAULT_THREAD_POOL_JOB_NAME() : str_dup(name);
	job->run = run;
	job->args = args;

	pthread_mutex_lock(pool->lock);

	if (pool->job_queue_capacity > 0 && pool->job_queue_size >= pool->job_queue_capacity)
	{
		pthread_mutex_unlock(pool->lock);
		thread_pool_job_free(job);
		log_error("Thread pool '%s' is full, max capacity: %d\n", pool->name, pool->job_queue_capacity);
		return 1;
	}

	if (!pool->first_job)
		pool->first_job = job;

	if (pool->last_job)
		pool->last_job->next = job;

	pool->last_job = job;
	pool->job_queue_size += 1;

	log_trace(pool->name, "Queued job - '%s'\n", job->name);

	pthread_mutex_unlock(pool->lock);
	pthread_cond_signal(pool->cond);

	return 0;
}

int thread_pool_stop_and_wait(struct thread_pool *pool)
{
	log_info("Thread_pool '%s' stopping\n", pool->name);

	pthread_mutex_lock(pool->lock);
	if (pool->state != RUNNING)
	{
		pthread_mutex_unlock(pool->lock);
		log_error("Unable to stop thread_pool '%s' it is not running\n", pool->name);
		return 1;
	}

	pool->state = EXITING;

	pthread_mutex_unlock(pool->lock);
	pthread_cond_broadcast(pool->cond);

	wait_for_workers(pool);

	log_info("Thread_pool '%s' stopped\n", pool->name);

	pool->state = IDLE;

	return 0;
}

int thread_pool_pause_and_wait(struct thread_pool *pool)
{
	log_info("Thread_pool '%s' pausing\n", pool->name);

	pthread_mutex_lock(pool->lock);
	if (pool->state != RUNNING)
	{
		pthread_mutex_unlock(pool->lock);
		log_error("Unable to pause thread_pool '%s' it is not running\n", pool->name);
		return 1;
	}

	pool->state = IDLE;

	pthread_cond_broadcast(pool->cond);
	pthread_mutex_unlock(pool->lock);

	wait_for_workers(pool);

	log_info("Thread_pool '%s' paused\n", pool->name);

	return 0;
}

int thread_pool_free(struct thread_pool *pool)
{
	log_trace(pool->name, "Destroying thread_pool\n");

	pthread_mutex_lock(pool->lock);
	if (pool->state != IDLE)
	{
		pthread_mutex_unlock(pool->lock);
		log_error("Unable to free thread_pool '%s' it is not idle\n", pool->name);
		return 1;
	}

	free(pool->name);
	free(pool->worker_threads);

	struct thread_pool_job *job = pool->first_job;
	while (job)
	{
		struct thread_pool_job *tmp = job->next;
		thread_pool_job_free(job);
		job = tmp;
	}

	pthread_cond_destroy(pool->cond);
	pthread_mutex_t *lock = pool->lock;
	free(pool->cond);

	pool->name = NULL;
	pool->cond = NULL;
	pool->lock = NULL;
	pool->first_job = NULL;
	pool->last_job = NULL;
	pool->worker_threads = NULL;

	free(pool);

	pthread_mutex_unlock(lock);
	pthread_mutex_destroy(lock);
	free(lock);

	return 0;
}

static void *run_worker(void *arg)
{
	struct thread_pool *pool = arg;
	while (1)
	{
		pthread_mutex_lock(pool->lock);

		if (pool->state == IDLE)
		{
			log_trace(pool->name, "State of the thread_pool is 'idle', exiting worker\n");
			pthread_mutex_unlock(pool->lock);
			break;
		}

		if (pool->first_job)
		{
			struct thread_pool_job *job = pool->first_job;
			pool->first_job = job->next;
			pool->job_queue_size -= 1;
			if (!pool->first_job)
				pool->last_job = NULL;

			pthread_mutex_unlock(pool->lock);

			log_trace(pool->name, "Executing job - '%s'\n", job->name);
			job->run(job->args);
			log_trace(pool->name, "Executed job - '%s'\n", job->name);

			thread_pool_job_free(job);
		}
		else if (pool->state == EXITING)
		{
			log_trace(pool->name,
				  "State of the thread_pool is 'exiting' and the queue is empty, exiting worker\n");
			pthread_mutex_unlock(pool->lock);
			break;
		}
		else
		{
			log_trace(pool->name, "Queue of the thread_pool is empty, waiting for a signal\n");
			pthread_cond_wait(pool->cond, pool->lock);
			pthread_mutex_unlock(pool->lock);
		}
	}

	return NULL;
}

static void wait_for_workers(struct thread_pool *pool)
{
	log_trace(pool->name, "Waiting for worker threads to finish\n");
	for (int i = 0; i < pool->size; i++)
	{
		pthread_join(pool->worker_threads[i], NULL);
		log_trace(pool->name, "Joined worker - '%d'\n", i);
	}
}

static void thread_pool_job_free(struct thread_pool_job *job)
{
	free(job->name);

	job->name = NULL;
	job->run = NULL;
	job->args = NULL;

	free(job);
}
