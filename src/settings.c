#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>
#include <ctype.h>

#include "external/cJSON.h"

#include "utils/log.h"
#include "utils/string.h"
#include "utils/vector.h"

#include "settings.h"

#define SETTINGS_PARSE_ERROR() str_dup("settings.parse")
#define SETTINGS_INVALID_NAME_ERROR() str_dup("settings.name.invalid")
#define SETTINGS_MISSING_SERVICES_ERROR() str_dup("settings.services.missing")
#define SETTINGS_INVALID_SERVICE_NAME_ERROR(s) str_concat("settings.service.", s, ".name.invalid", NULL)
#define SETTINGS_INVALID_SERVICE_COMMAND_ERROR(s) str_concat("settings.service.", s, ".command.invalid", NULL)

static struct service_settings service_settings_parse(cJSON *json);
static inline bool is_name_valid(const char *name);

char *project_settings_parse(const char *data, struct project_settings *settings)
{
    settings->services = vector_create(struct service_settings);

    cJSON *json = cJSON_Parse(data);
    if (json == NULL)
        return SETTINGS_PARSE_ERROR();

    cJSON *js = json->child;
    do
    {
        if (strcmp(js->string, "name") == 0)
            settings->name = str_dup(js->valuestring);

        if (strcmp(js->string, "services") == 0 && js->type == cJSON_Array && js->child)
        {
            cJSON *arr = js->child;
            do
            {
                struct service_settings service = service_settings_parse(arr);
                vector_push(settings->services, service);

                if (!is_name_valid(service.name))
                {
                    cJSON_Delete(json);
                    return service.name
                               ? SETTINGS_INVALID_SERVICE_NAME_ERROR(service.name)
                               : SETTINGS_INVALID_SERVICE_NAME_ERROR("");
                }

                if (vector_length(service.command) < 1)
                {
                    cJSON_Delete(json);
                    return SETTINGS_INVALID_SERVICE_COMMAND_ERROR(service.name);
                }

            } while ((arr = arr->next));
        }
    } while (((js = js->next)));

    cJSON_Delete(json);

    if (!is_name_valid(settings->name))
        return SETTINGS_INVALID_NAME_ERROR();

    if (vector_length(settings->services) < 1)
        return SETTINGS_MISSING_SERVICES_ERROR();

    return NULL;
}

char *project_settings_stringify(const struct project_settings settings)
{
    cJSON *root = cJSON_CreateObject();
    cJSON_AddItemToObject(root, "name", cJSON_CreateString(settings.name));

    cJSON *services = cJSON_CreateArray();
    cJSON_AddItemToObject(root, "services", services);

    for (size_t i = 0; i < vector_length(settings.services); i++)
    {
        struct service_settings service_settings = settings.services[i];

        cJSON *service = cJSON_CreateObject();
        cJSON_AddItemToArray(services, service);
        cJSON_AddItemToObject(service, "name", cJSON_CreateString(service_settings.name));
        if (service_settings.pwd)
            cJSON_AddItemToObject(service, "pwd", cJSON_CreateString(service_settings.pwd));

        cJSON *command = cJSON_CreateArray();
        cJSON_AddItemToObject(service, "command", command);

        for (size_t j = 0; j < vector_length(service_settings.command); j++)
            cJSON_AddItemToArray(command, cJSON_CreateString(service_settings.command[j]));
    }

    char *result = cJSON_PrintUnformatted(root);
    cJSON_Delete(root);

    return result;
}

struct project_settings project_settings_dup(const struct project_settings settings)
{
    struct project_settings copy;
    copy.name = str_dup(settings.name);

    size_t service_count = vector_length(settings.services);
    copy.services = vector_create_prealloc(struct service_settings, service_count);

    for (size_t i = 0; i < service_count; i++)
    {
        struct service_settings service_settings = service_settings_dup(settings.services[i]);
        vector_push(copy.services, service_settings);
    }

    return copy;
}

struct service_settings service_settings_dup(const struct service_settings settings)
{
    struct service_settings copy;
    copy.name = str_dup(settings.name);
    if (settings.pwd)
        copy.pwd = str_dup(settings.pwd);

    size_t command_len = vector_length(settings.command);
    copy.command = vector_create_prealloc(char *, command_len);

    for (size_t i = 0; i < command_len; i++)
    {
        char *part = str_dup(settings.command[i]);
        vector_push(copy.command, part);
    }

    return copy;
}

void project_settings_free(struct project_settings settings)
{
    if (settings.services != NULL)
    {
        vector_for_each(settings.services, service_settings_free);
        vector_free(settings.services);
    }
    free(settings.name);

    settings.name = NULL;
    settings.services = NULL;
}

void service_settings_free(struct service_settings settings)
{
    free(settings.name);
    free(settings.pwd);
    if (settings.command != NULL)
    {
        vector_for_each(settings.command, free);
        vector_free(settings.command);
    }

    settings.name = NULL;
    settings.pwd = NULL;
    settings.command = NULL;
}

static struct service_settings service_settings_parse(cJSON *json)
{
    struct service_settings settings = {0};
    settings.command = vector_create_prealloc(char *, 2);

    cJSON *js = json->child;
    do
    {
        if (strcmp(js->string, "name") == 0)
            settings.name = str_dup(js->valuestring);
        if (strcmp(js->string, "pwd") == 0)
            settings.pwd = str_dup(js->valuestring);
        if (strcmp(js->string, "command") == 0 && js->type == cJSON_Array && js->child)
        {
            cJSON *cmd = js->child;
            do
            {
                char *commandPart = str_dup(cmd->valuestring);
                vector_push(settings.command, commandPart);
            } while ((cmd = cmd->next));
        }
    } while ((js = js->next));

    return settings;
}

static inline bool is_name_valid(const char *name)
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
