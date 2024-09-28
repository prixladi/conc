#include <stdio.h>
#include <pthread.h>
#include <stdbool.h>

#include "settings.h"
#include "driver.h"
#include "manager.h"

#include "utils/vector.h"
#include "utils/log.h"
#include "utils/string.h"

struct project
{
    struct project_settings settings;
    pthread_mutex_t *lock;
};

struct project_store
{
    struct project *projects;
    pthread_mutex_t *lock;
};

static bool try_find_project(const char *project_name, struct project *project, int *pos);
static bool try_find_service(const char *service_name, const struct project project, struct service_settings *service);

static struct project_info project_info_create(struct project project);
static struct service_info service_info_create(const char *project_name, const char *service_name);

static struct project project_create(const struct project_settings settings);
static void project_free(struct project project);

static struct project_store store;

int
manager_init(void)
{
    if (driver_mount() != 0)
    {
        log_critical("Unable to mount the driver.\n");
        return 2;
    }

    store.lock = malloc(sizeof(pthread_mutex_t));
    if (pthread_mutex_init(store.lock, NULL) != 0)
    {
        free(store.lock);
        store.lock = NULL;
        log_critical("Manager settings store mutex init has failed.\n");
        return 1;
    }

    pthread_mutex_lock(store.lock);

    char **stored_settings = d_get_all_stored_settings();
    size_t settings_count = vector_length(stored_settings);
    store.projects = vector_create_prealloc(struct project, settings_count);
    for (size_t i = 0; i < settings_count; i++)
    {
        struct project_settings settings = { 0 };

        char *parse_error = project_settings_parse(stored_settings[i], &settings);
        if (parse_error)
        {
            log_error("Unable to parse settings '%s' because of error '%s'.\n", stored_settings[i], parse_error);
            free(parse_error);
            project_settings_free(settings);
            continue;
        }

        log_debug("Loaded stored project '%s'\n", settings.name);
        struct project project = project_create(settings);

        pthread_mutex_lock(project.lock);
        d_project_stop(project.settings);
        pthread_mutex_unlock(project.lock);

        vector_push(store.projects, project);
    }

    vector_for_each(stored_settings, free);
    vector_free(stored_settings);

    pthread_mutex_unlock(store.lock);

    log_info("Manager initialized\n");

    return 0;
}

void
manager_stop(void)
{
    pthread_mutex_lock(store.lock);

    for (size_t i = 0; i < vector_length(store.projects); i++)
    {
        struct project project = store.projects[i];

        pthread_mutex_lock(project.lock);
        d_project_stop(project.settings);
        pthread_mutex_unlock(project.lock);

        project_free(project);
    }

    driver_unmount();

    vector_free(store.projects);
    store.projects = NULL;
    pthread_mutex_unlock(store.lock);
    pthread_mutex_destroy(store.lock);

    free(store.lock);
    store.lock = NULL;

    log_info("Manager stopped\n");
}

struct project_settings *
projects_settings_get(void)
{
    pthread_mutex_lock(store.lock);

    size_t project_count = vector_length(store.projects);
    struct project_settings *copy = vector_create_prealloc(struct project_settings, project_count);
    for (size_t i = 0; i < project_count; i++)
    {
        struct project project = store.projects[i];
        pthread_mutex_lock(project.lock);
        vector_push_rval(copy, project_settings_dup(project.settings));
        pthread_mutex_unlock(project.lock);
    }

    pthread_mutex_unlock(store.lock);

    return copy;
}

struct project_info *
projects_info_get(void)
{
    pthread_mutex_lock(store.lock);

    size_t project_count = vector_length(store.projects);
    struct project_info *infos = vector_create_prealloc(struct project_info, project_count);
    for (size_t i = 0; i < project_count; i++)
    {
        struct project project = store.projects[i];
        pthread_mutex_lock(project.lock);
        vector_push_rval(infos, project_info_create(project));
        pthread_mutex_unlock(project.lock);
    }

    pthread_mutex_unlock(store.lock);

    return infos;
}

int
project_settings_get(const char *project_name, struct project_settings *settings)
{
    pthread_mutex_lock(store.lock);

    struct project project;
    if (try_find_project(project_name, &project, NULL) == false)
    {
        pthread_mutex_unlock(store.lock);
        return 127;
    }

    pthread_mutex_lock(project.lock);
    pthread_mutex_unlock(store.lock);

    (*settings) = project_settings_dup(project.settings);

    pthread_mutex_unlock(project.lock);

    return 0;
}

int
project_info_get(const char *project_name, struct project_info *info)
{
    pthread_mutex_lock(store.lock);

    struct project project;
    if (try_find_project(project_name, &project, NULL) == false)
    {
        pthread_mutex_unlock(store.lock);
        return 127;
    }

    pthread_mutex_lock(project.lock);
    pthread_mutex_unlock(store.lock);

    (*info) = project_info_create(project);

    pthread_mutex_unlock(project.lock);

    return 0;
}

int
project_upsert(const struct project_settings settings)
{
    struct project new_project = project_create(project_settings_dup(settings));

    pthread_mutex_lock(store.lock);

    for (size_t i = 0; i < vector_length(store.projects); i++)
    {
        struct project project = store.projects[i];
        if (strcmp(new_project.settings.name, project.settings.name))
            continue;

        pthread_mutex_lock(project.lock);

        d_project_stop(project.settings);
        d_project_remove(project.settings);
        vector_remove(store.projects, i, NULL);

        pthread_mutex_unlock(project.lock);

        project_free(project);
    }

    int result = d_project_init(new_project.settings);
    if (result == 0)
        vector_push(store.projects, new_project);

    pthread_mutex_unlock(store.lock);

    return result;
}

