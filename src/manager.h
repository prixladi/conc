#ifndef MANAGER__H
#define MANAGER__H

#include <stdio.h>
#include <pthread.h>
#include <stdbool.h>

#include "utils/vector.h"
#include "utils/log.h"

#include "settings.h"

typedef enum ServiceStatus
{
    IDLE,
    RUNNING,
    STOPPED,
} ServiceStatus;

typedef struct ServiceInfo
{
    char *name;
    ServiceStatus status;
} ServiceInfo;

typedef struct ProjectInfo
{
    char *name;
    ServiceInfo *services;
} ProjectInfo;

int manager_init();
void manager_free();

ProjectSettings *projects_settings_get();
ProjectInfo *projects_info_get();
int project_settings_get(const char *project_name, ProjectSettings *settings);
int project_info_get(const char *project_name, ProjectInfo *info);
int project_upsert(const ProjectSettings settings);
int project_remove(const char *project_name);
int project_start(const char *project_name);
int project_stop(const char *project_name);

int service_info_get(const char *project_name, const char *service_name, ServiceInfo *info);
int service_start(const char *project_name, const char *service_name);
int service_stop(const char *project_name, const char *service_name);

void service_info_free(ServiceInfo info);
void project_info_free(ProjectInfo info);

#endif