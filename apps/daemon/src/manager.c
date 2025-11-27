#include <stdio.h>
#include <pthread.h>
#include <stdbool.h>
#include <unistd.h>

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

static enum d_result project_services_start(struct project_settings project, const struct env_variable *env);
static enum d_result project_services_stop(struct project_settings project);
static enum d_result project_services_stop_and_remove(struct project_settings project);
static enum d_result project_services_clear_logs(struct project_settings project);

static bool try_find_project(const char *proj_name, struct project *project, int *pos);
static bool try_find_service(const char *serv_name, const struct project project, struct service_settings *service);

static struct project_info project_info_create(struct project project);
static struct service_info service_info_create(const char *proj_name, const char *serv_name);

static struct project project_create(const struct project_settings settings);
static void project_free(struct project project);

static struct project_store store;

enum m_result
manager_init(void)
{
    if (driver_mount() < D_OK)
    {
        log_critical("Unable to mount the driver.\n");
        return M_DRIVER_ERROR;
    }

    store.lock = malloc(sizeof(pthread_mutex_t));
    if (pthread_mutex_init(store.lock, NULL) != 0)
    {
        free(store.lock);
        store.lock = NULL;
        log_critical("Manager settings store mutex init has failed.\n");
        return M_ERROR;
    }

    pthread_mutex_lock(store.lock);

    vec_scoped char **stored_settings = d_get_all_stored_settings();
    size_t settings_count = vec_length(stored_settings);
    store.projects = vec_create_prealloc(struct project, settings_count);
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

        log_info("Loaded stored project '%s'\n", settings.name);
        struct project project = project_create(settings);

        pthread_mutex_lock(project.lock);
        // This is here in case of the application exited abruptly and cold not stop services when stopping
        project_services_stop(project.settings);
        pthread_mutex_unlock(project.lock);

        vec_push(store.projects, project);
    }

    vec_for_each(stored_settings, free);

    pthread_mutex_unlock(store.lock);

    log_info("Manager initialized\n");
    return M_OK;
}

enum m_result
manager_stop(void)
{
    pthread_mutex_lock(store.lock);

    for (size_t i = 0; i < vec_length(store.projects); i++)
    {
        struct project project = store.projects[i];

        pthread_mutex_lock(project.lock);
        project_services_stop(project.settings);
        pthread_mutex_unlock(project.lock);

        project_free(project);
    }

    driver_unmount();

    vec_free(store.projects);
    store.projects = NULL;
    pthread_mutex_unlock(store.lock);
    pthread_mutex_destroy(store.lock);

    free(store.lock);
    store.lock = NULL;

    log_info("Manager stopped\n");
    return M_OK;
}

struct project_settings *
projects_settings_get(void)
{
    pthread_mutex_lock(store.lock);

    size_t project_count = vec_length(store.projects);
    struct project_settings *copy = vec_create_prealloc(struct project_settings, project_count);
    for (size_t i = 0; i < project_count; i++)
    {
        struct project project = store.projects[i];
        pthread_mutex_lock(project.lock);
        vec_push(copy, project_settings_dup(project.settings));
        pthread_mutex_unlock(project.lock);
    }

    pthread_mutex_unlock(store.lock);

    return copy;
}

struct project_info *
projects_info_get(void)
{
    pthread_mutex_lock(store.lock);

    size_t project_count = vec_length(store.projects);
    struct project_info *infos = vec_create_prealloc(struct project_info, project_count);
    for (size_t i = 0; i < project_count; i++)
    {
        struct project project = store.projects[i];
        pthread_mutex_lock(project.lock);
        vec_push(infos, project_info_create(project));
        pthread_mutex_unlock(project.lock);
    }

    pthread_mutex_unlock(store.lock);

    return infos;
}

enum m_result
project_settings_get(const char *proj_name, struct project_settings *settings)
{
    pthread_mutex_lock(store.lock);

    struct project project;
    if (try_find_project(proj_name, &project, NULL) == false)
    {
        pthread_mutex_unlock(store.lock);
        return M_PROJECT_NOT_FOUND;
    }

    pthread_mutex_lock(project.lock);

    pthread_mutex_unlock(store.lock);

    (*settings) = project_settings_dup(project.settings);

    pthread_mutex_unlock(project.lock);

    return M_OK;
}

