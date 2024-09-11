#include <stdio.h>
#include <pthread.h>
#include <stdbool.h>

#include "settings.h"
#include "driver.h"
#include "manager.h"

#include "utils/vector.h"
#include "utils/log.h"
#include "utils/string.h"

typedef struct Project
{
    ProjectSettings settings;
    pthread_mutex_t lock;
} Project;

typedef struct ProjectStore
{
    Project *projects;
    pthread_mutex_t lock;
} ProjectStore;

static bool try_find_project(const char *project_name, Project *project, int *pos);
static bool try_find_service(const char *service_name, const Project project, ServiceSettings *service);

static ProjectInfo project_info_create(Project project);
static ServiceInfo service_info_create(const char *project_name, const char *service_name);

static Project project_create(const ProjectSettings settings);
static void project_free(Project project);

static ProjectStore store;

int manager_init()
{
    if (pthread_mutex_init(&store.lock, NULL) != 0)
    {
        LOG_ERROR("Settings store mutex init has failed\n");
        return 1;
    }

    store.projects = vector_create(Project);

    LOG_INFO("GTD init done\n");

    return 0;
}

void manager_free()
{
    pthread_mutex_destroy(&store.lock);

    vector_for_each(store.projects, project_free);
    vector_free(store.projects);

    store.projects = NULL;
}

ProjectSettings *projects_settings_get()
{
    pthread_mutex_lock(&store.lock);

    size_t project_count = vector_length(store.projects);
    ProjectSettings *copy = vector_create_prealloc(ProjectSettings, project_count);
    for (size_t i = 0; i < project_count; i++)
    {
        Project project = store.projects[i];
        pthread_mutex_lock(&project.lock);
        vector_push_rval(copy, project_settings_dup(project.settings));
        pthread_mutex_unlock(&project.lock);
    }

    pthread_mutex_unlock(&store.lock);

    return copy;
}

ProjectInfo *projects_info_get()
{
    pthread_mutex_lock(&store.lock);

    size_t project_count = vector_length(store.projects);
    ProjectInfo *infos = vector_create_prealloc(ProjectInfo, project_count);
    for (size_t i = 0; i < project_count; i++)
    {
        Project project = store.projects[i];
        pthread_mutex_lock(&project.lock);
        vector_push_rval(infos, project_info_create(project));
        pthread_mutex_unlock(&project.lock);
    }

    pthread_mutex_unlock(&store.lock);

    return infos;
}

int project_settings_get(const char *project_name, ProjectSettings *settings)
{
    pthread_mutex_lock(&store.lock);

    Project project;
    if (try_find_project(project_name, &project, NULL) == false)
    {
        pthread_mutex_unlock(&store.lock);
        return 127;
    }

    pthread_mutex_lock(&project.lock);
    pthread_mutex_unlock(&store.lock);

    (*settings) = project_settings_dup(project.settings);

    pthread_mutex_unlock(&project.lock);

    return 0;
}

int project_info_get(const char *project_name, ProjectInfo *info)
{
    pthread_mutex_lock(&store.lock);

    Project project;
    if (try_find_project(project_name, &project, NULL) == false)
    {
        pthread_mutex_unlock(&store.lock);
        return 127;
    }

    pthread_mutex_lock(&project.lock);
    pthread_mutex_unlock(&store.lock);

    (*info) = project_info_create(project);

    pthread_mutex_unlock(&project.lock);

    return 0;
}

int project_upsert(const ProjectSettings settings)
{
    Project new_project = project_create(project_settings_dup(settings));

    pthread_mutex_lock(&store.lock);

    bool replaced = false;
    for (size_t i = 0; i < vector_length(store.projects); i++)
    {
        Project project = store.projects[i];
        if (strcmp(new_project.settings.name, project.settings.name))
            continue;

        pthread_mutex_lock(&project.lock);

        d_project_stop(project.settings);
        store.projects[i] = new_project;

        pthread_mutex_unlock(&project.lock);

        project_free(project);
        replaced = true;
    }

    if (!replaced)
        vector_push(store.projects, new_project);

    pthread_mutex_lock(&new_project.lock);
    pthread_mutex_unlock(&store.lock);

    d_project_init(new_project.settings);

    pthread_mutex_unlock(&new_project.lock);

    return 0;
}

int project_start(const char *project_name)
{
    pthread_mutex_lock(&store.lock);

    Project project;
    if (try_find_project(project_name, &project, NULL) == false)
    {
        pthread_mutex_unlock(&store.lock);
        return 127;
    }

    pthread_mutex_lock(&project.lock);
    pthread_mutex_unlock(&store.lock);

    d_project_start(project.settings);

    pthread_mutex_unlock(&project.lock);

    return 0;
}

