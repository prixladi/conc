#include <signal.h>
#include <unistd.h>
#include <stdlib.h>

#include "utils/log.h"

#include "socket-server.h"
#include "protocol.h"
#include "manager.h"
#include "app-config.h"

static void graceful_stop_handler(int signal);
static void restart_handler(int signal);

static struct server *server;
static volatile bool restarting = false;

int
main(int argc, char **argv)
{
    char stdout_buffer[1024];
    setvbuf(stdout, stdout_buffer, _IOLBF, 1024);

    struct app_config config;
    char *config_error = app_config_init(argc, argv, &config);
    if (config_error)
    {
        fprintf(stderr, "%s", config_error);
        free(config_error);
        return 1;
    }

    if (config.print_help)
    {
        char *help_message = get_help_message(argv[0]);
        printf("%s", help_message); // intentionally not using log_* to ignore log level
        free(help_message);
        return config.is_daemon ? 1 : 0; // deamon should never ask for help
    }

    if (config.work_dir)
        chdir(config.work_dir);

    log_init(config.log_level);

    signal(SIGCHLD, SIG_IGN);
    signal(SIGPIPE, SIG_IGN);

    signal(SIGTERM, graceful_stop_handler);
    signal(SIGINT, graceful_stop_handler);
    // systemd sends 'hang up' with expectation of a restart
    signal(SIGHUP, config.is_daemon ? restart_handler : graceful_stop_handler);

    do
    {
        restarting = false;
        if (manager_init() != 0)
        {
            log_critical("Unable to init the manager, exiting.\n");
            return 1;
        }

        struct server_options server_opts = {
            .dispatch = dispatch_command,
        };

        server = server_run_async(server_opts);
        server_wait_and_free(server);
        server = NULL;

        manager_stop();
    } while (restarting);

    return 0;
}

static void
graceful_stop_handler(int signal)
{
    (void)signal;
    // Intentionally not using 'log_*' or 'printf' because it uses non-async-signal-safe functions
    write(STDOUT_FILENO, "[SGN] Received terminate signal, stopping\n", 43);
    if (server != NULL)
        server_stop(server);
}

static void
restart_handler(int signal)
{
    (void)signal;
    restarting = true;
    // Intentionally not using 'log_*' or 'printf' because it uses non-async-signal-safe functions
    write(STDOUT_FILENO, "[SGN] Received restart signal, restarting\n", 43);
    if (server != NULL)
        server_stop(server);
}