enum m_result
project_info_get(const char *proj_name, struct project_info *info)
{
    pthread_mutex_lock(store.lock);

    struct project project;
    if (try_find_project(proj_name, &project, NULL) == false)
    {
        pthread_mutex_unlock(store.lock);
        return M_PROJECT_NOT_FOUND;
    }

    pthread_mutex_lock(project.lock);

    pthread_mutex_unlock(store.lock);

    (*info) = project_info_create(project);

    pthread_mutex_unlock(project.lock);

    return M_OK;
}

enum m_result
project_upsert(const struct project_settings settings)
{
    pthread_mutex_lock(store.lock);

    for (size_t i = 0; i < vec_length(store.projects); i++)
    {
        struct project project = store.projects[i];
        if (strcmp(settings.name, project.settings.name))
            continue;

        pthread_mutex_lock(project.lock);

        enum d_result result = project_services_stop_and_remove(project.settings);
        if (result < D_OK)
        {
            pthread_mutex_unlock(project.lock);
            pthread_mutex_unlock(store.lock);
            return M_DRIVER_ERROR;
        }

        vec_remove(store.projects, i, NULL);

        pthread_mutex_unlock(project.lock);

        project_free(project);
    }

    struct project new_project = project_create(project_settings_dup(settings));
    if (d_project_init(new_project.settings) < D_OK)
    {
        project_free(new_project);
        pthread_mutex_unlock(store.lock);
        return M_DRIVER_ERROR;
    }
    vec_unshift(store.projects, new_project);

    pthread_mutex_unlock(store.lock);

    return M_OK;
}

enum m_result
project_start(const char *proj_name, const struct env_variable *env)
{
    pthread_mutex_lock(store.lock);

    struct project project;
    if (try_find_project(proj_name, &project, NULL) == false)
    {
        pthread_mutex_unlock(store.lock);
        return M_PROJECT_NOT_FOUND;
    }

    pthread_mutex_lock(project.lock);

    pthread_mutex_unlock(store.lock);
    enum d_result result = project_services_start(project.settings, env);
    pthread_mutex_unlock(project.lock);

    if (result < D_OK)
        return M_DRIVER_ERROR;
    if (result == D_NO_ACTION)
        return M_NO_ACTION;
    return M_OK;
}

enum m_result
project_restart(const char *proj_name, const struct env_variable *env)
{
    pthread_mutex_lock(store.lock);

    struct project project;
    if (try_find_project(proj_name, &project, NULL) == false)
    {
        pthread_mutex_unlock(store.lock);
        return M_PROJECT_NOT_FOUND;
    }

    pthread_mutex_lock(project.lock);

    pthread_mutex_unlock(store.lock);
    enum d_result result = project_services_stop(project.settings);

    if (result < D_OK)
    {
        pthread_mutex_unlock(project.lock);
        return M_DRIVER_ERROR;
    }

    result = project_services_start(project.settings, env);

    pthread_mutex_unlock(project.lock);

    if (result < D_OK)
        return M_DRIVER_ERROR;
    return M_OK;
}

enum m_result
project_stop(const char *proj_name)
{
    pthread_mutex_lock(store.lock);

    struct project project;
    if (try_find_project(proj_name, &project, NULL) == false)
    {
        pthread_mutex_unlock(store.lock);
        return M_PROJECT_NOT_FOUND;
    }

    pthread_mutex_lock(project.lock);

    pthread_mutex_unlock(store.lock);
    enum d_result result = project_services_stop(project.settings);
    pthread_mutex_unlock(project.lock);

    if (result < D_OK)
        return M_DRIVER_ERROR;
    if (result == D_NO_ACTION)
        return M_NO_ACTION;
    return M_OK;
}

enum m_result
project_clear_logs(const char *proj_name)
{
    pthread_mutex_lock(store.lock);

    struct project project;
    if (try_find_project(proj_name, &project, NULL) == false)
    {
        pthread_mutex_unlock(store.lock);
        return M_PROJECT_NOT_FOUND;
    }

    pthread_mutex_lock(project.lock);

    pthread_mutex_unlock(store.lock);
    enum d_result result = project_services_clear_logs(project.settings);
    pthread_mutex_unlock(project.lock);

    if (result < D_OK)
        return M_DRIVER_ERROR;
    if (result == D_NO_ACTION)
        return M_NO_ACTION;
    return M_OK;
}

