#include <unistd.h>
#include <string.h>
#include <stdlib.h>

#include "utils/memory.h"
#include "utils/fs.h"

#include "app-config.h"

#ifdef __DEBUG__
#define DEFAULT_LOG_LEVEL DEBUG
#define DEFAULT_WORK_DIR "./tmp"
#else
#define DEFAULT_LOG_LEVEL INFO
#define DEFAULT_WORK_DIR NULL
#endif

static char *error_message_create(char *app_name, char *error);
static bool try_parse_log_level(char *str, enum log_level *level);

char *
app_config_init(int argc, char **argv, struct app_config *config)
{
    config->is_daemon = !isatty(STDOUT_FILENO);
    config->log_level = DEFAULT_LOG_LEVEL;
    config->print_help = false;
    config->work_dir = DEFAULT_WORK_DIR;

    for (int i = 1; i < argc; i++)
    {
        scoped char *error = NULL;

        if (strcmp(argv[i], "-h") == 0 || strcmp(argv[i], "--help") == 0)
            config->print_help = true;
        else if (strcmp(argv[i], "-d") == 0 || strcmp(argv[i], "--daemon") == 0)
            config->is_daemon = true;
        else if (strcmp(argv[i], "-l") == 0 || strcmp(argv[i], "--log-level") == 0)
        {
            i++;
            bool was_last = i >= argc;
            if (was_last)
                error = str_dup("Expected a log level");
            else if (!try_parse_log_level(argv[i], &config->log_level))
                error = str_printf("Invalid log level '%s'", argv[i]);
        }
        else if (strcmp(argv[i], "-w") == 0 || strcmp(argv[i], "--work-dir") == 0)
        {
            i++;
            bool was_last = i >= argc;
            if (was_last || strlen(argv[i]) == 0)
                error = str_dup("Expected a work dir");
            else if (!dir_exists(argv[i]))
                error = str_dup("Work directory does not exist");
            else
                config->work_dir = argv[i];
        }
        else
            error = str_printf("Invalid argument '%s'", argv[i]);

        if (error)
            return error_message_create(argv[0], error);
    }

    return NULL;
}

char *
get_help_message(char *app_name)
{
    return str_printf("Usage: %s [options]...\n\
Process manager service.\n\n\
Flags:\n\
    -l, --log-level <T|D|I|W|E|C>     Changes default log level\n\
    -h, --help                        Prints help\n\
    -d, --daemon                      Forces the app to run in daemon mode (defaults to true when run outside tty)\n\
    -w, --work-dir                    Working directory (for relative paths it is relative to the current work directory)\n\n\
Examples:\n\
    %s --log-level I                  Starts services with log level set to INFO\n\
    %s --log-level E                  Starts services with log level set to ERROR\n\
    %s --work-dir /var/lib/conc       Starts services with root work directory in /var/lib/conc\n\
    %s --daemon                       Starts services as a daemon\n",
                      app_name, app_name, app_name, app_name, app_name);
}

static char *
error_message_create(char *app_name, char *error)
{
    return str_printf("%s\nUsage: %s [options]..., run again with flag --help for more details\n", error, app_name);
}

static bool
try_parse_log_level(char *str, enum log_level *level)
{
    if (str == NULL || strlen(str) != 1)
        return false;

    char c = str[0];
    switch (c)
    {
    case 'T':
        (*level) = TRACE;
        return true;
    case 'D':
        (*level) = DEBUG;
        return true;
    case 'I':
        (*level) = INFO;
        return true;
    case 'W':
        (*level) = WARN;
        return true;
    case 'E':
        (*level) = ERROR;
        return true;
    case 'C':
        (*level) = CRITICAL;
        return true;
    default:
        return false;
    }
}
