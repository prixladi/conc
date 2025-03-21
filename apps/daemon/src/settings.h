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
    char *cwd;
    struct env_variable *env;
    struct service_settings *services;
};

char *project_settings_parse(const char *data, struct project_settings *settings);
char *environment_vars_parse(const char *data, struct env_variable **vars);

char *project_settings_stringify(const struct project_settings settings);

struct project_settings project_settings_dup(const struct project_settings settings);

void project_settings_free(struct project_settings settings);
void environment_vars_free(struct env_variable *vars);

#endif
