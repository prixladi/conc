#include <string.h>
#include <assert.h>
#include <stdbool.h>

#include "utils/log.h"
#include "utils/vector.h"
#include "utils/string.h"

#include "settings.h"
#include "manager.h"

#define resp_error(msg) str_concat("ERROR\n", msg, NULL)
#define resp_ok(msg) str_concat("OK\n", msg, NULL)
#define resp_ok_no_content() str_dup("OK")

static char **tokenize(const char *input);
static void tokenize_free(char **tokens);

typedef char *(*Handler)(char **command);
static inline char *match_and_handle(const char *name, char **tokens, size_t argc, Handler handler);

static char *handle_project_upsert(char **command);
static char *handle_projects_names();
static char *handle_projects_settings();
static char *handle_projects_info();
static char *handle_project_settings(char **command);
static char *handle_project_info(char **command);
static char *handle_project_start(char **command);
static char *handle_project_stop(char **command);
static char *handle_project_remove(char **command);
static char *handle_services_names(char **command);
static char *handle_service_info(char **command);
static char *handle_service_start(char **command);
static char *handle_service_stop(char **command);

static char *format_list(char **items);
static char *format_service_info(ServiceInfo info);

char *dispatch_command(const char *input)
{
    char **command = tokenize(input);

    char *response = NULL;
    if (response == NULL)
        response = match_and_handle("PROJECTS-NAMES", command, 0, handle_projects_names);
    if (response == NULL)
        response = match_and_handle("PROJECTS-SETTINGS", command, 0, handle_projects_settings);
    if (response == NULL)
        response = match_and_handle("PROJECTS-INFO", command, 0, handle_projects_info);
    if (response == NULL)
        response = match_and_handle("PROJECT-SETTINGS", command, 1, handle_project_settings);
    if (response == NULL)
        response = match_and_handle("PROJECT-INFO", command, 1, handle_project_info);
    if (response == NULL)
        response = match_and_handle("PROJECT-UPSERT", command, 1, handle_project_upsert);
    if (response == NULL)
        response = match_and_handle("PROJECT-START", command, 1, handle_project_start);
    if (response == NULL)
        response = match_and_handle("PROJECT-STOP", command, 1, handle_project_stop);
    if (response == NULL)
        response = match_and_handle("PROJECT-REMOVE", command, 1, handle_project_remove);
    if (response == NULL)
        response = match_and_handle("SERVICES-NAMES", command, 1, handle_services_names);
    if (response == NULL)
        response = match_and_handle("SERVICE-INFO", command, 2, handle_service_info);
    if (response == NULL)
        response = match_and_handle("SERVICE-START", command, 2, handle_service_start);
    if (response == NULL)
        response = match_and_handle("SERVICE-STOP", command, 2, handle_service_stop);

    tokenize_free(command);

    if (response)
        return response;
    return resp_error("unknown_command");
}

// Probably don't want to use strtok_r because it requires gnu extension to c99 standard and it would reduce portability
static char **tokenize(const char *input)
{
    char **result = vector_create_prealloc(char *, 8);

    size_t len = strlen(input);
    size_t last_pos = 0;
    for (size_t i = 0; i < len; i++)
    {
        bool is_newline = input[i] == '\n';
        bool is_last = i + 1 == len;
        if (!is_newline && !is_last)
            continue;

        int ind = is_newline ? 0 : 1;

        size_t part_len = i - last_pos + ind;
        char *part = malloc(sizeof(char) * (part_len + 1));
        part[part_len] = '\0';

        for (size_t j = 0; j <= part_len - 1; j++)
            part[j] = input[last_pos + j];

        vector_push(result, part);
        last_pos = i + 1 + ind;
    }

    return result;
}

static void tokenize_free(char **tokens)
{
    for (size_t i = 0; i < vector_length(tokens); i++)
    {
        free(tokens[i]);
        tokens[i] = NULL;
    }

    vector_free(tokens);
}

