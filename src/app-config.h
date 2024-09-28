#ifndef APP_CONFIG__H
#define APP_CONFIG__H

#include <stdbool.h>

#include "utils/log.h"

struct app_config
{
    bool is_daemon;
    enum log_level log_level;
    bool print_help;
};

char *app_config_init(int argc, char **argv, struct app_config *config);
char *get_help_message(char *app_name);

#endif