enum m_result
project_remove(const char *proj_name)
{
    pthread_mutex_lock(store.lock);

    struct project project;
    int pos;
    if (try_find_project(proj_name, &project, &pos) == false)
    {
        pthread_mutex_unlock(store.lock);
        return M_PROJECT_NOT_FOUND;
    }

    pthread_mutex_lock(project.lock);

    enum d_result result = project_services_stop_and_remove(project.settings);
    if (result >= D_OK)
        vec_remove(store.projects, pos, NULL);

    pthread_mutex_unlock(project.lock);
    pthread_mutex_unlock(store.lock);

    if (result >= D_OK)
    {
        project_free(project);
        return M_OK;
    }

    return M_DRIVER_ERROR;
}

enum m_result
service_info_get(const char *proj_name, const char *serv_name, struct service_info *info)
{
    pthread_mutex_lock(store.lock);

    struct project project;
    if (try_find_project(proj_name, &project, NULL) == false)
    {
        pthread_mutex_unlock(store.lock);
        return M_PROJECT_NOT_FOUND;
    }

    pthread_mutex_lock(project.lock);

    pthread_mutex_unlock(store.lock);

    struct service_settings service;
    if (try_find_service(serv_name, project, &service) == false)
    {
        pthread_mutex_unlock(project.lock);
        return M_SERVICE_NOT_FOUND;
    }

    (*info) = service_info_create(project.settings.name, service.name);

    pthread_mutex_unlock(project.lock);

    return M_OK;
}

enum m_result
service_start(const char *proj_name, const char *serv_name, const struct env_variable *env)
{
    pthread_mutex_lock(store.lock);

    struct project project;
    if (try_find_project(proj_name, &project, NULL) == false)
    {
        pthread_mutex_unlock(store.lock);
        return M_PROJECT_NOT_FOUND;
    }

    pthread_mutex_lock(project.lock);

    pthread_mutex_unlock(store.lock);

    struct service_settings service;
    if (try_find_service(serv_name, project, &service) == false)
    {
        pthread_mutex_unlock(project.lock);
        return M_SERVICE_NOT_FOUND;
    }

    enum d_result start_result = d_service_start(project.settings, service, env);

    pthread_mutex_unlock(project.lock);

    if (start_result < D_OK)
        return M_DRIVER_ERROR;
    if (start_result == D_NO_ACTION)
        return M_NO_ACTION;
    return M_OK;
}

enum m_result
service_restart(const char *proj_name, const char *serv_name, const struct env_variable *env)
{
    pthread_mutex_lock(store.lock);

    struct project project;
    if (try_find_project(proj_name, &project, NULL) == false)
    {
        pthread_mutex_unlock(store.lock);
        return M_PROJECT_NOT_FOUND;
    }

    pthread_mutex_lock(project.lock);

    pthread_mutex_unlock(store.lock);

    struct service_settings service;
    if (try_find_service(serv_name, project, &service) == false)
    {
        pthread_mutex_unlock(project.lock);
        return M_SERVICE_NOT_FOUND;
    }

    enum d_result result = d_service_stop(project.settings.name, service);
    if (result < D_OK)
    {
        pthread_mutex_unlock(project.lock);
        return M_DRIVER_ERROR;
    }

    result = d_service_start(project.settings, service, env);

    pthread_mutex_unlock(project.lock);

    if (result < D_OK)
        return M_DRIVER_ERROR;
    return M_OK;
}

enum m_result
service_stop(const char *proj_name, const char *serv_name)
{
    pthread_mutex_lock(store.lock);

    struct project project;
    if (try_find_project(proj_name, &project, NULL) == false)
    {
        pthread_mutex_unlock(store.lock);
        return M_PROJECT_NOT_FOUND;
    }

    pthread_mutex_lock(project.lock);

    pthread_mutex_unlock(store.lock);

    struct service_settings service;
    if (try_find_service(serv_name, project, &service) == false)
    {
        pthread_mutex_unlock(project.lock);
        return M_SERVICE_NOT_FOUND;
    }

    enum d_result stop_result = d_service_stop(project.settings.name, service);

    pthread_mutex_unlock(project.lock);

    if (stop_result < D_OK)
        return M_DRIVER_ERROR;
    if (stop_result == D_NO_ACTION)
        return M_NO_ACTION;
    return M_OK;
}

