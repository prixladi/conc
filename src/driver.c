#include <sys/stat.h>
#include <sys/types.h>
#include <stdlib.h>
#include <dirent.h>
#include <stdio.h>
#include <stdbool.h>
#include <unistd.h>
#include <signal.h>
#include <dirent.h>

#include "utils/string.h"
#include "utils/vector.h"
#include "utils/log.h"
#include "utils/threading.h"

#include "driver.h"
#include "process.h"

static const char *root_projects_dir = "./projects";
static const char *log_file_name = "log.txt";
static const char *meta_file_name = "meta";

typedef struct ServiceProcessInfo
{
    int pid;
    time_t c_time;
} ServiceProcessInfo;

static int ensure_project_dir_exists(const char *project_name);
static int ensure_service_dir_exists(const char *project_name, const char *service_name);

static void write_service_meta_file(const char *project_name, const char *service_name, ServiceProcessInfo info);
static bool try_parse_service_meta_file(const char *project_name, const char *service_name, ServiceProcessInfo *info);

static FILE *open_service_meta_file(const char *project_name, const char *service_name, const char *mode);
static FILE *open_project_meta_file(const char *project_name, const char *mode);

static int get_running_service_pid(const char *project_name, const char *service_name);
static bool try_get_pid_info(int pid, struct stat *sts);
static int kill_pid(int pid);

int driver_mount()
{
    mkdir(root_projects_dir, S_IRWXU | S_IRWXG | S_IRWXO);

    DIR *dir = opendir(root_projects_dir);
    if (dir == NULL)
    {
        LOG_ERROR("(System) Driver root project dir init failed\n");
        return 1;
    }

    closedir(dir);

    LOG_INFO("(System) Driver mounted\n");
    return 0;
}

void driver_unmount()
{
    LOG_INFO("(System) Driver unmounted\n");
}

int d_project_init(const ProjectSettings settings)
{
    ensure_project_dir_exists(settings.name);

    FILE *fptr = open_project_meta_file(settings.name, "w");
    if (fptr == NULL)
        LOG_ERROR("Unable to open meta file for project. Project: '%s'", settings.name);

    char *stringified_settings = project_settings_stringify(settings);
    fprintf(fptr, "%s", stringified_settings);
    fclose(fptr);
    free(stringified_settings);

    for (size_t i = 0; i < vector_length(settings.services); i++)
        ensure_service_dir_exists(settings.name, settings.services[i].name);

    return 0;
}

int d_project_start(const ProjectSettings settings)
{
    for (size_t i = 0; i < vector_length(settings.services); i++)
    {
        ServiceSettings service = settings.services[i];
        d_service_start(settings.name, service);
    }

    return 0;
}

int d_project_stop(const ProjectSettings settings)
{
    for (size_t i = 0; i < vector_length(settings.services); i++)
    {
        ServiceSettings service = settings.services[i];
        d_service_stop(settings.name, service);
    }

    return 0;
}

D_ServiceInfo d_service_info_get(const char *project_name, const char *service_name)
{
    int running_pid = get_running_service_pid(project_name, service_name);

    D_ServiceInfo info = {
        .status = running_pid > 0 ? D_RUNNING : running_pid == 0 ? D_STOPPED
                                                                 : D_NONE,
    };

    return info;
}

int d_service_start(const char *project_name, const ServiceSettings service_settings)
{
    int running_pid = get_running_service_pid(project_name, service_settings.name);
    if (running_pid > 0)
        return 409;

    char *logfile_path = str_concat(root_projects_dir, "/", project_name, "/", service_settings.name, "/", log_file_name, NULL);
    int pid = process_start(project_name, service_settings, logfile_path);

    free(logfile_path);

    struct stat sts;
    if (try_get_pid_info(pid, &sts) == false)
        return 1;

    ServiceProcessInfo info = {
        .pid = pid,
        .c_time = sts.st_ctime};

    write_service_meta_file(project_name, service_settings.name, info);

    return 0;
}

