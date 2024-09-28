#ifndef DRIVER__H
#define DRIVER__H

#include "settings.h"

enum d_service_status
{
    D_NONE,
    D_RUNNING,
    D_STOPPED,
};

struct d_service_info
{
    enum d_service_status status;
};

int driver_mount(void);
void driver_unmount(void);

char **d_get_all_stored_settings(void);

int d_project_init(const struct project_settings settings);
int d_project_start(const struct project_settings settings);
int d_project_stop(const struct project_settings settings);
int d_project_remove(const struct project_settings settings);

struct d_service_info d_service_info_get(const char *project_name, const char *service_name);
int d_service_start(const char *project_name, const struct service_settings service_settings);
int d_service_stop(const char *project_name, const struct service_settings service_settings);

#endif