enum m_result
service_clear_logs(const char *proj_name, const char *serv_name)
{
    pthread_mutex_lock(store.lock);

    struct project project;
    if (try_find_project(proj_name, &project, NULL) == false)
    {
        pthread_mutex_unlock(store.lock);
        return M_PROJECT_NOT_FOUND;
    }

    pthread_mutex_lock(project.lock);

    pthread_mutex_unlock(store.lock);

    struct service_settings service;
    if (try_find_service(serv_name, project, &service) == false)
    {
        pthread_mutex_unlock(project.lock);
        return M_SERVICE_NOT_FOUND;
    }

    enum d_result stop_result = d_service_clear_logs(project.settings.name, service);

    pthread_mutex_unlock(project.lock);

    if (stop_result < D_OK)
        return M_DRIVER_ERROR;
    return M_OK;
}

void
service_info_free(struct service_info info)
{
    free(info.name);
    free(info.logfile_path);

    info.name = NULL;
    info.logfile_path = NULL;
}

void
project_info_free(struct project_info info)
{
    free(info.name);
    if (info.services != NULL)
    {
        vec_for_each(info.services, service_info_free);
        vec_free(info.services);
    }

    info.name = NULL;
    info.services = NULL;
}

static enum d_result
project_services_start(struct project_settings project, const struct env_variable *env)
{
    enum d_result final_result = D_NO_ACTION;
    for (size_t i = 0; i < vec_length(project.services); i++)
    {
        enum d_result result = d_service_start(project, project.services[i], env);
        if (result <= D_OK && final_result >= D_OK)
            final_result = result;
    }

    return final_result;
}

static enum d_result
project_services_stop(struct project_settings project)
{
    enum d_result final_result = D_NO_ACTION;
    for (size_t i = 0; i < vec_length(project.services); i++)
    {
        enum d_result result = d_service_stop(project.name, project.services[i]);
        if (result <= D_OK && final_result >= D_OK)
            final_result = result;
    }

    return final_result;
}

static enum d_result
project_services_clear_logs(struct project_settings project)
{
    enum d_result final_result = D_NO_ACTION;
    for (size_t i = 0; i < vec_length(project.services); i++)
    {
        enum d_result result = d_service_clear_logs(project.name, project.services[i]);
        if (result <= D_OK && final_result >= D_OK)
            final_result = result;
    }

    return final_result;
}

static enum d_result
project_services_stop_and_remove(struct project_settings project)
{
    enum d_result stop_result = project_services_stop(project);
    if (stop_result < D_OK)
        return stop_result;

    return d_project_remove(project);
}

static bool
try_find_project(const char *proj_name, struct project *project, int *pos)
{
    for (size_t i = 0; i < vec_length(store.projects); i++)
    {
        if (pos != NULL)
            (*pos) = i;
        (*project) = store.projects[i];

        if (strcmp(proj_name, project->settings.name) == 0)
            return true;
    }

    return false;
}

static bool
try_find_service(const char *serv_name, const struct project project, struct service_settings *service)
{
    for (size_t i = 0; i < vec_length(project.settings.services); i++)
    {
        (*service) = project.settings.services[i];
        if (strcmp(serv_name, service->name) == 0)
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
    size_t service_count = vec_length(project.settings.services);

    info.name = str_dup(project.settings.name);
    info.services = vec_create_prealloc(struct service_info, service_count);
    for (size_t i = 0; i < service_count; i++)
    {
        struct service_settings service_settings = project.settings.services[i];
        struct service_info service_info = service_info_create(project.settings.name, service_settings.name);
        vec_push(info.services, service_info);
    }

    return info;
}

static struct service_info
service_info_create(const char *proj_name, const char *serv_name)
{
    struct d_service_info d_info = { 0 };
    d_service_info_get(proj_name, serv_name, &d_info);
    enum service_status status;

    char *logfile_path = d_info.logfile_path ? str_dup(d_info.logfile_path) : NULL;

    switch (d_info.status)
    {
    case D_RUNNING:
        status = RUNNING;
        break;
    case D_STOPPED:
        status = STOPPED;
        break;
    case D_EXITED:
        status = EXITED;
        break;
    default:
        status = IDLE;
        break;
    }

    struct service_info info = { .name = str_dup(serv_name),
                                 .status = status,
                                 .logfile_path = logfile_path,
                                 .pid = d_info.pid,
                                 .start_time = d_info.start_time,
                                 .stop_time = d_info.stop_time };

    d_service_info_free(d_info);

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
