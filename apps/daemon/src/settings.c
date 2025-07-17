#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>
#include <ctype.h>
#include <assert.h>

#include "external/cJSON.h"

#include "utils/log.h"
#include "utils/string.h"
#include "utils/vector.h"

#include "settings.h"

#define SETTINGS_PARSE_ERROR() str_dup("settings.parse")
#define SETTINGS_INVALID_NAME_ERROR() str_dup("settings.name.invalid")
#define SETTINGS_INVALID_CWD_ERROR() str_dup("settings.cwd.invalid")
#define SETTINGS_MISSING_SERVICES_ERROR() str_dup("settings.services.missing")
#define SETTINGS_INVALID_SERVICE_NAME_ERROR(s) str_printf("settings.service.%s.name.invalid", s ? s : "")
#define SETTINGS_DUPLICATE_SERVICE_NAME_ERROR(s) str_printf("settings.service.%s.name.duplicate", s ? s : "")
#define SETTINGS_INVALID_SERVICE_COMMAND_ERROR(s) str_printf("settings.service.%s.command.invalid", s ? s : "")

#define ENV_PARSE_ERROR() str_dup("env.parse")

static struct service_settings service_settings_parse(struct cJSON *json);
static struct env_variable *env_vars_parse(struct cJSON *js);

static struct service_settings service_settings_dup(const struct service_settings settings);
static struct env_variable *env_variable_dup(struct env_variable *env);

static void service_settings_free(struct service_settings settings);
static void env_variable_free(struct env_variable e);

static bool is_name_valid(const char *name);
static bool is_cwd_valid(const char *cwd);

char *
project_settings_parse(const char *data, struct project_settings *settings)
{
    settings->services = vec_create(struct service_settings);

    cJSON *json = cJSON_Parse(data);
    if (json == NULL)
        return SETTINGS_PARSE_ERROR();

    cJSON *js = json->child;
    do
    {
        if (strcmp(js->string, "name") == 0)
            settings->name = str_dup(js->valuestring);
        if (strcmp(js->string, "cwd") == 0)
            settings->cwd = str_dup(js->valuestring);
        if (strcmp(js->string, "env") == 0 && js->type == cJSON_Object && js->child)
            settings->env = env_vars_parse(js->child);

        if (strcmp(js->string, "services") == 0 && js->type == cJSON_Array && js->child)
        {
            cJSON *arr = js->child;
            do
            {
                struct service_settings service = service_settings_parse(arr);

                if (!is_name_valid(service.name))
                {
                    cJSON_Delete(json);
                    char *error = service.name ? SETTINGS_INVALID_SERVICE_NAME_ERROR(service.name)
                                               : SETTINGS_INVALID_SERVICE_NAME_ERROR("");
                    service_settings_free(service);
                    return error;
                }

                if (vec_length(service.command) < 1)
                {
                    cJSON_Delete(json);
                    char *error = SETTINGS_INVALID_SERVICE_COMMAND_ERROR(service.name);
                    service_settings_free(service);
                    return error;
                }

                for (size_t i = 0; i < vec_length(settings->services); i++)
                {
                    assert(service.name);
                    if (strcmp(service.name, settings->services[i].name) == 0)
                    {
                        cJSON_Delete(json);
                        char *error = SETTINGS_DUPLICATE_SERVICE_NAME_ERROR(service.name);
                        service_settings_free(service);
                        return error;
                    }
                }

                vec_push(settings->services, service);

            } while ((arr = arr->next));
        }
    } while (((js = js->next)));

    if (settings->env == NULL)
        settings->env = vec_create(struct env_variable);

    cJSON_Delete(json);

    if (!is_name_valid(settings->name))
        return SETTINGS_INVALID_NAME_ERROR();
    if (!is_cwd_valid(settings->cwd))
        return SETTINGS_INVALID_CWD_ERROR();
    if (vec_length(settings->services) < 1)
        return SETTINGS_MISSING_SERVICES_ERROR();

    return NULL;
}

char *
environment_vars_parse(const char *data, struct env_variable **vars)
{
    cJSON *json = cJSON_Parse(data);
    if (json == NULL || json->type != cJSON_Object)
    {
        cJSON_Delete(json);
        return ENV_PARSE_ERROR();
    }

    (*vars) = env_vars_parse(json->child);
    cJSON_Delete(json);
    return NULL;
}

char *
project_settings_stringify(const struct project_settings settings)
{
    cJSON *root = cJSON_CreateObject();
    cJSON_AddItemToObject(root, "name", cJSON_CreateString(settings.name));

    cJSON_AddItemToObject(root, "cwd", cJSON_CreateString(settings.cwd));

    cJSON *env = cJSON_CreateObject();
    cJSON_AddItemToObject(root, "env", env);
    for (size_t i = 0; i < vec_length(settings.env); i++)
    {
        struct env_variable e = settings.env[i];
        cJSON_AddItemToObject(env, e.key, cJSON_CreateString(e.value));
    }

    cJSON *services = cJSON_CreateArray();
    cJSON_AddItemToObject(root, "services", services);
    for (size_t i = 0; i < vec_length(settings.services); i++)
    {
        struct service_settings service_settings = settings.services[i];
        cJSON *service = cJSON_CreateObject();
        cJSON_AddItemToArray(services, service);

        cJSON_AddItemToObject(service, "name", cJSON_CreateString(service_settings.name));
        if (service_settings.pwd)
            cJSON_AddItemToObject(service, "pwd", cJSON_CreateString(service_settings.pwd));

        cJSON *command = cJSON_CreateArray();
        cJSON_AddItemToObject(service, "command", command);
        for (size_t j = 0; j < vec_length(service_settings.command); j++)
            cJSON_AddItemToArray(command, cJSON_CreateString(service_settings.command[j]));

        cJSON *env = cJSON_CreateObject();
        cJSON_AddItemToObject(service, "env", env);
        for (size_t j = 0; j < vec_length(service_settings.env); j++)
        {
            struct env_variable e = service_settings.env[j];
            cJSON_AddItemToObject(env, e.key, cJSON_CreateString(e.value));
        }
    }

    char *result = cJSON_PrintUnformatted(root);
    cJSON_Delete(root);

    return result;
}

