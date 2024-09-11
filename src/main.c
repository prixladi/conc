#include <signal.h>
#include <unistd.h>

#include "socket-server.h"
#include "protocol.h"
#include "manager.h"

int main()
{
    signal(SIGCHLD, SIG_IGN);
    chdir("./run");

    manager_init();

    ServerOptions server_opts = {
        .dispatch = dispatch};

    Server server = server_run_async(server_opts);
    server_wait(server);

    manager_free();

    return 0;
}