int d_service_stop(const char *project_name, const ServiceSettings service_settings)
{
    int running_pid = get_running_service_pid(project_name, service_settings.name);
    if (running_pid <= 0)
        return 409;

    kill_pid(running_pid);

    return 0;
}

static int get_running_service_pid(const char *project_name, const char *service_name)
{
    ServiceProcessInfo info = {0};
    if (try_parse_service_meta_file(project_name, service_name, &info) == false)
        return -1;

    struct stat sts;
    if (try_get_pid_info(info.pid, &sts) == false)
        return 0;

    return info.pid;
}

static int ensure_service_dir_exists(const char *project_name, const char *service_name)
{
    char *service_dir = str_concat(root_projects_dir, "/", project_name, "/", service_name, NULL);
    int result = mkdir(service_dir, S_IRWXU | S_IRWXG | S_IRWXO);
    free(service_dir);
    return result;
}

static int ensure_project_dir_exists(const char *project_name)
{
    char *project_dir = str_concat(root_projects_dir, "/", project_name, NULL);
    int result = mkdir(project_dir, S_IRWXU | S_IRWXG | S_IRWXO);
    free(project_dir);
    return result;
}

static void write_service_meta_file(const char *project_name, const char *service_name, ServiceProcessInfo info)
{
    FILE *fptr = open_service_meta_file(project_name, service_name, "w");
    if (fptr == NULL)
        LOG_ERROR("Unable to open meta file for service. Project: '%s', service: %s", project_name, service_name);

    fprintf(fptr, "%d\n%ld", info.pid, info.c_time);
    fclose(fptr);
}

#define MAX_META_LINE_LEN 1024 // This should be sufficient but probably should handle cases when it is not
static bool try_parse_service_meta_file(const char *project_name, const char *service_name, ServiceProcessInfo *info)
{
    FILE *fptr = open_service_meta_file(project_name, service_name, "r");
    if (fptr == NULL)
        return false;

    char buffer[MAX_META_LINE_LEN];

    int parsed = 0;
    if (fgets(buffer, MAX_META_LINE_LEN, fptr))
    {
        buffer[strcspn(buffer, "\n")] = '\0';
        int pid = atoi(buffer);
        if (pid != 0)
        {
            info->pid = pid;
            parsed++;
        }
    }

    if (fgets(buffer, MAX_META_LINE_LEN, fptr))
    {
        buffer[strcspn(buffer, "\n")] = '\0';
        time_t c_time = atoll(buffer);
        if (c_time != 0)
        {
            info->c_time = c_time;
            parsed++;
        }
    }

    fclose(fptr);
    return parsed == 2;
}

static FILE *open_service_meta_file(const char *project_name, const char *service_name, const char *modes)
{
    char *meta_file_path = str_concat(root_projects_dir, "/", project_name, "/", service_name, "/", meta_file_name, NULL);
    FILE *fptr;
    fptr = fopen(meta_file_path, modes);
    free(meta_file_path);
    return fptr;
}

static FILE *open_project_meta_file(const char *project_name, const char *modes)
{
    char *meta_file_path = str_concat(root_projects_dir, "/", project_name, "/", meta_file_name, NULL);
    FILE *fptr;
    fptr = fopen(meta_file_path, modes);
    free(meta_file_path);

    return fptr;
}

static bool try_get_pid_info(int pid, struct stat *sts)
{
    char *pid_string = int_to_string(pid);
    char *proc = str_concat("/proc/", pid_string, NULL);
    free(pid_string);

    int result = stat(proc, sts);
    free(proc);

    return result != -1;
}

static int kill_pid(int pid)
{
    struct stat sts;
    if (try_get_pid_info(pid, &sts) == false)
        return 0;

    int attempt = 0;
    while (attempt < 10)
    {
        if (attempt)
            sleep_ms(50);

        kill(pid, attempt > 6 ? SIGKILL : SIGTERM);

        if (try_get_pid_info(pid, &sts) == false)
            return 0;

        attempt++;
    }

    return attempt;
}