#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <pthread.h>
#include <poll.h>
#include <sys/types.h>
#include <sys/socket.h>
#include <sys/un.h>

#include "utils/log.h"
#include "utils/memory.h"
#include "utils/thread-pool.h"

#include "socket-server.h"

#define SOCKET_PATH "conc.sock"

#define BUFFER_SIZE 1024
#define MAX_WAITING_REQUESTS 10

#define THREAD_POOL_CONCURRENCY 5
#define THREAD_POOL_QUEUE_CAPACITY 1024

#define TRACE_NAME "socket_server"

struct server
{
    volatile struct server_options opts;
    volatile pthread_t main_thread;
    volatile bool running;
    int pipe_fds[2];
};

struct handler_options
{
    Dispatch dispatch;
    int client_socket;
};

static void *client_socket_handle(void *data);
static char *read_input(int client_socket, int *totalLength);

static void *server_run(void *data);

struct server *
server_run_async(struct server_options opts)
{
    struct server *server = malloc(sizeof(struct server));
    server->running = true;
    server->opts = opts;

    server->pipe_fds[0] = -1;
    server->pipe_fds[1] = -1;
    if (pipe(server->pipe_fds) != 0)
    {
        log_critical("Unable to create pipe for socket server\n");
        return NULL;
    }

    pthread_t thr;
    pthread_create(&thr, NULL, server_run, (void *)server);

    server->main_thread = thr;

    return server;
}

void
server_stop(struct server *server)
{
    server->running = false;
    char buf = 0;
    write(server->pipe_fds[1], &buf, 1);
}

void
server_wait_and_free(struct server *server)
{
    pthread_join(server->main_thread, NULL);
    if (server->pipe_fds[0] != -1)
        close(server->pipe_fds[0]);
    if (server->pipe_fds[1] != -1)
        close(server->pipe_fds[1]);
    free(server);
    log_info("Socket server stopped\n");
}

static void *
server_run(void *data)
{
    struct server *server = data;

    int server_socket = socket(AF_UNIX, SOCK_STREAM, 0);
    if (server_socket < 0)
    {
        log_critical("Unable to create server_socket FD\n");
        return NULL;
    }

    struct sockaddr_un server_addr = { .sun_family = AF_UNIX };
    strcpy(server_addr.sun_path, SOCKET_PATH);

    unlink(SOCKET_PATH);
    if (bind(server_socket, (struct sockaddr *)&server_addr, sizeof(server_addr)) < 0)
    {
        log_critical("Unable to bind socket server port\n");
        return NULL;
    }

    listen(server_socket, MAX_WAITING_REQUESTS);

    log_info("Socket server started\n");

    struct thread_pool *pool = thread_pool_create(THREAD_POOL_CONCURRENCY, THREAD_POOL_QUEUE_CAPACITY, TRACE_NAME);
    thread_pool_start(pool);

    while (server->running)
    {
        struct pollfd fds[2] = {
            { .fd = server_socket, .events = POLLIN },
            { .fd = server->pipe_fds[0], .events = POLLIN },
        };

        int poll_status = poll(fds, 2, -1);
        if (poll_status <= 0 || !(fds[0].revents & POLLIN) || !server->running)
        {
            if (poll_status < 0)
                log_error("Socket server poll returned %d while polling for data from socket.\n", poll_status);
            continue;
        }

        struct sockaddr_un client_addr;
        unsigned int clen = sizeof(client_addr);

        int client_socket = accept(server_socket, (struct sockaddr *)&client_addr, &clen);
        log_trace(TRACE_NAME, "Accepted socket connection '%d'\n", client_socket);

        struct handler_options *handler_opts = malloc(sizeof(struct handler_options));
        handler_opts->dispatch = server->opts.dispatch;
        handler_opts->client_socket = client_socket;

        if (thread_pool_queue_job(pool, NULL, client_socket_handle, handler_opts) != 0)
        {
            log_critical("Unable to queue client socket handle to on threadpool.");
            shutdown(client_socket, SHUT_WR);
            close(client_socket);
        }
    }

    log_info("Socket server stopping\n");

    thread_pool_finish_and_stop(pool);
    thread_pool_free(pool);

    return NULL;
}

static void *
client_socket_handle(void *data)
{
    scoped struct handler_options *opts = data;

    int total_length = 0;
    scoped char *input = read_input(opts->client_socket, &total_length);
    shutdown(opts->client_socket, SHUT_RD);

    // one character message containing just '\0' is threated as a health check
    bool is_health_check = input[0] == '\0';
    scoped char *response = NULL;
    if (is_health_check)
    {
        log_trace(TRACE_NAME, "Received health check from connection '%d'\n", opts->client_socket);
        response = calloc(1, sizeof(char));
    }
    else
    {
        log_trace(TRACE_NAME, "Received command '%s' from connection '%d'\n", input, opts->client_socket);
        response = opts->dispatch(input);
        log_trace(TRACE_NAME, "Sending response '%s' to connection '%d'\n", response, opts->client_socket);
    }

    write(opts->client_socket, response, strlen(response) + 1); // we also want to send '\0' as a end of message indicator
    shutdown(opts->client_socket, SHUT_WR);

    log_trace(TRACE_NAME, "Closing socket connection '%d'\n", opts->client_socket);
    if (close(opts->client_socket) != 0)
        log_error("Unable to close client socket '%d'\n", opts->client_socket);

    return NULL;
}

static char *
read_input(int client_socket, int *totalLength)
{
    char *input = calloc(1, sizeof(char));
    char buffer[BUFFER_SIZE + 1];
    int len = 0;

    while ((len = read(client_socket, buffer, BUFFER_SIZE)) > 0)
    {
        (*totalLength) += len;
        input = realloc(input, sizeof(char) * (*totalLength) + 1);
        buffer[len] = '\0';
        strncat(input, buffer, len);

        if (buffer[len - 1] == '\0')
            break;
    }

    return input;
}
