#ifndef SETTINGS__H
#define SETTINGS__H

typedef struct ServiceSettings
{
    char *name;
    char *pwd;
    char **command;
} ServiceSettings;

typedef struct ProjectSettings
{
    char *name;
    ServiceSettings *services;
} ProjectSettings;

char *project_settings_parse(const char *data, ProjectSettings *settings);
char *project_settings_stringify(const ProjectSettings settings);

ProjectSettings project_settings_dup(const ProjectSettings settings);
ServiceSettings service_settings_dup(const ServiceSettings settings);

void project_settings_free(ProjectSettings settings);
void service_settings_free(ServiceSettings settings);

#endif