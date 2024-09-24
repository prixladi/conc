#include <signal.h>
#include <unistd.h>

#include "utils/log.h"

#include "socket-server.h"
#include "protocol.h"
#include "manager.h"

static struct server *server;

static void graceful_stop_handler(int signal);
static void restart_handler(int signal);

static volatile bool running = true;

int main()
{
    char stdout_buffer[1024];
    setvbuf(stdout, stdout_buffer, _IOLBF, 1024);

    signal(SIGCHLD, SIG_IGN);

    signal(SIGTERM, graceful_stop_handler);
    signal(SIGINT, graceful_stop_handler);
    signal(SIGHUP, restart_handler);

    chdir("./run");

    while (running)
    {
        if (manager_init() != 0)
        {
            LOG_ERROR("(System) Unable to init the manager, exiting.");
            return 1;
        }

        struct server_options server_opts = {
            .dispatch = dispatch_command};

        server = server_run_async(server_opts);
        server_wait_and_free(server);

        manager_stop();
    }

    return 0;
}

static void graceful_stop_handler(int signal)
{
    running = false;
    LOG_INFO("(System) Received '%d' signal, exiting gracefully\n", signal);
    server_stop(server);
}

static void restart_handler(int signal)
{
    LOG_INFO("(System) Received '%d' signal, restarting\n", signal);
    server_stop(server);
}