int project_stop(const char *project_name)
{
    pthread_mutex_lock(&store.lock);

    Project project;
    if (try_find_project(project_name, &project, NULL) == false)
    {
        pthread_mutex_unlock(&store.lock);
        return 127;
    }

    pthread_mutex_lock(&project.lock);

    d_project_stop(project.settings);

    pthread_mutex_unlock(&project.lock);
    pthread_mutex_unlock(&store.lock);

    return 0;
}

int project_remove(const char *project_name)
{
    pthread_mutex_lock(&store.lock);

    Project project;
    int pos;
    if (try_find_project(project_name, &project, &pos) == false)
    {
        pthread_mutex_unlock(&store.lock);
        return 127;
    }

    pthread_mutex_lock(&project.lock);

    vector_remove(store.projects, pos, NULL);
    d_project_stop(project.settings);

    pthread_mutex_unlock(&project.lock);

    project_free(project);

    pthread_mutex_unlock(&store.lock);
    return 0;
}

int service_info_get(const char *project_name, const char *service_name, ServiceInfo *info)
{
    pthread_mutex_lock(&store.lock);

    Project project;
    if (try_find_project(project_name, &project, NULL) == false)
    {
        pthread_mutex_unlock(&store.lock);
        return 127;
    }

    pthread_mutex_lock(&project.lock);
    pthread_mutex_unlock(&store.lock);

    ServiceSettings service;
    if (try_find_service(service_name, project, &service) == false)
    {
        pthread_mutex_unlock(&project.lock);
        return 128;
    }

    (*info) = service_info_create(project.settings.name, service.name);

    pthread_mutex_unlock(&project.lock);

    return 0;
}

int service_start(const char *project_name, const char *service_name)
{
    pthread_mutex_lock(&store.lock);

    Project project;
    if (try_find_project(project_name, &project, NULL) == false)
    {
        pthread_mutex_unlock(&store.lock);
        return 127;
    }

    pthread_mutex_lock(&project.lock);
    pthread_mutex_unlock(&store.lock);

    ServiceSettings service;
    if (try_find_service(service_name, project, &service) == false)
    {
        pthread_mutex_unlock(&project.lock);
        return 128;
    }

    int start_result = d_service_start(project.settings.name, service);

    pthread_mutex_unlock(&project.lock);

    return start_result;
}

int service_stop(const char *project_name, const char *service_name)
{
    pthread_mutex_lock(&store.lock);

    Project project;
    if (try_find_project(project_name, &project, NULL) == false)
    {
        pthread_mutex_unlock(&store.lock);
        return 127;
    }

    pthread_mutex_lock(&project.lock);
    pthread_mutex_unlock(&store.lock);

    ServiceSettings service;
    if (try_find_service(service_name, project, &service) == false)
    {
        pthread_mutex_unlock(&project.lock);
        return 128;
    }

    int stop_result = d_service_stop(project.settings.name, service);

    pthread_mutex_unlock(&project.lock);

    return stop_result;
}

void service_info_free(ServiceInfo info)
{
    free(info.name);
    info.name = NULL;
}

void project_info_free(ProjectInfo info)
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

static bool try_find_project(const char *project_name, Project *project, int *pos)
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

static bool try_find_service(const char *service_name, const Project project, ServiceSettings *service)
{
    for (size_t i = 0; i < vector_length(project.settings.services); i++)
    {
        (*service) = project.settings.services[i];
        if (strcmp(service_name, service->name) == 0)
            return true;
    }

    return false;
}

static Project project_create(const ProjectSettings settings)
{
    Project project = {
        .settings = settings};

    pthread_mutex_init(&project.lock, NULL);

    return project;
}

static ProjectInfo project_info_create(Project project)
{
    ProjectInfo info;
    size_t service_count = vector_length(project.settings.services);

    info.name = str_dup(project.settings.name);
    info.services = vector_create_prealloc(ServiceInfo, service_count);
    for (size_t i = 0; i < service_count; i++)
    {
        ServiceInfo service_info = service_info_create(project.settings.name, project.settings.services[i].name);
        vector_push(info.services, service_info);
    }

    return info;
}

static ServiceInfo service_info_create(const char *project_name, const char *service_name)
{
    D_ServiceInfo d_info = d_service_info_get(project_name, service_name);
    ServiceStatus status;

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

    ServiceInfo info = {
        .name = str_dup(service_name),
        .status = status};

    return info;
}

static void project_free(Project project)
{
    pthread_mutex_destroy(&project.lock);
    project_settings_free(project.settings);
}
