#ifndef DRIVER__H
#define DRIVER__H

#include "settings.h"

typedef enum D_ServiceStatus
{
    D_NONE,
    D_RUNNING,
    D_STOPPED,
} D_ServiceStatus;

typedef struct D_ServiceInfo
{
    D_ServiceStatus status;
} D_ServiceInfo;

int d_project_init(const ProjectSettings settings);
int d_project_start(const ProjectSettings settings);
int d_project_stop(const ProjectSettings settings);

D_ServiceInfo d_service_info_get(const char *project_name, const char *service_name);
int d_service_start(const char *project_name, const ServiceSettings service_settings);
int d_service_stop(const char *project_name, const ServiceSettings service_settings);

#endif
