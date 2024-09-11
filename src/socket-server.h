#ifndef SOCKET_SERVER__H
#define SOCKET_SERVER__H

#include <pthread.h>

#define BUFFER_SIZE 10

#define MAX_WAITING_REQUESTS 10
#define SOCKET_PATH "conc.sock"

typedef char *(*Dispatch)(const char *command);

typedef struct ServerOptions
{
    Dispatch dispatch;
} ServerOptions;

typedef struct Server
{
    pthread_t main_thread;
} Server;

Server server_run_async(ServerOptions opts);
void server_wait(Server server);

#endif