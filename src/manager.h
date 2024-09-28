#ifndef MANAGER__H
#define MANAGER__H

#include <stdio.h>
#include <pthread.h>
#include <stdbool.h>

#include "settings.h"

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

int manager_init(void);
void manager_stop(void);

struct project_settings *projects_settings_get(void);
struct project_info *projects_info_get(void);
int project_settings_get(const char *project_name, struct project_settings *settings);
int project_info_get(const char *project_name, struct project_info *info);
int project_upsert(const struct project_settings settings);
int project_remove(const char *project_name);
int project_start(const char *project_name);
int project_stop(const char *project_name);

int service_info_get(const char *project_name, const char *service_name, struct service_info *info);
int service_start(const char *project_name, const char *service_name);
int service_stop(const char *project_name, const char *service_name);

void service_info_free(struct service_info info);
void project_info_free(struct project_info info);

#endif
