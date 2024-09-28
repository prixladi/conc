#ifndef SETTINGS__H
#define SETTINGS__H

struct env_variable
{
    char *key;
    char *value;
};

struct service_settings
{
    char *name;
    char *pwd;
    struct env_variable *env;
    char **command;
};

struct project_settings
{
    char *name;
    struct service_settings *services;
};

char *project_settings_parse(const char *data, struct project_settings *settings);
char *project_settings_stringify(const struct project_settings settings);

struct project_settings project_settings_dup(const struct project_settings settings);
struct service_settings service_settings_dup(const struct service_settings settings);

void project_settings_free(struct project_settings settings);
void service_settings_free(struct service_settings settings);

#endif
