#include <unistd.h>
#include <string.h>
#include <stdlib.h>

#include "app-config.h"

#ifdef __DEBUG__
#define DEFAULT_LOG_LEVEL DEBUG
#else
#define DEFAULT_LOG_LEVEL INFO
#endif

static char *error_message_create(char *app_name, char *error);
static bool try_parse_log_level(char *str, enum log_level *level);

char *
app_config_init(int argc, char **argv, struct app_config *config)
{
    config->is_daemon = !isatty(STDOUT_FILENO);
    config->log_level = DEFAULT_LOG_LEVEL;
    config->print_help = false;

    char *error = NULL;
    for (int i = 1; i < argc && !error; i++)
    {
        if (strcmp(argv[i], "-h") == 0 || strcmp(argv[i], "--help") == 0)
            config->print_help = true;
        else if (strcmp(argv[i], "-d") == 0 || strcmp(argv[i], "--daemon") == 0)
            config->is_daemon = true;
        else if (strcmp(argv[i], "-l") == 0 || strcmp(argv[i], "--log-level") == 0)
        {
            i++;
            bool was_last = i >= argc;
            if (was_last || !try_parse_log_level(argv[i], &config->log_level))
                error = was_last ? str_dup("Expected log level") : STR_CONCAT("Invalid log level '", argv[i], "'");
        }
        else
            error = STR_CONCAT("Invalid argument '", argv[i], "'");
    }

    if (error)
    {
        char *error_message = error_message_create(argv[0], error);
        free(error);
        return error_message;
    }

    return NULL;
}

char *
get_help_message(char *app_name)
{
    char *str = "Usage: %s [options]...\n\
Process manager service.\n\n\
    -l, --log-level <T|D|I|W|E|C>     Changes default log level\n\
    -h, --help                      Prints help\n\
    -d, --daemon                    Forces the app to run in daemon mode (defaults to true when run outside tty)\n\n\
Examples:\n\
    %s --log-level I        Starts services with log level set to INFO\n\
    %s --log-level E        Starts services with log level set to ERROR\n";

    int len = snprintf(NULL, 0, str, app_name, app_name, app_name);
    char *buff = malloc(sizeof(char *) * len + 1);
    snprintf(buff, len + 1, str, app_name, app_name, app_name);

    return buff;
}

static char *
error_message_create(char *app_name, char *error)
{
    char *str = "%s\nUsage: %s [options]..., run with again --help for more details\n";

    int len = snprintf(NULL, 0, str, error, app_name);
    char *buff = malloc(sizeof(char *) * len + 1);
    snprintf(buff, len + 1, str, error, app_name);

    return buff;
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