struct project_settings
project_settings_dup(const struct project_settings settings)
{
    struct project_settings copy = { 0 };
    copy.name = str_dup(settings.name);
    copy.cwd = str_dup(settings.cwd);
    copy.env = env_variable_dup(settings.env);

    size_t service_count = vec_length(settings.services);
    copy.services = vec_create_prealloc(struct service_settings, service_count);

    for (size_t i = 0; i < service_count; i++)
    {
        struct service_settings service_settings = service_settings_dup(settings.services[i]);
        vec_push(copy.services, service_settings);
    }

    return copy;
}

void
project_settings_free(struct project_settings settings)
{
    if (settings.services != NULL)
    {
        vec_for_each(settings.services, service_settings_free);
        vec_free(settings.services);
    }
    if (settings.env != NULL)
    {
        environment_vars_free(settings.env);
    }
    free(settings.name);
    free(settings.cwd);

    settings.name = NULL;
    settings.env = NULL;
    settings.cwd = NULL;
    settings.services = NULL;
}

void
environment_vars_free(struct env_variable *vars)
{
    vec_for_each(vars, env_variable_free);
    vec_free(vars);
}

static struct service_settings
service_settings_parse(struct cJSON *json)
{
    struct service_settings settings = { 0 };
    settings.command = vec_create_prealloc(char *, 2);

    struct cJSON *js = json->child;
    do
    {
        if (strcmp(js->string, "name") == 0)
            settings.name = str_dup(js->valuestring);
        if (strcmp(js->string, "pwd") == 0)
            settings.pwd = str_dup(js->valuestring);
        if (strcmp(js->string, "env") == 0 && js->type == cJSON_Object && js->child)
            settings.env = env_vars_parse(js->child);
        if (strcmp(js->string, "command") == 0 && js->type == cJSON_Array && js->child)
        {
            struct cJSON *cmd = js->child;
            do
            {
                char *commandPart = str_dup(cmd->valuestring);
                vec_push(settings.command, commandPart);
            } while ((cmd = cmd->next));
        }

    } while ((js = js->next));

    if (settings.env == NULL)
        settings.env = vec_create(struct env_variable);

    return settings;
}

static struct env_variable *
env_vars_parse(struct cJSON *js)
{
    struct env_variable *env_vars = vec_create(struct env_variable);
    struct cJSON *e = js;
    while (e)
    {
        if (e->type != cJSON_String)
            continue;

        struct env_variable env_var = {
            .key = str_dup(e->string),
            .value = str_dup(e->valuestring),
        };
        vec_push(env_vars, env_var);
        e = e->next;
    };

    return env_vars;
}

static struct service_settings
service_settings_dup(const struct service_settings settings)
{
    struct service_settings copy = { 0 };
    copy.name = str_dup(settings.name);
    if (settings.pwd)
        copy.pwd = str_dup(settings.pwd);
    copy.env = env_variable_dup(settings.env);

    size_t command_len = vec_length(settings.command);
    copy.command = vec_create_prealloc(char *, command_len);
    for (size_t i = 0; i < command_len; i++)
    {
        char *part = str_dup(settings.command[i]);
        vec_push(copy.command, part);
    }

    return copy;
}

static struct env_variable *
env_variable_dup(struct env_variable *env)
{
    size_t env_len = vec_length(env);
    struct env_variable *new_env = vec_create_prealloc(struct env_variable, env_len);
    for (size_t i = 0; i < env_len; i++)
    {
        struct env_variable e = {
            .key = str_dup(env[i].key),
            .value = str_dup(env[i].value),
        };
        vec_push(new_env, e);
    }

    return new_env;
}

static void
service_settings_free(struct service_settings settings)
{
    free(settings.name);
    free(settings.pwd);
    if (settings.env != NULL)
    {
        vec_for_each(settings.env, env_variable_free);
        vec_free(settings.env);
    }
    if (settings.command != NULL)
    {
        vec_for_each(settings.command, free);
        vec_free(settings.command);
    }

    settings.name = NULL;
    settings.pwd = NULL;
    settings.command = NULL;
    settings.env = NULL;
}

static void
env_variable_free(struct env_variable e)
{
    free(e.key);
    free(e.value);

    e.key = NULL;
    e.value = NULL;
}

static bool
is_name_valid(const char *name)
{
    if (name == NULL)
        return false;

    size_t len = strlen(name);

    for (size_t i = 0; i < len; i++)
    {
        char c = name[i];
        if (!isalnum(name[i]) && c != '_' && c != '-')
            return false;
    }

    return len > 0;
}

static bool
is_cwd_valid(const char *cwd)
{
    if (cwd == NULL)
        return false;

    return strlen(cwd) > 0;
}
