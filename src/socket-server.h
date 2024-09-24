#ifndef SOCKET_SERVER__H
#define SOCKET_SERVER__H

#include <pthread.h>
#include <stdbool.h>

typedef char *(*Dispatch)(const char *command);

struct server_options
{
    Dispatch dispatch;
};

struct server
{
    volatile struct server_options opts;
    volatile pthread_t main_thread;
    volatile bool running;
};

struct server *server_run_async(struct server_options opts);
void server_stop(struct server *server);
void server_wait_and_free(struct server *server);

#endif