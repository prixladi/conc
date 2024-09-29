#ifndef MANAGER__H
#define MANAGER__H

#include <stdio.h>
#include <pthread.h>
#include <stdbool.h>

#include "settings.h"

enum m_result
{
    M_DRIVER_ERROR = -501,
    M_PROJECT_NOT_FOUND = -404,
    M_SERVICE_NOT_FOUND = -414,
    M_ERROR = -1,
    M_OK = 0,
    M_NO_ACTION = 204,
};

enum service_status
{
    IDLE,
    RUNNING,
    STOPPED,
};

struct service_info
{
    char *name;
    enum service_status status;
};

struct project_info
{
    char *name;
    struct service_info *services;
};

enum m_result manager_init(void);
enum m_result manager_stop(void);

struct project_settings *projects_settings_get(void);
struct project_info *projects_info_get(void);

enum m_result project_settings_get(const char *proj_name, struct project_settings *settings);
enum m_result project_info_get(const char *proj_name, struct project_info *info);
enum m_result project_upsert(const struct project_settings settings);
enum m_result project_remove(const char *proj_name);
enum m_result project_start(const char *proj_name);
enum m_result project_stop(const char *proj_name);

enum m_result service_info_get(const char *proj_name, const char *serv_name, struct service_info *info);
enum m_result service_start(const char *proj_name, const char *serv_name);
enum m_result service_stop(const char *proj_name, const char *serv_name);

void service_info_free(struct service_info info);
void project_info_free(struct project_info info);

#endif
