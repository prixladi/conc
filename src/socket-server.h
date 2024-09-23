#ifndef SOCKET_SERVER__H
#define SOCKET_SERVER__H

#include <pthread.h>
#include <stdbool.h>

typedef char *(*Dispatch)(const char *command);

typedef struct ServerOptions
{
    Dispatch dispatch;
} ServerOptions;

typedef struct Server
{
    volatile ServerOptions opts;
    volatile pthread_t main_thread;
    volatile bool running;

} Server;

Server *server_run_async(ServerOptions opts);
void server_stop(Server *server);
void server_wait_and_free(Server *server);

#endif