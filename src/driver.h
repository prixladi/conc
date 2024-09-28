#ifndef DRIVER__H
#define DRIVER__H

#include "settings.h"

enum d_result
{
    D_FS_ERROR = -1,
    D_PROC_ERROR = -2,
    D_OK = 0,
    D_NO_ACTION = 204
};

enum d_service_status
{
    D_NONE,
    D_RUNNING,
    D_STOPPED,
};

struct d_service_info
{
    enum d_service_status status;
    int pid;
};

enum d_result driver_mount(void);
enum d_result driver_unmount(void);

char **d_get_all_stored_settings(void);

enum d_result d_project_init(const struct project_settings settings);
enum d_result d_project_remove(const struct project_settings settings);

enum d_result d_service_info_get(const char *proj_name, const char *serv_name, struct d_service_info *info);
enum d_result d_service_start(const char *proj_name, const struct service_settings service_settings);
enum d_result d_service_stop(const char *proj_name, const struct service_settings service_settings);

#endif