int
project_start(const char *project_name)
{
    pthread_mutex_lock(store.lock);

    struct project project;
    if (try_find_project(project_name, &project, NULL) == false)
    {
        pthread_mutex_unlock(store.lock);
        return 127;
    }

    pthread_mutex_lock(project.lock);
    pthread_mutex_unlock(store.lock);

    d_project_start(project.settings);

    pthread_mutex_unlock(project.lock);

    return 0;
}

int
project_stop(const char *project_name)
{
    pthread_mutex_lock(store.lock);

    struct project project;
    if (try_find_project(project_name, &project, NULL) == false)
    {
        pthread_mutex_unlock(store.lock);
        return 127;
    }

    pthread_mutex_lock(project.lock);
    pthread_mutex_unlock(store.lock);

    d_project_stop(project.settings);

    pthread_mutex_unlock(project.lock);

    return 0;
}

int
project_remove(const char *project_name)
{
    pthread_mutex_lock(store.lock);

    struct project project;
    int pos;
    if (try_find_project(project_name, &project, &pos) == false)
    {
        pthread_mutex_unlock(store.lock);
        return 127;
    }

    pthread_mutex_lock(project.lock);

    vector_remove(store.projects, pos, NULL);
    d_project_stop(project.settings);
    int remove_res = d_project_remove(project.settings);

    pthread_mutex_unlock(project.lock);

    project_free(project);

    pthread_mutex_unlock(store.lock);

    if (remove_res > 0)
        return 2;
    return 0;
}

int
service_info_get(const char *project_name, const char *service_name, struct service_info *info)
{
    pthread_mutex_lock(store.lock);

    struct project project;
    if (try_find_project(project_name, &project, NULL) == false)
    {
        pthread_mutex_unlock(store.lock);
        return 127;
    }

    pthread_mutex_lock(project.lock);
    pthread_mutex_unlock(store.lock);

    struct service_settings service;
    if (try_find_service(service_name, project, &service) == false)
    {
        pthread_mutex_unlock(project.lock);
        return 128;
    }

    (*info) = service_info_create(project.settings.name, service.name);

    pthread_mutex_unlock(project.lock);

    return 0;
}

int
service_start(const char *project_name, const char *service_name)
{
    pthread_mutex_lock(store.lock);

    struct project project;
    if (try_find_project(project_name, &project, NULL) == false)
    {
        pthread_mutex_unlock(store.lock);
        return 127;
    }

    pthread_mutex_lock(project.lock);
    pthread_mutex_unlock(store.lock);

    struct service_settings service;
    if (try_find_service(service_name, project, &service) == false)
    {
        pthread_mutex_unlock(project.lock);
        return 128;
    }

    int start_result = d_service_start(project.settings.name, service);

    pthread_mutex_unlock(project.lock);

    return start_result;
}

int
service_stop(const char *project_name, const char *service_name)
{
    pthread_mutex_lock(store.lock);

    struct project project;
    if (try_find_project(project_name, &project, NULL) == false)
    {
        pthread_mutex_unlock(store.lock);
        return 127;
    }

    pthread_mutex_lock(project.lock);
    pthread_mutex_unlock(store.lock);

    struct service_settings service;
    if (try_find_service(service_name, project, &service) == false)
    {
        pthread_mutex_unlock(project.lock);
        return 128;
    }

    int stop_result = d_service_stop(project.settings.name, service);

    pthread_mutex_unlock(project.lock);

    return stop_result;
}

void
service_info_free(struct service_info info)
{
    free(info.name);
    info.name = NULL;
}

void
project_info_free(struct project_info info)
{
    free(info.name);
    if (info.services != NULL)
    {
        vector_for_each(info.services, service_info_free);
        vector_free(info.services);
    }

    info.name = NULL;
    info.services = NULL;
}

static bool
try_find_project(const char *project_name, struct project *project, int *pos)
{
    for (size_t i = 0; i < vector_length(store.projects); i++)
    {
        if (pos != NULL)
            (*pos) = i;
        (*project) = store.projects[i];

        if (strcmp(project_name, project->settings.name) == 0)
            return true;
    }

    return false;
}

static bool
try_find_service(const char *service_name, const struct project project, struct service_settings *service)
{
    for (size_t i = 0; i < vector_length(project.settings.services); i++)
    {
        (*service) = project.settings.services[i];
        if (strcmp(service_name, service->name) == 0)
            return true;
    }

    return false;
}

static struct project
project_create(const struct project_settings settings)
{
    struct project project = {
        .settings = settings,
        .lock = malloc(sizeof(pthread_mutex_t)),
    };

    pthread_mutex_init(project.lock, NULL);

    return project;
}

static struct project_info
project_info_create(struct project project)
{
    struct project_info info;
    size_t service_count = vector_length(project.settings.services);

    info.name = str_dup(project.settings.name);
    info.services = vector_create_prealloc(struct service_info, service_count);
    for (size_t i = 0; i < service_count; i++)
    {
        struct service_settings service_settings = project.settings.services[i];
        struct service_info service_info = service_info_create(project.settings.name, service_settings.name);
        vector_push(info.services, service_info);
    }

    return info;
}

static struct service_info
service_info_create(const char *project_name, const char *service_name)
{
    struct d_service_info d_info = d_service_info_get(project_name, service_name);
    enum service_status status;

    switch (d_info.status)
    {
    case D_RUNNING:
        status = RUNNING;
        break;
    case D_STOPPED:
        status = STOPPED;
        break;
    default:
        status = IDLE;
        break;
    }

    struct service_info info = {
        .name = str_dup(service_name),
        .status = status,
    };

    return info;
}

static void
project_free(struct project project)
{
    project_settings_free(project.settings);
    pthread_mutex_destroy(project.lock);
    free(project.lock);
    project.lock = NULL;
}
