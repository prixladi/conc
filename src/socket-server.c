#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <pthread.h>
#include <sys/types.h>
#include <sys/socket.h>
#include <sys/un.h>

#include "utils/log.h"

#include "socket-server.h"

typedef struct HandlerOptions
{
    Dispatch dispatch;
    int client_socket;
} HandlerOptions;

static void *client_socket_handle(void *data);

static void *server_run(void *data);

Server server_run_async(ServerOptions opts)
{
    ServerOptions *opts_ptr = malloc(sizeof(ServerOptions));
    memcpy(opts_ptr, &opts, sizeof(ServerOptions));

    pthread_t thr;
    pthread_create(&thr, NULL, server_run, (void *)opts_ptr);

    Server server = {.main_thread = thr};
    return server;
}

void server_wait(Server server)
{
    pthread_join(server.main_thread, NULL);
}

static void *server_run(void *data)
{
    ServerOptions *opts = (ServerOptions *)data;

    LOG_INFO("Starting socket server\n");
    int server_socket = socket(AF_UNIX, SOCK_STREAM, 0);

    struct sockaddr_un server_addr;
    server_addr.sun_family = AF_UNIX;
    strcpy(server_addr.sun_path, SOCKET_PATH);

    unlink(SOCKET_PATH);
    bind(server_socket, (struct sockaddr *)&server_addr, sizeof(server_addr));

    listen(server_socket, MAX_WAITING_REQUESTS);

    int i = 10;
    while (i--)
    {
        struct sockaddr_un client_addr;
        unsigned int clen = sizeof(client_addr);

        int client_socket = accept(server_socket, (struct sockaddr *)&client_addr, &clen);
        LOG_DEBUG("Accepted socket connection '%d'\n", client_socket);

        HandlerOptions *handler_opts = malloc(sizeof(HandlerOptions));
        handler_opts->dispatch = opts->dispatch;
        handler_opts->client_socket = client_socket;

        // TODO: Implement some sort of thread_pool
        pthread_t thr;
        pthread_create(&thr, NULL, client_socket_handle, (void *)handler_opts);
    }
    sleep(2);

    free(opts);

    return NULL;
}

static void *client_socket_handle(void *data)
{
    HandlerOptions *opts = data;
    int client_socket = opts->client_socket;
    Dispatch dispatch = opts->dispatch;
    free(opts);

    char *input = calloc(1, sizeof(char));
    char buffer[BUFFER_SIZE + 1];
    int totalLength;
    int len = 0;

    while ((len = read(client_socket, buffer, BUFFER_SIZE)) > 0)
    {
        totalLength += len;
        input = realloc(input, sizeof(char) * totalLength + 1);
        buffer[len] = '\0';
        strcat(input, buffer);

        if (buffer[len - 1] == '\0')
            break;
    }

    LOG_DEBUG("Received command '%s' from connection '%d'\n", input, client_socket);

    char *response = dispatch(input);
    write(client_socket, response, strlen(response) + 1); // we also want to send '\0' as a end of message indicator

    free(input);
    free(response);

    LOG_DEBUG("Closing socket connection '%d'\n", client_socket);
    close(client_socket);

    return NULL;
}