static inline char *match_and_handle(const char *name, char **command, size_t argc, Handler handler)
{
    if (strcmp(name, command[0]))
        return NULL;

    size_t command_len = vector_length(command);

    if (command_len - 1 != argc)
        return resp_error("invalid_argument_count");

    char **argv = command + 1;
    return handler(argv);
}

static char *handle_projects_names()
{
    ProjectSettings *projects = projects_settings_get();
    size_t projects_count = vector_length(projects);
    if (projects_count == 0)
    {
        vector_free(projects);
        return resp_ok_no_content();
    }

    char **items = vector_create_prealloc(char *, projects_count);

    for (size_t i = 0; i < projects_count; i++)
        vector_push(items, projects[i].name);

    char *response = format_list(items);

    vector_free(items);
    vector_for_each(projects, project_settings_free);
    vector_free(projects);

    char *ok_response = resp_ok(response);
    free(response);

    return ok_response;
}

static char *handle_projects_settings()
{
    ProjectSettings *projects = projects_settings_get();
    size_t projects_count = vector_length(projects);
    if (projects_count == 0)
    {
        vector_free(projects);
        return resp_ok_no_content();
    }

    char **items = vector_create_prealloc(char *, projects_count);

    for (size_t i = 0; i < projects_count; i++)
    {
        char *json = project_settings_stringify(projects[i]);
        vector_push(items, json);
    }

    char *response = format_list(items);

    vector_for_each(items, free);
    vector_free(items);

    vector_for_each(projects, project_settings_free);
    vector_free(projects);

    char *ok_response = resp_ok(response);
    free(response);

    return ok_response;
}

static char *handle_projects_info()
{
    ProjectInfo *infos = projects_info_get();

    size_t project_count = vector_length(infos);
    if (project_count == 0)
    {
        vector_free(infos);
        return resp_ok_no_content();
    }

    char **parts = vector_create(char *);
    for (size_t i = 0; i < project_count; i++)
    {
        ProjectInfo info = infos[i];
        size_t service_count = vector_length(info.services);

        vector_push_rval(parts, str_dup(info.name));
        for (size_t i = 0; i < service_count; i++)
            vector_push_rval(parts, format_service_info(info.services[i]));
    }

    char *response = format_list(parts);

    vector_for_each(infos, project_info_free);
    vector_free(infos);

    vector_for_each(parts, free);
    vector_free(parts);

    char *ok_response = resp_ok(response);
    free(response);
    return ok_response;
}

static char *handle_project_settings(char **command)
{
    ProjectSettings settings = {0};
    int result = project_settings_get(command[0], &settings);
    if (result > 0)
    {
        switch (result)
        {
        case 127:
            return resp_error("project_not_found");
        default:
            return resp_error("unknown");
        }
    }

    char *json = project_settings_stringify(settings);

    char *ok_response = resp_ok(json);
    free(json);
    project_settings_free(settings);

    return ok_response;
}

static char *handle_project_info(char **command)
{
    ProjectInfo info = {0};
    int result = project_info_get(command[0], &info);
    if (result > 0)
    {
        switch (result)
        {
        case 127:
            return resp_error("project_not_found");
        default:
            return resp_error("unknown");
        }
    }

    size_t service_count = vector_length(info.services);
    char **parts = vector_create_prealloc(char *, service_count + 1);

    vector_push_rval(parts, str_dup(info.name));
    for (size_t i = 0; i < service_count; i++)
        vector_push_rval(parts, format_service_info(info.services[i]));
    project_info_free(info);

    char *response = format_list(parts);
    vector_for_each(parts, free);
    vector_free(parts);

    char *ok_response = resp_ok(response);
    free(response);
    return ok_response;
}

static char *handle_project_upsert(char **command)
{
    ProjectSettings settings = {0};
    char *parse_error = project_settings_parse(command[0], &settings);
    if (parse_error != NULL)
    {
        char *error = resp_error(parse_error);
        free(parse_error);
        project_settings_free(settings);
        return error;
    }

    project_upsert(settings);

    char *info_command[1];
    info_command[0] = settings.name;
    char *info_response = handle_project_info(info_command);

    project_settings_free(settings);
    return info_response;
}

