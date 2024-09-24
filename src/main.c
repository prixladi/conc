#include <signal.h>
#include <unistd.h>

#include "utils/log.h"

#include "socket-server.h"
#include "protocol.h"
#include "manager.h"

static Server *server;

void sig_term_int_handler(int signal)
{
    LOG_INFO("(System) Received '%d' signal, exiting gracefully\n", signal);
    server_stop(server);
}

int main()
{
    signal(SIGCHLD, SIG_IGN);
    signal(SIGTERM, sig_term_int_handler);
    signal(SIGINT, sig_term_int_handler);

    chdir("./run");

    if (manager_init() != 0)
    {
        LOG_ERROR("(System) Unable to init the manager, exiting.");
        return 1;
    }

    ServerOptions server_opts = {
        .dispatch = dispatch_command};

    server = server_run_async(server_opts);
    server_wait_and_free(server);

    manager_stop();

    return 0;
}