static char *handle_project_start(char **command)
{
    int result = project_start(command[0]);
    if (result > 0)
    {
        switch (result)
        {
        case 127:
            return resp_error("project_not_found");
        default:
            return resp_error("unknown");
        }
    }

    return handle_project_info(command);
}

static char *handle_project_stop(char **command)
{
    int result = project_stop(command[0]);
    if (result > 0)
    {
        switch (result)
        {
        case 127:
            return resp_error("project_not_found");
        default:
            return resp_error("unknown");
        }
    }

    return handle_project_info(command);
}

static char *handle_project_remove(char **command)
{
    int result = project_remove(command[0]);
    switch (result)
    {
    case 0:
        return resp_ok_no_content();
    case 127:
        return resp_error("project_not_found");
    default:
        return resp_error("unknown");
    }
}

static char *handle_services_names(char **command)
{
    ProjectSettings project = {0};
    int result = project_settings_get(command[0], &project);
    if (result > 0)
    {
        switch (result)
        {
        case 127:
            return resp_error("project_not_found");
        default:
            return resp_error("unknown");
        }
    }

    size_t services_count = vector_length(project.services);
    if (services_count == 0)
    {
        project_settings_free(project);
        return resp_ok_no_content();
    }

    char **items = vector_create_prealloc(char *, services_count);

    for (size_t i = 0; i < services_count; i++)
        vector_push(items, project.services[i].name);

    char *response = format_list(items);

    vector_free(items);
    project_settings_free(project);

    char *ok_response = resp_ok(response);
    free(response);

    return ok_response;
}

static char *handle_service_info(char **command)
{
    ServiceInfo info;
    int result = service_info_get(command[0], command[1], &info);

    if (result > 0)
    {
        switch (result)
        {
        case 127:
            return resp_error("project_not_found");
        case 128:
            return resp_error("service_not_found");
        default:
            return resp_error("unknown");
        }
    }

    char *formatted = format_service_info(info);
    service_info_free(info);

    char *response = resp_ok(formatted);
    free(formatted);

    return response;
}

static char *handle_service_start(char **command)
{
    int result = service_start(command[0], command[1]);
    if (result > 0)
    {
        switch (result)
        {
        case 1:
            return resp_error("exited");
        case 127:
            return resp_error("project_not_found");
        case 128:
            return resp_error("service_not_found");
        case 409:
            return resp_ok("already_running");
        default:
            return resp_error("unknown");
        }
    }

    return handle_service_info(command);
}

static char *handle_service_stop(char **command)
{
    int result = service_stop(command[0], command[1]);
    if (result > 0)
    {
        switch (result)
        {
        case 127:
            return resp_error("project_not_found");
        case 128:
            return resp_error("service_not_found");
        case 409:
            return resp_ok("already_stoped");
        default:
            return resp_error("unknown");
        }
    }

    return handle_service_info(command);
}

static char *format_list(char **lines)
{
    size_t item_count = vector_length(lines);
    size_t response_length = item_count; // newline after each line and the '\0' at the end
    for (size_t i = 0; i < item_count; i++)
        response_length += strlen(lines[i]);

    char *response = calloc(response_length, sizeof(char));
    for (size_t i = 0; i < item_count; i++)
    {
        strcat(response, lines[i]);
        if (i + 1 < item_count) // last line should not be suffixed with the '\n'
            strcat(response, "\n");
    }

    return response;
}

static char *format_service_info(const ServiceInfo info)
{
    char *status;
    switch (info.status)
    {
    case RUNNING:
        status = "RUNNING";
        break;
    case STOPPED:
        status = "STOPPED";
        break;
    default:
        status = "IDLE";
        break;
    }

    return str_concat(info.name, " ", status, NULL);
}